#!/usr/bin/env python3
"""Output tokens per assistant message, before vs after the output-saver block.

Reads Claude Code transcripts (~/.claude/projects/*/*.jsonl) on this machine and
compares the `usage.output_tokens` of assistant messages written before and
after a cut-over date (default: when ~/.claude/trs.md was created). Nothing
leaves the machine; only aggregates are printed.

This is observational, not an experiment. The task mix, the model and the
prompts changed between the two periods, so a difference is a hint, never proof
that the block caused it. Numbers are labelled "estimated": `output_tokens`
includes thinking and tool-call arguments, not only the prose the block steers.
Each model is compared only with itself. The confidence interval is a
bootstrap on the difference of means and medians (seeded, so reruns agree).

Usage:  python3 docs/development/benchmarks/transcript_output.py [--since YYYY-MM-DD]
        [--projects DIR] [--min N] [--resamples N]
"""

import argparse
import datetime as dt
import glob
import json
import os
import random
import statistics
import sys


def default_since():
    path = os.path.expanduser("~/.claude/trs.md")
    try:
        st = os.stat(path)
    except OSError:
        return None
    born = getattr(st, "st_birthtime", st.st_mtime)
    return dt.datetime.fromtimestamp(born, dt.timezone.utc).date().isoformat()


def messages(projects_dir):
    """One (timestamp, model, output_tokens, has_text) per assistant message id."""
    seen = {}
    for path in glob.glob(os.path.join(projects_dir, "*", "*.jsonl")):
        try:
            lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
        except OSError:
            continue
        for line in lines:
            try:
                d = json.loads(line)
            except ValueError:
                continue
            if d.get("type") != "assistant" or d.get("isSidechain"):
                continue
            m = d.get("message") or {}
            mid, usage = m.get("id"), m.get("usage") or {}
            if str(m.get("model", "")).startswith("<") or not mid or "output_tokens" not in usage or not d.get("timestamp"):
                continue
            text = any(c.get("type") == "text" for c in m.get("content") or [])
            prev = seen.get(mid)
            seen[mid] = (
                d["timestamp"],
                m.get("model") or "?",
                usage["output_tokens"],
                text or (prev[3] if prev else False),
            )
    return list(seen.values())


def boot_diff(before, after, fn, resamples, rng):
    diffs = []
    for _ in range(resamples):
        b = [rng.choice(before) for _ in before]
        a = [rng.choice(after) for _ in after]
        diffs.append(fn(a) - fn(b))
    diffs.sort()
    return diffs[int(0.025 * resamples)], diffs[int(0.975 * resamples) - 1]


def report(name, before, after, resamples, rng):
    if not before or not after:
        return
    row = [name]
    for label, fn in (("mean", statistics.fmean), ("median", statistics.median)):
        b, a = fn(before), fn(after)
        lo, hi = boot_diff(before, after, fn, resamples, rng)
        row.append(f"{label} {b:.0f} -> {a:.0f} ({a - b:+.0f}, 95% CI {lo:+.0f}..{hi:+.0f})")
    print(f"  {row[0]:34} n={len(before)}/{len(after)}  " + "  ".join(row[1:]))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--since", default=default_since())
    ap.add_argument("--projects", default=os.path.expanduser("~/.claude/projects"))
    ap.add_argument("--min", type=int, default=30, help="minimum messages per side")
    ap.add_argument("--resamples", type=int, default=2000)
    args = ap.parse_args()
    if not args.since:
        sys.exit("no --since given and ~/.claude/trs.md not found")
    cut = args.since
    msgs = messages(args.projects)
    rng = random.Random(7)
    print(f"estimated output tokens per assistant message, before vs after {cut}")
    print("observational: task mix and model changed between the periods\n")
    by_model = {}
    for ts, model, tokens, text in msgs:
        side = "after" if ts[:10] >= cut else "before"
        by_model.setdefault(model, {"before": [], "after": [], "tb": [], "ta": []})
        by_model[model][side].append(tokens)
        if text:
            by_model[model]["ta" if side == "after" else "tb"].append(tokens)
    shown = 0
    for model, d in sorted(by_model.items(), key=lambda kv: -len(kv[1]["after"])):
        if len(d["before"]) >= args.min and len(d["after"]) >= args.min:
            report(f"{model} (all messages)", d["before"], d["after"], args.resamples, rng)
            if len(d["tb"]) >= args.min and len(d["ta"]) >= args.min:
                report(f"{model} (with prose)", d["tb"], d["ta"], args.resamples, rng)
            shown += 1
    if not shown:
        print(f"  no model has {args.min}+ messages on both sides of {cut}")


if __name__ == "__main__":
    main()
