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

## trs against similar tools

2026-09-24, one repo state, all tools on one machine. Reproduce:

```bash
bash docs/development/benchmarks/compare/setup.sh     # builds the others from their clones
python3 docs/development/benchmarks/compare/compare.py
```

Every hook-based tool runs through its own agent hook: the hook decides
(`hook ms`, paid on every agent call), then the shell runs what it returned
(`exec +ms` over raw). claw-compactor has no hook, so its text API compresses
the output instead. Anchors are paths, `file:line` refs, hashes and error lines
taken from the raw output. "pipelines" counts the 9 checks from `truth.py`,
compared after whitespace normalization.

36 commands, tokens: o200k_base

| tool | tokens cut | anchors kept | RISK | grew | pipelines | hook ms | exec +ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| trs | 81% | 60% | 2 | 2 | 9/9 | 4.5 | 4.6 |
| rtk | 79% | 44% | 3 | 4 | 5/9 | 11.4 | 5.1 |
| token-saver | 28% | 58% | 4 | 0 | 9/9 | 41.6 | 50.2 |
| squeez | 89% | 29% | 5 | 1 | 9/9 | 3.8 | 48.7 |
| token-optimizer | 36% | 43% | 3 | 0 | 9/9 | 25.6 | 58.6 |
| claw-compactor | 30% | 83% | 0 | 2 | n/a | 0.0 | 0.0 |

Per command, `cut / anchors kept`:

| command | raw tok | trs | rtk | token-saver | squeez | token-optimizer | claw-compactor |
|---|---:|---:|---:|---:|---:|---:|---:|
| `grep -n "fn " src/report.rs` | 145 | 0% / 100% | -24% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% |
| `grep -rn "emit_compressed" src` | 228 | -2% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | -8% / 47% |
| `grep -rn "fn main" src` | 1236 | 15% / 100% | 0% / 100% | 42% / 68% | 42% / 63% | 36% / 71% | 15% / 32% |
| `grep -rn -A3 "pub(crate) fn" src/report.rs` | 331 | 9% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 4% / 100% |
| `grep -rn -C2 "is_verbatim_invocation" src` | 1784 | 22% / 100% | 0% / 100% | 88% / 44% | 25% / 39% | 77% / 44% | 13% / 100% |
| `grep -rn "TODO\|FIXME" src` | 596 | 10% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 11% / 33% |
| `rg -n "captures_output" src` | 40 | 2% / 100% | -15% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | -20% / 33% |
| `git log -5` | 3162 | 84% / 42% | 85% / 19% | 96% / 16% | 84% / 16% | 0% / 100% | 35% / 52% |
| `git log -20` | 15494 | 87% / 59% | 88% / 28% | 98% / 11% | 96% / 9% | 0% / 100% | 40% / 64% |
| `git log --oneline -30` | 635 | 0% / 100% | 0% / 100% | 65% / 36% | 54% / 43% | 0% / 100% | 0% / 100% |
| `git log --stat -3` | 1764 | 0% / 100% | 82% / 35% | 96% / 11% | 83% / 8% | 0% / 100% | 28% / 97% |
| `git diff HEAD~1` | 6450 | 8% / 80% | 25% / 80% | 40% / 77% | 93% / 24% | 94% / 16% | 4% / 93% |
| `git diff HEAD~5` | 39946 | 98% / 24% | 79% / 33% | 27% / 72% | 99% / 7% | 99% / 3% | 9% / 77% |
| `git diff HEAD~3 --stat` | 534 | 20% / 100% | 0% / 100% | 52% / 50% | 0% / 100% | 0% / 100% | 23% / 100% |
| `git show HEAD` | 6796 | 8% / 80% | 29% / 80% | 38% / 78% | 94% / 25% | 94% / 17% | 8% / 93% |
| `git show HEAD --stat` | 527 | 18% / 100% | 0% / 100% | 0% / 100% | 40% / 77% | 0% / 100% | 22% / 100% |
| `git show HEAD~2:src/report.rs` | 1675 | 0% / 100% | 0% / 100% | 0% / 100% | 82% / 25% | 77% / 0% | 34% / 100% |
| `git status` | 17 | 47% / 100% | 18% / 100% | 6% / 100% | -153% / 100% | 0% / 100% | 6% / 100% |
| `git branch -a` | 544 | 41% / 100% | 77% / 50% | 88% / 0% | 39% / 100% | 74% / 0% | 18% / 100% |
| `ls -la` | 1108 | 69% / 100% | 68% / 100% | 74% / 100% | 21% / 100% | 0% / 100% | 12% / 100% |
| `ls -la src` | 2245 | 65% / 100% | 60% / 100% | 80% / 65% | 60% / 38% | 36% / 63% | 12% / 100% |
| `find src -name "*.rs"` | 1955 | 83% / 49% | 91% / 0% | 85% / 17% | 83% / 18% | 80% / 23% | 12% / 100% |
| `find . -name "*.md" -not -path "./target/*"` | 1314 | 48% / 100% | 98% / 0% | 4% / 87% | 74% / 42% | 64% / 57% | 8% / 96% |
| `wc -l src/rewrite.rs src/rewrite_decide.rs src/report.rs src/init.rs` | 38 | -3% / 100% | 40% / 0% | 0% / 100% | 0% / 100% | 0% / 100% | 29% / 100% |
| `tail -50 src/rewrite_decide.rs` | 401 | 0% / 100% | 0% / 100% | 0% / 100% | 39% / 100% | 0% / 100% | 46% / 100% |
| `cargo build` | 24 | 0% / 100% | -25% / 100% | 4% / 100% | 0% / 100% | 0% / 100% | 17% / 100% |
| `cargo clippy --all-targets` | 24 | 83% / 100% | 67% / 100% | 4% / 100% | 0% / 100% | 0% / 100% | 17% / 100% |
| `gh pr list --state all --limit 30` | 1238 | 52% / 0% | 56% / 0% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% |
| `gh pr view 162` | 922 | 4% / 100% | 0% / 100% | 50% / 0% | 0% / 100% | 0% / 100% | 22% / 75% |
| `gh run list --limit 10` | 459 | 69% / 100% | 73% / 100% | 0% / 100% | 0% / 100% | 0% / 100% | 0% / 100% |
| `ps aux` | 68382 | 99% / 0% | 98% / 0% | 0% / 100% | 95% / 33% | 0% / 100% | 50% / 100% |
| `cargo test` | 832 | 76% / 56% | 77% / 67% | 65% / 100% | 75% / 50% | 0% / 100% | 64% / 89% |
| `cargo build` | 24 | 0% / 100% | -25% / 100% | 4% / 100% | 0% / 100% | 0% / 100% | 17% / 100% |
| `bun test` | 548 | 55% / 70% | 0% / 90% | 90% / 50% | 15% / 90% | 0% / 90% | 22% / 100% |
| `pytest` | 372 | 67% / 58% | 59% / 42% | 41% / 92% | 73% / 42% | 0% / 100% | 7% / 100% |
| `pytest -q` | 223 | 44% / 64% | 32% / 45% | 9% / 91% | 0% / 100% | 0% / 100% | 8% / 91% |

Read by eye before trusting a cell:

- **No tool gets both.** squeez cuts the most (89%) and keeps the fewest anchors
  (29%). claw-compactor keeps the most (83%) and cuts 30%. Among the tools that
  cut more than half, trs keeps the most anchors (60%).
- **Speed.** trs has the fastest hook plus run: 4.5 ms to decide, 4.6 ms over
  raw. The Python hooks cost 25-42 ms to decide and ~50 ms to run.
- **Pipelines.** One tool still compresses the first command of a pipeline:
  `git diff | grep -c '^+'` gives 0. trs had this bug until #162.
- **Where trs cuts little, on purpose.** `grep` keeps every match and the
  context lines the caller asked for. `gh pr view` keeps the PR body. `git log
  --stat`/`-p`, `tail` and `git show rev:file` are passed through verbatim.
- **Where trs still loses anchors.** `git diff` on a large range becomes a
  summary (now with a pointer to the full output). `find` over a big tree lists
  per directory. `ps aux` keeps the top processes. `gh pr list` omits branch
  names.
- **Zero-anchor rows are not always a loss.** For `ps aux`, the anchors are
  PIDs in a list the agent rarely reads whole, and the full list is one path
  away.
