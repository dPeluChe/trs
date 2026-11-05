#!/usr/bin/env python3
"""How often does an agent go back to a command family's output?

A cut that forces a second call can cost more than it saved, and no byte count
shows that. This walks Claude Code transcripts in order, subagents included,
and for each Bash call of a family trs compresses looks at the next calls for:

  tee     read the saved full output
  bypass  same command through a pipe or redirect, which trs never rewrites
  repeat  same command again, same result
  drill   narrower command on the same subject (`git diff -- path`, `git show`)

This is a re-fetch rate with no control: a transcript does not say whether trs
cut that output, and agents narrow and pipe out of habit too. Use it to rank
families, not to price a cut. Calls are deduped by tool_use_id within a file.
Each step of a `&&` chain counts; tokens come from single-command calls only,
because a chain's result is mixed. Nothing leaves the machine.

Usage:  python3 docs/development/benchmarks/recovery.py [--since YYYY-MM-DD]
        [--projects DIR] [--window N]
"""

import argparse
import collections
import glob
import os
import re
import shlex
import statistics

from bench_stats import records
from truth import tokenizer

FAMILIES = ("git diff", "git show", "git log", "git status", "git branch", "gh pr", "gh run",
            "cargo test", "cargo build", "cargo clippy", "pytest", "grep", "rg", "find", "ls",
            "ps", "docker")
HEADS = {f.split()[0] for f in FAMILIES}
DRILLS = {"git diff": ("git diff", "git show"), "git log": ("git show",), "git show": ("git show",)}
TEE = ".trs/tee/"
CHAIN = re.compile(r"&&|\|\||;|\n")
PIPED = re.compile(r"\||>")
# Their state moves on its own, so a second look is polling, not recovery.
VOLATILE = {"gh pr", "gh run", "docker", "ps"}
SAME_OUTPUT = ("tee", "bypass", "repeat")


def segments(cmd):
    parts = map(str.strip, CHAIN.split(cmd.replace("2>&1", "")))
    return [p for p in parts if p and not p.startswith("cd ")]


def family(cmd):
    if PIPED.search(cmd) or cmd.split(None, 1)[0] not in HEADS:
        return None
    try:
        words = shlex.split(cmd)
    except ValueError:
        return None
    return next((f for f in FAMILIES if words[: len(f.split())] == f.split()), None)


def body(result):
    content = result.get("content")
    if isinstance(content, str):
        return content
    return "\n".join(c.get("text", "") for c in content or [] if isinstance(c, dict))


def calls(path, since):
    """One session's tool calls in order, each with its result text."""
    uses, done = {}, {}
    for d in records(path, '"tool_use"', '"tool_result"'):
        if since and str(d.get("timestamp", "")) < since:
            continue
        content = (d.get("message") or {}).get("content")
        for c in content if isinstance(content, list) else []:
            if not isinstance(c, dict):
                continue
            if c.get("type") == "tool_use":
                uses.setdefault(c.get("id"), (c.get("name"), c.get("input") or {}))
            elif c.get("type") == "tool_result":
                done[c.get("tool_use_id")] = body(c)
    return [(name, inp, done.get(i, "")) for i, (name, inp) in uses.items()]


def recovery(cmd, fam, text, after):
    """The first later call that went back for this output, if any."""
    for name, inp, later_text in after:
        target = str(inp.get("file_path", "")) if name == "Read" else str(inp.get("command", ""))
        if TEE in target:
            return "tee", later_text
        if name != "Bash":
            continue
        for later in segments(target):
            if later == cmd and later_text == text and fam not in VOLATILE:
                return "repeat", later_text
            if later.startswith(cmd) and PIPED.match(later[len(cmd):].lstrip()):
                return "bypass", later_text
            if later != cmd and not PIPED.search(later) and later.startswith(DRILLS.get(fam, ())):
                return "drill", later_text
    return None


def tally(session, window, count, stats, first, back):
    for i, (name, inp, text) in enumerate(session):
        parts = segments(str(inp.get("command", ""))) if name == "Bash" else []
        for cmd in parts:
            fam = family(cmd)
            if not fam:
                continue
            stats[fam]["calls"] += 1
            if len(parts) == 1:
                first[fam].append(count(text))
            found = recovery(cmd, fam, text, session[i + 1 : i + 1 + window])
            if found:
                stats[fam][found[0]] += 1
                back[fam].append(count(found[1]))


def median(values):
    return statistics.median(values or [0])


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
        tally(calls(path, a.since), a.window, count, stats, first, back)
    print(f"{len(sessions)} transcripts, next {a.window} calls\n")
    print("| family | calls | re-fetched | tee | bypass | repeat | drill | median tok | median tok back |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    rows = sorted(stats.items(), key=lambda kv: -kv[1]["calls"])
    for fam, s in rows:
        same = sum(s[k] for k in SAME_OUTPUT)
        print(f"| {fam} | {s['calls']} | {(same + s['drill']) * 100 / s['calls']:.1f}% | {s['tee']} | "
              f"{s['bypass']} | {s['repeat']} | {s['drill']} | {median(first[fam]):.0f} | {median(back[fam]):.0f} |")
    total = sum(s["calls"] for _, s in rows) or 1
    same = sum(s[k] for _, s in rows for k in SAME_OUTPUT)
    drill = sum(s["drill"] for _, s in rows)
    print(f"\n{total} calls: {same * 100 / total:.1f}% same output again, {drill * 100 / total:.1f}% drilled in")


if __name__ == "__main__":
    main()
