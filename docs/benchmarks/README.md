# trs Benchmarks

Living laboratory for trs. These benchmarks exist to help us **learn, measure, and iterate** — not to be marketing material or regression gates.

## Why this folder exists

Every CLI in this space (rtk, token-saver, ccp, repomix, claw-compactor, pi)
ships different tradeoffs. Some compress harder, some preserve more signal,
some are faster on specific inputs. Instead of guessing, we run the
comparisons here and let the numbers guide the decisions we make in trs.

The goal is internal knowledge — "what do we actually do better, and where
should we improve?" — not to publish a leaderboard.

## What's in here

| Script | Purpose |
|--------|---------|
| [`benchmark.sh`](./benchmark.sh) | Comparative runs against rtk and token-saver on a curated set of real-world commands |
| [`benchmark-real.sh`](./benchmark-real.sh) | Longer, more varied workload (slower, more representative) |
| [`chain-rewrite.sh`](./chain-rewrite.sh) | Verifies the hook rewriter correctly handles `A && B` chains, pipes, redirections, and edge cases |

## How to use

```bash
# Quick comparative run (from repo root)
./docs/benchmarks/benchmark.sh --all

# Full workload
./docs/benchmarks/benchmark-real.sh

# Chain rewriter sanity check (runs in <1s)
./docs/benchmarks/chain-rewrite.sh
```

All scripts prefer `./target/release/trs` if present (so you're testing your
latest changes), falling back to the `trs` in `$PATH`.

## What these benchmarks are NOT

- Not part of the CI pipeline — runtime varies too much between environments.
- Not reproducible science — network latency, disk cache, and terminal
  buffering all move the needle.
- Not a commitment to future behavior — results change as parsers evolve.

## What they are

Quick, approximate signals that help us answer questions like:

- Did the new chain rewriter regress the simple-command case?
- Is trs's compact formatter faster than rtk's for git status?
- How does `trs ingest --budget` compare to repomix's default output?

When in doubt, **read the output by eye**, not the summary number. The
interesting signal is usually in the cases that surprise you.

## Contributing

If you find a command or workflow where trs underperforms, add a case to the
relevant script and let the numbers speak. That's how we learn.
