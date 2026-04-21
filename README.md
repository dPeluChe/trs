<p align="center">
  <strong>trs</strong> — terminal compression for AI agents
</p>

<p align="center">
  <a href="https://dpeluche.github.io/trs/"><strong>dpeluche.github.io/trs</strong></a> ·
  <a href="https://github.com/dPeluChe/trs">GitHub</a> ·
  <a href="https://www.npmjs.com/package/@dpeluche/trs">npm</a> ·
  <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="https://github.com/dPeluChe/trs/actions"><img src="https://github.com/dPeluChe/trs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/dPeluChe/trs/releases"><img src="https://img.shields.io/github/v/release/dPeluChe/trs" alt="Release"></a>
  <a href="https://www.npmjs.com/package/@dpeluche/trs"><img src="https://img.shields.io/npm/v/@dpeluche/trs" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="#install-recommended">Install</a> ·
  <a href="#quick-start-tldr">Quick start</a> ·
  <a href="#what-it-does">What it does</a> ·
  <a href="#project-digest">Project digest</a> ·
  <a href="#from-source-development">From source</a>
</p>

---

## Why

Token pricing kept climbing. Every `git status`, `cargo test`, and `ls -la` the agent rendered into its context cost real money, and the signal-to-noise ratio on those commands was painfully low. We started writing small tools — first for ourselves, then for the team — to reduce what the agent actually had to read.

Along the way we came across [**rtk**](https://github.com/rtk-ai/rtk) (Rust Token Killer). By then our tools had been evolving on their own, so we faced the honest choice: migrate to rtk and drop what we'd built, or continue and publish our own take. We chose to continue — more options in this space means a better fit for more workflows. trs kept iterating and expanding as we learned more about where tokens actually burn.

The more we used it, the more we saw the opportunity was bigger than input hooks. `trs output-saver` installs rules into each agent's global config so replies come back shorter too. `trs audit-docs` inspects CLAUDE.md / AGENTS.md for the bloat every session re-loads. `trs ingest` compresses whole repos into a budget-aware, LLM-ready context index. Still a single static binary with zero runtime deps — the story just got bigger than hooks.

The landing page has the full write-up: <https://dpeluche.github.io/trs/>

## What it does

Prefix any command with `trs` (or let `trs init` wire it into your AI tool for you). The binary spawns your command, parses the output, and emits a compact version built for both humans and LLMs.

```bash
$ trs git status
main [ahead 1]
unstaged (3):
  M src/main.rs
  M src/lib.rs
  A src/new.rs
# 1.4 KB → 336 B (76% reduction)

$ trs cargo test
cargo test: 2186 passed (71 suites, 4.9s)
# 55 KB → 58 B (99% reduction)

$ trs cargo clippy
lint: 102 issues in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  W dead_code 112:8
# 55 KB → 5.5 KB (90% reduction)
```

Commands without a dedicated parser still get generic compression (whitespace collapse, ANSI stripping) — ~30-40% for free.

## Install (recommended)

Platform support: **macOS (arm64/x64), Linux (arm64/x64), Windows (x64)**.
Single static binary, zero runtime deps, ~12ms startup.

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.ps1 | iex
```

### npm (all platforms)

```bash
npm install -g @dpeluche/trs
```

### cargo (builds from source — requires Rust)

```bash
cargo install tars-cli
```

### Prebuilt binaries

[GitHub Releases](https://github.com/dPeluChe/trs/releases) — Linux x64/arm64,
macOS x64/arm64, Windows x64. All methods ship the same native binary (~6 MB).

### Pin a specific version

```bash
TRS_VERSION=v0.5.9 curl -fsSL https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.sh | sh
```

### Upgrading

```bash
trs upgrade --check    # show what would run
trs upgrade            # auto-detects channel (npm / curl|sh), refreshes hooks too
```

## Quick start (TL;DR)

```bash
# 1. Try it — prefix any command with trs
trs git status
trs cargo test

# 2. Let your AI agent do it automatically — wires hooks into Claude /
#    Gemini / Cursor / OpenCode / Kilo / Codex / Droid / Windsurf / Antigravity
trs init --all --global

# 3. See your savings
trs stats                          # dashboard
trs stats --by-agent               # breakdown per AI agent
```

Flags work anywhere and stdin is supported too:

```bash
trs git status --json              # structured JSON
trs --json git status              # flags work anywhere
git status | trs parse git-status  # pipe syntax too
```

Full command reference below, or see [`docs/commands/`](docs/commands/) for
per-command deep-dives.

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

# Containers & GitHub CLI
trs docker ps / logs   ·   trs gh pr/issue/run list

# System
trs env / wc / curl -I / wget
```

## Built-in tools (not just wrappers)

```bash
# JSON query (jq-lite, no dependency)
curl -s api.com/users | trs json                    # structure
curl -s api.com/users | trs json -q '.users[].name' # query

# Intelligent file reader
trs read src/main.rs -l aggressive    # signatures only
trs read src/main.rs -l minimal       # strip comments, keep code

# Search & replace (ripgrep powered)
trs search src "TODO" --extension rs
trs replace src "old_fn" "new_fn" --dry-run

# Error filter
trs err cargo build                   # only errors/warnings

# Text processing
trs tail app.log --errors
trs clean --no-ansi --collapse-blanks
trs html2md https://example.com

# Fast find (gitignore-aware walker)
trs find --gitignore . -name "*.rs"

# Utilities
trs is-clean
trs raw gh api /repos/user/repo       # passthrough, tracked in stats
trs stats --history                   # savings dashboard
```

## Project digest

```bash
trs ingest                     # writes digest, stdout = path
trs ingest --budget 128k       # fit to token budget (signatures first)
trs ingest --deps              # dependency graph only, no content
trs ingest --changed           # only files with uncommitted changes
trs ingest --since-last        # only files changed since last ingest
trs ingest --fresh             # reuse cached digest if HEAD unchanged
trs ingest -o ~/ctx.md         # write to a custom path, no shadow save
trs ingest --print             # emit content to stdout (default: path)
trs ingest --warn-at 40k       # stderr warning if digest exceeds N tokens
trs ingest --list              # saved digests + HEAD sha + stale markers
trs ingest --read myproject    # read a saved digest
```

Everything about `trs ingest` — stale detection, dependency graphs, Ollama post-processing, budget-aware truncation — is documented on the [landing page](https://dpeluche.github.io/trs/#digest).

## AI tool hooks (`trs init`)

`trs init --all` installs hooks for every detected tool. Programmatic hooks for Claude Code, Gemini CLI, Cursor, OpenCode, Kilo, Factory Droid; prompt-level rules files for Codex, Google Antigravity, and Windsurf. The installer smart-merges into existing settings.json so your other config is preserved.

```bash
trs init --show           # status of all integrations
trs init --all --global   # install everything it detects
trs init claude           # or pick one
trs init claude --replace # cut over from an existing compressor hook
```

Before writing, `trs init` runs a pre-install collision check: it scans
target configs (following `@imports` for Claude/Gemini) for existing
rtk or token-optimizer hooks and aborts by default. `--replace` clears
the previous compressor's hook cleanly before installing trs; `--force`
installs alongside (risky — double-compression).

## Output saver (`trs output-saver`)

trs compresses what agents **see** via `trs rewrite`. `trs output-saver`
closes the symmetric gap — it installs a compact rules block into each
agent's global config to compress what agents **emit**: no preambles,
no narration, result-first, structured output where appropriate, no
hallucinated paths.

```bash
trs output-saver            # read-only scan of all detected agents
trs output-saver --install  # write the block where the scan was clean
trs output-saver --print    # dump the raw block (pipe-friendly)
trs output-saver --remove   # clean uninstall
```

Eight of nine agents are covered (Antigravity is per-project only —
use `trs init antigravity`). Claude/Gemini get a standalone file plus
`@import`; Cursor gets an auto-loaded `.mdc`; Codex/Windsurf/OpenCode/
Kilo/Droid get an inline block wrapped in HTML-comment sentinels so
re-installs are idempotent.

## Output formats

Every command supports 6 output formats:

```bash
trs git status                # compact (default)
trs git status --json         # structured JSON
trs git status --csv          # CSV with headers
trs git status --tsv          # tab-separated
trs git status --agent        # AI-optimized markdown
trs git status --raw          # unprocessed passthrough
```

## Features

- **30+ dedicated parsers** — git, cargo, go, npm, pnpm, docker, gh, pytest, jest, vitest, eslint, ruff, biome, golangci-lint, and more.
- **Chain-aware rewrite** — `cd X && git status` or `cargo fmt && cargo clippy` get each rewritable segment wrapped with trs; pipes and semicolons pass through.
- **9 AI tool integrations** — Claude, Gemini, Cursor, OpenCode, Kilo, Droid (programmatic) + Codex, Antigravity, Windsurf (rules files).
- **JSON query engine** — built-in jq-lite, no dependency on `jq`.
- **Token savings dashboard** — `trs stats` shows cumulative compression and tokens saved per day. `trs stats --by-agent` breaks the totals down by which AI agent fired each rewrite (Claude / Gemini / Cursor / OpenCode / Kilo).
- **Generic compression fallback** — commands without a parser still get ANSI stripping, whitespace collapse, repeated-line dedup.

## Configuration

Optional — trs works without config. For tuning:

```toml
# ~/.trs/config.toml (or .trs/config.toml per project)
[limits]
grep_max_results = 200
status_max_files = 15
passthrough_max_chars = 2000
json_max_depth = 10
```

## How it stays safe

- `--no-verify` blocked on `git commit`/`git push` (protects pre-commit hooks from agents).
- Commands with `--json` / `--porcelain` flags pass through untouched.
- If a parser fails, output falls back to truncated passthrough — never silent failure.
- Exit codes always propagated from the wrapped command.
- On failure, full output saved to `~/.trs/tee/` for recovery.
- `trs read` never strips content from JSON/YAML/TOML/XML data files.

## Tech stack

| | |
|---|---|
| Language | Rust |
| Binary | ~6 MB (LTO + strip), no runtime deps |
| Startup | ~12ms on macOS / Linux (native binary or shell launcher) |
| CLI | clap 4 (bypassed on hot path) |
| Tests | 2,186 passing, 0 warnings |
| Architecture | 200+ modular files across parsers, handlers, and integrations — [details](AGENTS.md) |

## From source (development)

Prefer the prebuilt install paths above unless you're contributing. For a
source checkout:

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs

# Build + install into ~/.cargo/bin/
cargo install --path .

# Dev loop
cargo test                     # 2,186 tests across 71 suites
cargo clippy -- -D warnings    # no warnings allowed
cargo fmt -- --check           # formatting must match
cargo run -- git status        # run locally against the workspace
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for code guidelines,
[AGENTS.md](AGENTS.md) for architecture, and
[docs/TASK_TODO.md](docs/TASK_TODO.md) for the roadmap.

## License

MIT

---

<p align="center">
  A product by <a href="https://iteris.tech"><strong>Iteris</strong></a> · Published and maintained by <a href="https://dpeluche.dev"><strong>@dPeluChe</strong></a>
</p>
