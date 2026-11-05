# trs Benchmarks

Living laboratory for trs. These benchmarks exist to help us **learn, measure, and iterate**, not to be marketing material or regression gates.

## Why this folder exists

Every CLI in this space (rtk, token-saver, ccp, repomix, claw-compactor, pi)
ships different tradeoffs. Some compress harder, some preserve more signal,
some are faster on specific inputs. Instead of guessing, we run the
comparisons here and let the numbers guide the decisions we make in trs.

The goal is internal knowledge: "what do we actually do better, and where
should we improve?", not to publish a leaderboard.

## What we learned

[`output-cost-findings.md`](../output-cost-findings.md) records where agent cost actually goes, what the output-saver tests showed and how to run the next one.

## What's in here

| Script | Purpose |
|--------|---------|
| [`benchmark.sh`](./benchmark.sh) | Comparative runs against rtk and token-saver on a curated set of real-world commands |
| [`benchmark-real.sh`](./benchmark-real.sh) | Longer, more varied workload (slower, more representative) |
| [`recovery.py`](./recovery.py) | How often a compact output sends the agent back for what it cut, from your own Claude Code transcripts |
| [`transcript_output.py`](./transcript_output.py) | Output tokens per assistant message before vs after the output-saver block, from your own Claude Code transcripts, per model, with a bootstrap 95% CI. Observational and labelled estimated |
| [`output_saver_ab.py`](./output_saver_ab.py) | A/B for the output-saver block with a control: no block, "Answer concisely.", the trs block; isolated `claude -p` runs, style markers, fidelity cases (negation, number, destructive warning), bootstrap CI. Spends usage, refuses without `--yes`; `--dry-run` tests the harness for free |
| [`output_saver_agentic.py`](./output_saver_agentic.py) | Agentic A/B for the output-saver block: tools on, small repo tasks with a test as success check, compares arms (`--arm NAME=FILE`) on `total_cost_usd`, turns, output tokens and success with a bootstrap CI. Parallel (`--jobs`), spend caps (`--total-cap-usd`), no `bypassPermissions` (scoped Bash allowlist). Spends usage, refuses without `--yes` |
| [`output_saver_stale.py`](./output_saver_stale.py) | Fixture for `output_saver_agentic.py --suite stale`: a repo whose docs contradict its code, to score correct answers per arm |
| [`truth.py`](./truth.py) | Whether trs kept what the agent needed. Pipelines sent through the real hook must print what the raw pipeline prints; paths, `file:line` refs, hashes and error lines from the raw output must survive; a plain `tail` cut to trs's size is the control; anchors found only in the middle half of the raw output are scored on their own (BLIND when a cut of 50% or more keeps under half of them). Tokens via tiktoken `o200k_base` when installed |
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

## Context lines in small diffs

`git diff` keeps only the changed lines and each hunk header (it names the
function); the unchanged context lines go. Keeping some was tried against
32 real diffs from this repo (4-500 changed lines each, last 60 commits,
anchors from `truth.py`, tokens counted with o200k_base), by temporarily
varying how many context lines surround each change:

| context lines kept | tokens cut | anchors kept (median) |
|---:|---:|---:|
| 0 (current) | 23.7% | 58.7% |
| 1 | 16.1% | 60.0% |
| 2 | 8.5% | 61.2% |
| 3 (nearly the raw diff) | 1.8% | 64.1% |

One line of context costs 7.6 points of savings for 1.3 points of anchors, and
even the near-raw diff keeps only 64%: the anchors lost are mostly the
`index abc..def` hashes and the `diff --git a/x b/x` paths, which the file
summary replaces on purpose. So the default stays at 0.

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
  per directory. `ps aux` keeps the top processes.
- **Zero-anchor rows are not always a loss.** For `ps aux`, the anchors are
  PIDs in a list the agent rarely reads whole, and the full list is one path
  away.

## Facts, not literal text

2026-11-05, v0.9.1 against the latest release of the closest similar tool, same
corpus, `compare.py --only trs,rtk --reps 3`. The anchor score counted a reworded
line as lost: `tests::x: src/main.rs:65:60: ...` scored as missing the `test x ...
FAILED` it replaces. `truth.facts` reads the output for meaning, for every tool alike:

- a line counts as a diagnostic by its shape (`error:`, `FAILED`, `panicked at`),
  not by holding the word `fail`, and it survives when most of its words do;
- `index` blob hashes stop counting;
- a Docker row can be reached by either of its two names.

RISK, BLIND and `tail-wins` now use facts, so they are not comparable with the
sections below. `anchors kept` stays as the old literal column.

| | trs 0.9.1 | rtk 1.1.0 |
|---|---:|---:|
| tokens cut, whole corpus | 87% | 83% |
| mean cut per command | 32% | 34% |
| median cut per command | 23% | 33% |
| facts kept, mean per command | 91% | 80% |
| facts kept, pooled | 57% | 46% |
| commands with no fact left | 1 | 4 |
| RISK rows | 2 | 1 |
| pipelines | 9/9 | 8/9 |
| hook ms | 5.2 | 18.2 |

`ps aux` and `git diff HEAD~5` hold 80% of the corpus tokens, so per-command scores
say more than the totals. The real loss is `git diff HEAD~5`: 99% cut, 94 of 367
facts, and BLIND. The corpus runs on this repo, so its diffs move with each commit.

## Do agents go back to an output?

2026-11-05, `recovery.py --since 2026-06-01`, 2,594 transcripts, next 5 calls. Of
35,130 calls, 1.2% asked for the same output again and 1.8% drilled in with a
narrower command.

| family | calls | re-fetched | tee | bypass | repeat | drill |
|---|---:|---:|---:|---:|---:|---:|
| git show | 173 | 26.0% | 0 | 2 | 2 | 41 |
| git diff | 2695 | 22.2% | 10 | 31 | 7 | 549 |
| git status | 1934 | 9.9% | 1 | 161 | 29 | 0 |
| git log | 5577 | 1.8% | 11 | 12 | 33 | 44 |
| gh pr | 2887 | 0.9% | 3 | 24 | 0 | 0 |
| ls | 6634 | 0.6% | 4 | 29 | 10 | 0 |
| git branch | 1129 | 0.6% | 1 | 6 | 0 | 0 |
| grep | 12773 | 0.3% | 8 | 14 | 16 | 0 |

This has no control: a transcript does not say whether trs cut that output, and
agents narrow a diff or pipe `git status` out of habit too. It ranks families; it
does not price a cut. The calls span many trs versions.

## A large diff shares a budget

2026-11-05, same corpus after the change. Past 500 change lines, `git diff` used to
print a file list and nothing else. It now has one renderer and a 24 KB budget of
hunks, with no threshold: code shares it fairly, generated files get what the code
leaves, and each hunk of a large file gets a share. A diff that fits prints whole.
Each cut says `[trs: N lines omitted]`, and the raw output is saved.

| `git diff HEAD~5` | tokens cut | facts kept |
|---:|---:|---:|
| trs before | 99% | 94 of 367 |
| trs now | 85% | 152 of 383 |
| rtk 1.1.0 | 85% | 155 of 383 |

| | before | now | rtk 1.1.0 |
|---|---:|---:|---:|
| tokens cut, whole corpus | 87% | 84% | 84% |
| facts kept, pooled | 57% | 61% | 46% |
| facts kept, mean per command | 91% | 92% | 79% |
| RISK rows | 2 | 1 | 1 |

The whole-corpus cut fell because that one diff now spends tokens on code. BLIND
stays on that row: 24 KB cannot carry the middle of a 2,500-line diff.

`truth.py` adds SILENT: under 70% of facts kept with no way back named. No row has
it. Zero context lines in small diffs stays as measured above: nothing is noted
there, because the file itself is the way back.

`git branch -a` drops nothing to gain ground. The tool that cuts more here prints
`... +20 more` and hides real branches, so trs only nests repeated prefixes.

## trs alone on v0.9.1

2026-11-04, same corpus and command as below. This is what usetrs.dev shows.
`truth.py`: 9/9 pipelines match, 1 RISK row (`git diff HEAD~5`), 0 BLIND rows.
`chain-rewrite.sh`: 43/43. Startup: 3.65 ms median, against 3.6 ms on v0.9.0 with
the same loop.

| | tokens cut | anchors kept |
|---|---:|---:|
| all 38 commands (token-weighted) | 85% | 62% (mean 84% per command) |
| median command | 27% | |
| commands cut by half or more | 13 of 38 | |

Do not compare the rows without `ps aux` to v0.9.0 (44% then, 61% now): the corpus
runs on this repo, and `git diff HEAD~5` grew past the 500-changed-line limit, so
it became a file summary (22,799 to 640 tokens, 97% cut). That row is the RISK:
the file list plus the path to the raw output, not the changes. Corrections that
trade savings for truth do not show in this corpus: it has no `make` target and no
translated `git status`.

### Rereads: measured, not built

A cache for repeated reads looked large because `sed` and `cat` are 38% of what
agents receive. Over 40 days of transcripts (233 MB of tool output, deduplicated by
tool-use id, since a resumed session repeats its history):

| shape | share of tool output |
|---|---:|
| same command, same output | 0.04% |
| `sed -n` range already seen in full | 0.07% |
| `sed -n` lines already seen, upper bound | 0.15% |

Agents read each range once, and a cache would claim "unchanged" to an agent whose
context was compacted. Counting lines without that deduplication gives a false 7%.

## trs alone on v0.9.0

2026-11-04, same corpus, this repo's state that day, `compare.py --only trs
--reps 1`, tokens counted with o200k_base. This is what usetrs.dev shows.
`truth.py`: 9/9 pipelines match, 0 RISK rows, 1 BLIND row (`git diff HEAD~1`).
`chain-rewrite.sh`: 43/43.

| | tokens cut | anchors kept |
|---|---:|---:|
| all 38 commands (token-weighted) | 84% | 64% (mean 84% per command) |
| without `ps aux` | 44% | |
| median command | 29% | |
| commands cut by half or more | 11 of 38 | |

### The "up to 60%" on usetrs.dev

The site said ~70% until v0.8.10. That came from 2026-10-21 data, before `trs
stats` left failed runs and outputs over 30 KB out of its totals. Measured
again on 2026-11-04, from the author's history:

| window | commands | saved |
|---|---:|---:|
| all history (203 days) | 84,325 | 58% |
| 90 days | 25,730 | 48% |
| 30 days | 6,030 | 24% |
| 7 days | 1,370 | 35% |

The windows swing with the work: a month of low-output commands saves little.
"Up to 60%" is a ceiling from the history, on the output trs handles, not on
the bill: see [`output-cost-findings.md`](../output-cost-findings.md). Quote the
table above for a given command. With oversized outputs counted back in, 30 days is 58%, but a few
huge outputs carry that number.

## trs alone on v0.8.9

2026-10-21, same corpus (now 38 commands: `docker ps -a` and `docker images`
were added), this repo's state that day, `compare.py --only trs --reps 3`,
tokens counted with o200k_base.

| | tokens cut | anchors kept |
|---|---:|---:|
| all 38 commands (token-weighted) | 81% | 65% (mean 88% per command) |
| without `ps aux` | 39% | |
| median command | 28% | |
| commands cut by half or more | 12 of 38 | |

`ps aux` is 97,702 of the 139,322 raw tokens, so the headline figure is mostly
that one row. Every number here moves with the repo state: the September run
had a 40k-token `git diff HEAD~5` (98% cut) where this one has 7.7k (26%).
Quote the per-command rows, not one percentage.

The ~70% that usetrs.dev showed then was a different measurement: the author's history over 30
days (2026-10-21), only commands run by agents through a hook (3,259, 10.4M
input tokens): 73.3% saved. `trs stats --days 30` shows 68.9% over 8,359
commands because it also counts test suites and hand-typed runs (3,649 of them
are test noise, such as another project's tests calling `trs digest`). Same
caveat as the table: a few verbose outputs carry the total. Without the top 1%
of outputs it is 58.7%, the median command saves 1%, and the last 7 days were
23.6%.
