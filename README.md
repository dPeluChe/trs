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
trs cargo test / go test / pytest / jest / vitest / npm test / pnpm test / bun test

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

# Fast find (explicit gitignore-aware walker)
trs find --gitignore . -name "*.rs"   # 5ms vs 280ms raw find
trs find . -name "*.rs"               # executes real find (always honest)

# Benchmark
trs benchmark git status              # compression metrics per command
trs benchmark cargo test --repeat 5   # averaged over N runs
trs benchmark ls -la --json           # machine-readable output

# Project digest (LLM-ready codebase index)
trs ingest                            # digest current project (minimal)
trs ingest --budget 32k               # fit to token budget
trs ingest --changed                  # only uncommitted files
trs ingest --since HEAD~5             # last 5 commits
trs ingest -l aggressive              # signatures only
trs ingest --ollama auto              # LLM-formatted summary
trs ingest --deps                     # dependency graph only (no file content)
trs ingest --since-last               # only files changed since last ingest
trs ingest --fresh                    # reuse cached digest if HEAD unchanged
trs ingest --list                     # show saved digests with HEAD sha + stale info
trs ingest --read myproject           # read a saved digest

# Utilities
trs is-clean                          # exit 0=clean, 1=dirty
trs raw gh api /repos/user/repo       # no compression, tracked in stats
trs stats --history                   # token savings dashboard

# Hook installer for AI tools
trs init claude                       # install Claude Code hook
trs init gemini --global              # install Gemini CLI hook globally
trs init cursor                       # install Cursor hook
trs init codex                        # append trs instructions to AGENTS.md
trs init opencode                     # install OpenCode plugin
trs init kilo                         # install Kilo Code plugin
trs init antigravity                  # install Google Antigravity hook
trs init --all --global               # install all hooks globally
trs init --show                       # show which tools are configured

# Discover missed savings (scan Claude Code history)
trs discover                          # current project, last 7 days
trs discover --all --since 30         # all projects, last 30 days
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

### Compression (18 synthetic tests)

vs [rtk](https://github.com/rtk-ai/rtk) 0.35.0 and [token-saver](https://github.com/nicobailey/token-saver) 2.2.1:

| Command | Raw | trs | rtk | Winner |
|---------|-----|-----|-----|--------|
| `cargo test` | 8.1 KB | 58 B | 58 B | tie |
| `git status` | 1.4 KB | 336 B | 599 B | trs |
| `git log -10` | 7.6 KB | 689 B | 2.8 KB | trs |
| `git diff` | 14.6 KB | 8.0 KB | 12.1 KB | trs |
| `ls -la` | 1.4 KB | 270 B | 257 B | rtk |
| `env` | 2.9 KB | 728 B | 1.1 KB | trs |
| `find *.rs` | 4.4 KB | 2.4 KB | 760 B | rtk |
| `curl -I` | 201 B | 115 B | 192 B | trs |
| `gh pr list` | 560 B | 348 B | 384 B | trs |

**Score: trs 13 / rtk 5 / token-saver 0** across 18 tests.

### Speed (local commands, deterministic)

| | trs | rtk | token-saver |
|---|---|---|---|
| Overhead vs raw | **27%** | 45% | 274% |
| Speed wins | **12** | 2 | 0 |
| Startup (`--version`) | **3.2ms** | 4.5ms | ~55ms |

### Real-world project (labs-mundialito, 11 modified files, TypeScript + Convex)

| | trs | rtk |
|---|---|---|
| Speed wins | **13/15** | 2/15 |
| Compression wins | **11/15** | 4/15 |
| Total time | **302ms** | 460ms |
| Total bytes | **303 KB** | 387 KB |

Note: rtk silently replaces `find` with an internal walker (undocumented), which makes it appear faster on file searches. trs always executes the real command. Use `trs find --gitignore` for an explicit fast walker that respects `.gitignore`.

Run it yourself: `./docs/benchmarks/benchmark.sh` or `./docs/benchmarks/benchmark-real.sh [project-path]`

### Ingest quality (vs repomix)

`trs ingest` produces LLM-ready project digests. We benchmarked quality by sending both trs and [repomix](https://github.com/yamadashy/repomix) digests to Ollama (gemma4) and scoring what the model understood:

| | trs ingest | repomix (truncated) | repomix (full) |
|---|---|---|---|
| Mundialito (React+Convex) | **19/20** | 7/20 | 1/20 |
| Spark (Rust TUI) | 13/20 | **14/20** | 0/20 |
| gstack (TS CLI) | **11/20** | **11/20** | n/a |
| Input size | **16-58 KB** | 24 KB (truncated) | 665-813 KB |
| Hallucinations | **0** | 0 | Yes ("Next.js", "JWT") |

trs achieves equal or better quality with 50-113x fewer tokens. repomix full (raw concatenation) consistently causes models to hallucinate or produce generic responses.

Tested on 9 repos: Rust, TypeScript, Python, React+Convex, Tauri, Go, docs/skills collections.

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

- `--no-verify` blocked on git commit/push (protects pre-commit hooks from agents)
- Commands with `--json` / `--porcelain` flags pass through untouched
- If a parser fails, output falls back to truncated passthrough (never silent failure)
- Exit codes always propagated from the wrapped command
- On failure, full output saved to `~/.trs/tee/` for recovery
- `trs read` never strips content from JSON/YAML/TOML/XML data files

## Tech stack

| | |
|---|---|
| Language | Rust |
| Binary | ~5.9 MB (LTO + strip), no runtime dependencies |
| CLI | clap 4 (bypassed on hot path for ~3ms overhead) |
| Search | ripgrep (grep crate) |
| Tests | 2,104 passing, 0 warnings |
| Architecture | 210+ files, <500 lines each — [details](AGENTS.md) |

## Contributing

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs
cargo test                     # 2,104 tests must pass
cargo clippy -- -D warnings    # no warnings allowed
cargo fmt -- --check           # formatting must match
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for code guidelines, [AGENTS.md](AGENTS.md) for the architecture, and [docs/TASK_TODO.md](docs/TASK_TODO.md) for the roadmap.

## Acknowledgments

This project wouldn't exist without the work of others in the token-reduction space. These are all interesting projects worth exploring:

- [rtk](https://github.com/rtk-ai/rtk) — the project that sparked this one. Their approach to token reduction for AI agents showed me the problem was worth solving, and studying their Rust CLI architecture taught me a lot. We benchmark against rtk regularly to keep both projects honest and push each other forward.
- [token-saver](https://github.com/nicobailey/token-saver) — a Python-based alternative with a different design philosophy (wrap.py pipeline). Comparing against it helped us understand the tradeoffs between native binaries and scripted approaches, especially around startup time.
- [caveman](https://github.com/JuliusBrussee/caveman) — a complementary approach: instead of compressing terminal output, caveman reduces AI *response* tokens by making Claude speak concisely. Different layer, same goal (token efficiency). Their benchmark methodology and validation pipeline are well worth studying.
- [repomix](https://github.com/yamadashy/repomix) — the most popular codebase-to-markdown tool (~22k stars). Studying it helped us design `trs ingest` with a different philosophy: intelligent extraction instead of raw concatenation. Their tree-sitter compression feature is worth exploring.
- [claw-compactor](https://github.com/open-compress/claw-compactor) — compression patterns (LogCrunch, DiffCrunch, Ionizer) that influenced our log/diff/json handlers.
- [tokf](https://github.com/mpecan/tokf) — TOML filter pipeline concept.

## License

MIT
