# labs-tarscli (187 files, 26.4k tokens, rust)

## Structure

//
  README.md  .gitignore  AGENTS.md  CODE_OF_CONDUCT.md  
  CONTRIBUTING.md  Cargo.toml  LICENSE  README.es.md

.github/workflows/
  ci.yml  release.yml

docs/
  CNAME  install.ps1  install.sh

docs/development/
  agent-integrations.md

docs/development/benchmarks/
  README.md  benchmark-real.sh  benchmark.sh  chain-rewrite.sh

docs/features/
  audit-docs.md  doctor.md  formats.md  ingest.md  init.md  
  output-saver.md  stats.md  upgrade.md

docs/roadmap/
  TASK_TODO.md

docs/roadmap/completed/
  2603.md  2604.md  README.md

docs/support/
  agents.md  commands.md  install.md  other-token-savers.md

npm/
  README.md  package.json

npm/bin/
  trs  trs.cmd

npm/platforms/darwin-arm64/
  README.md  package.json

npm/platforms/darwin-x64/
  README.md  package.json

npm/platforms/linux-arm64/
  README.md  package.json

npm/platforms/linux-x64/
  README.md  package.json

npm/platforms/win32-x64/
  README.md  package.json

scripts/
  docker-smoke.sh  sync-version.sh

src/
  audit_docs.rs  benchmark.rs  classifier.rs  classifier_exec.rs  
  classifier_transfer.rs  cli.rs  commands.rs  commands_parse.rs  
  config.rs  debug_info.rs  discover.rs  doctor.rs  doctor_tests.rs  
  fast_find.rs  help.rs  help_tests.rs  init.rs  init_collision.rs  
  init_templates.rs  main.rs  main_tests.rs  main_tests_precedence.rs  
  output_saver.rs  process.rs  process_helpers.rs  process_tests.rs  
  rewrite.rs  tracker.rs  upgrade.rs

src/formatter/  — Formatter system for trs (Token-Reducing Shell)
  agent.rs  agent_schema.rs  compact.rs  compact_schema_git.rs  
  compact_schema_output.rs  csv.rs  csv_schema.rs  helpers.rs  json.rs  
  json_schema.rs  mod.rs  raw.rs  tsv.rs  tsv_schema.rs

src/ingest/  — Project digest generator for LLM consumption
  collect.rs  collect_compress.rs  collect_index.rs  
  collect_manifests.rs  deps.rs  deps_extract.rs  format.rs  meta.rs  
  mod.rs  ollama.rs  remote.rs  store.rs

src/reducer/  — Reducer system for trs (Token-Reducing Shell)
  mod.rs  output.rs  registry.rs  truncation.rs

src/router/  — Command routing system for trs (Token-Reducing Shell)
  mod.rs

src/router/handlers/
  clean.rs  common.rs  err.rs  html2md.rs  isclean.rs  json.rs  
  json_query.rs  json_tests.rs  mod.rs  read.rs  read_filters.rs  
  read_tests.rs  replace.rs  run.rs  search.rs  stats.rs  tail.rs  
  trim.rs

src/router/handlers/parse/
  brew.rs  bun_format.rs  bun_parse.rs  extra_cargo_test.rs  
  extra_db.rs  extra_download.rs  extra_env.rs  extra_network.rs  
  extra_services.rs  extra_system.rs  find.rs  git_branch.rs  
  git_diff.rs  git_diff_format.rs  git_log.rs  git_status.rs  
  git_status_format.rs  go_test.rs  grep.rs  grep_format.rs  
  jest_format.rs  jest_parse.rs  lint.rs  logs.rs  logs_format.rs  
  logs_helpers.rs  ls.rs  mod.rs  npm_format.rs  npm_parse.rs  
  pnpm_format.rs  pnpm_parse.rs  ps.rs  pytest_format.rs  
  pytest_parse.rs  python_traceback.rs  test.rs  vitest_format.rs  
  vitest_parse.rs

src/router/handlers/txt2md/  — Handler for the `txt2md` command - converts plain text to Markdown
  detect_headings.rs  detect_lists.rs  format.rs  mod.rs  parser.rs

src/router/handlers/types/  — Shared data structures and types for command handlers
  fs.rs  git.rs  grep_types.rs  logs.rs  mod.rs  test_types_core.rs  
  test_types_runners.rs

src/schema/  — Stable JSON schemas for trs (Token-Reducing Shell) reducers
  fs.rs  git.rs  logs.rs  mod.rs  process.rs  search.rs  test.rs  
  tests.rs


## Key Dependencies

  common.rs ← clean.rs, err.rs, handlers/html2md.rs, handlers/isclean.rs (+10)
  types/mod.rs ← handlers/html2md.rs, handlers/isclean.rs, parse/mod.rs, handlers/replace.rs (+5)
  helpers.rs ← formatter/agent.rs, agent_schema.rs, formatter/compact.rs, compact_schema_output.rs (+2)
  formatter/mod.rs ← formatter/agent.rs, formatter/compact.rs, csv.rs, formatter/json.rs (+2)
  tests.rs ← formatter/mod.rs, help.rs, reducer/mod.rs, schema/mod.rs

## README.md

<strong>trs</strong> — <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · terminal compression for AI agents

  <a href="https://usetrs.dev"><strong>usetrs.dev</strong></a> ·
  <a href="https://github.com/dPeluChe/trs">GitHub</a> ·
  <a href="https://www.npmjs.com/package/@dpeluche/trs">npm</a> ·

  <a href="#what-is-trs">What</a> ·
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#supported-ai-agents">Agents</a> ·
  <a href="#supported-commands">Commands</a> ·
  <a href="#built-in-trs-tools">Built-in</a> ·
  <a href="#project-digest">Digest</a> ·

---

## What is trs

Prefix any command with `trs` (or let `trs init` wire it into your AI tool for you). The binary spawns your command, parses the output, and emits a compact version built for both humans and LLMs.

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

... (254 lines, hidden sections: Install · macOS / Linux · Windows (PowerShell) · npm (all platforms) · cargo (builds from source) · Quick start · 1. Try it — prefix any command with trs · 2. Let your AI agent do it automatically — wires hooks into Claude /)

**.gitignore**
# Build artifacts
/target/
**/*.rs.bk
*.pdb

# Cargo lock file for binary projects (keep for libraries)
# Uncomment if this becomes a library:
# Cargo.lock
... (65 lines)

## AGENTS.md

# AGENTS.md — trs (Token-Reducing Shell)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Reduces token consumption by 68-99% for developers, AI agents, and automation pipelines.

## Related commands worth knowing

- `trs stats` — cumulative savings across commands (shows date range, today
  vs. average, last command, top-reducers).
- `trs doctor` — installation health check. Warns when AGENTS.md / CLAUDE.md
  exceed ~5k tokens and points at `trs audit-docs`.
- `trs audit-docs` — static analysis of agent instruction files
  (CLAUDE.md, AGENTS.md, rules files). Surfaces cross-file duplicates,
  embedded code/SQL/JSON blocks that belong elsewhere, dead `@imports`,
  and — for code fences — cross-references declared symbols against
  the actual source tree (so you can REMOVE ones already defined in src/
  and EXTRACT ones that don't live anywhere yet).
- `trs output-saver` — install a short output-reduction rules block
  into each agent's global config (AGENTS.md / CLAUDE.md / Cursor rules).
  Closes the symmetric gap: trs compresses what agents SEE via
  `trs rewrite`; output-saver compresses what they EMIT via
  anti-preamble / anti-narration / structured-output directives.
  Check-first by default (`--install` to commit, `--remove` to undo).
- `trs ingest` — project digest for LLM consumption. Use symbol index,
  compression levels, or `owner/repo` URL shorthand.
- `trs init --show` — see which AI agents have trs hooks installed.
- `trs upgrade` — detects the install channel (npm / curl) and re-runs
  it for the latest release. `--check` dry-runs, `-y` skips the
  confirmation prompt. See [`docs/features/upgrade.md`](./docs/features/upgrade.md).
- `trs init <tool>` — now runs a collision pre-check: detects hooks
  from other token-compression tools (via `@imports` too) and aborts
  by default. `--replace` removes competitor hooks cleanly, `--force`
  installs alongside (risky — double-compression can corrupt output).
  See [`docs/support/other-token-savers.md`](./docs/support/other-token-savers.md)
  for the list of tools we coexist with.
- `trs stats --by-agent` — breakdown by which AI agent fired each
  rewrite. Reads the `TRS_AGENT` env var that `trs rewrite` and
  plugin templates inject into the command line. Rules-only agents
... (165 lines, hidden sections: Architecture · Key Design Decisions · Development · Testing)

## CODE_OF_CONDUCT.md

# Code of Conduct

## Our Standard

Be respectful, constructive, and welcoming. We're all here to learn and build.

## Expected Behavior

- Be kind in code reviews and discussions
- Accept feedback gracefully
- Focus on what's best for the project
- Respect differing viewpoints

## Unacceptable Behavior

- Harassment, insults, or personal attacks
- Trolling or deliberately inflammatory comments
- Publishing others' private information

## Enforcement
... (26 lines, hidden sections: Scope)

## CONTRIBUTING.md

# Contributing to trs

Thanks for your interest in contributing! trs is a personal project that grew into something useful, and contributions are welcome — whether it's a new parser, a bug fix, or just better docs.

## Getting started

git clone https://github.com/dPeluChe/trs.git
cd trs
cargo build
cargo test

All three checks must pass before submitting a PR:

cargo fmt -- --check           # formatting
cargo clippy -- -D warnings    # no warnings allowed
cargo test                     # 2,186+ tests, 0 failures

## Code guidelines

### File size
- Max 500 lines per file. If a file grows past this, split it.
- Rust allows multiple `impl` blocks in separate files — use this pattern.
- Tests go in `tests/` (integration) or `src/*_tests.rs` (unit).

### Naming
- Parser files: `{tool}_parse.rs` + `{tool}_format.rs` (e.g. `npm_parse.rs`, `npm_format.rs`)
- Test files: `test_{feature}_{category}.rs` (e.g. `test_replace_edge.rs`)
- Fixture data: `tests/fixture_data/{tool}_{scenario}.txt`

### Style
- Run `cargo fmt` before committing. No exceptions.
- No `unwrap()` in production code — use `?` or explicit error handling.
- `unwrap()` is fine in tests.
- Prefer simple code over clever code. Three similar lines > premature abstraction.
- Don't add doc comments to every function — only where the logic isn't obvious.

### Tests
- Every new parser needs at least 3 tests: basic input, edge case, empty input.
- Integration tests (in `tests/`) test the CLI binary end-to-end.
- Unit tests (in `src/`) test individual functions.
... (166 lines, hidden sections: Adding a new parser · Proposing new commands · Project structure · Commit messages · Questions?)

**Cargo.toml**
name: trs-cli
version: 0.5.9
edition: 2021
description: Token-reducing shell for AI agents — compact terminal output at 68-99% reduction
[dependencies]
  clap = 4
  grep = 0.3
  grep-matcher = 0.1
  grep-regex = 0.1
  grep-searcher = 0.1
  htmd = 0.5
  ignore = 0.4
  regex = 1
  serde = 1
  serde_json = 1
  tempfile = 3
  time = 0.3
  toml = 0.8
  ureq = 3
[dev-dependencies]
  assert_cmd = 2
  predicates = 3
  serde_json = 1
bin: trs

**LICENSE**
MIT License

Copyright (c) 2026 dPeluChe

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
... (21 lines)

## README.es.md

<strong>trs</strong> — <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · compresión de salida terminal para agentes de IA

  <a href="https://usetrs.dev"><strong>usetrs.dev</strong></a> ·
  <a href="https://github.com/dPeluChe/trs">GitHub</a> ·
  <a href="https://www.npmjs.com/package/@dpeluche/trs">npm</a> ·

  <a href="#qué-es-trs">Qué</a> ·
  <a href="#instalación">Instalar</a> ·
  <a href="#inicio-rápido">Inicio rápido</a> ·
  <a href="#agentes-de-ia-soportados">Agentes</a> ·
  <a href="#comandos-soportados">Comandos</a> ·
  <a href="#herramientas-built-in">Built-in</a> ·
  <a href="#digest-del-proyecto">Digest</a> ·

---

## Qué es trs

Prefija cualquier comando con `trs` (o deja que `trs init` lo conecte a tu herramienta de IA). El binario ejecuta tu comando, parsea la salida, y emite una versión compacta pensada para humanos y LLMs.

... (254 lines, hidden sections: 1.4 KB → 336 B (76% reducción) · 55 KB → 58 B (99% reducción) · 55 KB → 5.5 KB (90% reducción) · Instalación · macOS / Linux · Windows (PowerShell) · npm (todas las plataformas) · cargo (compila desde fuente))

## .github/workflows/

**ci.yml**
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

... (21 lines)

**release.yml**
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
... (21 lines)

## docs/

**CNAME**: usetrs.dev
**install.ps1**
function Write-Info    { param($msg) Write-Host "▸ $msg" -ForegroundColor Cyan
function Write-Ok      { param($msg) Write-Host "✓ $msg" -ForegroundColor Green
function Write-Warning2 { param($msg) Write-Host "! $msg" -ForegroundColor Yellow
function Write-Err     { param($msg) Write-Host "✗ $msg" -ForegroundColor Red; exit 1
function Get-PathEntries
function Test-InUserPath
function Get-InstallDir

**install.sh**
REPO="dPeluChe/trs"
BIN_NAME="trs"
C_RESET='\033[0m'
C_BOLD='\033[1m'
C_GREEN='\033[0;32m'
C_YELLOW='\033[0;33m'
C_RED='\033[0;31m'
C_CYAN='\033[0;36m'
... (19 lines)

## docs/development/

**agent-integrations.md**
# AI Agent Integrations — Reference

How `trs` integrates with each supported AI coding agent. Use this doc when
adding a new agent, debugging a broken integration, or reviewing why a
specific quirk exists.

Last validated: 2026-04-19 against `trs` v0.5.7.

## Output-saver matrix

`trs output-saver` installs output-reduction rules into each agent's
global config. Orthogonal to the input-side hook/plugin/rules install
matrix below. Six distinct target paths across three mechanisms:

| Agent | Mechanism | Path |
|---|---|---|
| Claude Code | standalone file + `@import` | `~/.claude/trs-output-saver.md` + line in `~/.claude/CLAUDE.md` |
| Gemini CLI | standalone file + `@import` | `~/.gemini/trs-output-saver.md` + line in `~/.gemini/GEMINI.md` |
| Cursor | auto-loaded rules file | `~/.cursor/rules/trs-output-saver.mdc` |
| Codex | inline with sentinels | `~/.codex/AGENTS.md` |
... (363 lines, hidden sections: Integration types · Wire-format dispatch (hook agents) · Agent attribution (`TRS_AGENT`) · Per-agent reference · Test prompts · Debugging a broken integration · 1. Create a logging wrapper · 2. Point the agent's hook at the wrapper (back up first))

## docs/development/benchmarks/

**README.md**
# trs Benchmarks

Living laboratory for trs. These benchmarks exist to help us learn, measure, and iterate — not to be marketing material or regression gates.

## Why this folder exists

Every CLI in this space (rtk, token-saver, ccp, repomix, claw-compactor, pi)
ships different tradeoffs. Some compress harder, some preserve more signal,
some are faster on specific inputs. Instead of guessing, we run the
comparisons here and let the numbers guide the decisions we make in trs.

The goal is internal knowledge — "what do we actually do better, and where
should we improve?" — not to publish a leaderboard.

## What's in here

| Script | Purpose |
|--------|---------|
| [`benchmark.sh`](./benchmark.sh) | Comparative runs against rtk and token-saver on a curated set of real-world commands |
| [`benchmark-real.sh`](./benchmark-real.sh) | Longer, more varied workload (slower, more representative) |
... (41 lines)

**benchmark-real.sh**
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'
... (13 lines)

**benchmark.sh**
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'
... (45 lines)

**chain-rewrite.sh**
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'
TRS_BIN="$REPO_ROOT/target/release/trs"
... (11 lines)

## docs/features/

**audit-docs.md**
# `trs audit-docs` — find bloat in CLAUDE.md / AGENTS.md / rules files

Agent instruction files (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`,
`.windsurfrules`) get loaded into every agent session — every turn,
every project open, every conversation start. Bloat in these files is
the single most expensive kind of bloat because it multiplies across
every interaction.

`trs audit-docs` is a static analyzer that finds:

- Cross-file duplicate sections (SimHash over 3-word shingles; flags
  blocks with Hamming distance ≤ 6, i.e. ≥ 90% similar).
- Dead `@imports` — references to files that don't exist.
- Embedded code / SQL / JSON / YAML / tables that belong in their own
  files rather than inline in rules.
- Code fences whose declared symbols already exist in the project's
  source tree (so you can replace the snippet with a `src/…:NN` link)
  or don't exist yet (so you can extract them into new files).

## Quick reference
... (127 lines, hidden sections: What it scans · Duplicate detection (SimHash) · Dead `@imports` · Embedded bloat · Language support for symbol extraction · Integration with `trs doctor` · Philosophy · Output format example)

**doctor.md**
# `trs doctor` — installation health check

`trs doctor` runs 10 checks that cover every surface where a trs
install can go wrong: the binary, the PATH, dependencies, the config
dir, the history file, the hook pipeline, agent integrations, and
agent-doc budgets. Use it after a fresh install, when an agent hook
suddenly stops firing, or when debugging CI.

## Quick reference

trs doctor          # human-readable report
trs doctor --json   # machine-readable; exits non-zero on any Fail

## What each check does

| Check | What it verifies |
|---|---|
| `trs binary` | Reports version + binary path (never fails — always informational). |
| `trs in PATH` | `which -a trs` / `where trs`. Warns if multiple binaries exist so a shadowed install doesn't silently win. |
| `git available` | `git --version`. Required — several parsers assume git is present. |
... (100 lines, hidden sections: Reading the report · JSON mode · Typical fixes · See also)

**formats.md**
# Output formats

Every trs command supports six output formats. Pick the one that
matches the consumer: humans / agents read compact, scripts read
json/csv/tsv, pipelines sometimes want raw passthrough.

| Flag | Name | Who it's for |
|---|---|---|
| *(default)* | compact | humans + agents — terse single-pass form |
| `--json` | JSON | scripts, dashboards, anything structured |
| `--csv` | CSV | spreadsheets, basic data import |
| `--tsv` | TSV | tab-friendly tooling (`cut -f`, spreadsheets) |
| `--agent` | agent-optimized markdown | LLMs specifically — same compact form with marker syntax for section parsing |
| `--raw` | raw passthrough | unchanged — no compression, still tracked in stats |

Flags work anywhere in the invocation — `trs --json git status` and
`trs git status --json` are equivalent.

## Examples

... (126 lines, hidden sections: When to use what · Built-in tools vs wrapped commands · See also)

**ingest.md**
# `trs ingest` — project digest for AI agents

`trs ingest` walks a repo and produces a compact, token-budget-aware
Markdown digest of the codebase — structure + key files + signatures
— ready to paste into an AI agent's context.

## Quick reference

trs ingest                      # write digest, print path to stdout
trs ingest --budget 128k        # fit to token budget (signatures first)
trs ingest --deps               # dependency graph only, no content
trs ingest --changed            # only files with uncommitted changes
trs ingest --since-last         # only files changed since last ingest
trs ingest --fresh              # reuse cached digest if HEAD unchanged
trs ingest -o ~/ctx.md          # custom output path (no shadow save)
trs ingest --print              # emit content to stdout instead of path
trs ingest --warn-at 40k        # stderr warning if digest exceeds N tokens
trs ingest --list               # list saved digests + HEAD sha + stale markers
trs ingest --read myproject     # read a saved digest by name

... (139 lines, hidden sections: What the digest contains · <project name> · Structure · Dependencies · Files (highlights) · Files (signatures) · Budget-aware truncation · Staleness detection)

**init.md**
# `trs init` — install hooks for AI agents

`trs init` wires your AI coding agent's shell-execution pipeline through
`trs rewrite` so every command gets compressed automatically. Nine
agents are supported end-to-end. See [`docs/development/agent-integrations.md`](../development/agent-integrations.md)
for the full per-agent reference.

## Quick reference

trs init --show                      # status of all 9 agents
trs init --all --global              # install for every detected agent
trs init <agent>                     # install for one: claude, gemini, cursor, …
trs init --all --global --force      # refresh templates (see "Refreshing hooks")
trs init <agent> --replace           # migrate cleanly from another compressor

## What gets installed where

| Agent | Type | Target |
|---|---|---|
| Claude Code | JSON hook | `~/.claude/settings.json` (or `~/.claude/hooks.json`) |
... (181 lines, hidden sections: Collision handling · Refreshing hooks · `--global` vs project-local · Bypassing the hook for one command · Agent attribution (`TRS_AGENT`) · Uninstalling · See also)

**output-saver.md**
# `trs output-saver` — reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents see — the output of the shell commands they run. Agents
still emit verbose replies: preambles ("Sure!"), narration
("Now I will…"), speculative suggestions, hallucinated file paths.

`trs output-saver` installs a short rules block into each supported
agent's global config so those replies come back tighter.

## Quick reference

trs output-saver                 # read-only scan of all 9 agents
trs output-saver --install       # write to every detected agent
trs output-saver <agent> --install  # scope to one
trs output-saver --remove        # clean uninstall
trs output-saver --print         # dump the block to stdout (pipe-friendly)

## What the block says

... (142 lines, hidden sections: Coverage matrix · How the install is idempotent · Output saver — keep replies cheap · Check-first semantics · `--refresh` — pick up template changes without adding new installs · `--remove` behavior · Measuring impact · Interaction with `trs init`)

**stats.md**
# `trs stats` — token savings dashboard

Every trs invocation logs an entry to `~/.trs/history.jsonl`:
timestamp, command, input bytes, output bytes, duration. `trs stats`
reads that log and produces a dashboard of cumulative savings.

## Quick reference

trs stats              # summary dashboard (top 15 commands)
trs stats --history    # per-command log (most recent 20)
trs stats -n 30        # override row cap (top 30 in summary, last 30 in --history)
trs stats --by-agent   # breakdown by which AI agent triggered the run
trs stats --json       # machine-readable summary

## Summary (default)

trs savings — Apr 15 23:04 → Apr 20 17:12 (5 days)
────────────────────────────────────────────────────
  input:       4.2 MB    output:  930 KB     saved: 3.3 MB
  tokens in:   1.0M      out:     232K        saved: 800K (77%)
... (152 lines, hidden sections: History view · `--by-agent` — attribution breakdown · JSON mode · What gets tracked · Clearing / trimming history · See also)

**upgrade.md**
# `trs upgrade` — re-run the install pipeline for the latest release

`trs upgrade` detects how trs was installed on your machine and runs
the matching install command so you don't have to remember which
channel you used.

Added in v0.5.8.

## Quick reference

trs upgrade                # detect + confirm + binary + refresh configs
trs upgrade -y             # skip confirmation (useful for scripts / cron)
trs upgrade --check        # dry-run: show detection + planned commands
trs upgrade --binary-only  # upgrade only the binary, skip config refresh

## What gets upgraded

By default `trs upgrade` runs three steps in order:

1. Binary — the shell install command for your detected channel
... (152 lines, hidden sections: Detection logic · Why detection is path-based · Confirmation prompt · Roadmap for unsupported channels · What happens after a successful upgrade · Interaction with hooks · See also)

## docs/roadmap/

**TASK_TODO.md**
# trs — Roadmap

Binary: `trs` | Language: Rust | Status: Active development

---

## Phase 1 — Release & Distribution

- [x] Create first GitHub Release — v0.1.0 shipped; at v0.5.7 now
- [x] npm publish (`@dpeluche/trs`)
- [ ] Homebrew tap (low priority — npm + curl|sh covers 99% of users)
- [ ] Publish to crates.io (`cargo install trs-cli` — currently source-only)
- [ ] Shell completions (bash, zsh, fish)
- [ ] Copilot hook — see Phase 3 "VSCode ecosystem" for the full research scope
- [x] ~~Detect pipe context — skip rewriting find/fd when piped~~ —
      replaced in v0.5.6: rewrite the producer segment and pass the pipe
      through unchanged. `git status | head -3` now becomes
      `trs git status | head -3` instead of being skipped entirely.
- [x] Rewrite hook: detect `cd X && git Y` chains — done in v0.5.5
      (chain-aware per-segment rewrite)
... (234 lines, hidden sections: Phase 2 — New Parsers · Phase 2.5 — Ideas from competitor analysis (token-optimizer) · Phase 3 — Agent integration follow-ups · Phase 4 — Analytics & Configuration · Phase 5 — Plugin System (future evaluation))

## docs/roadmap/completed/

**2603.md**
# Marzo 2026 — trs development log

---

# 2026-03-18 — Phase 1: New Features + Competitor Analysis

## Context
Competitive analysis of RTK (v0.31.0), tokf, ccp, claw-compactor, QMD, and Pi Coding Agent.
Identified gaps vs RTK and implemented Phase 1 features based on findings.
Evaluated and discarded MCP Server integration (adds complexity without value for a unidirectional pipe tool).

## Completed

### Emoji stripping (global, default)
- Added `strip_emojis()` to `src/router/handlers/common.rs`
- Applied in `ParseHandler::read_input()` and `Router::process_stdin()`
- Strips decorative emoji (U+1Fxxx ranges + select dingbats like U+274C, U+2705, U+2728)
- Preserves functional symbols: checkmarks, crosses, bullets, warning signs (used by test runners/build tools)
- Why: RTK #603 found emojis confuse non-Claude LLMs causing retry loops

... (369 lines, hidden sections: Decisions · Files Changed · 2026-03-18 (session 2) — Safety, Config, Read, Core Improvements · Context · Completed · Decisions · Files Changed · 2026-03-18 (session 3) — Opensource Architecture Refactor)

**2604.md**
# Abril 2026 — trs development log

---

# 2026-04-15 — Dependency graph for ingest + Parity fixes

## Context
Analyzed https://github.com/braedonsaunders/codeflow for ideas to improve `trs ingest`.
Implemented dependency graph feature. Then reviewed RTK v0.36.0 changelog for parity gaps
and addressed UTF-8 safety, pytest -q, go test parser, and Google Antigravity support.

## Completed

### Ingest dependency graph (PR #1)
- New module `src/ingest/deps.rs` — graph building, resolution, formatting
- New module `src/ingest/deps_extract.rs` — language-specific import extractors (Rust, TS/JS, Python, Go)
- `## Key Dependencies` section auto-injected after `## Structure` in code project digests
- `--deps` flag — outputs only the import graph, no file content
- Import resolution: `./`/`../` relative, `@/` alias (Next.js/Vite), Go module paths, Python absolute packages
- `short_label()` disambiguates generic filenames (mod.rs, index.ts) with parent dir context
... (676 lines, hidden sections: Decisions · Files Changed · 2026-04-17 — v0.5.6 qualitative polish: 9-agent integration audit · Context · Completed · Decisions · Files Changed · 2026-04-18 — v0.5.7: feedback branch — handlers + safety + docs audit)

**README.md**
# TASK_COMPLETED — Changelog de trabajo

Registro mensual de tareas completadas, decisiones tomadas y archivos modificados.

## Formato de archivos

Cada archivo se nombra `YYMM.md` (ej: `2603.md` = marzo 2026).

## Estructura de cada entrada

# YYYY-MM-DD — Titulo breve de la sesion

## Context
Por que se hizo este trabajo. Contexto del problema o requerimiento.

## Completed
### Feature/Fix nombre
- Que se hizo (bullet points concretos)
- Archivos clave modificados
- Tests agregados/modificados
... (35 lines)

## docs/support/

**agents.md**
# Supported AI agents

Nine AI coding agents are supported end-to-end. Each row lists the
install method, which sides of the loop trs touches (input / output),
how `trs stats --by-agent` labels runs from that agent, and the
install scope.

| Agent | Install method | Input hook (rewrite) | Output-saver | Attribution label | Scope |
|---|---|---|---|---|---|
| Claude Code | programmatic hook | ✓ | ✓ (`@import`) | `claude` | global + project |
| Gemini CLI | programmatic hook | ✓ | ✓ (`@import`) | `gemini` | global + project |
| Cursor | programmatic hook | ✓ | ✓ (`.mdc`) | `cursor` | global + project |
| OpenCode | plugin template | ✓ | ✓ (inline block) | `opencode` | global |
| Kilo Code | plugin template | ✓ | ✓ (inline block) | `kilo` | global |
| Factory Droid | programmatic hook | ✓ | ✓ (inline block) | `claude` (see caveat) | global + project |
| Codex CLI | rules file only | — | ✓ (inline block) | `(untagged)` | global + project |
| Google Antigravity | rules file only | — | — | `(untagged)` | project only |
| Windsurf | rules file only | — | ✓ (inline block) | `(untagged)` | global + project |

## Column legend
... (41 lines)

**commands.md**
# Supported commands

Every command supported by trs falls into one of four levels.

1. Dedicated parser. trs spawns the tool, parses its native output,
   and emits a structured compact form. Typical reduction 68–99%.
2. Dispatched alias. A different binary with the same semantics
   (e.g. `rg` for `grep`, `eza` for `ls`) gets routed to the same
   parser. No configuration — the dispatcher recognizes the binary
   name.
3. Generic compression. Commands without a parser still get ANSI
   stripping, whitespace collapse, and repeated-line deduplication.
   Typical reduction 30–40% "free."
4. Passthrough. Commands where trs detects a flag that already
   produces structured output (`--json`, `--porcelain`) are passed
   through untouched — the agent gets the raw structured form.

## Commands with dedicated parsers

### VCS — git
... (189 lines, hidden sections: Built-in trs tools (not wrappers) · Dispatch mechanisms · Generic compression (the fallback))

**install.md**
# Installing trs

Five install channels. All of them ship the same native binary
(~6 MB, zero runtime deps, ~12 ms startup). Pick whichever fits your
existing toolchain.

## Quick list

| Channel | One-liner |
|---|---|
| curl / sh | `curl -fsSL https://usetrs.dev/install.sh \| sh` |
| PowerShell | `irm https://usetrs.dev/install.ps1 \| iex` |
| npm | `npm install -g @dpeluche/trs` |
| cargo | `cargo install trs-cli` |
| Prebuilt binary | [GitHub Releases](https://github.com/dPeluChe/trs/releases) |

The `curl|sh` and `irm|iex` scripts are the recommended default: they
detect arch, download the right prebuilt binary, place it in
`~/.local/bin/` (or `$USERPROFILE\.local\bin\` on Windows), and add
that dir to PATH automatically if it isn't already.
... (143 lines, hidden sections: Platform support · Prebuilt binaries — manual install · Pinning a specific version · Custom install directory · Upgrading · Shadowed installs (multi-channel) · Uninstall · Troubleshooting)

**other-token-savers.md**
# Other token-saving tools

trs is one of several tools in the shell-output-compression space for
AI agents. This page lists the alternatives we're aware of, for folks
evaluating options or migrating between tools.

We don't link to these projects directly — go search for them if you
want to compare. The list is descriptive, not promotional, and we
update it as new tools appear.

## Alternatives we've analyzed

- rtk (Rust Token Killer) — another Rust-based CLI proxy for
  shell compression. TOML filter pipeline, SQLite usage tracking,
  dedicated `rtk gain` analytics. Overlaps significantly with trs on
  the core rewrite surface.
- token-optimizer — Node-based compressor, installs as a global
  npm package. Hook integration focused on Claude Code.
- token-saver — early-stage shell wrapper, smaller scope than
  rtk / trs.
... (60 lines, hidden sections: How trs positions itself · Installing alongside another tool)

## npm/

**README.md**
# trs — Token-Reducing Shell

Transform noisy terminal output into compact, structured signal.
A CLI toolkit for developers, automation pipelines, and AI agents.

68-99% token savings on common dev operations.

## Install

npm install -g @dpeluche/trs

Or with other package managers:

cargo install trs-cli    # from source (Rust required)

## Usage

# Git (compressed output)
trs git status
trs git diff
... (41 lines)

**package.json**: name: @dpeluche/trs | version: 0.5.9
## npm/bin/

**trs**
#!/bin/sh
# trs — shell wrapper that execs the native binary directly.
# Saves ~25ms vs the previous Node wrapper by skipping the node runtime.
#
# The platform-specific binary is installed by npm as an optionalDependency
# into node_modules/@dpeluche/trs-cli-<os>-<arch>/trs.

set -e
... (72 lines)

**trs.cmd**
@echo off
rem trs launcher for Windows — execs the native binary directly.
rem See bin/trs for the Unix equivalent.
setlocal

set "DIR=%~dp0"
set "PKG=trs-cli-win32-x64"
set "BIN="
... (31 lines)

## npm/platforms/darwin-arm64/

**README.md**
Platform binary package for trs (darwin-arm64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-darwin-arm64 | version: 0.5.9
## npm/platforms/darwin-x64/

**README.md**
Platform binary package for trs (darwin-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-darwin-x64 | version: 0.5.9
## npm/platforms/linux-arm64/

**README.md**
Platform binary package for trs (linux-arm64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-linux-arm64 | version: 0.5.9
## npm/platforms/linux-x64/

**README.md**
Platform binary package for trs (linux-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-linux-x64 | version: 0.5.9
## npm/platforms/win32-x64/

**README.md**
Platform binary package for trs (win32-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-win32-x64 | version: 0.5.9
## scripts/

**docker-smoke.sh**
BINARY="$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/trs"
DISTROS=(
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'
PASS=0
FAIL=0
... (13 lines)

**sync-version.sh**
#!/bin/bash

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ -n "$1" ]; then
  VERSION="$1"
... (32 lines)

## src/

**audit_docs.rs**
  struct DocFile
  struct Block
  struct DupPair
  struct DeadRef
  struct InlineBloat
  struct SymbolMatch
  enum BloatKind
  pub fn run_audit_docs(root: &Path)
  fn discover(root: &Path) -> Vec<DocFile>
  fn collect_markdown_dir(dir: &Path, out: &mut Vec<DocFile>, root: &Path)
  fn load_doc(path: &Path, root: &Path) -> Option<DocFile>
  fn split_into_blocks(content: &str) -> Vec<Block>
  fn flush_block(blocks: &mut Vec<Block>, buf: &mut String, start: usize, end_exclusive: usize)
  fn compute_simhash(text: &str) -> u64
  fn fnv1a_64(bytes: &[u8]) -> u64
  fn find_near_duplicates(blocks: &[Block]) -> Vec<DupPair>
  fn find_dead_refs(docs: &[DocFile], root: &Path) -> Vec<DeadRef>
  fn extract_references(line: &str) -> Vec<String>
  fn looks_like_import_path(s: &str) -> bool
  fn looks_like_local_markdown_link(s: &str) -> bool
  fn ref_resolves(reference: &str, doc_dir: &Path, root: &Path) -> bool
  fn find_inline_bloat(docs: &[DocFile]) -> Vec<InlineBloat>
  fn collect_code_fences(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>)
  fn extract_fence_symbols(lang: &str, body: &str) -> Vec<String>
  fn is_meaningful_symbol(name: &str) -> bool
  fn extract_js_like_symbol(line: &str) -> Option<String>
  fn extract_python_symbol(line: &str) -> Option<String>
  fn extract_rust_symbol(line: &str) -> Option<String>
  fn extract_go_symbol(line: &str) -> Option<String>
  fn extract_swift_symbol(line: &str) -> Option<String>
  fn first_ident(s: &str) -> Option<String>
  fn fence_open_lang(line: &str) -> Option<String>
  fn is_fence_close(line: &str) -> bool
  fn collect_large_tables(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>)
  fn is_table_row(line: &str) -> bool
  fn is_table_separator(line: &str) -> bool
  fn resolve_symbol_matches(bloat: &mut [InlineBloat], root: &Path)
  fn contains_symbol_definition(content: &str, sym: &str, ext: &str) -> bool
  fn last_commit_days_ago(path: &Path, root: &Path) -> Option<u64>
  fn render_report(
  fn human_tokens(n: usize) -> String

**benchmark.rs**
  struct IterResult
  struct BenchReport
  fn run_once(cmd: &str, args: &[String]) -> Option<IterResult>
  fn run_through_trs(cmd: &str, args: &[String], raw_bytes: usize) -> usize
  fn estimate_compressed_size(cmd: &str, args: &[String], raw_bytes: usize) -> usize
  fn print_json(r: &BenchReport)
  fn print_table(r: &BenchReport)
  fn format_number(n: u64) -> String
  mod tests
  fn test_format_number()
  fn test_estimate_compressed_size()
  fn test_bytes_per_token_constant()
  fn test_reduction_pct_zero_input()

**classifier.rs**: fn strip_git_global_opts(args: &[String]) -> Vec<String> | fn has_structured_output_flag(args: &[String]) -> bool
**classifier_exec.rs**
  fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String>
  fn generic_compress(input: &str) -> String
  fn collapse_whitespace(s: &str) -> String

**classifier_transfer.rs**
  mod tests
  fn test_push_normal()
  fn test_push_up_to_date()
  fn test_pull_already_up_to_date()
  fn test_pull_fast_forward()
  fn test_fetch_new_branch()
  fn test_push_fatal_error()
  fn test_push_empty_output()

**cli.rs**
  pub struct Cli
  pub enum OutputFormat

  impl Cli
  pub fn output_format_precedence() -> &'static [OutputFormat]
  pub fn output_format(&self) -> OutputFormat
  pub fn enabled_format_flags(&self) -> Vec<OutputFormat>
  pub fn has_conflicting_format_flags(&self) -> bool
  pub fn current_format_precedence(&self) -> u8

**commands.rs**
  mod commands_parse
  pub use commands_parse::ParseCommands
  pub enum Commands
  pub enum ReadLevel
  pub enum TestRunner

**commands_parse.rs**: pub enum ParseCommands
**config.rs**
  pub fn config() -> &'static Config
  pub struct Config
  pub struct Limits

  impl Default for Config
  fn default() -> Self

  impl Default for Limits

  impl Config
  fn load() -> Self
  fn try_load(path: &PathBuf) -> Option<Self>
  mod tests
  fn test_defaults()
  fn test_parse_partial_toml()
  fn test_parse_empty_toml()
  fn test_parse_full_toml()

**debug_info.rs**
  fn build_report() -> String
  fn section(out: &mut String, title: &str, body: String)
  fn version_section() -> String
  fn platform_section() -> String
  fn doctor_section() -> String
  fn history_section() -> String
  fn tee_section() -> String
  fn truncate_for_report(cmd: &str, max_chars: usize) -> String
  mod tests
  fn report_has_expected_sections()
  fn truncate_folds_newlines()
  fn truncate_enforces_max()

**discover.rs**
  fn scan_directory(
  fn extract_command(line: &str) -> Option<String>
  mod tests
  fn test_extract_command()
  fn test_extract_command_escaped()
  fn test_extract_command_none()

**doctor.rs**
  impl Check
  fn pass(name: &'static str, detail: impl Into<String>) -> Self
  fn warn(name: &'static str, detail: impl Into<String>) -> Self
  fn fail(name: &'static str, detail: impl Into<String>) -> Self
  fn with_sub(mut self, lines: Vec<String>) -> Self
  fn with_hint(mut self, hint: impl Into<String>) -> Self

  impl fmt::Display for CheckStatus
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result

  struct Summary
  impl Summary
  fn from_checks(checks: &[Check]) -> Self
  fn check_output_saver_installed() -> Check
  fn check_agent_docs_health() -> Check
  fn human_k(n: usize) -> String
  fn check_version() -> Check
  fn check_path_accessible() -> Check
  fn check_dep(cmd: &str, label: &str, required: bool, hint: &str) -> Check
  fn check_config_dir() -> Check
  fn check_history_writable() -> Check
  fn check_stdin_pipeline() -> Check
  fn check_hooks_installed() -> Check
  mod tests

**doctor_tests.rs**
  fn test_check_version_passes()
  fn test_check_path_accessible()
  fn test_check_dep_git()
  fn test_check_dep_rg_name()
  fn test_check_dep_unknown_name()
  fn test_check_dep_missing_required()
  fn test_check_config_dir()
  fn test_check_history_writable()
  fn test_run_checks_returns_all()
  fn test_check_status_display()
  fn test_count_hooks_via_init()
  fn test_summary_from_checks()
  fn test_check_builders()

**fast_find.rs**
  fn print_compact(files: &[String], dirs: &[String], _root: &str)
  fn print_raw(files: &[String], dirs: &[String])
  fn print_json(files: &[String], dirs: &[String])
  fn glob_match(pattern: &str, name: &str) -> bool
  fn glob_match_inner(p: &[char], n: &[char]) -> bool
  mod tests
  fn test_glob_match()

**help.rs**
  pub fn get_command_help(command: &str) -> Option<&'static str>
  pub fn get_format_precedence_help() -> &'static str
  mod tests

**help_tests.rs**
  fn test_get_command_help_search()
  fn test_get_command_help_replace()
  fn test_get_command_help_tail()
  fn test_get_command_help_clean()
  fn test_get_command_help_parse()
  fn test_get_command_help_html2md()
  fn test_get_command_help_txt2md()
  fn test_get_command_help_run()
  fn test_get_command_help_trim()
  fn test_get_command_help_unknown()
  fn test_format_precedence_help()
  fn test_long_about_not_empty()

**init.rs**
  struct HookSpec
  impl AiTool
  fn spec(&self) -> Option<HookSpec>
  fn is_trs_in_path() -> bool
  fn install_from_spec(spec: &HookSpec, opts: InstallOpts) -> Result<String>
  fn install_codex() -> Result<String>
  fn install_rules(path_rel: &str, content: &str) -> Result<String>
  fn has_trs_marker(content: &str) -> bool
  fn file_has_any_trs_marker(content: &str) -> bool
  fn has_any_trs_marker_at(path_str: &str) -> bool
  fn has_any_trs_marker_at_path(path: &Path) -> bool
  fn home_dir() -> Result<PathBuf>
  fn check_file_contains_path(path: &Path, needle: &str) -> bool
  fn write_hook(dir: &Path, path: &Path, content: &str, replace: bool) -> Result<String>
  fn merge_json_hook(
  fn contains_trs_rewrite(val: &serde_json::Value) -> bool

**init_collision.rs**
  fn target_paths(tool: &AiTool, _global: bool) -> Vec<PathBuf>
  fn scan_json(path: &Path) -> Vec<Collision>
  fn scan_text(path: &Path, depth: usize, visited: &mut HashSet<PathBuf>) -> Vec<Collision>
  fn extract_imports(content: &str, base_file: &Path) -> Vec<PathBuf>
  fn resolve_import(target: &str, base_file: &Path) -> Option<PathBuf>
  fn collect_string_values(val: &serde_json::Value, out: &mut Vec<String>)
  fn truncate(s: &str, max: usize) -> String
  mod tests
  fn scan_json_flags_rtk_hook()
  fn scan_json_ignores_trs_hook()
  fn scan_text_flags_rtk_rules()
  fn scan_text_follows_at_imports()
  fn scan_text_breaks_import_cycle()
  fn resolve_import_handles_home_and_relative()
  fn is_competitor_hook_matches_nested()
  fn is_competitor_hook_rejects_trs()

**init_templates.rs**: (260 lines)
**main.rs**
  mod audit_docs
  mod benchmark
  mod classifier
  mod classifier_exec
  mod classifier_transfer
  mod cli
  mod commands
  mod debug_info
  mod discover
  mod doctor
  mod formatter
  mod help
  mod ingest
  mod init
  mod init_collision
  mod init_templates
  mod output_saver
  mod process
  mod reducer
  mod rewrite
  mod router
  mod schema
  mod upgrade
  pub use cli::{Cli, OutputFormat
  pub use commands::{Commands, ParseCommands, TestRunner
  mod fast_find
  fn main()
  fn is_external_fast_path(args: &[String]) -> bool
  fn parse_token_budget(s: &str) -> usize
  mod tests

**main_tests.rs**
  fn test_output_format_default()
  fn test_output_format_json_precedence()
  fn test_output_format_csv()
  fn test_output_format_tsv()
  fn test_output_format_agent()
  fn test_output_format_raw()
  fn test_output_format_compact()
  fn test_output_format_precedence_json_over_csv()
  fn test_output_format_precedence_csv_over_tsv()
  fn test_output_format_precedence_tsv_over_agent()
  fn test_output_format_precedence_agent_over_compact()
  fn test_stats_flag()
  fn test_search_command_parsing()
  fn test_replace_command_parsing()
  fn test_replace_command_parsing_with_count()
  fn test_tail_command_parsing()
  fn test_clean_command_parsing()
  fn test_parse_git_status()
  fn test_parse_test_runner()
  fn test_html2md_command()
  fn test_txt2md_command()
  mod precedence

**main_tests_precedence.rs**
  fn test_precedence_order()
  fn test_format_precedence_values()
  fn test_current_format_precedence()
  fn test_enabled_format_flags_single()
  fn test_enabled_format_flags_multiple()
  fn test_enabled_format_flags_none()
  fn test_has_conflicting_format_flags_true()
  fn test_has_conflicting_format_flags_false_single()
  fn test_has_conflicting_format_flags_false_none()
  fn test_precedence_json_over_all()
  fn test_precedence_csv_over_all_except_json()
  fn test_precedence_tsv_over_all_except_json_csv()
  fn test_precedence_agent_over_compact_raw()
  fn test_precedence_compact_over_raw()
  fn test_precedence_json_over_csv()
  fn test_precedence_json_over_tsv()
  fn test_precedence_json_over_agent()
  fn test_precedence_json_over_compact()
  fn test_precedence_json_over_raw()
  fn test_precedence_csv_over_tsv()
  fn test_precedence_csv_over_agent()
  fn test_precedence_csv_over_compact()
  fn test_precedence_csv_over_raw()
  fn test_precedence_tsv_over_agent()
  fn test_precedence_tsv_over_compact()
  fn test_precedence_tsv_over_raw()
  fn test_precedence_agent_over_compact()
  fn test_precedence_agent_over_raw()
  fn test_precedence_with_run_command()
  fn test_precedence_with_parse_command()
  fn test_precedence_with_replace_command()
  fn test_precedence_with_tail_command()
  fn test_precedence_with_clean_command()
  fn test_precedence_with_html2md_command()
  fn test_precedence_with_txt2md_command()
  fn test_stdin_no_command()
  fn test_stdin_with_format_flags()

**output_saver.rs**
  fn standalone_file() -> String
  fn sentinel_wrapped() -> String

  enum Target
  fn resolve_target_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Target
  fn scan_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Status
  fn install_agent_with_home(
  fn remove_agent_with_home(
  fn replace_between(content: &str, start: &str, end: &str, new_block: &str) -> String
  fn agent_display(id: &str) -> &'static str
  fn run_scan(targets: &[&str])
  fn run_install(targets: &[&str])
  fn run_refresh(targets: &[&str])
  fn run_remove(targets: &[&str])
  mod tests
  fn replace_between_swaps_segment()
  fn standalone_file_contains_block()
  fn sentinel_wrapped_is_idempotent_on_replace()
  fn scan_unknown_agent_returns_unsupported()
  fn install_and_remove_imported_agent_roundtrip()
  fn install_inline_file_is_idempotent()

**process.rs**
  mod helpers
  pub struct ProcessOutput

  impl ProcessOutput
  pub fn success(&self) -> bool
  pub fn code(&self) -> i32
  pub fn has_stdout(&self) -> bool
  pub fn has_stderr(&self) -> bool
  pub enum ProcessError

  impl std::fmt::Display for ProcessError
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result

  impl std::error::Error for ProcessError

  impl ProcessError
  pub fn exit_code(&self) -> Option<i32>
  pub fn is_timeout(&self) -> bool
  pub fn is_command_not_found(&self) -> bool
  pub fn is_permission_denied(&self) -> bool
  pub struct ProcessBuilder

  impl ProcessBuilder
  pub fn new(command: impl Into<String>) -> Self
  pub fn arg(mut self, arg: impl Into<String>) -> Self
  pub fn args<I, S>(mut self, args: I) -> Self
  pub fn current_dir(mut self, dir: impl Into<PathBuf>) -> Self
  pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
  pub fn envs<I, K, V>(mut self, vars: I) -> Self
  pub fn env_clear(mut self, clear: bool) -> Self
  pub fn timeout(mut self, timeout: Duration) -> Self
  pub fn capture_stdout(mut self, capture: bool) -> Self
  pub fn capture_stderr(mut self, capture: bool) -> Self
  pub fn capture_exit_code(mut self, capture: bool) -> Self
  pub fn capture_duration(mut self, capture: bool) -> Self
  pub fn run(&self) -> Result<ProcessOutput, ProcessError>
  pub fn run_checked(&self) -> Result<ProcessOutput, ProcessError>
  pub fn run(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>
  pub fn run_checked(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>
  pub fn run_with_timeout(
  mod tests

**process_helpers.rs**
  fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<ExitStatus>>

  impl ChildExt for std::process::Child

**process_tests.rs**
  fn test_process_output_success()
  fn test_process_output_failure()
  fn test_process_output_no_exit_code()
  fn test_process_error_display()
  fn test_process_error_helpers()
  fn test_process_builder_basic()
  fn test_process_builder_args_iter()
  fn test_process_builder_env()
  fn test_process_builder_envs()
  fn test_process_builder_timeout()
  fn test_process_builder_capture()
  fn test_run_echo()
  fn test_run_command_not_found()
  fn test_run_non_zero_exit()
  fn test_run_checked_non_zero_exit()
  fn test_run_with_args()
  fn test_run_with_env()
  fn test_run_with_working_dir()
  fn test_run_with_timeout_success()
  fn test_run_with_timeout_exceeded()
  fn test_run_checked_success()
  fn test_run_capture_stderr()
  fn test_run_no_capture_stdout()
  fn test_run_no_capture_stderr()
  fn test_run_no_capture_exit_code()
  fn test_run_capture_exit_code_default()
  fn test_run_capture_exit_code_non_zero()
  fn test_run_no_capture_duration()
  fn test_run_capture_duration_default()
  fn test_process_builder_env_clear()
  fn test_process_output_has_output()

**rewrite.rs**
  fn handle_json_protocol(json: &serde_json::Value)

  enum HookEvent
  impl HookEvent
  fn parse(name: &str) -> Self
  fn agent_label(&self) -> &'static str
  fn build_hook_response(json: &serde_json::Value) -> Option<serde_json::Value>
  fn split_env_prefix(cmd: &str) -> Option<(String, &str)>
  fn looks_like_env_assignment(token: &str) -> bool
  fn tag_with_agent(cmd: &str, agent: &str) -> String
  fn maybe_rewrite(cmd: &str) -> Option<String>
  fn split_at_shell_op(s: &str) -> Option<(&str, &str)>
  mod tests
  fn test_rewrite_git()
  fn test_rewrite_cargo()
  fn test_skip_already_trs()
  fn test_skip_cd()
  fn test_skip_trs_skip_env_var()
  fn test_trs_skip_does_not_match_other_env_vars()
  fn test_env_prefix_stays_in_front()
  fn test_multi_env_prefix()
  fn test_env_prefix_before_unknown_command_still_rewrites()
  fn test_flag_looks_like_assignment_not_matched()
  fn test_split_env_prefix_empty_when_none()
  fn test_stderr_redirects_survive_rewrite()
  fn test_rewrite_pipe_first_segment()
  fn test_rewrite_multi_pipe_first_segment_only()
  fn test_rewrite_redirect_first_segment()
  fn test_skip_pipe_when_first_segment_is_skipped()
  fn test_skip_subshells()
  fn test_skip_assignments()
  fn test_skip_empty()
  fn test_rewrite_unknown_command()
  fn test_json_protocol()
  fn test_skip_echo()
  fn test_rewrite_env()
  fn test_skip_shell_builtins()
  fn test_rewrite_cd_chain()
  fn test_rewrite_multi_chain()
  fn test_skip_cd_chain_with_pipe()
  fn test_skip_cd_chain_all_skips()
  fn parse_input(s: &str) -> serde_json::Value
  fn test_hook_response_claude_code_format()
  fn test_hook_response_cursor_format()
  fn test_hook_response_gemini_format()
  fn test_hook_response_default_is_claude_format()
  fn test_hook_response_no_rewrite_returns_none()
  fn test_hook_response_missing_command_returns_none()
  fn test_hook_response_chain_preserved_across_formats()

**tracker.rs**
  pub struct HistoryEntry
  fn history_path() -> Option<PathBuf>
  fn dirs_path() -> Option<PathBuf>
  pub fn log_execution(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64)
  pub fn read_history() -> Vec<HistoryEntry>
  pub fn read_project_history() -> Vec<HistoryEntry>
  pub fn format_bytes_human(bytes: usize) -> String
  mod tests
  fn test_format_bytes_human()
  fn test_saved_pct_calculation()
  fn test_history_entry_serialization()

**upgrade.rs**
  impl InstallMethod
  fn label(&self) -> &'static str
  fn prompt_yes(question: &str) -> bool
  fn run_shell(cmd: &str) -> bool
  fn refresh_configs()
  fn first_broken_hook_json() -> Option<std::path::PathBuf>
  mod tests
  fn detects_npm_install()
  fn detects_curl_install_default_path()
  fn detects_curl_install_trs_bin_path()
  fn detects_brew_install()
  fn detects_cargo_install()
  fn unknown_for_built_from_source()
  fn unknown_for_none()

## src/formatter/

**agent.rs**
  pub struct AgentFormatter

  impl Formatter for AgentFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl AgentFormatter
  pub fn section_header(title: &str) -> String
  pub fn subsection_header(title: &str) -> String
  pub fn list_item(item: &str, label: Option<&str>) -> String
  pub fn key_value_item(key: &str, value: &str, label: Option<&str>) -> String
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_counts(label: &str, counts: &[(&str, usize)]) -> String
  pub fn format_section_header(name: &str, count: Option<usize>) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> String
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String
  pub fn format_error(message: &str) -> String
  pub fn format_error_with_code(message: &str, exit_code: i32) -> String
  pub fn format_not_implemented(message: &str) -> String
  pub fn format_command_result(
  pub fn format_list(items: &[impl AsRef<str>]) -> String
  pub fn format_count(count: usize) -> String
  pub fn format_flag(name: &str, value: bool) -> String
  pub fn format_array(items: &[impl AsRef<str>]) -> String
  pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String
  pub fn format_key_value(key: &str, value: &str) -> String
  pub fn format_metadata(items: &[(&str, &str)]) -> String
  pub fn format_code_block(code: &str, language: Option<&str>) -> String
  pub fn format_divider() -> String
  pub fn format_bold(text: &str) -> String
  pub fn format_italic(text: &str) -> String
  pub fn format_code_inline(text: &str) -> String
  pub fn format_link(text: &str, url: &str) -> String
  pub fn start_document(title: &str) -> String

**agent_schema.rs**
  impl AgentFormatter
  pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String
  pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String
  pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String
  pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String
  pub fn format_find(find: &crate::schema::FindOutputSchema) -> String
  pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String
  pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String
  pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String
  pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String
  pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String

**compact.rs**
  pub struct CompactFormatter

  impl Formatter for CompactFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl CompactFormatter
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_counts(label: &str, counts: &[(&str, usize)]) -> String
  pub fn format_section_header(name: &str, count: Option<usize>) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> &'static str
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String

**compact_schema_git.rs**
  impl CompactFormatter
  pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String
  pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String

**compact_schema_output.rs**
  impl CompactFormatter
  pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String
  pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String
  pub fn format_find(find: &crate::schema::FindOutputSchema) -> String
  pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String
  pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String
  pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String
  pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String
  pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String

**csv.rs**
  pub struct CsvFormatter

  impl Formatter for CsvFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl CsvFormatter
  pub fn escape_field(field: &str) -> String
  pub fn format_header(fields: &[&str]) -> String
  pub fn format_row(values: &[&str]) -> String
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_key_value(key: &str, value: &str) -> String
  pub fn format_object(pairs: &[(&str, &str)]) -> String
  pub fn format_counts(counts: &[(&str, usize)]) -> String
  pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> String
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String
  pub fn format_error(message: &str) -> String
  pub fn format_error_with_code(message: &str, exit_code: i32) -> String
  pub fn format_not_implemented(message: &str) -> String
  pub fn format_command_result(
  pub fn format_list(items: &[impl AsRef<str>]) -> String
  pub fn format_count(count: usize) -> String
  pub fn format_flag(name: &str, value: bool) -> String
  pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String

**csv_schema.rs**
  impl CsvFormatter
  pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String
  pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String
  pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String
  pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String
  pub fn format_find(find: &crate::schema::FindOutputSchema) -> String
  pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String
  pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String
  pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String
  pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String
  pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String

**helpers.rs**
  #[allow(dead_code)]
  pub fn format_count_if_positive(label: &str, count: usize) -> Option<String> {
      if count > 0 {
          Some(format!("{}={}", label, count))
      } else {
          None
      }
  }

  #[allow(dead_code)]
  pub fn format_list_with_count(label: &str, items: &[String]) -> String {
      let mut output = String::new();
      if !items.is_empty() {
          output.push_str(&format!("{} ({}):\n", label, items.len()));
          for item in items {
              output.push_str(&format!("  {}\n", item));
          }
      }
      output
  }

  #[allow(dead_code)]
  pub fn format_key_value(key: &str, value: &str, label: Option<&str>) -> String {
      match label {
          Some(l) => format!("{} [{}]: {}\n", key, l, value),
          None => format!("{}: {}\n", key, value),
      }
  }

  #[allow(dead_code)]
  pub fn format_line(key: &str, value: impl std::fmt::Display) -> String {
      format!("{}: {}\n", key, value)
  }

  pub fn truncate(s: &str, max_len: usize) -> String {
      if s.len() <= max_len {
          s.to_string()
      } else {
          let mut end = max_len.saturating_sub(3);
          while end > 0 && !s.is_char_boundary(end) {
              end -= 1;
          }
          format!("{}...", &s[..end])
      }
  }

  #[allow(dead_code)]
  pub fn format_duration(ms: u64) -> String {
      if ms < 1000 {
          format!("{}ms", ms)
      } else if ms < 60000 {
          format!("{:.2}s", ms as f64 / 1000.0)
      } else {
          let mins = ms / 60000;
          let secs = (ms % 60000) / 1000;
          format!("{}m {}s", mins, secs)
      }
  }

  #[allow(dead_code)]
  pub fn format_bytes(bytes: usize) -> String {
      const KB: usize = 1024;
      const MB: usize = 1024 * KB;
      const GB: usize = 1024 * MB;

      if bytes >= GB {
          format!("{:.2}GB", bytes as f64 / GB as f64)
      } else if bytes >= MB {
          format!("{:.2}MB", bytes as f64 / MB as f64)
      } else if bytes >= KB {
          format!("{:.2}KB", bytes as f64 / KB as f64)
      } else {
          format!("{}B", bytes)
      }
  }

**json.rs**
  pub struct JsonFormatter

  impl Formatter for JsonFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl JsonFormatter
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_key_value(key: &str, value: impl serde::Serialize) -> String
  pub fn format_object(pairs: &[(&str, serde_json::Value)]) -> String
  pub fn format_counts(counts: &[(&str, usize)]) -> String
  pub fn format_section(name: &str, items: &[impl serde::Serialize]) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> String
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String
  pub fn format_error(message: &str) -> String
  pub fn format_error_with_code(message: &str, exit_code: i32) -> String
  pub fn format_not_implemented(message: &str) -> String
  pub fn format_command_result(
  pub fn format_list(items: &[impl AsRef<str>]) -> String
  pub fn format_count(count: usize) -> String
  pub fn format_flag(name: &str, value: bool) -> String
  pub fn format_array<T: serde::Serialize>(items: &[T]) -> String

**json_schema.rs**

  #[allow(dead_code)]
  impl JsonFormatter {
      pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
          serde_json::to_string_pretty(status).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
          serde_json::to_string_pretty(diff).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
          serde_json::to_string_pretty(ls).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
          serde_json::to_string_pretty(grep).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
          serde_json::to_string_pretty(find).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
          serde_json::to_string_pretty(test).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
          serde_json::to_string_pretty(logs).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
          serde_json::to_string_pretty(state).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
          serde_json::to_string_pretty(process).unwrap_or_else(|_| "{}".to_string())
      }

      pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
          serde_json::to_string_pretty(error).unwrap_or_else(|_| "{}".to_string())
      }
  }

**mod.rs**
  mod agent;
  mod agent_schema;
  mod compact;
  mod compact_schema_git;
  mod compact_schema_output;
  mod csv;
  mod csv_schema;
  pub mod helpers;
  mod json;
  mod json_schema;
  mod raw;
  mod tsv;
  mod tsv_schema;

  #[cfg(test)]
  mod tests;

  pub use agent::AgentFormatter;
  pub use compact::CompactFormatter;
  pub use csv::CsvFormatter;
  #[allow(unused_imports)]
  pub use helpers::*;
  pub use json::JsonFormatter;
  pub use raw::RawFormatter;
  pub use tsv::TsvFormatter;


  #[allow(dead_code)]
  pub trait Formatter {
      fn name() -> &'static str;

      fn format() -> OutputFormat;
  }

  #[allow(dead_code)]
  pub fn select_formatter(format: OutputFormat) -> &'static str {
      match format {
          OutputFormat::Json => JsonFormatter::name(),
          OutputFormat::Csv => CsvFormatter::name(),
          OutputFormat::Tsv => TsvFormatter::name(),
          OutputFormat::Agent => AgentFormatter::name(),
          OutputFormat::Compact => CompactFormatter::name(),
          OutputFormat::Raw => RawFormatter::name(),
      }
  }

**raw.rs**
  pub struct RawFormatter

  impl Formatter for RawFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl RawFormatter
  pub fn format_list(items: &[impl AsRef<str>]) -> String
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_counts(counts: &[(&str, usize)]) -> String
  pub fn format_section_header(name: &str, count: Option<usize>) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> &'static str
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String
  pub fn format_key_value(key: &str, value: &str) -> String
  pub fn format_raw(content: &str) -> String

**tsv.rs**
  pub struct TsvFormatter

  impl Formatter for TsvFormatter
  fn name() -> &'static str
  fn format() -> OutputFormat

  impl TsvFormatter
  pub fn escape_field(field: &str) -> String
  pub fn format_header(fields: &[&str]) -> String
  pub fn format_row(values: &[&str]) -> String
  pub fn format_message(key: &str, value: &str) -> String
  pub fn format_key_value(key: &str, value: &str) -> String
  pub fn format_object(pairs: &[(&str, &str)]) -> String
  pub fn format_counts(counts: &[(&str, usize)]) -> String
  pub fn format_section(status_col: &str, path_col: &str, items: &[(&str, &str)]) -> String
  pub fn format_item(status: &str, path: &str) -> String
  pub fn format_item_renamed(status: &str, old_path: &str, new_path: &str) -> String
  pub fn format_test_summary(
  pub fn format_status(success: bool) -> String
  pub fn format_failures(failures: &[String]) -> String
  pub fn format_log_levels(error: usize, warn: usize, info: usize, debug: usize) -> String
  pub fn format_grep_match(file: &str, line: Option<usize>, content: &str) -> String
  pub fn format_grep_file(file: &str, match_count: usize) -> String
  pub fn format_diff_file(
  pub fn format_diff_summary(
  pub fn format_clean() -> String
  pub fn format_dirty(
  pub fn format_branch_with_tracking(branch: &str, ahead: usize, behind: usize) -> String
  pub fn format_empty() -> String
  pub fn format_truncated(shown: usize, total: usize) -> String
  pub fn format_error(message: &str) -> String
  pub fn format_error_with_code(message: &str, exit_code: i32) -> String
  pub fn format_not_implemented(message: &str) -> String
  pub fn format_command_result(
  pub fn format_list(items: &[impl AsRef<str>]) -> String
  pub fn format_count(count: usize) -> String
  pub fn format_flag(name: &str, value: bool) -> String
  pub fn format_table(headers: &[&str], rows: &[Vec<&str>]) -> String

**tsv_schema.rs**
  impl TsvFormatter
  pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String
  pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String
  pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String
  pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String
  pub fn format_find(find: &crate::schema::FindOutputSchema) -> String
  pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String
  pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String
  pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String
  pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String
  pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String

## src/ingest/

**collect.rs**: (247 lines)
**collect_compress.rs**
  fn has_multiline_python_sig(content: &str) -> bool
  fn join_python_multiline_sigs(content: &str) -> String
  fn extract_signatures(content: &str, ext: &str) -> String
  fn clean_signature(line: &str) -> String

**collect_index.rs**
  fn contains_manifest_field(s: &str) -> bool
  fn looks_like_config_line(s: &str) -> bool
  fn first_rust_module_doc(content: &str) -> Option<String>
  fn first_python_docstring(content: &str) -> Option<String>
  fn first_jsdoc_summary(content: &str) -> Option<String>
  fn symbol_from_rust(line: &str) -> Option<String>
  fn symbol_from_python(line: &str) -> Option<String>
  fn symbol_from_ts(line: &str) -> Option<String>
  fn symbol_from_go(line: &str) -> Option<String>
  fn symbol_from_swift(line: &str) -> Option<String>
  fn symbol_from_java(line: &str) -> Option<String>
  fn first_ident(s: &str) -> Option<String>

**collect_manifests.rs**: (277 lines)
**deps.rs**
  impl DepGraph
  pub fn top_central(&self, n: usize) -> Vec<(&str, &Vec<String>)>
  pub fn is_empty(&self) -> bool
  fn resolve_imports(
  fn resolve_relative(import: &str, importer_dir: &str, all_paths: &[&str]) -> Option<String>
  fn resolve_by_suffix(suffix: &str, all_paths: &[&str]) -> Option<String>
  fn resolve_by_stem(name: &str, stem_index: &HashMap<String, Vec<&str>>) -> Option<String>
  fn resolve_module_path(import: &str, all_paths: &[&str]) -> Option<String>
  fn normalize_path(base: &str, import: &str) -> String
  fn short_label(rel_path: &str, all_paths: &[&str]) -> String
  mod tests
  fn test_normalize_path()
  fn test_build_dep_graph_empty()
  fn test_dep_summary_skips_singletons()

**deps_extract.rs**
  fn extract_rust(content: &str) -> Vec<String>
  fn extract_ts(content: &str) -> Vec<String>
  fn extract_from_path(line: &str) -> Option<String>
  fn extract_python(content: &str) -> Vec<String>
  fn extract_go(content: &str) -> Vec<String>
  fn is_go_project_import(path: &str) -> bool
  mod tests
  fn test_extract_rust_imports()
  fn main()
  fn test_extract_rust_mod()
  fn test_extract_ts_imports()
  fn test_extract_python_imports()

**format.rs**: fn format_file_entry(out: &mut String, name: &str, content: &str) | fn collect_dir_annotations(files: &[DigestFile]) -> BTreeMap<String, String>
**meta.rs**


  #[derive(Debug, Clone, Serialize, Deserialize, Default)]
  pub(crate) struct IngestMeta {
      pub head_sha: Option<String>,
      pub timestamp: u64,
      pub file_count: usize,
      pub tokens: usize,
      pub project_root: String,
      pub trs_version: String,
  }

  pub(crate) fn get_head_sha(root: &Path) -> Option<String> {
      let output = Command::new("git")
          .args(["rev-parse", "--short=7", "HEAD"])
          .current_dir(root)
          .output()
          .ok()?;
      if !output.status.success() {
          return None;
      }
      let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
      if sha.is_empty() {
          None
      } else {
          Some(sha)
      }
  }

  pub(crate) fn load_meta(digest_path: &Path) -> Option<IngestMeta> {
      let meta_path = meta_path_for(digest_path);
      let content = std::fs::read_to_string(&meta_path).ok()?;
      serde_json::from_str(&content).ok()
  }

  pub(crate) fn save_meta(digest_path: &Path, meta: &IngestMeta) -> std::io::Result<()> {
      let meta_path = meta_path_for(digest_path);
      let json = serde_json::to_string_pretty(meta).map_err(std::io::Error::other)?;
      std::fs::write(meta_path, json)
  }

  fn meta_path_for(digest_path: &Path) -> PathBuf {
      digest_path.with_extension("json")
  }

  pub(crate) fn commits_since(root: &Path, stored_sha: &str) -> Option<usize> {
      let output = Command::new("git")
          .args(["rev-list", "--count", &format!("{}..HEAD", stored_sha)])
          .current_dir(root)
          .output()
          .ok()?;
      if !output.status.success() {
          return None;
      }
      let n_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
      n_str.parse::<usize>().ok()
  }

  pub(crate) fn now_unix() -> u64 {
      std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map(|d| d.as_secs())
          .unwrap_or(0)
  }

**mod.rs**
  mod collect
  mod collect_compress
  mod collect_index
  mod collect_manifests
  mod deps
  mod deps_extract
  mod format
  mod meta
  mod ollama
  mod remote
  mod store
  pub use remote::{is_remote_ref, resolve_remote, TmpMode
  pub use store::{list_ingests, read_digest
  pub enum IngestLevel

  impl IngestLevel
  pub fn from_str(s: &str) -> Self
  pub struct IngestConfig
  pub fn resolve_project_root(path: &Path) -> Result<PathBuf>
  fn suggest_budget(n: usize) -> &'static str
  pub fn run_ingest(config: &IngestConfig)
  mod tests
  fn test_format_tokens()
  fn test_format_bytes()
  fn test_ingest_level_from_str()
  fn test_skip_extensions()
  fn test_skip_files()
  fn test_build_tree()

**ollama.rs**
  pub fn list_ollama_models()
  fn get_ollama_models() -> Option<Vec<(String, String, String)>>
  fn pick_default_model() -> Option<String>

**remote.rs**
  pub struct ResolvedSource
  pub fn is_remote_ref(s: &str) -> bool
  fn is_owner_repo_shorthand(s: &str) -> bool
  fn is_ident_char(c: char) -> bool
  pub enum TmpMode
  pub fn resolve_remote(input: &str, tmp: TmpMode) -> Result<ResolvedSource>
  fn normalize_to_url(input: &str) -> Result<String>
  fn owner_repo_from(url: &str) -> Result<(String, String)>
  fn owner_repo_slash(url: &str) -> Result<String>
  pub fn has_spark() -> bool
  fn spark_clone(url: &str) -> Result<()>
  fn spark_search_first(owner_repo: &str) -> Result<PathBuf>
  fn git_shallow_clone(url: &str, repo: &str) -> Result<TempDir>
  mod tests
  fn detects_remote_refs()
  fn rejects_local_paths()
  fn normalizes_shorthand()
  fn extracts_owner_repo()

**store.rs**
  struct ListEntry
  pub fn list_ingests()
  pub fn read_digest(name: Option<&str>, project_path: &Path)
  fn ingest_store_dir() -> Option<PathBuf>
  fn get_repo_identity(root: &Path) -> (String, String)
  fn get_repo_name(root: &Path) -> String

## src/reducer/

**mod.rs**
  mod registry
  mod truncation
  mod tests
  pub use output::{ReducerItem, ReducerMetadata, ReducerOutput, ReducerSection, ReducerStats
  pub use registry::{BaseReducer, ReducerRegistry
  pub use truncation::{TruncationConfig, TruncationInfo
  pub struct ReducerContext

  impl ReducerContext
  pub fn from_cli(cli: &crate::Cli) -> Self
  pub fn has_conflicting_formats(&self) -> bool
  pub type ReducerResult<T = ()> = Result<T, ReducerError>
  pub enum ReducerError

  impl ReducerError
  pub fn is_not_implemented(&self) -> bool
  pub fn is_invalid_input(&self) -> bool
  pub fn is_io_error(&self) -> bool
  pub fn is_processing_error(&self) -> bool

  impl std::fmt::Display for ReducerError
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result

  impl std::error::Error for ReducerError
  pub trait Reducer
  type Input
  type Output
  fn reduce(&self, input: &Self::Input, context: &ReducerContext) -> ReducerResult<Self::Output>
  fn name(&self) -> &'static str

**output.rs**
  pub struct ReducerMetadata
  fn estimate_tokens(bytes: usize) -> usize
  pub struct ReducerStats

  impl ReducerStats
  pub fn new(
  pub struct ReducerOutput

  impl ReducerOutput
  pub fn new<T: Serialize>(data: T) -> Self
  pub fn empty() -> Self
  pub fn with_metadata(mut self, metadata: ReducerMetadata) -> Self
  pub fn with_stats(mut self, stats: ReducerStats) -> Self
  pub fn with_summary(mut self, summary: impl Into<String>) -> Self
  pub fn with_items(mut self, items: Vec<ReducerItem>) -> Self
  pub fn with_sections(mut self, sections: Vec<ReducerSection>) -> Self
  pub fn format(&self, context: &ReducerContext) -> String
  pub fn format_json(&self) -> String
  pub fn format_csv(&self) -> String
  pub fn format_tsv(&self) -> String
  pub fn format_agent(&self) -> String
  pub fn format_compact(&self) -> String
  pub fn format_raw(&self) -> String
  pub struct ReducerItem

  impl ReducerItem
  pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self
  pub fn with_label(mut self, label: impl Into<String>) -> Self
  pub fn with_data(mut self, data: serde_json::Value) -> Self
  pub struct ReducerSection

  impl ReducerSection
  pub fn new(name: impl Into<String>) -> Self
  pub fn with_count(mut self, count: usize) -> Self
  pub fn add_item(&mut self, item: ReducerItem)

**registry.rs**
  pub struct BaseReducer<T: Serialize>
  pub fn new(name: &'static str) -> Self
  pub fn format_json(output: &T) -> String
  pub fn format_compact(output: &T) -> String
  pub fn format_raw(output: &T) -> String
  type Input = String
  type Output = T
  fn reduce(
  fn name(&self) -> &'static str
  type ReducerFn = Box<dyn Fn(&str, &ReducerContext) -> ReducerResult<ReducerOutput>>
  pub struct ReducerRegistry

  impl ReducerRegistry
  pub fn new() -> Self
  pub fn register<R>(&mut self, reducer: R)
  pub fn execute(
  pub fn reducer_names(&self) -> Vec<&'static str>

**truncation.rs**
  pub struct TruncationInfo

  impl TruncationInfo
  pub fn none() -> Self
  pub fn limited(total: usize, shown: usize, limit: usize) -> Self
  pub fn size_threshold(total_bytes: usize, shown_bytes: usize, threshold_bytes: usize) -> Self
  pub fn detected(pattern: &str, original_size: usize) -> Self
  pub fn detect_from_output(output: &str) -> Self
  fn detect_incomplete_json(output: &str) -> bool
  fn detect_cutoff_line(output: &str) -> bool
  pub fn is_truncated(&self) -> bool
  pub fn summary(&self) -> Option<String>
  pub struct TruncationConfig

  impl Default for TruncationConfig
  fn default() -> Self

  impl TruncationConfig
  pub fn with_max_items(max_items: usize) -> Self
  pub fn with_max_bytes(max_bytes: usize) -> Self
  pub fn truncate_items<T>(&self, items: Vec<T>) -> (Vec<T>, TruncationInfo)
  pub fn truncate_output(&self, output: String) -> (String, TruncationInfo)

## src/router/

**mod.rs**
  pub use handlers::common::{CommandContext, CommandError, CommandResult
  pub struct Router

  impl Router
  pub fn new() -> Self
  pub fn route(&self, command: &Commands, ctx: &CommandContext) -> CommandResult
  pub fn execute_and_print(&self, command: &Commands, ctx: &CommandContext)
  pub fn process_stdin(&self, input: &str, ctx: &CommandContext) -> CommandResult<String>
  fn format_not_implemented(msg: &str, format: OutputFormat) -> String
  fn format_command_error(error: &CommandError, format: OutputFormat) -> String

  impl Default for Router
  fn default() -> Self
  mod tests

## src/router/handlers/

**clean.rs**
  impl CleanHandler

  impl CommandHandler for CleanHandler
  type Input = CleanInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**common.rs**
  fn is_emoji(c: char) -> bool
  pub struct CommandContext

  impl CommandContext
  pub fn default_compact() -> Self
  pub fn from_cli(cli: &Cli) -> Self
  pub fn has_conflicting_formats(&self) -> bool
  pub type CommandResult<T = ()> = Result<T, CommandError>
  pub enum CommandError

  impl CommandError
  pub fn exit_code(&self) -> Option<i32>

  impl std::fmt::Display for CommandError
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result

  impl std::error::Error for CommandError
  pub struct CommandStats

  impl CommandStats
  pub fn new() -> Self
  pub fn with_input_bytes(mut self, bytes: usize) -> Self
  pub fn with_output_bytes(mut self, bytes: usize) -> Self
  pub fn with_items_processed(mut self, count: usize) -> Self
  pub fn with_items_filtered(mut self, count: usize) -> Self
  pub fn with_duration_ms(mut self, ms: u64) -> Self
  pub fn with_command(mut self, cmd: impl Into<String>) -> Self
  pub fn with_exit_code(mut self, code: i32) -> Self
  pub fn with_reducer(mut self, reducer: impl Into<String>) -> Self
  pub fn with_output_mode(mut self, mode: OutputFormat) -> Self
  pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
  pub fn reduction_percent(&self) -> f64
  pub fn token_reduction_percent(&self) -> f64
  pub fn print(&self)

**err.rs**: fn is_error_line(line: &str) -> bool | fn is_error_not_warning(line: &str) -> bool
**html2md.rs**
  impl Html2mdHandler

  impl CommandHandler for Html2mdHandler
  type Input = Html2mdInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**isclean.rs**
  impl IsCleanHandler

  impl CommandHandler for IsCleanHandler
  type Input = IsCleanInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**json.rs**
  mod json_query

  impl JsonHandler
  fn non_json_hint(file: &Option<PathBuf>, err: &serde_json::Error) -> CommandError
  fn read_json_input(file: &Option<PathBuf>) -> CommandResult<String>
  fn is_id_key(key: &str) -> bool
  fn has_error_keys(value: &serde_json::Value) -> bool
  fn sample_array(arr: &[serde_json::Value]) -> (Vec<&serde_json::Value>, bool)
  fn format_structure(value: &serde_json::Value, buf: &mut String, depth: usize, max_depth: usize)
  fn all_same_type(arr: &[serde_json::Value]) -> bool
  fn value_type(value: &serde_json::Value) -> &'static str
  fn to_schema_json(value: &serde_json::Value, depth: usize, max_depth: usize) -> serde_json::Value
  mod tests

**json_query.rs**
  enum QuerySegment
  fn parse_query(query: &str) -> Result<Vec<QuerySegment>, CommandError>
  fn resolve_segments(
  fn value_type(value: &serde_json::Value) -> &'static str

**json_tests.rs**
  fn test_simple_object()
  fn test_array_of_objects()
  fn test_empty_structures()
  fn test_nested_depth_limit()
  fn test_long_string_shows_length()
  fn test_null_value()
  fn test_schema_json_output()
  fn test_small_array_no_sampling()
  fn test_large_array_sampling()
  fn test_large_array_sampling_numbers()
  fn test_sample_array_preserves_error_items()
  fn test_sample_array_exactly_at_threshold()
  fn test_sample_array_just_above_threshold()
  fn test_large_array_json_schema_output()
  fn test_id_key_detection()
  fn test_id_annotation_in_structure()
  fn test_id_annotation_nested_object()
  fn test_id_annotation_json_schema()
  fn test_non_json_hint_toml()
  fn test_non_json_hint_yaml()
  fn test_non_json_hint_yml()
  fn test_non_json_hint_csv()
  fn test_non_json_hint_unknown_ext()
  fn test_non_json_hint_no_file()
  fn test_has_error_keys()
  fn test_combined_large_array_with_id_fields()
  fn test_query_key()
  fn test_query_nested()
  fn test_query_array_index()
  fn test_query_iterate()
  fn test_query_root()
  fn test_query_missing_key()
  fn test_query_index_out_of_bounds()
  fn test_query_format_string()
  fn test_query_format_array_primitives()
  fn test_query_format_number()

**mod.rs**
  pub mod clean;
  pub mod common;
  pub mod err;
  pub mod html2md;
  pub mod isclean;
  pub mod json;
  pub mod parse;
  pub mod read;
  pub mod read_filters;
  pub mod replace;
  pub mod run;
  pub mod search;
  pub mod stats;
  pub mod tail;
  pub mod trim;
  pub mod txt2md;
  pub mod types;

**read.rs**: impl ReadHandler | mod tests
**read_filters.rs**
  fn is_comment_line(line: &str, lang: Language) -> bool
  fn is_import_line(trimmed: &str, lang: Language) -> bool
  fn is_decorator(trimmed: &str, lang: Language) -> bool
  fn is_definition_line(trimmed: &str, lang: Language) -> bool
  fn is_type_or_const(trimmed: &str, lang: Language) -> bool

**read_tests.rs**
  fn test_detect_language()
  fn test_minimal_filter_strips_comments()
  fn test_minimal_filter_preserves_todo()
  fn test_aggressive_filter_rust()
  pub fn hello(name: &str) -> String
  pub struct Config
  fn test_aggressive_filter_python()

  class MyClass
  fn test_data_files_passthrough()
  fn test_line_range_head()
  fn test_line_range_tail()
  fn test_count_braces_ignores_strings()

**replace.rs**
  impl ReplaceHandler

  impl CommandHandler for ReplaceHandler
  type Input = ReplaceInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**run.rs**
  impl RunHandler

  impl CommandHandler for RunHandler
  type Input = RunInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

  impl From<(&String, &Vec<String>, bool, bool, bool, bool, Option<u64>)> for RunInput
  fn from(

**search.rs**
  impl SearchHandler

  struct MatchResult
  struct MatchSink
  impl MatchSink
  fn new(matcher: RegexMatcher) -> Self

  impl Sink for MatchSink
  type Error = std::io::Error
  fn matched(
  fn context(

  impl CommandHandler for SearchHandler
  type Input = SearchInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**stats.rs**
  fn local_offset() -> time::UtcOffset
  fn today_date_label(offset: time::UtcOffset) -> String
  fn format_date(ts: u64) -> String
  fn format_timestamp(ts: u64, offset: time::UtcOffset) -> String
  pub struct StatsInput

  struct CommandAgg
  impl CommandAgg
  fn saved(&self) -> usize
  fn avg_reduction_pct(&self) -> f64
  pub fn handle_stats(input: &StatsInput)
  fn print_by_agent(entries: &[HistoryEntry])
  fn print_summary(entries: &[HistoryEntry], top_limit: usize)
  fn today_entries(entries: &[HistoryEntry], offset: time::UtcOffset) -> Vec<&HistoryEntry>
  fn print_history(entries: &[HistoryEntry], limit: usize)
  fn display_cmd(cmd: &str) -> String
  fn print_json(
  fn truncate_cmd(cmd: &str, max_len: usize) -> String

**tail.rs**
  impl TailHandler
  fn format_body_lines(output: &TailOutput) -> String

  impl CommandHandler for TailHandler
  type Input = TailInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**trim.rs**
  impl TrimHandler

  impl CommandHandler for TrimHandler
  type Input = TrimInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

## src/router/handlers/parse/

**brew.rs**
  impl ParseHandler
  fn is_progress_bar(line: &str) -> bool
  fn condense_keg_line(line: &str) -> String
  fn format_brew_compact(installed: &[String], errors: &[String], warnings: &[String]) -> String

**bun_format.rs**: impl ParseHandler
**bun_parse.rs**: impl ParseHandler
**extra_cargo_test.rs**: impl ParseHandler
**extra_db.rs**
  impl ParseHandler
  fn parse_psql(input: &str) -> DbResult
  fn parse_mysql(input: &str) -> DbResult
  fn parse_sqlite(input: &str) -> DbResult
  fn format_row_compact(cells: &[String]) -> String
  fn format_db_compact(result: &DbResult) -> String
  fn csv_escape(field: &str) -> String

**extra_download.rs**
  impl ParseHandler
  fn looks_like_http_trace(input: &str) -> bool
  fn compress_http_body(input: &str) -> String
  fn decode_github_content(val: &serde_json::Value) -> Option<String>
  fn base64_decode(s: &str) -> Option<Vec<u8>>
  mod tests
  fn detects_http_trace()
  fn compact_json_body_reduces_size()
  fn github_contents_base64_is_decoded()
  fn unknown_body_is_passthrough()
  fn base64_decode_handles_padding()
  fn base64_decode_rejects_invalid()

**extra_env.rs**
  impl ParseHandler
  fn is_env_noise(key: &str) -> bool
  fn is_secret_key(key: &str) -> bool
  fn env_category(key: &str) -> &'static str

**extra_network.rs**: impl ParseHandler | fn format_ping_compact(
**extra_services.rs**: impl ParseHandler | fn truncate_str(s: &str, max_len: usize) -> String
**extra_system.rs**: impl ParseHandler
**find.rs**: impl ParseHandler | fn common_path_prefix(paths: &[&str]) -> String
**git_branch.rs**
  impl ParseHandler
  fn render_compact(current: &str, local: &[String], remote: &[String]) -> String
  fn render_group(out: &mut String, branches: &[&String], indent: &str)

**git_diff.rs**: impl ParseHandler | fn parse_git_diff_stat(input: &str) -> CommandResult<GitDiff>
**git_diff_format.rs**
  impl ParseHandler
  fn format_hunk_compressed(hunk: &GitDiffHunk) -> Vec<String>
  fn build_file_summary(diff: &GitDiff) -> String

**git_log.rs**
  fn apply_truncate(subject: &str, max: Option<usize>) -> String

  impl ParseHandler
  fn extract_subject(msg: &[String]) -> String
  fn relative_time(date_str: &str) -> String

**git_status.rs**: impl ParseHandler
**git_status_format.rs**
  impl ParseHandler
  fn format_entries_capped(entries: &[GitStatusEntry], max: usize, output: &mut String)
  fn format_entries_listed(entries: &[GitStatusEntry], max: usize, output: &mut String)
  fn format_entries_grouped(entries: &[GitStatusEntry], output: &mut String)
  mod tests
  fn make_entry(status: &str, path: &str) -> GitStatusEntry
  fn make_entries(status: &str, paths: &[&str]) -> Vec<GitStatusEntry>
  fn test_small_list_shows_individual_files()
  fn test_large_list_groups_by_directory()
  fn test_grouped_shows_status_counts()
  fn test_grouped_single_status_no_count()
  fn test_root_files_grouped_as_dot()

**go_test.rs**
  impl ParseHandler

  struct GoTestResult
  struct GoFailedTest
  fn parse_go_test(input: &str) -> GoTestResult
  fn format_go_test_compact(r: &GoTestResult) -> String
  fn format_go_test_json(r: &GoTestResult) -> String
  mod tests
  fn test_go_test_verbose_all_pass()
  fn test_go_test_verbose_with_failure()
  fn test_go_test_default_mode()
  fn test_go_test_with_skip()
  fn test_go_test_compile_error()
  fn test_go_test_empty()
  fn test_go_test_compact_format()
  fn test_go_test_json_format()

**grep.rs**: impl ParseHandler
**grep_format.rs**: impl ParseHandler
**jest_format.rs**: impl ParseHandler
**jest_parse.rs**: impl ParseHandler | fn extract_count(text: &str, label: &str) -> usize
**lint.rs**
  struct LintIssue
  enum LintLevel
  impl ParseHandler
  fn parse_lint_issues(input: &str) -> Vec<LintIssue>
  fn extract_clippy_rule(lines: &[&str], start: usize) -> String
  fn parse_colon_format(line: &str) -> Option<LintIssue>
  fn find_eslint_file_context(lines: &[&str], from: usize) -> String
  fn format_lint_compact(issues: &[LintIssue], errors: usize, warnings: usize) -> String
  fn format_lint_json(issues: &[LintIssue], errors: usize, warnings: usize) -> String
  mod tests
  fn test_parse_clippy_format()
  fn test_parse_ruff_colon_format()
  fn test_format_compact_clean()
  fn test_format_compact_grouped()
  fn test_format_json()

**logs.rs**: impl ParseHandler
**logs_format.rs**
  impl ParseHandler
  fn level_indicator(level: LogLevel) -> &'static str
  fn level_name(level: LogLevel) -> &'static str
  fn preview_msg(msg: &str, max: usize) -> String
  fn is_stack_trace_line(entry: &LogEntry) -> bool

**logs_helpers.rs**: impl ParseHandler
**ls.rs**: impl ParseHandler
**mod.rs**
  impl ParseHandler

  impl CommandHandler for ParseHandler
  type Input = ParseCommands
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**npm_format.rs**: impl ParseHandler
**npm_parse.rs**: impl ParseHandler
**pnpm_format.rs**: impl ParseHandler
**pnpm_parse.rs**: impl ParseHandler
**ps.rs**
  struct Proc
  impl ParseHandler
  fn parse_ps(input: &str) -> Option<Vec<Proc>>
  fn split_at_field(row: &str, n: usize) -> (Vec<&str>, &str)
  fn find_col(cols: &[&str], names: &[&str]) -> Option<usize>
  fn shorten_command(cmd: &str) -> String
  fn render_compact(procs: &[Proc]) -> String
  fn render_json(procs: &[Proc]) -> String
  mod tests
  fn parses_ps_aux_sample()
  fn compact_is_shorter_than_raw()
  fn passthrough_on_unknown_header()
  fn shorten_command_strips_huge_rustc_args()

**pytest_format.rs**: impl ParseHandler
**pytest_parse.rs**
  impl ParseHandler
  fn parse_pytest_quiet_progress(line: &str) -> Vec<TestResult>
  fn extract_count(text: &str, label: &str) -> usize

**python_traceback.rs**
  struct Frame
  struct Traceback
  impl ParseHandler
  fn parse_traceback(input: &str) -> Traceback
  fn parse_file_line(line: &str) -> Option<Frame>
  fn basename(path: &str) -> String
  fn render_compact(tb: &Traceback) -> String
  fn render_json(tb: &Traceback) -> String
  mod tests
  fn parses_full_traceback()
  fn compact_render_is_shorter()
  fn preamble_is_preserved()
  fn missing_code_snippet_is_tolerated()

**test.rs**: impl ParseHandler
**vitest_format.rs**: impl ParseHandler
**vitest_parse.rs**: impl ParseHandler | fn extract_count(text: &str, label: &str) -> usize
## src/router/handlers/txt2md/

**detect_headings.rs**: impl Txt2mdHandler
**detect_lists.rs**: impl Txt2mdHandler
**format.rs**: impl Txt2mdHandler
**mod.rs**
  impl Txt2mdHandler

  impl CommandHandler for Txt2mdHandler
  type Input = Txt2mdInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**parser.rs**: impl Txt2mdHandler
## src/router/handlers/types/

**fs.rs**: impl Default for LsEntryType | fn default() -> Self
**git.rs**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitStatusSection {
    None,
    Staged,
    Unstaged,
    Untracked,
    Unmerged,
}
... (59 lines)

**grep_types.rs**
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepMatch {
    pub(crate) line_number: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) line: String,
    pub(crate) is_context: bool,
    pub(crate) excerpt: Option<String>,
}
... (28 lines)

**logs.rs**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    Unknown,
... (51 lines)

**mod.rs**
  pub(crate) mod fs;
  pub(crate) mod git;
  pub(crate) mod grep_types;
  pub(crate) mod logs;
  pub(crate) mod test_types_core;
  pub(crate) mod test_types_runners;

  pub(crate) use fs::*;
  pub(crate) use git::*;
  pub(crate) use grep_types::*;
  pub(crate) use logs::*;
  pub(crate) use test_types_core::*;
  pub(crate) use test_types_runners::*;


  pub trait CommandHandler {
      type Input;
      fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult;
  }

**test_types_core.rs**: (262 lines)
**test_types_runners.rs**: (284 lines)
## src/schema/

**fs.rs**
  pub struct LsOutputSchema

  impl LsOutputSchema
  pub fn new() -> Self

  impl Default for LsOutputSchema
  fn default() -> Self
  pub enum LsEntryType
  pub struct LsEntry

  impl LsEntry
  pub fn new(name: &str, entry_type: LsEntryType) -> Self
  pub struct LsError
  pub struct LsCounts
  pub struct FindOutputSchema

  impl FindOutputSchema

  impl Default for FindOutputSchema
  pub struct FindEntry

  impl FindEntry
  pub fn new(path: &str) -> Self
  pub struct FindError
  pub struct FindCounts

**git.rs**
  pub struct GitStatusSchema

  impl GitStatusSchema
  pub fn new(branch: &str) -> Self
  pub struct GitFileEntry

  impl GitFileEntry
  pub fn new(status: &str, path: &str) -> Self
  pub fn renamed(status: &str, old_path: &str, new_path: &str) -> Self
  pub struct GitStatusCounts
  pub struct GitDiffSchema

  impl GitDiffSchema
  pub fn new() -> Self

  impl Default for GitDiffSchema
  fn default() -> Self
  pub struct GitDiffEntry

  impl GitDiffEntry
  pub fn new(path: &str, change_type: &str) -> Self
  pub struct GitDiffCounts
  pub struct RepositoryStateSchema

  impl RepositoryStateSchema

  impl Default for RepositoryStateSchema

**logs.rs**
  pub struct LogsOutputSchema

  impl LogsOutputSchema
  pub fn new() -> Self

  impl Default for LogsOutputSchema
  fn default() -> Self
  pub enum LogLevel
  pub struct LogEntry

  impl LogEntry
  pub fn new(line: &str, line_number: usize) -> Self
  pub struct RepeatedLine
  pub struct LogCounts

**mod.rs**
  mod fs;
  mod git;
  mod logs;
  mod process;
  mod search;
  mod test;

  #[cfg(test)]
  mod tests;


  #[allow(unused_imports)]
  pub use fs::{
      FindCounts, FindEntry, FindError, FindOutputSchema, LsCounts, LsEntry, LsEntryType, LsError,
      LsOutputSchema,
  };
  #[allow(unused_imports)]
  pub use git::{
      GitDiffCounts, GitDiffEntry, GitDiffSchema, GitFileEntry, GitStatusCounts, GitStatusSchema,
      RepositoryStateSchema,
  };
  #[allow(unused_imports)]
  pub use logs::{LogCounts, LogEntry, LogLevel, LogsOutputSchema, RepeatedLine};
  pub use process::{ErrorSchema, ProcessOutputSchema};
  #[allow(unused_imports)]
  pub use search::{
      GrepCounts, GrepFile, GrepMatch, GrepOutputSchema, ReplaceCounts, ReplaceFile, ReplaceMatch,
      ReplaceOutputSchema,
  };
  #[allow(unused_imports)]
  pub use test::{TestOutputSchema, TestResult, TestRunnerType, TestStatus, TestSuite, TestSummary};

  pub const SCHEMA_VERSION: &str = "1.0.0";

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct SchemaVersion {
      pub version: String,
      #[serde(rename = "type")]
      pub schema_type: String,
  }

  impl SchemaVersion {
      pub fn new(schema_type: &str) -> Self {
          Self {
              version: SCHEMA_VERSION.to_string(),
              schema_type: schema_type.to_string(),
          }
      }
  }

**process.rs**
  pub struct ProcessOutputSchema

  impl ProcessOutputSchema
  pub fn new(command: &str) -> Self
  pub struct ErrorSchema

  impl ErrorSchema
  pub fn new(message: &str) -> Self
  pub fn with_type(message: &str, error_type: &str) -> Self

**search.rs**
  pub struct GrepOutputSchema

  impl GrepOutputSchema
  pub fn new() -> Self

  impl Default for GrepOutputSchema
  fn default() -> Self
  pub struct GrepFile

  impl GrepFile
  pub fn new(path: &str) -> Self
  pub struct GrepMatch

  impl GrepMatch
  pub fn new(line: &str) -> Self
  pub struct GrepCounts
  pub struct ReplaceOutputSchema

  impl ReplaceOutputSchema
  pub fn new(search_pattern: &str, replacement: &str, dry_run: bool) -> Self
  pub fn with_files(mut self, files: Vec<ReplaceFile>) -> Self
  pub fn with_counts(mut self, counts: ReplaceCounts) -> Self
  pub struct ReplaceFile

  impl ReplaceFile
  pub struct ReplaceMatch

  impl ReplaceMatch
  pub fn new(line_number: usize, original: &str, replaced: &str) -> Self
  pub struct ReplaceCounts

**test.rs**
  pub struct TestOutputSchema

  impl TestOutputSchema
  pub fn new(runner: TestRunnerType) -> Self
  pub enum TestRunnerType

  impl std::fmt::Display for TestRunnerType
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  pub struct TestSuite

  impl TestSuite
  pub fn new(file: &str) -> Self
  pub struct TestResult

  impl TestResult
  pub fn new(name: &str, status: TestStatus) -> Self
  pub enum TestStatus
  pub struct TestSummary

**tests.rs**
  mod tests
  fn test_schema_version()
  fn test_schema_version_serialization()
  fn test_git_status_schema_new()
  fn test_git_status_schema_serialization()
  fn test_git_file_entry_new()
  fn test_git_file_entry_renamed()
  fn test_git_status_counts_default()
  fn test_git_diff_schema_new()
  fn test_git_diff_entry_new()
  fn test_repository_state_schema_new()
  fn test_ls_output_schema_new()
  fn test_ls_entry_new()
  fn test_ls_entry_hidden()
  fn test_ls_entry_type_serialization()
  fn test_find_output_schema_new()
  fn test_find_entry_new()
  fn test_find_entry_hidden_detection()
  fn test_grep_output_schema_new()
  fn test_grep_file_new()
  fn test_grep_match_new()
  fn test_replace_output_schema_new()
  fn test_replace_output_schema_dry_run()
  fn test_replace_file_new()
  fn test_replace_match_new()
  fn test_replace_output_schema_serialization()
  fn test_replace_counts_default()
  fn test_replace_output_schema_with_files()
  fn test_replace_output_round_trip()
  fn test_test_output_schema_new()
  fn test_test_suite_new()
  fn test_test_result_new()
  fn test_test_runner_type_display()
  fn test_test_status_serialization()
  fn test_logs_output_schema_new()
  fn test_log_entry_new()
  fn test_log_level_serialization()
  fn test_process_output_schema_new()
  fn test_process_output_schema_serialization()
  fn test_error_schema_new()
  fn test_error_schema_with_type()
  fn test_error_schema_serialization()
  fn test_git_status_schema_deserialization()
  fn test_ls_entry_type_deserialization()
  fn test_test_status_deserialization()
  fn test_git_status_round_trip()
  fn test_ls_output_round_trip()
  fn test_test_output_round_trip()

---
*trs ingest v0.5.9 | 112ms | 101.5KB*
