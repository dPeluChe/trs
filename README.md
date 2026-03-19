# trs - TARS CLI

Transform noisy terminal output into compact, structured signal.

A CLI toolkit that reduces token consumption by **68-90%** for developers, automation pipelines, and AI agents.

## The Problem

Every terminal command produces verbose output. When AI agents run `git status`, `ls -la`, or `grep`, they consume hundreds of unnecessary tokens parsing boilerplate text.

## The Solution

Just prefix your command with `trs`:

```
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs

# Raw: 497 bytes → trs: 81 bytes (80% reduction)
```

```
$ trs git log -5
<Antonio>
  4ca8fe4 Optimize parsers (12 min ago)
  b49dda3 Fix all tests (21 min ago)
  581541e Fix 3 replace bugs (28 min ago)

# Raw: 7.2 KB → trs: 690 bytes (90% reduction)
```

```
$ trs cargo test
cargo test: ok (2012 passed, 0 failed, 70 suites, 3.21s)

# Raw: 55 KB → trs: 58 bytes (99% reduction)
```

## Install

```bash
# npm (recommended)
npm install -g tars-cli

# Try without installing
npx tars-cli git status

# From source (requires Rust)
cargo install --path .
```

## Quick Start

```bash
# Just prefix any command with trs
trs git status
trs git diff
trs git log -10
trs ls -la
trs npm test
trs cargo test
trs env

# Add --json for structured output (flags work anywhere)
trs git status --json
trs --json git status      # same thing
```

## Supported Commands

`trs` auto-detects the command and applies the right parser:

```bash
# Git
trs git status              # branch, ahead/behind, file changes
trs git diff                # files changed with +adds/-dels (DiffCrunch compression)
trs git diff --stat         # stat format supported too
trs git log -10             # grouped by author, relative time
trs git branch -a           # current + local/remote

# Files
trs ls -la                  # dirs first, files with sizes, no noise
trs tree -L 2               # summary with dir/file counts
trs find . -name "*.rs"     # grouped by directory

# Search
trs grep -rn "pattern" .    # grouped by file with line numbers

# Test runners
trs cargo test              # pass/fail/ignore/filter/suite counts
trs pytest                  # passed/failed/skipped summary
trs npm test                # works with npm, pnpm, yarn, bun
trs jest                    # jest, vitest supported

# Package managers
trs npm ls                  # clean dependency list
trs pip list                # name@version format
trs npm install             # summary: added, warnings, errors

# Linters (grouped by file + rule, 89% reduction)
trs cargo clippy            # Rust clippy warnings/errors
trs eslint src/             # ESLint issues grouped
trs ruff check .            # Python ruff issues
trs biome check src/        # Biome linting
trs golangci-lint run       # Go linting

# Build tools
trs cargo build             # errors + warnings only
trs tsc                     # TypeScript build errors
trs make                    # build output summary

# Docker
trs docker ps               # container status (up/down)
trs docker logs <name>      # log level detection (LogCrunch folding)

# GitHub CLI
trs gh pr list              # compact pull request list
trs gh issue list           # compact issue list
trs gh run list             # workflow runs with status

# System
trs env                     # grouped, filtered, PATH summarized
trs wc src/*.rs             # compact: file, lines, words, bytes

# HTTP
trs curl -I https://api.com    # compact headers
trs curl -v https://api.com    # verbose → compact
trs wget https://file.com      # strip progress bars

# Any command — generic compression (collapse whitespace, strip ANSI)
trs ollama list               # 39% reduction from whitespace collapse
trs kubectl get pods          # tabular output compacted automatically
trs echo "hello"              # short output passes through unchanged
```

## Built-in Tools

```bash
# JSON structure (without values)
echo '{"users":[...]}' | trs json         # shows keys + types + array lengths
trs json --depth 2                        # limit depth
cat data.json | trs json --json           # schema as JSON

# File reader with filter levels
trs read src/main.rs                      # raw with line numbers
trs read src/main.rs -l minimal           # strip comments, normalize blanks
trs read src/main.rs -l aggressive        # signatures-only (imports + fn defs)

# Search (ripgrep-powered)
trs search src "TODO" --extension rs
trs search . "error" -i --context 2

# Replace
trs replace src "old" "new" --dry-run     # preview
trs replace src "old" "new"               # execute
trs replace . "TODO" "DONE" --count       # count only

# Error filter (any command)
trs err cargo build                       # only errors/warnings
trs err npm install                       # only problems

# Tail
trs tail app.log -n 20                    # last N lines
trs tail app.log --errors                 # errors only
trs tail app.log --follow                 # streaming

# Clean text
some-command | trs clean --no-ansi --collapse-blanks --trim

# Convert
trs html2md https://example.com           # HTML to Markdown
cat notes.txt | trs txt2md                # text to Markdown

# Git state
trs is-clean                              # exit 0=clean, 1=dirty
trs is-clean --json                       # {"is_clean": true}

# Raw execution (no filtering, but tracked in stats)
trs raw gh api /repos/user/repo           # full JSON output, tracked
trs raw kubectl get pods -o json          # bypass compression

# Token savings tracker
trs stats                                 # cumulative savings
trs stats --history                       # recent commands
trs stats --project                       # current project only
```

## Output Formats

Every command supports 6 formats. Flags work before or after the command:

```bash
trs git status                # compact (default)
trs git status --json         # structured JSON
trs git status --csv          # CSV with headers
trs git status --tsv          # tab-separated
trs git status --agent        # AI-optimized text
trs git status --raw          # unprocessed passthrough
trs git status --stats        # show reduction metrics
```

## Pipe Syntax

Also works with pipes for any parser:

```bash
git status | trs parse git-status
git diff   | trs parse git-diff
ls -la     | trs parse ls
cat app.log | trs parse logs
pytest | trs parse test --runner pytest
cargo test 2>&1 | trs parse cargo-test
cargo clippy 2>&1 | trs parse lint
```

## Benchmarks

Across 18 tests vs [rtk](https://github.com/rtk-ai/rtk): **trs 13 wins, rtk 4 wins, 1 tie**.

| Command | Raw | trs | Reduction |
|---------|-----|-----|-----------|
| `cargo test` | 55 KB | 58 B | **99%** |
| `trs read -l aggressive` | 4.7 KB | 295 B | **93%** |
| `cargo clippy` | 55 KB | 5.5 KB | **89%** |
| `git status` | 13.6 KB | 876 B | **93%** |
| `git log -10` | 8.5 KB | 842 B | **90%** |
| `git branch -a` | 65 B | 6 B | **90%** |
| `ls -la` | 1.4 KB | 227 B | **83%** |
| `env` | 3.0 KB | 807 B | **73%** |
| `gh run list` | 618 B | 202 B | **67%** |
| `cargo build` | 71 B | 32 B | **54%** |
| `find *.rs` | 3.9 KB | 2.1 KB | **46%** |
| `curl -I` | 201 B | 115 B | **42%** |
| `ollama list` | 1.0 KB | 626 B | **39%** |
| `trs json` | 308 B | 210 B | **31%** |

Commands without a dedicated parser still get **generic compression** (whitespace collapse, ANSI stripping) — typically 20-40% reduction on tabular output.

Run the benchmark: `./scripts/benchmark.sh`

## Configuration

Optional config at `~/.trs/config.toml` (or `.trs/config.toml` per-project):

```toml
[limits]
grep_max_results = 200
grep_max_per_file = 25
status_max_files = 15
passthrough_max_chars = 2000
json_max_depth = 10
```

## Safety

- **Smart passthrough**: `--json`, `--porcelain`, `--format=json` flags skip parsing entirely
- **3-tier fallback**: parser OK → degraded → truncated passthrough with `[trs:passthrough]`
- **Generic fallback**: commands without a parser still get whitespace/ANSI compression
- **Exit code propagation**: always preserves the wrapped command's exit code
- **Tee system**: on failure, saves full output to `~/.trs/tee/` for recovery
- **Data protection**: JSON/YAML/TOML/XML files never stripped in `trs read`

## For AI Agents

`trs` is the output layer between system commands and LLMs:

```bash
trs git status --json
# → {"branch":"main","is_clean":false,"staged_count":2,...}

trs search src "handleError" --extension ts --json
# → {"files":[{"path":"src/handler.ts","matches":[...]}],...}

trs is-clean --json
# → {"is_clean":true}
```

## Tech Stack

- **Language**: Rust
- **Binary**: ~7 MB, self-contained, no runtime dependencies
- **CLI**: clap 4
- **Search**: ripgrep (grep crate)
- **HTML**: htmd
- **HTTP**: ureq
- **Tests**: 2,012 passing, 0 failures
- **Architecture**: 202 files, max 506 lines per file

## License

MIT
