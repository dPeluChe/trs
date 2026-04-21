<p align="center">
  <strong>trs</strong> — <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · terminal compression for AI agents
</p>

<p align="center">
  <a href="https://usetrs.dev"><strong>usetrs.dev</strong></a> ·
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
  <a href="#what-is-trs">What</a> ·
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#supported-ai-agents">Agents</a> ·
  <a href="#supported-commands">Commands</a> ·
  <a href="#built-in-trs-tools">Built-in</a> ·
  <a href="#project-digest">Digest</a> ·
  <a href="#why">Why</a>
</p>

---

## What is trs

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

Commands without a dedicated parser still get generic compression (whitespace collapse, ANSI stripping) — ~30–40% for free.

## Install

Single static binary, zero runtime deps, ~12 ms startup. Platforms: **macOS (arm64/x64), Linux (arm64/x64), Windows (x64)**.

```bash
# macOS / Linux
curl -fsSL https://usetrs.dev/install.sh | sh

# Windows (PowerShell)
irm https://usetrs.dev/install.ps1 | iex

# npm (all platforms)
npm install -g @dpeluche/trs

# cargo (builds from source)
cargo install trs-cli
```

Full install options — prebuilt binaries, version pinning, custom install dirs, troubleshooting: [`docs/support/install.md`](docs/support/install.md).

### Upgrading

```bash
trs upgrade --check    # show what would run (auto-detects channel)
trs upgrade            # upgrade binary + refresh hooks
```

See [`docs/features/upgrade.md`](docs/features/upgrade.md).

## Quick start

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
trs stats -n 30                    # custom row limit
```

Flags work anywhere and stdin is supported too:

```bash
trs git status --json              # structured JSON
trs --json git status              # flags work anywhere
git status | trs parse git-status  # pipe syntax too
```

## Supported AI agents

Nine agents supported end-to-end. Programmatic hook for Claude Code, Gemini CLI, Cursor, OpenCode, Kilo Code, Factory Droid. Rules file only for Codex CLI, Google Antigravity, Windsurf.

| Agent | Install method | Input hook | Output-saver | Attribution |
|---|---|---|---|---|
| Claude Code · Gemini · Cursor | programmatic hook | ✓ | ✓ | `claude` / `gemini` / `cursor` |
| OpenCode · Kilo Code | plugin template | ✓ | ✓ | `opencode` / `kilo` |
| Factory Droid | programmatic hook | ✓ | ✓ | `claude` (shares envelope) |
| Codex CLI · Windsurf | rules file only | — | ✓ | `(untagged)` |
| Google Antigravity | rules file only | — | — | `(untagged)` |

Full compatibility matrix + caveats + per-agent config paths: [`docs/support/agents.md`](docs/support/agents.md).

## Supported commands

| Category | Tools with dedicated parsers |
|---|---|
| VCS | `git` (status, diff, log, branch, push, pull, fetch) |
| Rust | `cargo` (build, check, test, clippy, fmt, install) |
| JS/TS | `npm`, `pnpm`, `yarn`, `bun`, `npx`, `pnpm dlx` |
| Python | `pytest`, `pip`, `uv`, `python3 -m <mod>` routing |
| Go | `go` (test, build, mod) |
| Tests | `pytest`, `jest`, `vitest` (full runner parsing) |
| Linters | `eslint`, `biome`, `ruff`, `pylint`, `golangci-lint`, `cargo clippy` |
| Files | `ls` (+ `eza`, `lsd`, `exa`), `find` (+ `fd`), `grep` (+ `rg`, `ag`, `ack`), `tree`, `tail` |
| Containers | `docker` (ps, logs, build) |
| GitHub | `gh` (pr/issue/run list + `gh api`) |
| System | `ps`, `env`, `wc`, `brew`, `curl`, `wget` |

Plus **chain-aware rewrite** (`cd X && cargo test`), **env-prefix preservation** (`RUSTFLAGS=x cargo build`), **pipe syntax** (`cmd | trs parse …`), and `TRS_SKIP=1` to bypass any wrapping.

Full command reference with subcommands and examples: [`docs/support/commands.md`](docs/support/commands.md).

## Built-in trs tools

Native trs commands — no external binary behind them, all shipped in the single binary.

```bash
trs json              # jq-lite query engine (-q '.users[].name')
trs read              # file reader (-l minimal / -l aggressive)
trs search            # ripgrep-powered content search
trs replace           # ripgrep-powered replace (--dry-run)
trs err               # error filter (only errors/warnings)
trs tail              # log tail with --errors
trs clean             # ANSI / whitespace / dedup cleanup
trs html2md           # HTML → Markdown
trs find              # gitignore-aware walker
trs is-clean          # repo clean check (exit code)
trs raw               # passthrough, still tracked in stats
trs stats             # savings dashboard
trs debug-info        # bundle version + doctor + logs for bug reports
```

## Project digest

`trs ingest` walks a repo and emits a compact Markdown digest — structure + key files + signatures — ready to paste into any AI agent's context. Budget-aware, staleness-aware, incremental.

```bash
trs ingest                     # writes digest, prints path
trs ingest --budget 128k       # fit to token budget (signatures first)
trs ingest --changed           # only uncommitted files
trs ingest --since-last        # incremental since last ingest
trs ingest --deps              # dependency graph only
trs ingest --fresh             # reuse cached digest if HEAD unchanged
trs ingest --list              # saved digests + HEAD sha + stale markers
```

Full reference: [`docs/features/ingest.md`](docs/features/ingest.md). A live example — `trs ingest` run against this repo — ships at [`docs/development/codebase-digest.md`](docs/development/codebase-digest.md).

## Output saver

trs compresses what agents **see** via `trs rewrite`. `trs output-saver` closes the symmetric gap — installs a compact rules block into each agent's global config to compress what agents **emit**: no preambles, no narration, result-first, structured output where appropriate, no hallucinated paths.

```bash
trs output-saver               # read-only scan
trs output-saver --install     # install on detected agents
trs output-saver --remove      # clean uninstall
```

Eight of nine agents supported (Antigravity is per-project only). Full reference: [`docs/features/output-saver.md`](docs/features/output-saver.md).

## Output formats

Every command supports six output formats:

```bash
trs git status                 # compact (default, humans + agents)
trs git status --json          # structured JSON
trs git status --csv           # CSV with headers
trs git status --tsv           # tab-separated
trs git status --agent         # AI-optimized markdown
trs git status --raw           # unprocessed passthrough
```

Full format reference with side-by-side examples: [`docs/features/formats.md`](docs/features/formats.md).

## Features

- **30+ dedicated parsers** — see [`docs/support/commands.md`](docs/support/commands.md) for the full list.
- **Chain-aware rewrite** — `cd X && git status` or `cargo fmt && cargo clippy` wrap each rewritable segment with trs; pipes and semicolons pass through untouched.
- **Env-var prefix preservation** — `RUSTFLAGS=x cargo build` stays functional after rewrite.
- **Generic compression fallback** — commands without a parser still get ANSI stripping, whitespace collapse, repeated-line dedup (~30–40% free).
- **Token savings dashboard** — `trs stats` shows cumulative compression and tokens saved per day. `trs stats --by-agent` breaks totals down by which AI agent fired each rewrite. `-n <N>` customizes row limits.
- **Collision check** — `trs init` detects hooks from other token-compression tools (via `@imports` too) and aborts by default; `--replace` migrates cleanly. See [`docs/support/other-token-savers.md`](docs/support/other-token-savers.md).
- **Self-upgrade** — `trs upgrade` auto-detects your install channel (npm / curl|sh) and runs it, then refreshes hook templates and the output-saver block.
- **Bug-report bundle** — `trs debug-info` packages version + platform + doctor checks + recent history + tee logs into one paste-ready report.

### How it stays safe

- `--no-verify` blocked on `git commit` / `git push` (protects pre-commit hooks from agents).
- Commands with `--json` / `--porcelain` flags pass through untouched.
- If a parser fails, output falls back to truncated passthrough — never silent failure.
- Exit codes always propagated from the wrapped command.
- On failure, full output saved to `~/.trs/tee/` for recovery.
- `trs read` never strips content from JSON/YAML/TOML/XML data files.

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

## Tech stack

| | |
|---|---|
| Language | Rust |
| Binary | ~6 MB (LTO + strip), no runtime deps |
| Startup | ~12 ms on macOS / Linux |
| CLI | clap 4 (bypassed on hot path) |
| Tests | 2,186+ passing, 0 warnings |
| Architecture | 200+ modular files — see [AGENTS.md](AGENTS.md) |

## Why

<details>
<summary>The honest backstory behind trs.</summary>

Token pricing kept climbing. Every `git status`, `cargo test`, and `ls -la` the agent rendered into its context cost real money, and the signal-to-noise ratio on those commands was painfully low. We started writing small tools — first for ourselves, then for the team — to reduce what the agent actually had to read.

Along the way we came across [**rtk**](https://github.com/rtk-ai/rtk) (Rust Token Killer). By then our tools had been evolving on their own, so we faced the honest choice: migrate to rtk and drop what we'd built, or continue and publish our own take. We chose to continue — more options in this space means a better fit for more workflows. trs kept iterating and expanding as we learned more about where tokens actually burn.

The more we used it, the more we saw the opportunity was bigger than input hooks. `trs output-saver` installs rules into each agent's global config so replies come back shorter too. `trs audit-docs` inspects CLAUDE.md / AGENTS.md for the bloat every session re-loads. `trs ingest` compresses whole repos into a budget-aware, LLM-ready context index. Still a single static binary with zero runtime deps — the story just got bigger than hooks.

The landing page has the full write-up: <https://usetrs.dev>

</details>

## From source (development)

Prefer the prebuilt install paths above unless you're contributing. For a source checkout:

```bash
git clone https://github.com/dPeluChe/trs.git
cd trs

# Build + install into ~/.cargo/bin/
cargo install --path .

# Dev loop
cargo test                     # 2,186+ tests across 71 suites
cargo clippy -- -D warnings    # no warnings allowed
cargo fmt -- --check           # formatting must match
cargo run -- git status        # run locally against the workspace
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for code guidelines, [AGENTS.md](AGENTS.md) for architecture, [`docs/development/`](docs/development/) for deeper internals, and [`docs/roadmap/TASK_TODO.md`](docs/roadmap/TASK_TODO.md) for the roadmap.

## License

MIT

---

<p align="center">
  A product by <a href="https://iteris.tech"><strong>Iteris</strong></a> · Published and maintained by <a href="https://dpeluche.dev"><strong>@dPeluChe</strong></a>
</p>
