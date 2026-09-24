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

## Seven tools, one scoring rule

2026-09-24, same repo state, `o200k_base` tokens, scored by `truth.py`'s
anchor and pipeline checks. trs is main at #163. Each tool runs through its own
agent hook where it has one (trs, rtk 0.42.3, token-saver, squeez 1.48.9,
token-optimizer at fbe2070), else its text API on the command's output
(claw-compactor 7.1.0, headroom 0.38.0 with the downloaded ML model off). Clones
live under the spark root, tagged `trs` (`spark tag list trs`); the adapters are
per-machine and not in this repo.

| tool | tokens cut | anchors kept | RISK rows | pipelines correct |
|---|---:|---:|---:|---:|
| trs | 69% | 66% | 2 | 9/9 |
| rtk | 67% | 45% | 2 | 5/9 |
| token-saver | 54% | 43% | 1 | 9/9 |
| squeez | 87% | 29% | 2 | 9/9 |
| token-optimizer | 65% | 41% | 1 | 9/9 |
| claw-compactor | 17% | 87% | 0 | 8/9 |
| headroom | 2% | 92% | 0 | 9/9 |

Per command, `cut / anchors kept`:

| command | trs | rtk | token-saver | squeez | token-optimizer | claw-compactor | headroom |
|---|---:|---:|---:|---:|---:|---:|---:|
| `git log -10` | 16% / 100% | 91% / 23% | 95% / 3% | 94% / 23% | 0% / 100% | 40% / 75% | 0% / 100% |
| `git log --oneline -30` | 0% / 100% | 0% / 100% | 68% / 32% | 58% / 39% | 0% / 100% | 0% / 100% | 0% / 100% |
| `git diff HEAD~1` | 29% / 59% | 17% / 62% | 9% / 68% | 19% / 88% | 59% / 53% | 8% / 97% | 0% / 100% |
| `git diff HEAD~5` | 99% / 42% | 68% / 55% | 38% / 66% | 98% / 7% | 98% / 10% | 10% / 88% | 2% / 73% |
| `git show HEAD --stat` | 91% / 67% | 0% / 100% | 0% / 100% | 52% / 44% | 51% / 11% | 29% / 89% | 0% / 100% |
| `git status` | 83% / 100% | 48% / 100% | 4% / 100% | -78% / 100% | 78% / 100% | 4% / 100% | 0% / 100% |
| `ls -la src` | 66% / 100% | 60% / 100% | 80% / 66% | 60% / 39% | 36% / 64% | 12% / 100% | 0% / 100% |
| `find src -name '*.rs'` | 84% / 50% | 91% / 0% | 85% / 17% | 83% / 18% | 80% / 23% | 12% / 100% | 0% / 100% |
| `grep -rn 'fn main' src` | 8% / 100% | 0% / 100% | 42% / 68% | 42% / 63% | 36% / 71% | 15% / 32% | 20% / 100% |
| `grep -rn 'emit_compressed' src` | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | -8% / 47% | 20% / 47% |
| `cargo clippy --all-targets` | 93% / 100% | 86% / 100% | 61% / 100% | 59% / 100% | 59% / 100% | 66% / 100% | 59% / 100% |

Read by eye before trusting a cell:

- **Cut and kept trade off, and nobody escapes it.** squeez cuts most (87%) and
  keeps least (29%); its own header agrees (`[anchors: 7%]` on the big diff).
  headroom keeps 92% by barely compressing shell output (2%): its text
  compressor is an ML model that needs a download. trs sits at 69% cut, 66%
  kept, the best kept figure among the tools that cut over half.
- **Recoverable is not the same as kept.** squeez and token-optimizer store the
  full output and tell the agent how to fetch it (`squeez_retrieve`,
  `expand <key>`). trs's big-diff summary drops the hunks with no way back.
  That is the gap worth closing first.
- **Pipelines.** Scored on whitespace-normalized output. token-saver passes
  the whole pipeline to its wrapper and compresses the final text, so filters
  see raw bytes: 9/9, an approach trs's backlog already lists. One tool still
  compresses a pipeline's first command: `git diff | grep -c '^+'` gives 0
  instead of 804, the bug #162 removed from trs.
- **Where trs loses outright:** `git log -10` (16% cut, where others cut 90%+
  by truncating bodies) and `grep` (8% cut vs ~40% for three others).
