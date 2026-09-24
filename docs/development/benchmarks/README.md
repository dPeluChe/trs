# trs Benchmarks

Living laboratory for trs. These benchmarks exist to help us **learn, measure, and iterate**, not to be marketing material or regression gates.

## Why this folder exists

Every CLI in this space (rtk, token-saver, ccp, repomix, claw-compactor, pi)
ships different tradeoffs. Some compress harder, some preserve more signal,
some are faster on specific inputs. Instead of guessing, we run the
comparisons here and let the numbers guide the decisions we make in trs.

The goal is internal knowledge: "what do we actually do better, and where
should we improve?", not to publish a leaderboard.

## What's in here

| Script | Purpose |
|--------|---------|
| [`benchmark.sh`](./benchmark.sh) | Comparative runs against rtk and token-saver on a curated set of real-world commands |
| [`benchmark-real.sh`](./benchmark-real.sh) | Longer, more varied workload (slower, more representative) |
| [`truth.py`](./truth.py) | Whether trs kept what the agent needed. Pipelines sent through the real hook must print what the raw pipeline prints; paths, `file:line` refs, hashes and error lines from the raw output must survive; a plain `tail` cut to trs's size is the control. Tokens via tiktoken `o200k_base` when installed |
| [`chain-rewrite.sh`](./chain-rewrite.sh) | Verifies the hook rewriter handles `A && B` chains, pipes, redirections, transparent wrappers (`time`, `nohup`, `noglob`, etc.), and edge cases |

## How to use

```bash
# Quick comparative run (from repo root)
./docs/development/benchmarks/benchmark.sh --all

# Full workload
./docs/development/benchmarks/benchmark-real.sh

# Chain rewriter sanity check (runs in <1s)
./docs/development/benchmarks/chain-rewrite.sh

# Signal survival (exits 1 if any pipeline differs from raw)
python3 docs/development/benchmarks/truth.py
```

All scripts prefer `./target/release/trs` if present (so you're testing your
latest changes), falling back to the `trs` in `$PATH`.

## What these benchmarks are NOT

- Not part of the CI pipeline: runtime varies too much between environments.
- Not reproducible science: network latency, disk cache, and terminal
  buffering all move the needle.
- Not a commitment to future behavior: results change as parsers evolve.

## What they are

Quick, approximate signals that help us answer questions like:

- Did the new chain rewriter regress the simple-command case?
- Is trs's compact formatter faster than rtk's for git status?
- How does `trs ingest --budget` compare to repomix's default output?

When in doubt, **read the output by eye**, not the summary number.

Bytes saved is the easy half. `benchmark.sh` crowns whoever prints the fewest
bytes, so a tool that dropped the failing test's name would win it, and so did
a trs that answered `find src | wc -l` with 14 instead of 241. `truth.py`
exists for that half. The
interesting signal is usually in the cases that surprise you.

## Contributing

If you find a command or workflow where trs underperforms, add a case to the
relevant script and let the numbers speak. That's how we learn.

## Latest truth run

2026-09-24, on this repo, tokens counted with `o200k_base`.

**Pipelines through the hook**: 1/9 matched raw on v0.8.1, 9/9 with #162.
Before it, the hook compressed the producer of any pipeline, so `| grep`,
`| wc -l` and `| jq` parsed trs's summary.

**Anchors**:

| command | raw tok | trs tok | cut | anchors kept | same-size `tail` | flag |
|---|---:|---:|---:|---:|---:|---|
| `git log -10` | 9556 | 8010 | 16% | 100% | 93% | |
| `git diff HEAD~1` | 3185 | 2474 | 22% | 59% | 74% | tail wins |
| `git diff HEAD~5` | 24350 | 345 | 99% | 42% | 3% | RISK |
| `git show HEAD --stat` | 784 | 69 | 91% | 67% | 22% | RISK |
| `git status` | 110 | 36 | 67% | 100% | 50% | |
| `ls -la src` | 2215 | 764 | 66% | 100% | 33% | |
| `find src -name '*.rs'` | 1936 | 305 | 84% | 50% | 19% | |
| `grep -rn 'fn main' src` | 1236 | 1133 | 8% | 100% | 80% | |

What the rows mean, read by eye:

- `git show --stat` drops the commit header (hash, author, date, message)
  and marks a modified file with only insertions as `+`, which reads as a
  new file.
- `git diff` on a large range becomes a file list with line counts and no
  hunks, and says nothing about how to get them.
- `git diff` on a small range drops context lines, which is where most of the
  lost anchors are. At a 22% cut, plain `tail` keeps more.
- `find` hides entries behind `+58 more`.

Anchor matching tolerates trs regrouping `grep` under file headers and `find`
under directories, so "kept" is a ceiling: a low number is a real loss.
