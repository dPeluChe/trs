<p align="center">
  <strong>trs</strong> — compact terminal output for humans and AI agents
</p>

<p align="center">
  <a href="https://github.com/dPeluChe/trs/actions"><img src="https://github.com/dPeluChe/trs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/dPeluChe/trs/releases"><img src="https://img.shields.io/github/v/release/dPeluChe/trs" alt="Release"></a>
  <a href="https://www.npmjs.com/package/tars-cli"><img src="https://img.shields.io/npm/v/tars-cli" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="#install">Install</a> •
  <a href="#what-it-does">What it does</a> •
  <a href="#benchmarks">Benchmarks</a> •
  <a href="CONTRIBUTING.md">Contributing</a> •
  <a href="AGENTS.md">Architecture</a> •
  <a href="docs/TASK_TODO.md">Roadmap</a> •
  <a href="README.es.md">Español</a>
</p>

---

## Origin Story

trs started as a learning project. While exploring how tools like [rtk](https://github.com/rtk-ai/rtk) compress terminal output for AI agents, I wanted to understand the problem deeply — not just use a solution, but build one from scratch in Rust.

What began as "let me see if I can replicate this" quickly became a daily driver. The process of building each parser taught me what actually matters for token reduction, and along the way trs grew its own features: a JSON query engine, a lint parser, 6 output formats, built-in search/replace, and a generic compression fallback that works on any command.

This is now the tool I use every day with Claude Code. I'm sharing it in case it's useful to others or as a reference for anyone who wants to learn how terminal output compression works.

## What it does

Prefix any command with `trs` to get compact output:

```bash
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs
# 497 bytes → 81 bytes

$ trs cargo test
cargo test: ok (2012 passed, 0 failed, 70 suites, 3.21s)
# 55 KB → 58 bytes

$ trs cargo clippy
lint: 102 (102 warnings) in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  ...
# 55 KB → 5.5 KB
```

Commands without a dedicated parser still get basic compression (whitespace collapse, ANSI stripping) — so `trs ollama list` or `trs kubectl get pods` gives you ~30-40% reduction for free.

## Install

```bash
# npm (downloads precompiled binary)
npm install -g tars-cli

# Try without installing
npx tars-cli git status

# From source
cargo install --path .

# Pre-built binaries: https://github.com/dPeluChe/trs/releases
```

## Quick Start

```bash
trs git status                 # compact output
trs git status --json          # structured JSON
trs --json git status          # flags work anywhere
git status | trs parse git-status  # pipe syntax too
```

## Commands with dedicated parsers

```bash
# Git
trs git status / diff / log / branch / push / pull / fetch

# Linters (grouped by file + rule)
trs cargo clippy / eslint / ruff / biome / golangci-lint

# Test runners
trs cargo test / pytest / jest / vitest / npm test / pnpm test / bun test

# Files & search
trs ls -la / find / grep / tree

# Build & packages
trs cargo build / npm install / pip list

# Docker & GitHub CLI
trs docker ps / logs   |   trs gh pr/issue/run list

# System
trs env / wc / curl -I / wget
```

## Built-in tools (not just wrappers)

These are features trs has that go beyond output compression:

```bash
# JSON query (jq-lite, no dependency)
curl -s api.com/users | trs json                    # show structure
curl -s api.com/users | trs json -q '.users[].name' # extract values
curl -s api.com/users | trs json -q '.meta.total'   # nested paths

# File reader with intelligence
trs read src/main.rs -l aggressive    # signatures only (93% reduction)
trs read src/main.rs -l minimal       # strip comments, keep code

# Search & replace (ripgrep powered)
trs search src "TODO" --extension rs
trs replace src "old_fn" "new_fn" --dry-run

# Error filter (works with any command)
trs err cargo build                   # show only errors/warnings

# Text processing
trs tail app.log --errors             # only error lines
trs clean --no-ansi --collapse-blanks # clean piped text
trs html2md https://example.com       # HTML → Markdown

# Utilities
trs is-clean                          # exit 0=clean, 1=dirty
trs raw gh api /repos/user/repo       # no compression, tracked in stats
trs stats --history                   # token savings dashboard
```

## Output formats

Every command supports 6 output formats:

```bash
trs git status                # compact (default)
trs git status --json         # structured JSON
trs git status --csv          # CSV with headers
trs git status --tsv          # tab-separated
trs git status --agent        # AI-optimized
trs git status --raw          # unprocessed passthrough
```

## Benchmarks

vs [rtk](https://github.com/rtk-ai/rtk) (the tool that inspired this project):

| Command | Raw | trs | rtk | Winner |
|---------|-----|-----|-----|--------|
| `cargo test` | 55 KB | 58 B | 62 B | trs |
| `cargo clippy` | 55 KB | 5.5 KB | — | trs |
| `git status` | 13.6 KB | 876 B | 343 B | rtk |
| `git log -10` | 8.5 KB | 842 B | 811 B | rtk |
| `ls -la` | 1.4 KB | 227 B | 291 B | trs |
| `env` | 3.0 KB | 807 B | 1.1 KB | trs |
| `gh run list` | 618 B | 202 B | 240 B | trs |
| `find *.rs` | 3.9 KB | 2.1 KB | 3.9 KB | trs |
| `curl -I` | 201 B | 115 B | 192 B | trs |

**Score: trs 13 wins, rtk 4 wins, 1 tie** across 18 tests.

**Speed**: trs adds ~3ms overhead, rtk ~7ms.

Where rtk wins: git status/log with very aggressive truncation. Where trs wins: most other commands, plus features rtk doesn't have (json query, lint parser, 6 output formats, search/replace).

Run it yourself: `./scripts/benchmark.sh`

## Configuration

Optional — trs works without config. For tuning:

```toml
# ~/.trs/config.toml (or .trs/config.toml per-project)
[limits]
grep_max_results = 200
status_max_files = 15
passthrough_max_chars = 2000
json_max_depth = 10
```

## How it stays safe

- Commands with `--json` / `--porcelain` flags pass through untouched
- If a parser fails, output falls back to truncated passthrough (never silent failure)
- Exit codes always propagated from the wrapped command
- On failure, full output saved to `~/.trs/tee/` for recovery
- `trs read` never strips content from JSON/YAML/TOML/XML data files

## Tech stack

| | |
|---|---|
| Language | Rust |
| Binary | ~7 MB, no runtime dependencies |
| CLI | clap 4 |
| Search | ripgrep (grep crate) |
| Tests | 2,039 passing, 0 warnings |
| Architecture | 210+ files, <500 lines each — [details](AGENTS.md) |

## Contributing

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs
cargo test                     # 2,039 tests must pass
cargo clippy -- -D warnings    # no warnings allowed
cargo fmt -- --check           # formatting must match
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for code guidelines, [AGENTS.md](AGENTS.md) for the architecture, and [docs/TASK_TODO.md](docs/TASK_TODO.md) for the roadmap.

## Acknowledgments

- [rtk](https://github.com/rtk-ai/rtk) — the project that sparked this one. Their approach to token reduction for AI agents showed me the problem was worth solving, and studying their codebase taught me a lot about CLI design in Rust.
- [claw-compactor](https://github.com/open-compress/claw-compactor) — compression patterns (LogCrunch, DiffCrunch, Ionizer) that influenced our log/diff/json handlers.
- [tokf](https://github.com/mpecan/tokf) — TOML filter pipeline concept.

## License

MIT
