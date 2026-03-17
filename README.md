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

# Raw git status: 497 bytes → trs: 81 bytes (80% reduction)
```

```
$ trs git log -5
<Antonio>
  4ca8fe4 Optimize parsers: trs now wins benchmarks vs rtk (12 min ago)
  b49dda3 Fix all tests and eliminate all warnings (21 min ago)
  581541e Fix 3 replace bugs (28 min ago)

# Raw git log: 7.2 KB → trs: 690 bytes (90% reduction)
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
trs git diff                # files changed with +adds/-dels
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
trs pytest                  # passed/failed/skipped summary
trs npm test                # works with npm, pnpm, yarn, bun
trs jest                    # jest, vitest supported

# Package managers
trs npm ls                  # clean dependency list
trs pip list                # name@version format
trs npm install             # summary: added, warnings, errors

# Build tools
trs cargo build             # errors + warnings only
trs tsc                     # TypeScript build errors
trs make                    # build output summary

# Docker
trs docker ps               # container status (up/down)
trs docker logs <name>      # log level detection

# System
trs env                     # grouped, filtered, PATH summarized

# Unknown commands pass through unchanged
trs echo "hello"            # just prints "hello"
```

## Built-in Tools

```bash
# Search (ripgrep-powered)
trs search src "TODO" --extension rs
trs search . "error" -i --context 2

# Replace
trs replace src "old" "new" --dry-run    # preview
trs replace src "old" "new"              # execute
trs replace . "TODO" "DONE" --count      # count only

# Tail
trs tail app.log -n 20                   # last N lines
trs tail app.log --errors                # errors only
trs tail app.log --follow                # streaming

# Clean text
some-command | trs clean --no-ansi --collapse-blanks --trim

# Convert
trs html2md https://example.com          # HTML to Markdown
cat notes.txt | trs txt2md               # text to Markdown

# Git state
trs is-clean                             # exit 0=clean, 1=dirty
trs is-clean --json                      # {"is_clean": true}
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
```

## Benchmarks

| Command | Raw | trs | Reduction |
|---------|-----|-----|-----------|
| `git status` | 497 B | 81 B | **80%** |
| `git log -10` | 7.2 KB | 690 B | **90%** |
| `git diff --stat` | 723 B | 452 B | **37%** |
| `git branch -a` | 66 B | 7 B | **89%** |
| `ls -la` | 1.4 KB | 239 B | **82%** |
| `env` | 3.6 KB | 1.1 KB | **68%** |
| `find *.rs` | 2.2 KB | 1.2 KB | **48%** |

Run the benchmark: `./scripts/benchmark.sh`

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
- **Tests**: 2,399 passing, 0 warnings

## License

MIT
