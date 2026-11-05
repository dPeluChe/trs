import json
"""Shared by the benchmark scripts: a seeded bootstrap CI on a difference."""

import random
import statistics


def boot_diff(before, after, fn=statistics.fmean, resamples=2000, seed=11):
    """95% CI of fn(after) - fn(before) by resampling both sides."""
    rng = random.Random(seed)
    diffs = sorted(
        fn(rng.choices(after, k=len(after))) - fn(rng.choices(before, k=len(before)))
        for _ in range(resamples)
    )
    return diffs[int(0.025 * resamples)], diffs[int(0.975 * resamples) - 1]


def records(path, *needles):
    """Parsed JSONL records. `needles` skip the parse for lines without any of
    them, which is most of a 4 GB transcript folder."""
    try:
        lines = open(path, encoding="utf-8", errors="replace")
    except OSError:
        return
    for line in lines:
        if needles and not any(n in line for n in needles):
            continue
        try:
            yield json.loads(line)
        except ValueError:
            continue
