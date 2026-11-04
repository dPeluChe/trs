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
import statistics
import sys

from bench_stats import boot_diff


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
            lines = open(path, encoding="utf-8", errors="replace")
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


def report(name, before, after, resamples):
    if not before or not after:
        return
    cells = []
    for label, fn in (("mean", statistics.fmean), ("median", statistics.median)):
        b, a = fn(before), fn(after)
        lo, hi = boot_diff(before, after, fn, resamples)
        cells.append(f"{label} {b:.0f} -> {a:.0f} ({a - b:+.0f}, 95% CI {lo:+.0f}..{hi:+.0f})")
    print(f"  {name:34} n={len(before)}/{len(after)}  " + "  ".join(cells))


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
    print(f"estimated output tokens per assistant message, before vs after {cut}")
    print("observational: task mix and model changed between the periods\n")
    groups = {}
    for ts, model, tokens, text in msgs:
        side = "after" if ts[:10] >= cut else "before"
        groups.setdefault((model, side, False), []).append(tokens)
        if text:
            groups.setdefault((model, side, True), []).append(tokens)
    models = {k[0] for k in groups}
    shown = 0
    for model in sorted(models, key=lambda m: -len(groups.get((m, "after", False), []))):
        for prose, label in ((False, "all messages"), (True, "with prose")):
            before = groups.get((model, "before", prose), [])
            after = groups.get((model, "after", prose), [])
            if len(before) >= args.min and len(after) >= args.min:
                report(f"{model} ({label})", before, after, args.resamples)
                shown += not prose
    if not shown:
        print(f"  no model has {args.min}+ messages on both sides of {cut}")


if __name__ == "__main__":
    main()
