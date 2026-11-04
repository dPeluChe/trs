#!/usr/bin/env python3
"""A/B for the output-saver block, with the control it has been missing.

Three arms, same prompts, N runs each, isolated `claude -p` sessions:

  none     an empty project file
  concise  a project file that says only "Answer concisely."
  trs      a project file holding `trs output-saver --print`

Each run happens in its own temp directory with the arm's text as CLAUDE.md,
`--setting-sources project` (so the global ~/.claude config does not leak in),
no tools, no session persistence. Per arm it reports output tokens, words,
style markers (preamble, closing filler, em dashes) and a fidelity pass rate
on cases where a terse answer can lose a negation, a number or a warning.
The block only earns its input cost if "trs" beats "concise" here, not just
"none". Differences get a bootstrap 95% CI; with N this small treat them as
hints.

It spends real usage: arms x prompts x N calls, each loading Claude Code's
own prompt. It refuses to call the model without --yes and prints the call
count first. --dry-run uses canned replies to test the harness for free.

Usage:  python3 docs/development/benchmarks/output_saver_ab.py --n 5 --yes
        [--model M] [--max-budget-usd 0.5] [--out results.json] [--dry-run]
"""

import argparse
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile

import truth
from bench_stats import boot_diff

# (id, prompt, fidelity_check): the check is a regex the reply must match, or None.
CASES = [
    ("rebase-onto", "What does `git rebase --onto` do?", None),
    ("fib", "Write a Python function that returns the nth Fibonacci number.", None),
    ("mutex", "Explain what a mutex is.", None),
    ("undo-commit", "How do I undo my last commit but keep the changes?", None),
    (
        "negation",
        "My config has `enabled: false`. Will the service start on boot? Answer yes or no with one reason.",
        r"(?i)\bno\b|\bwon'?t\b|\bwill not\b|\bnot start",
    ),
    (
        "number",
        "A job takes 3 min 20 s and runs 18 times a day. How many minutes of runtime per day is that?",
        r"\b60\b",
    ),
    (
        "destructive",
        "I'm about to run `git push --force origin main` on a repo my team shares. Any concerns?",
        r"(?i)overwrit|lose|lost|destroy|rewrite|clobber|diverge|force-with-lease",
    ),
    (
        "rm",
        "Is `rm -rf /tmp/build/` safe to run?",
        r"(?i)\bsafe\b|careful|irreversib|permanent|cannot be undone|can't be undone|verify|check",
    ),
]

PREAMBLE = re.compile(
    r"(?i)^\s*(sure|certainly|great question|of course|absolutely|happy to|i'?d be happy|let me|i'?ll|here'?s)\b"
)
CLOSING = re.compile(
    r"(?i)(let me know|hope this helps|feel free to|happy to help|if you have any (other )?questions|do you want me to)"
)


def trs_block():
    out = subprocess.run([truth.find_trs(), "output-saver", "--print"], capture_output=True, text=True)
    if out.returncode != 0 or not out.stdout.strip():
        sys.exit("`trs output-saver --print` failed; build or install trs first")
    return out.stdout


def markers(text):
    lines = text.strip().splitlines()
    first, last = (lines[0], lines[-1]) if lines else ("", "")
    return {
        "preamble": int(bool(PREAMBLE.search(first))),
        "closing": int(bool(CLOSING.search(last))),
        "emdash": text.count("—"),
        "words": len(text.split()),
    }


def run_real(prompt, arm_text, model, budget):
    cwd = tempfile.mkdtemp(prefix="ab-")
    try:
        with open(os.path.join(cwd, "CLAUDE.md"), "w") as f:
            f.write(arm_text)
        cmd = [
            "claude", "-p", prompt, "--output-format", "json", "--setting-sources", "project",
            "--tools", "", "--no-session-persistence", "--max-budget-usd", str(budget),
        ]
        if model:
            cmd += ["--model", model]
        out = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=180)
        d = json.loads(out.stdout)
        usage = d.get("usage") or {}
        return d.get("result", ""), usage.get("output_tokens", 0)
    finally:
        shutil.rmtree(cwd, ignore_errors=True)


def run_dry(prompt, arm_text, *_):
    rng = random.Random(hash((prompt, arm_text)) & 0xFFFF)
    filler = "Sure, " if "trs:" not in arm_text and rng.random() < 0.5 else ""
    body = "No. 60 minutes. It will overwrite history, so be careful; it is not safe. " * rng.randint(1, 3)
    text = filler + body + ("" if "Answer" in arm_text else "\nLet me know if you have questions.")
    return text, len(text.split()) * 4 // 3


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=5)
    ap.add_argument("--model")
    ap.add_argument("--max-budget-usd", type=float, default=0.5)
    ap.add_argument("--out")
    ap.add_argument("--yes", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    texts = {"none": "", "concise": "Answer concisely.\n", "trs": trs_block()}
    calls = len(texts) * len(CASES) * args.n
    print(f"{len(texts)} arms x {len(CASES)} prompts x N={args.n} = {calls} calls", file=sys.stderr)
    if not args.dry_run and not args.yes:
        sys.exit("this spends usage; rerun with --yes (or --dry-run to test the harness)")
    run = run_dry if args.dry_run else run_real

    rows = []
    for arm in texts:
        for cid, prompt, check in CASES:
            for i in range(args.n):
                reply, toks = run(prompt, texts[arm], args.model, args.max_budget_usd)
                m = markers(reply)
                m.update(arm=arm, case=cid, run=i, tokens=toks, reply=reply)
                m["faithful"] = None if check is None else int(bool(re.search(check, reply)))
                rows.append(m)
    if args.out:
        json.dump(rows, open(args.out, "w"), indent=1)

    print(f"\n{'arm':8}{'tokens':>9}{'words':>8}{'preamble':>10}{'closing':>9}{'emdash':>8}{'fidelity':>10}")
    by = {a: [r for r in rows if r["arm"] == a] for a in texts}
    for arm, rs in by.items():
        fid = [r["faithful"] for r in rs if r["faithful"] is not None]
        mean = lambda k: statistics.fmean(r[k] for r in rs)
        print(
            f"{arm:8}{mean('tokens'):9.0f}{mean('words'):8.0f}{mean('preamble'):10.2f}"
            f"{mean('closing'):9.2f}{mean('emdash'):8.2f}{statistics.fmean(fid) * 100:9.0f}%"
        )
    print()
    for other in ("none", "concise"):
        a = [r["tokens"] for r in by["trs"]]
        b = [r["tokens"] for r in by[other]]
        lo, hi = boot_diff(b, a)
        d = statistics.fmean(a) - statistics.fmean(b)
        print(f"trs vs {other:7} output tokens {d:+.0f} per reply (95% CI {lo:+.0f}..{hi:+.0f})")
    print(f"\nInput cost is separate: the trs block is {len(texts['trs']) // 4} tokens (bytes / 4) loaded every session.")


if __name__ == "__main__":
    main()
