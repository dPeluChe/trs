#!/usr/bin/env python3
"""How often does a compact output send the agent back for what it cut?

A cut that forces a second call can cost more than it saved, and no byte count
shows that. This walks Claude Code transcripts in order and, for each Bash call
of a family trs compresses, looks at the next calls for one of these:

  tee     read the saved full output
  bypass  same command through a pipe or redirect, which trs never rewrites
  repeat  same command again, with no change between that could alter its output
  drill   narrower command on the same subject (`git diff -- path`, `git show`)

Drill is not always a loss: an agent narrows on purpose. Read it as a ceiling.
Each step of a `&&` chain counts; tokens come from single-command calls only,
because a chain's result is mixed. Resumed sessions repeat history, so calls are deduped by tool_use_id. Nothing
leaves the machine; only aggregates are printed.

Usage:  python3 docs/development/benchmarks/recovery.py [--since YYYY-MM-DD]
        [--projects DIR] [--window N]
"""

import argparse
import collections
import glob
import json
import os
import re
import shlex
import statistics

from truth import tokenizer

FAMILIES = {
    "git diff": ("git", "diff"),
    "git show": ("git", "show"),
    "git log": ("git", "log"),
    "git status": ("git", "status"),
    "git branch": ("git", "branch"),
    "gh pr": ("gh", "pr"),
    "gh run": ("gh", "run"),
    "cargo test": ("cargo", "test"),
    "cargo build": ("cargo", "build"),
    "cargo clippy": ("cargo", "clippy"),
    "pytest": ("pytest",),
    "grep": ("grep",),
    "rg": ("rg",),
    "find": ("find",),
    "ls": ("ls",),
    "ps": ("ps",),
    "docker": ("docker",),
}
DRILLS = {"git diff": ("git diff", "git show"), "git log": ("git show",), "git show": ("git show",)}
EDITS = {"Edit", "Write", "NotebookEdit"}
TEE = re.compile(r"\.trs/tee/")
CHAIN = re.compile(r"&&|;|\n")
PIPED = re.compile(r"\||>")
THEN_PIPED = re.compile(r"\s*(?:\||>)")
MUTATES = re.compile(r"\b(?:git (?:add|commit|checkout|pull|push|merge|rebase|reset|stash)|mkdir|rm|mv|cp|touch|sed -i|cargo (?:build|fmt|clean)|gh pr (?:create|merge|edit))\b")
# Their state moves on its own, so a second look is polling, not recovery.
VOLATILE = {"gh pr", "gh run", "docker", "ps"}


def segments(cmd):
    return [p.strip() for p in CHAIN.split(cmd) if p.strip() and not p.strip().startswith("cd ")]


def family(cmd):
    try:
        words = shlex.split(cmd)
    except ValueError:
        return None
    for name, head in FAMILIES.items():
        if tuple(words[: len(head)]) == head:
            return name
    return None


def body(result):
    content = result.get("content")
    if isinstance(content, str):
        return content
    return "\n".join(c.get("text", "") for c in content or [] if isinstance(c, dict))


def calls(path, since):
    """One session's tool calls in order, each with its result text."""
    seen, order, done = set(), [], {}
    try:
        lines = open(path, encoding="utf-8", errors="replace")
    except OSError:
        return []
    for line in lines:
        if '"tool_use"' not in line and '"tool_result"' not in line:
            continue
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if since and str(d.get("timestamp", "")) < since:
            continue
        content = (d.get("message") or {}).get("content")
        for c in content if isinstance(content, list) else []:
            if not isinstance(c, dict):
                continue
            if c.get("type") == "tool_use" and c.get("id") not in seen:
                seen.add(c.get("id"))
                order.append((c.get("id"), c.get("name"), c.get("input") or {}))
            elif c.get("type") == "tool_result":
                done[c.get("tool_use_id")] = body(c)
    return [(name, inp, done.get(i, "")) for i, name, inp in order]


def recovery(cmd, fam, after):
    """The first later call that went back for this output, if any."""
    changed = False
    drills = DRILLS.get(fam, ())
    for name, inp, text in after:
        target = str(inp.get("file_path", "")) if name == "Read" else str(inp.get("command", ""))
        if TEE.search(target):
            return "tee", text
        if name in EDITS or (name == "Bash" and MUTATES.search(target)):
            changed = True
        if name != "Bash":
            continue
        for later in segments(target):
            if later == cmd and not changed and fam not in VOLATILE:
                return "repeat", text
            if later.startswith(cmd) and THEN_PIPED.match(later[len(cmd):]):
                return "bypass", text
            if drills and later != cmd and not PIPED.search(later) and later.startswith(drills):
                return "drill", text
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--projects", default=os.path.expanduser("~/.claude/projects"))
    ap.add_argument("--since", default="")
    ap.add_argument("--window", type=int, default=5)
    a = ap.parse_args()
    _, count = tokenizer()
    stats = collections.defaultdict(collections.Counter)
    first, back = collections.defaultdict(list), collections.defaultdict(list)
    sessions = glob.glob(os.path.join(a.projects, "**", "*.jsonl"), recursive=True)
    for path in sessions:
        session = calls(path, a.since)
        for i, (name, inp, text) in enumerate(session):
            parts = segments(str(inp.get("command", ""))) if name == "Bash" else []
            for cmd in parts:
                fam = None if PIPED.search(cmd) else family(cmd)
                if not fam:
                    continue
                stats[fam]["calls"] += 1
                if len(parts) == 1:
                    first[fam].append(count(text))
                found = recovery(cmd, fam, session[i + 1 : i + 1 + a.window])
                if found:
                    stats[fam][found[0]] += 1
                    back[fam].append(count(found[1]))
    print(f"{len(sessions)} sessions, next {a.window} calls\n")
    print("| family | calls | went back | tee | bypass | repeat | drill | median tok | median tok back |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    rows = sorted(stats.items(), key=lambda kv: -kv[1]["calls"])
    for fam, s in rows:
        n = sum(s[k] for k in ("tee", "bypass", "repeat", "drill"))
        tail = statistics.median(back[fam]) if back[fam] else 0
        print(f"| {fam} | {s['calls']} | {n * 100 / s['calls']:.1f}% | {s['tee']} | {s['bypass']} | "
              f"{s['repeat']} | {s['drill']} | {statistics.median(first[fam] or [0]):.0f} | {tail:.0f} |")
    total = sum(s["calls"] for _, s in rows)
    hard = sum(s[k] for _, s in rows for k in ("tee", "bypass", "repeat"))
    drill = sum(s["drill"] for _, s in rows)
    print(f"\n{total} calls: {hard * 100 / max(total, 1):.1f}% went back for the same output, "
          f"{drill * 100 / max(total, 1):.1f}% drilled in")


if __name__ == "__main__":
    main()
