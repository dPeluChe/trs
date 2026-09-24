#!/usr/bin/env python3
"""Does trs keep what the agent needed? Bytes alone cannot say.

Three checks, each aimed at a way a byte benchmark scores a loss as a win:

1. Pipelines. Every pipeline goes through `trs rewrite`, exactly as an agent's
   hook would send it, and must print what the raw pipeline prints. A pipe
   hands stdout to a program that parses the real bytes; v0.8.1 compressed
   the producer and `find | wc -l` said 14 instead of 241.
2. Anchors. Paths, `file:line` refs, commit hashes and error lines are pulled
   from the raw output, and each one must appear in trs's output. A row that
   cuts 90% or more but keeps under 70% of its anchors is flagged.
3. A naive control. The raw output's last lines, cut to trs's byte count.
   Where `tail` keeps more anchors in the same space, trs is not earning
   the compression.

Tokens are counted with tiktoken (o200k_base) when it is installed, else
bytes/4, and the header says which.

Run from the repo root:  python3 docs/development/benchmarks/truth.py
Uses the newer of ./target/{release,debug}/trs, else trs on $PATH.
"""

import json
import os
import re
import shutil
import subprocess
import sys

PIPELINES = [
    "find src -name '*.rs' | wc -l",
    "git diff HEAD~3 | grep -c '^+'",
    "git diff HEAD~3 --stat | tail -1",
    "git log -10 | grep -c Author",
    "grep -rhoE 'fn [a-z_]+' src | sort -u | wc -l",
    "grep -rn 'TODO' src | wc -l",
    "ls -la src | wc -l",
    "git log --oneline -30 | grep -c fix",
    "git status | head -3",
]

COMMANDS = [
    "git log -10",
    "git log --oneline -30",
    "git diff HEAD~1",
    "git diff HEAD~5",
    "git show HEAD --stat",
    "git status",
    "ls -la src",
    "find src -name '*.rs'",
    "grep -rn 'fn main' src",
    "grep -rn 'emit_compressed' src",
    "cargo clippy --all-targets",
]

ANCHORS = {
    "path": re.compile(r"[\w./-]+\.(?:rs|ts|tsx|js|py|go|md|json|toml|sh|swift|lock)\b"),
    "file:line": re.compile(r"[\w./-]+\.\w+:\d+"),
    "hash": re.compile(r"\b(?=[0-9a-f]*\d)(?=[0-9a-f]*[a-f])[0-9a-f]{7,40}\b"),
}
ERROR_LINE = re.compile(r"error|fail|panick|warning|FAIL", re.I)
RISK_CUT, RISK_KEEP = 90.0, 70.0


def find_trs():
    # Newest build wins: a stale release binary silently measures old code.
    built = [p for p in ("./target/release/trs", "./target/debug/trs") if os.access(p, os.X_OK)]
    if built:
        return os.path.abspath(max(built, key=os.path.getmtime))
    return shutil.which("trs") or sys.exit("trs not found")


def tokenizer():
    try:
        import tiktoken

        enc = tiktoken.get_encoding("o200k_base")
        return "o200k_base", lambda s: len(enc.encode(s, disallowed_special=()))
    except ImportError:
        return "bytes/4 (install tiktoken for real counts)", lambda s: len(s.encode()) // 4


def sh(cmd, stderr=False):
    """stdout, plus stderr when asked: an agent's tool result carries both,
    and cargo writes its diagnostics to stderr."""
    env = dict(os.environ, TRS_SKIP="", GIT_PAGER="cat", PAGER="cat")
    r = subprocess.run(["bash", "-c", cmd], capture_output=True, text=True, env=env,
                       errors="replace")
    return r.stdout + r.stderr if stderr else r.stdout


def hook(trs, cmd):
    """The command an agent's hook would actually run, or the original."""
    payload = json.dumps({"tool_name": "Bash", "tool_input": {"command": cmd}})
    out = subprocess.run([trs, "rewrite"], input=payload, capture_output=True, text=True).stdout
    if not out.strip():
        return cmd
    new = json.loads(out)["hookSpecificOutput"]["updatedInput"]["command"]
    return new.replace("trs ", trs + " ", 1)


def anchors(text):
    found = set()
    for kind, rx in ANCHORS.items():
        found |= {(kind, m) for m in rx.findall(text)}
    for line in text.splitlines():
        if ERROR_LINE.search(line):
            found.add(("error", " ".join(line.split())[:40]))
    return found


def kept(found, text):
    """Anchors present in `text`, tolerating trs's two regroupings: `grep`
    under a per-file header, `find` under a per-directory one. Loose on
    purpose, so a high score is a ceiling and a low one is a real loss."""
    flat = " ".join(text.split())

    def has(kind, a):
        if a in text or a in flat:
            return True
        if kind == "hash":
            return a[:7] in text
        if kind == "file:line":
            path, line = a.rsplit(":", 1)
            return path in text and re.search(rf"(?m)^\s*{line}[:\s]", text) is not None
        if kind == "path":
            parent, _, base = a.rpartition("/")
            stem = base.rsplit(".", 1)[0]
            near = parent.rsplit("/", 1)[-1] if parent else ""
            return re.search(rf"\b{re.escape(stem)}\b", text) is not None and near in text
        return False

    return sum(has(k, a) for k, a in found)


def naive_tail(raw, budget):
    out = []
    size = 0
    for line in reversed(raw.splitlines()):
        size += len(line) + 1
        if size > budget:
            break
        out.append(line)
    return "\n".join(reversed(out))


def main():
    trs = find_trs()
    tok_name, count = tokenizer()
    print(f"trs: {trs}\ntokens: {tok_name}\n")

    print("Pipelines: through the hook, must print what the raw pipeline prints")
    bad = 0
    for p in PIPELINES:
        raw, via = sh(p), sh(hook(trs, p))
        ok = raw == via
        bad += not ok
        print(f"  {'ok  ' if ok else 'DIFF'}  {p:48} raw {raw.strip()[:24]!r:28} trs {via.strip()[:24]!r}")
    print(f"  {len(PIPELINES) - bad}/{len(PIPELINES)} match\n")

    print("Anchors: what survives, and whether plain tail would keep more")
    head = f"  {'command':30}{'raw tok':>8}{'trs tok':>8}{'cut':>6}{'anchors':>9}{'kept':>6}{'tail':>6}  flag"
    print(head)
    risks = 0
    for c in COMMANDS:
        raw = sh(c, stderr=True)
        out = sh(f"{trs} {c}", stderr=True)
        if not raw.strip():
            print(f"  {c:30} (no output, skipped)")
            continue
        rt, ot = count(raw), count(out)
        cut = (rt - ot) / rt * 100 if rt else 0.0
        found = anchors(raw)
        n = len(found)
        k = kept(found, out) * 100 / n if n else 100.0
        t = kept(found, naive_tail(raw, len(out))) * 100 / n if n else 100.0
        flags = []
        if ot > rt:
            flags.append("GREW")
        if cut >= RISK_CUT and k < RISK_KEEP:
            flags.append("RISK")
            risks += 1
        if t > k:
            flags.append("tail-wins")
        print(f"  {c[:30]:30}{rt:8}{ot:8}{cut:5.0f}%{n:9}{k:5.0f}%{t:5.0f}%  {' '.join(flags)}")
    print(f"\n  RISK = cut >= {RISK_CUT:.0f}% with under {RISK_KEEP:.0f}% of anchors kept ({risks} rows)")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
