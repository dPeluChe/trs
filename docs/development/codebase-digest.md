# labs-tarscli (274 files, 42.2k tokens, rust)

> Token-reducing shell for AI agents: compact terminal output at 68-99% reduction

## Structure

//
  README.md  .gitignore  AGENTS.md  CHANGELOG.md  CLAUDE.md  
  CODE_OF_CONDUCT.md  CONTRIBUTING.md  Cargo.toml  LICENSE  
  README.es.md  SECURITY.md  cliff.toml  lefthook.yml

.githooks/
  commit-msg  pre-commit  pre-push

.github/
  PULL_REQUEST_TEMPLATE.md  dependabot.yml

.github/ISSUE_TEMPLATE/
  bug_report.yml  config.yml  feature_request.yml

.github/workflows/
  ci.yml  release.yml

docs/
  CNAME  install.ps1  install.sh  llms.txt  robots.txt

docs/commands/
  audit-docs.md  doctor.md  init.md  output-saver.md  stats.md  
  upgrade.md

docs/development/
  agent-integrations.md  antigravity-hooks-research.md

docs/development/benchmarks/
  README.md  benchmark-real.sh  benchmark.sh  chain-rewrite.sh

docs/features/
  audit-docs.md  configuration.md  diff.md  doctor.md  formats.md  
  ingest.md  init.md  output-saver.md  stats.md  uninstall.md  
  upgrade.md

docs/roadmap/
  TASK_TODO.md

docs/roadmap/completed/
  2603.md  2604.md  2605.md  README.md

docs/support/
  agents.md  commands.md  install.md  other-token-savers.md  safety.md

memory/
  MEMORY.md  feedback_clippy_before_push.md  
  feedback_trs_raw_bypass.md  project_hook_migration.md

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
  docker-smoke.sh  sync-codebase-digest.sh  sync-version.sh

src/
  ai_tool.rs  ai_tool_tests.rs  audit_docs.rs  audit_docs_detect.rs  
  audit_docs_report.rs  audit_docs_symbols.rs  benchmark.rs  
  classifier.rs  classifier_args.rs  classifier_exec.rs  
  classifier_tests.rs  classifier_transfer.rs  cli.rs  codex.rs  
  command_registry.rs  command_registry_tests.rs  commands.rs  
  commands_parse.rs  config.rs  debug_info.rs  diff.rs  discover.rs  
  doctor.rs  doctor_checks.rs  doctor_tests.rs  exec.rs  fast_find.rs  
  help.rs  help_tests.rs  help_text.rs  help_text_more.rs  init.rs  
  init_collision.rs  init_collision_tests.rs  init_install.rs  
  init_install_plugins.rs  init_install_tests.rs  init_show.rs  
  init_templates.rs  init_templates_plugins.rs  main.rs  main_tests.rs  
  main_tests_precedence.rs  output_saver.rs  output_saver_core.rs  
  output_saver_core_tests.rs  parse_out.rs  path_display.rs  
  process.rs  process_helpers.rs  process_tests.rs  read_intercept.rs  
  rewrite.rs  rewrite_decide.rs  rewrite_decide_tests.rs  
  rewrite_tests.rs  text_util.rs  tracker.rs  tracker_tests.rs  
  uninstall.rs  upgrade.rs

src/formatter/: Formatter system for trs (Token-Reducing Shell)
  agent.rs  compact.rs  compact_schema_git.rs  
  compact_schema_output.rs  csv.rs  helpers.rs  json.rs  
  json_schema.rs  mod.rs  raw.rs  tsv.rs

src/ingest/: Project digest generator for LLM consumption
  collect.rs  collect_compress.rs  collect_index.rs  
  collect_manifests.rs  deps.rs  deps_extract.rs  dupes.rs  format.rs  
  format_html.rs  format_html_util.rs  format_tree.rs  meta.rs  mod.rs  
  mod_html.rs  ollama.rs  purpose.rs  remote.rs  resolve.rs  store.rs  
  tests.rs

src/reducer/: Reducer system for trs (Token-Reducing Shell)
  mod.rs  output.rs  registry.rs  truncation.rs

src/router/: Command routing system for trs (Token-Reducing Shell)
  mod.rs

src/router/handlers/
  ansi.rs  clean.rs  common.rs  err.rs  html2md.rs  isclean.rs  
  json.rs  json_query.rs  json_tests.rs  mod.rs  read.rs  
  read_filters.rs  read_tests.rs  replace.rs  run.rs  search.rs  
  stats.rs  stats_coverage.rs  stats_efficiency.rs  stats_render.rs  
  stats_render_tests.rs  tail.rs  trim.rs

src/router/handlers/parse/
  aws.rs  aws_tests.rs  brew.rs  bun_format.rs  bun_parse.rs  
  extra_cargo_test.rs  extra_db.rs  extra_download.rs  extra_env.rs  
  extra_network.rs  extra_system.rs  find.rs  fmt.rs  gh_api.rs  
  gh_api_tests.rs  gh_pr.rs  gh_run.rs  git_branch.rs  git_commit.rs  
  git_diff.rs  git_diff_format.rs  git_log.rs  git_pull.rs  
  git_status.rs  git_status_format.rs  go_test.rs  grep.rs  
  grep_format.rs  jest_format.rs  jest_parse.rs  lint.rs  
  lint_tests.rs  logs.rs  logs_format.rs  logs_helpers.rs  
  logs_json.rs  logs_json_tests.rs  ls.rs  mod.rs  npm_format.rs  
  npm_parse.rs  pnpm_format.rs  pnpm_parse.rs  ps.rs  pytest_format.rs  
  pytest_parse.rs  python_traceback.rs  sysinfo.rs  sysinfo_tests.rs  
  test.rs  vitest_format.rs  vitest_parse.rs

src/router/handlers/txt2md/: Handler for the `txt2md` command - converts plain text to Markdown
  detect_headings.rs  detect_lists.rs  format.rs  mod.rs  parser.rs

src/router/handlers/types/: Shared data structures and types for command handlers
  fs.rs  git.rs  grep_types.rs  logs.rs  mod.rs  test_types_core.rs  
  test_types_runners.rs

src/schema/: Stable JSON schemas for trs (Token-Reducing Shell) reducers
  fs.rs  git.rs  logs.rs  mod.rs  process.rs  search.rs  test.rs  
  tests.rs


## Key Dependencies

  parse_out.rs ← src/classifier_exec.rs, src/main.rs, parse/aws.rs, brew.rs (+26)
  tests.rs ← ai_tool.rs, classifier.rs, command_registry.rs, doctor.rs (+22)
  common.rs ← clean.rs, err.rs, html2md.rs, isclean.rs (+11)
  formatter/mod.rs ← agent.rs, compact.rs, csv.rs, formatter/json.rs (+8)
  src/tracker.rs ← src/classifier_exec.rs, debug_info.rs, doctor_checks.rs, fast_find.rs (+8)

## Architecture (module roles)

From the import graph (fan-in↓ / fan-out↑), no AST:

- **entry**: roots, nothing imports them (main, CLI) → `main` (0↓/49↑)
- **core**: high fan-in, everything routes through → `ingest` (20↓/5↑)
- **leaf**: used by many, import nothing (utils, types) → `path_display` (7↓/0↑), `config` (5↓/0↑)
- **internal**: mid-graph plumbing → `router/handlers` (5↓/12↑), `init` (8↓/7↑), `router/handlers/parse` (3↓/9↑), `classifier_exec` (2↓/8↑), `init_install` (3↓/7↑), `router` (7↓/3↑), `tracker` (9↓/1↑), `classifier` (5↓/4↑) +12 more

## README.md

<strong>trs</strong>: <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · terminal compression for AI agents

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

... (236 lines, hidden sections: Install · macOS / Linux · npm (all platforms) · cargo (builds from source) · Windows (PowerShell) · Quick start · 1. Wire hooks into every detected AI agent (the main path) · 2. See your savings)

**.gitignore**
# Build artifacts
/target/
**/*.rs.bk
*.pdb

# Cargo lock file for binary projects (keep for libraries)
# Uncomment if this becomes a library:
# Cargo.lock
... (74 lines)

## AGENTS.md

# AGENTS.md: trs (Token-Reducing Shell)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Reduces token consumption by 68-99% for developers, AI agents, and automation pipelines.

## Pre-generated codebase digest

[`docs/development/codebase-digest.md`](./docs/development/codebase-digest.md)
is a snapshot of the entire trs codebase produced by `trs ingest`.
Drop it into any agent's context for an instant map of the project
without having to run `trs ingest` yourself.

The digest can drift from HEAD between releases. Regenerate before
tagging a release, or whenever `src/` has moved meaningfully.

Run `./scripts/sync-codebase-digest.sh`. It uses `trs` from `PATH` and
falls back to `./target/release/trs` if one isn't installed.

## Related commands worth knowing

- `trs stats`: cumulative savings across commands (shows date range, today
  vs. average, last command, top-reducers).
- `trs doctor`: installation health check. Warns when AGENTS.md / CLAUDE.md
  exceed ~5k tokens and points at `trs audit-docs`.
- `trs audit-docs`: static analysis of agent instruction files
  (CLAUDE.md, AGENTS.md, rules files). Surfaces cross-file duplicates,
  embedded code/SQL/JSON blocks that belong elsewhere, dead `@imports`,
  and (for code fences) cross-references declared symbols against
  the actual source tree (so you can REMOVE ones already defined in src/
  and EXTRACT ones that don't live anywhere yet).
- `trs output-saver`: install a short output-reduction rules block
  into each agent's global config (AGENTS.md / CLAUDE.md / Cursor rules).
  Closes the symmetric gap: trs compresses what agents SEE via
  `trs rewrite`; output-saver compresses what they EMIT via
  anti-preamble / anti-narration / structured-output directives.
  Check-first by default (`--install` to commit, `--remove` to undo).
- `trs ingest`: project digest for LLM consumption. Use symbol index,
  compression levels, or `owner/repo` URL shorthand.
... (117 lines, hidden sections: Architecture · Key Design Decisions · Development · Testing)

## CHANGELOG.md

# Changelog

All notable changes to trs are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.5] - 2026-08-13

### Features

- aws: Compress recursive s3 output from receipts to counts (#131)
- find, stats: Tier find output by size, add `stats --gaps` (#132)
- stats: Show recent efficiency next to the lifetime mean (#139)
- stats: Add --days, and window the gaps view by default (#140)

### Refactor

- stats: Fold --gaps into --coverage, document the windows (#141)

### CI / Build

- Split CI into a Tenki quick tier and a GitHub full gate (#130)
- Move the quick tier back to GitHub runners, cancel superseded runs (#135)
- Adopt tenki-standard-small-2c-4g as the quick-tier default (#138)

### Dependencies

- deps: Bump the cargo-minor group across 1 directory with 4 updates (#134)

## [0.7.4] - 2026-07-29

### Bug Fixes

- rewrite: Stop doing text surgery on commands we can't parse (#127)

### Dependencies

- deps: Bump the cargo-minor group across 1 directory with 7 updates (#123)

## [0.7.3] - 2026-07-28
... (542 lines, hidden sections: [0.7.2] - 2026-07-28 · [0.7.1] - 2026-07-18 · [0.7.0] - 2026-07-18 · [0.6.17] - 2026-07-07 · [0.6.16] - 2026-06-09 · [0.6.15] - 2026-06-05 · [0.6.14] - 2026-06-04 · [0.6.13] - 2026-06-04)

## CLAUDE.md

# CLAUDE.md

Agent instructions for this repo live in [`AGENTS.md`](./AGENTS.md). Start
there. This file carries only the machine-readable config the hooks and the
ship gate read.

Personal agent setup (skills, flow imports) belongs in your own global config,
not here: this file is committed and shared by every contributor.

## ship config

# Mirrors .github/workflows/ci.yml so a local gate fails whatever CI would.
# --all-targets is a superset of CI's plain clippy: it also lints tests.
lint: cargo fmt -- --check && cargo clippy --all-targets -- -D warnings
# No separate typecheck: clippy type-checks the crate, so a `cargo check`
# step would just repeat the same work.
build: cargo build
# --no-fail-fast so one failing suite doesn't mask the rest (matters most
# on the blocking windows-latest job).
test: cargo test --no-fail-fast
# pre-push deliberately not delegated: it already mirrors CI and is stricter.
hooks_skip: pre-push: "already mirrors CI (fmt + clippy + full suite), stricter than the ship-config gate"
merge_policy: ask   # auto | ask
loc_limit: 500
simplify: 500       # run /simplify only if changed LOC > N (off = only on request)

## Git hooks

`core.hooksPath` points at the tracked `.githooks/`, so lefthook's own stubs
in `.git/hooks/` are never invoked. `.githooks/pre-commit` and
`.githooks/commit-msg` delegate to lefthook explicitly; both no-op when
lefthook isn't installed, so nobody is blocked by an optional tool.

`.githooks/pre-push` deliberately does NOT delegate: it already mirrors CI
(`fmt` + `clippy` + the full test suite), which is stricter than the
ship-config gate, and delegating would run clippy twice.

## Release

Tagging `vX.Y.Z` triggers `.github/workflows/release.yml` (5 binaries + npm
publish + GitHub Release). Bump `Cargo.toml`, regenerate `CHANGELOG.md` with
`git cliff -o CHANGELOG.md --tag vX.Y.Z`, merge, then tag. A tagged run uses
the workflow at the tag's commit, so fixing a failed release means re-tagging,
not re-running.

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

Thanks for your interest in contributing! trs is a personal project that grew into something useful, and contributions are welcome, whether it's a new parser, a bug fix, or just better docs.

## Getting started

git clone https://github.com/dPeluChe/trs.git
cd trs
cargo build
cargo test

All three checks must pass before submitting a PR:

cargo fmt -- --check           # formatting
cargo clippy -- -D warnings    # no warnings allowed
cargo test                     # 2,186+ tests, 0 failures

The repo ships a pre-push hook that runs these automatically. Activate it once after cloning:

git config core.hooksPath .githooks

## Code guidelines

### File size
- Max 500 lines per file. If a file grows past this, split it.
- Rust allows multiple `impl` blocks in separate files, use this pattern.
- Tests go in `tests/` (integration) or `src/*_tests.rs` (unit).

### Naming
- Parser files: `{tool}_parse.rs` + `{tool}_format.rs` (e.g. `npm_parse.rs`, `npm_format.rs`)
- Test files: `test_{feature}_{category}.rs` (e.g. `test_replace_edge.rs`)
- Fixture data: `tests/fixture_data/{tool}_{scenario}.txt`

### Style
- Run `cargo fmt` before committing. No exceptions.
- No `unwrap()` in production code, use `?` or explicit error handling.
- `unwrap()` is fine in tests.
- Prefer simple code over clever code. Three similar lines > premature abstraction.
- Don't add doc comments to every function, only where the logic isn't obvious.

... (214 lines, hidden sections: Adding a new parser · Proposing new commands · Project structure · Commit messages · Releasing · Questions?)

**Cargo.toml**
[package]
name = "trs-cli"
version = "0.7.5"
edition = "2021"
description = "Token-reducing shell for AI agents: compact terminal output at 68-99% reduction"
license = "MIT"
repository = "https://github.com/dPeluChe/trs"
homepage = "https://usetrs.dev"
keywords = ["cli", "terminal", "ai", "token-reduction", "parser"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "trs"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
grep = "0.4"
ignore = "0.4"
regex = "1"
htmd = "0.5"
ureq = { version = "3", features = ["rustls-webpki-roots"] }
toml = "1.1"
time = { version = "0.3", features = ["local-offset"] }
tempfile = "3"

[profile.release]
strip = true
lto = true
codegen-units = 1

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
serde_json = "1"

**LICENSE**
MIT License

Copyright (c) 2026 dPeluChe

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
... (21 lines)

## README.es.md

<strong>trs</strong>: <strong>T</strong>oken-<strong>R</strong>educing <strong>S</strong>hell · compresión de salida terminal para agentes de IA

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

... (238 lines, hidden sections: 1.4 KB → 336 B (76% reducción) · 55 KB → 58 B (99% reducción) · 55 KB → 5.5 KB (90% reducción) · Instalación · macOS / Linux · npm (todas las plataformas) · cargo (compila desde fuente) · Windows (PowerShell))

## SECURITY.md

# Security Policy

## Supported versions

Only the latest release receives fixes. Run `trs upgrade` to stay current.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting
(Security → Report a vulnerability) on this repository.
You should get a first response within 72 hours.

Please do not open public issues for security reports.

## Scope notes

trs executes the commands an AI agent (or you) already intended to run,
it does not grant new execution capability. Reports we especially care
about: credential leakage through compacted output (trs deliberately
preserves and redacts credential-bearing lines, see `redact_secrets`),
hook-template injection, and anything in `install.sh` / the release
pipeline.

**cliff.toml**
# git-cliff configuration: https://git-cliff.org
# Generates CHANGELOG.md (Keep a Changelog) and release notes from
# Conventional Commits. Regenerate: `git cliff -o CHANGELOG.md`.

[changelog]
header = """
# Changelog

... (21 lines)

**lefthook.yml**
# Central hooks live in dPeluChe/skills (hooks/lefthook-base.yml).
remotes:
  - git_url: https://github.com/dPeluChe/skills
    ref: main
    refetch_frequency: 24h
    configs:
      - hooks/lefthook-base.yml

## .githooks/

**commit-msg**
#!/bin/sh
# Delegates to lefthook: strips agent attribution so authorship stays human.
# "$@" carries the path to the commit message file. No-ops without lefthook.

if command -v lefthook >/dev/null 2>&1; then
  exec lefthook run commit-msg "$@"
fi
exit 0

**pre-commit**
#!/bin/sh
# Delegates to lefthook, whose config is fetched from dPeluChe/skills
# (secret scan on staged changes, .env block, LOC warning).
#
# core.hooksPath points here, so lefthook's own stubs in .git/hooks are never
# invoked, this file is the entry point.
#
# Optional by design: lefthook is a personal tool, not a build dependency. A
... (15 lines)

**pre-push**
#!/bin/sh
# Pre-push hook: mirrors CI checks exactly so nothing reaches GitHub red.
# Install once: git config core.hooksPath .githooks

set -e

echo "  [pre-push] cargo fmt -- --check"
cargo fmt -- --check
... (16 lines)

## .github/

**PULL_REQUEST_TEMPLATE.md**
## What

<!-- One or two lines: what does this change and why. -->

## Checklist

- [ ] `cargo fmt` + `cargo clippy --all-targets -- -D warnings` clean (CI gates on this)
- [ ] `cargo test` green; new behavior has tests (parsers: basic / edge / empty, see CONTRIBUTING.md)
- [ ] Files stay under ~500 LOC
- [ ] Conventional Commit title (`feat(scope): …` / `fix(scope): …`), the changelog is generated from it

**dependabot.yml**
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    groups:
      cargo-minor:
... (13 lines)

## .github/ISSUE_TEMPLATE/

**bug_report.yml**
name: Bug report
description: Something broke: wrong output, crash, hook not firing
labels: [bug]
body:
  - type: input
    id: version
    attributes:
      label: trs version
... (21 lines)

**config.yml**
blank_issues_enabled: true
contact_links:
  - name: Question / discussion
    url: https://github.com/dPeluChe/trs/discussions
    about: Usage questions and ideas, faster than issues for non-bugs

**feature_request.yml**
name: Feature request
description: New parser, new agent, or improvement
labels: [enhancement]
body:
  - type: dropdown
    id: kind
    attributes:
      label: Type
... (21 lines)

## .github/workflows/

**ci.yml**
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    # ready_for_review matters: without it, flipping a draft to ready does not
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
... (20 lines)

**llms.txt**: (30 lines)
**robots.txt**
User-agent: *
Allow: /

Sitemap: https://usetrs.dev/sitemap.xml

LLMs: https://usetrs.dev/llms.txt

## docs/commands/

**audit-docs.md**
# Moved

This doc is now at [`docs/features/audit-docs.md`](../features/audit-docs.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

**doctor.md**
# Moved

This doc is now at [`docs/features/doctor.md`](../features/doctor.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

**init.md**
# Moved

This doc is now at [`docs/features/init.md`](../features/init.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

**output-saver.md**
# Moved

This doc is now at [`docs/features/output-saver.md`](../features/output-saver.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

**stats.md**
# Moved

This doc is now at [`docs/features/stats.md`](../features/stats.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

**upgrade.md**
# Moved

This doc is now at [`docs/features/upgrade.md`](../features/upgrade.md).

Older `trs` binaries (≤ v0.5.9) link here from their "More:" hint.
Upgrade with `trs upgrade` to pick up the direct link.

## docs/development/

**agent-integrations.md**
# AI Agent Integrations: Reference

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
... (439 lines, hidden sections: Integration types · Wire-format dispatch (hook agents) · Agent attribution (`TRS_AGENT`) · Per-agent reference · Test prompts · Debugging a broken integration · 1. Create a logging wrapper · 2. Point the agent's hook at the wrapper (back up first))

**antigravity-hooks-research.md**
# Antigravity hooks: research notes

Status (2026-05-22): agy v1.0.1 does NOT expose user-configurable
PreTool hooks for shell (Bash) commands. trs cannot intercept Antigravity
tool calls until Google ships this surface upstream. v0.6.6 reverted the
v0.6.5 jetski PreToolUse integration and reclassified Antigravity (IDE +
CLI) as rules-only, same posture as Codex CLI and Windsurf.

This file records what we tested, what we found, and what would unblock
re-enabling the programmatic hook.

## Context

Google launched Antigravity 2.0 on 2026-05-19, simultaneously
releasing the desktop IDE and a CLI binary (`agy`). Both products are
built on Google's internal jetski agent framework, visible in the
binary as `google3/third_party/jetski/...` symbols.

trs v0.6.4 wrongly aliased both Antigravity variants to the Gemini CLI
hook harness (`BeforeTool` entry in `~/.gemini/settings.json`). agy
... (165 lines, hidden sections: Investigation summary · What unblocks re-enabling the integration · What still works in v0.6.6 · Why we didn't just leave v0.6.5 installed · Reproducing · Plant a side-channel probe in agy's hooks.json · Restart agy fresh, ask it to "ejecuta ls en bash y muéstrame la salida" · Then check:)

## docs/development/benchmarks/

**README.md**
# trs Benchmarks

Living laboratory for trs. These benchmarks exist to help us learn, measure, and iterate, not to be marketing material or regression gates.

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
# `trs audit-docs`: find bloat in CLAUDE.md / AGENTS.md / rules files

Agent instruction files (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`,
`.windsurfrules`) get loaded into every agent session, every turn,
every project open, every conversation start. Bloat in these files is
the single most expensive kind of bloat because it multiplies across
every interaction.

`trs audit-docs` is a static analyzer that finds:

- Cross-file duplicate sections (SimHash over 3-word shingles; flags
  blocks with Hamming distance ≤ 6, i.e. ≥ 90% similar).
- Dead `@imports`, references to files that don't exist.
- Embedded code / SQL / JSON / YAML / tables that belong in their own
  files rather than inline in rules.
- Code fences whose declared symbols already exist in the project's
  source tree (so you can replace the snippet with a `src/…:NN` link)
  or don't exist yet (so you can extract them into new files).

## Quick reference
... (127 lines, hidden sections: What it scans · Duplicate detection (SimHash) · Dead `@imports` · Embedded bloat · Language support for symbol extraction · Integration with `trs doctor` · Philosophy · Output format example)

**configuration.md**
# Configuration

trs works without any configuration: every default is tuned for
sensible output on typical workloads. Create a `config.toml` only
when you need to tighten or loosen specific limits.

## Files

Two lookup paths, merged (project overrides home):

1. `~/.trs/config.toml`: per-user defaults, applied globally.
2. `.trs/config.toml`: project override, committed to the repo.

Later keys win. Unrecognized keys are ignored with a warning on
`trs doctor`.

## Tunable limits

[limits]
grep_max_results = 200           # max rows surfaced by trs grep / search
... (121 lines, hidden sections: Hook-time wrappers · Bypassing trs for a single command · Inspecting the active config · When to skip the config file · See also)

**diff.md**
# `trs diff`: audit what compression drops

`trs diff <cmd>` runs a command twice (raw and through trs) and shows
both sides: the byte/token savings, the compact output the agent would
see, and every line that was dropped or collapsed. It exists so you
never have to *trust* the compression: you can check any command in
two seconds.

## Quick reference

trs diff git status          # raw vs compact + dropped lines
trs diff cargo test          # works with any supported command
trs diff git status --json   # machine-readable report

## What it shows

trs diff: git status
──────────────────────────────────────────────────
raw:          449 B  ~112 tok
compact:      135 B  ~34 tok   (70% smaller, 78 tok saved)
... (57 lines, hidden sections: When to use it · Escape hatches)

**doctor.md**
# `trs doctor`: installation health check

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
| `trs binary` | Reports version + binary path (never fails, always informational). |
| `trs in PATH` | `which -a trs` / `where trs`. Warns if multiple binaries exist so a shadowed install doesn't silently win. |
| `git available` | `git --version`. Required, several parsers assume git is present. |
... (100 lines, hidden sections: Reading the report · JSON mode · Typical fixes · See also)

**formats.md**
# Output formats

Every trs command supports six output formats. Pick the one that
matches the consumer: humans / agents read compact, scripts read
json/csv/tsv, pipelines sometimes want raw passthrough.

| Flag | Name | Who it's for |
|---|---|---|
| *(default)* | compact | humans + agents, terse single-pass form |
| `--json` | JSON | scripts, dashboards, anything structured |
| `--csv` | CSV | spreadsheets, basic data import |
| `--tsv` | TSV | tab-friendly tooling (`cut -f`, spreadsheets) |
| `--agent` | agent-optimized markdown | LLMs specifically, same compact form with marker syntax for section parsing |
| `--raw` | raw passthrough | unchanged, no compression, still tracked in stats |

Flags work anywhere in the invocation: `trs --json git status` and
`trs git status --json` are equivalent.

## Examples

... (126 lines, hidden sections: When to use what · Built-in tools vs wrapped commands · See also)

**ingest.md**
# `trs ingest`: project digest for AI agents

`trs ingest` walks a repo and produces a compact, token-budget-aware
Markdown digest of the codebase (structure + key files + signatures),
ready to paste into an AI agent's context.

## Quick reference

trs ingest                      # write digest, print path to stdout
trs ingest --budget 128k        # fit to token budget (signatures first)
trs ingest --deps               # dependency graph only, no content
trs ingest --symbols            # add a flat symbol → file index
trs ingest --changed            # only files with uncommitted changes
trs ingest --since-last         # only files changed since last ingest
trs ingest --fresh              # reuse cached digest if HEAD unchanged
trs ingest -o ~/ctx.md          # custom output path (no shadow save)
trs ingest --print              # emit content to stdout instead of path
trs ingest --agent              # agent format ⇒ implicit --print (content to stdout)
trs ingest --warn-at 40k        # stderr warning if digest exceeds N tokens
trs ingest --list               # list saved digests + HEAD sha + stale markers
... (218 lines, hidden sections: What the digest contains · <project name> · Structure · Dependencies · Architecture (module roles) · Files (highlights) · Files (signatures) · Budget-aware truncation)

**init.md**
# `trs init`: install hooks for AI agents

`trs init` wires your AI coding agent's shell-execution pipeline through
`trs rewrite` so every command gets compressed automatically. Sixteen
agents are supported end-to-end. See [`docs/development/agent-integrations.md`](../development/agent-integrations.md)
for the full per-agent reference.

## Quick reference

trs init --show                      # status of all 16 agents
trs init --all --global              # install for every detected agent
trs init <agent>                     # install for one: claude, gemini, cursor, …
trs init --all --global --dry-run    # preview every file that would change
trs init --all --global --force      # refresh templates (see "Refreshing hooks")
trs init <agent> --replace           # migrate cleanly from another compressor

## What gets installed where

| Agent | Type | Target |
|---|---|---|
... (240 lines, hidden sections: Preview with `--dry-run` · Collision handling · Refreshing hooks · `--global` vs project-local · Bypassing the hook for one command · Agent attribution (`TRS_AGENT`) · Uninstalling · See also)

**output-saver.md**
# `trs output-saver`: reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents see, the output of the shell commands they run. Agents
still emit verbose replies: preambles ("Sure!"), narration
("Now I will…"), speculative suggestions, hallucinated file paths.

`trs output-saver` installs a short rules block into each supported
agent's global config so those replies come back tighter.

## Quick reference

trs output-saver                 # read-only scan of all agents
trs output-saver --install       # write to every detected agent
trs output-saver <agent> --install  # scope to one
trs output-saver --verify        # per-agent: block matches current canonical?
trs output-saver --refresh       # re-write the block where it's already present
trs output-saver --remove        # clean uninstall
trs output-saver --print         # dump the block to stdout (pipe-friendly)

... (260 lines, hidden sections: What the block says · Why these rules: research backing · Coverage matrix · How the install is idempotent · Output saver: keep replies cheap · Check-first semantics · `--refresh`: pick up template changes without adding new installs · `--remove` behavior)

**stats.md**
# `trs stats`: token savings dashboard

Every trs invocation logs an entry to `~/.trs/history.jsonl`:
timestamp, command, input bytes, output bytes, duration. `trs stats`
reads that log and produces a dashboard of cumulative savings.

The active file rolls into a month-stamped archive
(`~/.trs/history.YYYY-MM.jsonl`) at the first append of each new month.
`trs stats` reads the active file plus every archive transparently, so
your cumulative numbers don't reset. Use `trs history --prune
--older-than 90` to retire archives older than your retention window.

`trs stats --history` lists commands newest first (top of output is
the most recent run). Same ordering in `--json` mode.

## Quick reference

trs stats              # summary dashboard (top 15 commands)
trs stats --history    # per-command log (most recent 20)
trs stats -n 30        # override row cap (top 30 in summary, last 30 in --history)
... (285 lines, hidden sections: Time windows · Summary (default) · History view · `--by-agent`: attribution breakdown · `--by-command`: command family breakdown · `--coverage`: parser-gap analysis · JSON mode · What gets tracked)

**uninstall.md**
# `trs uninstall`: remove trs from agent configs

`trs uninstall` is the inverse of [`trs init`](init.md). It walks every
surface `trs init` (and `trs output-saver`) wrote to and cleans up:
JSON hook entries, plugin files, sentinel-delimited rules blocks, and
the output-saver sidecar / `@import` line.

## Quick reference

trs uninstall                        # interactive, lists installed agents
trs uninstall <agent>                # one agent (claude, codex, gemini, …)
trs uninstall --all                  # every agent, with confirmation
trs uninstall --all --yes            # no prompt, for scripts / CI
trs uninstall --output-saver         # only the output-saver block
trs uninstall --dry-run              # preview without writing

## What gets removed per surface

| Surface | Action |
|---|---|
... (111 lines, hidden sections: Interactive mode · `--dry-run` · Preserving user-added hooks · Sentinel-based detection · What `trs uninstall` does NOT do · See also)

**upgrade.md**
# `trs upgrade`: re-run the install pipeline for the latest release

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

1. Binary: the shell install command for your detected channel
... (152 lines, hidden sections: Detection logic · Why detection is path-based · Confirmation prompt · Roadmap for unsupported channels · What happens after a successful upgrade · Interaction with hooks · See also)

## docs/roadmap/

**TASK_TODO.md**
# trs: Roadmap

Binary: `trs` | Language: Rust | Status: Active development

---

## Phase 1: Release & Distribution

- [x] Create first GitHub Release, v0.1.0 shipped
- [x] npm publish (`@dpeluche/trs`)
- [x] Rewrite hook: detect `cd X && git Y` chains, done in v0.5.5
- [x] Pipe/redirect first-segment rewrite, shipped in v0.5.6
- [ ] Homebrew tap (low priority, npm + curl|sh covers 99% of users)
- [ ] Publish to crates.io (`cargo install trs-cli`, currently source-only)
- [ ] Shell completions (bash, zsh, fish)
- [x] Copilot hook, see Phase 3 "VSCode ecosystem"
- [ ] `trs self-update` command, re-download latest binary from GitHub Releases

---

... (502 lines, hidden sections: Phase 2: New Parsers · Phase 2.5: Ideas from competitor analysis · Phase 3: Agent integration follow-ups · Documentation drift (carry-over from v0.5.9) · Phase 2.6: Internal architecture (May 2026 feedback) · Phase 3.5: Codex integration (stale, needs rework) · Phase 2.5b: Competitive landscape (May 2026) · Phase 2.7: Ingest upgrade: codebase intelligence (2026-07))

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

**2605.md**
# Mayo 2026 — trs development log

---

# 2026-05-01 — Bypass de-promotion + telemetry loop

## Context
Audited why agents kept burning savings with `TRS_SKIP=1 grep …`,
`TRS_SKIP=1 npm test`, and similar defensive bypass calls on
routine commands. Root cause: bypass mechanisms (`trs raw`,
`TRS_SKIP=1`) were promoted in agent-facing surfaces — the
`~/.claude/trs.md` template, `docs/llms.txt`, the README headline.
The previous v0.5.14 fix tried strengthening "Do NOT use" guidance;
agents kept reaching for the escape hatch anyway. Negative
instructions are weaker than absence; the visibility itself is the
temptation.

This session removed bypass mentions from prompt-level surfaces
(silent strategy) and added telemetry so the user can measure
whether the strategy is working.
... (222 lines, hidden sections: Completed · Decisions · Files Changed · Out of scope (deferred) · 2026-05-01 — Defensive-line iteration after field test (v0.5.16) · Context · Completed · Decisions)

**README.md**
# TASK_COMPLETED: changelog de trabajo

Registro mensual de tareas completadas, decisiones tomadas y archivos modificados.

## Formato de archivos

Cada archivo se nombra `YYMM.md` (ej: `2603.md` = marzo 2026).

## Estructura de cada entrada

# YYYY-MM-DD: titulo breve de la sesion

## Context
Por que se hizo este trabajo. Contexto del problema o requerimiento.

## Completed
### Feature/Fix nombre
- Que se hizo (bullet points concretos)
- Archivos clave modificados
- Tests agregados/modificados
... (45 lines)

## docs/support/

**agents.md**
# Supported AI agents

Sixteen AI coding agents are supported end-to-end. Each row lists the
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
| Pi Coding Agent | programmatic hook (extension) | ✓ | — | `pi` | global + project |
| Factory Droid | programmatic hook | ✓ | ✓ (inline block) | `droid` | global + project |
| Antigravity IDE | rules file only (see [research notes](../development/antigravity-hooks-research.md)) | — | ✓ (`@import`) | `antigravity` (env fallback) | global |
| Antigravity CLI (`agy`) | rules file only (see [research notes](../development/antigravity-hooks-research.md)) | — | ✓ (`@import`) | `antigravity` (env fallback) | global |
| Codex CLI | programmatic hook (codex-cli ≥ 0.134), rules fallback | ✓ (≥ 0.134) | ✓ (inline block) | `codex` (fallback `(untagged)`) | global + project |
| Devin Desktop | rules file only | — | ✓ (inline block) | `(untagged)` | global + project |
... (41 lines)

**commands.md**
# Supported commands

Every command supported by trs falls into one of four levels.

1. Dedicated parser. trs spawns the tool, parses its native output,
   and emits a structured compact form. Typical reduction 68–99%.
2. Dispatched alias. A different binary with the same semantics
   (e.g. `rg` for `grep`, `eza` for `ls`) gets routed to the same
   parser. No configuration, the dispatcher recognizes the binary
   name.
3. Generic compression. Commands without a parser still get ANSI
   stripping, whitespace collapse, and repeated-line deduplication.
   Typical reduction 30–40% "free."
4. Passthrough. Commands where trs detects a flag that already
   produces structured output (`--json`, `--porcelain`) are passed
   through untouched, the agent gets the raw structured form.

## Commands with dedicated parsers

### VCS: git
... (283 lines, hidden sections: Built-in trs tools (not wrappers) · Dispatch mechanisms · Generic compression (the fallback))

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
... (143 lines, hidden sections: Platform support · Prebuilt binaries: manual install · Pinning a specific version · Custom install directory · Upgrading · Shadowed installs (multi-channel) · Uninstall · Troubleshooting)

**other-token-savers.md**
# Other token-saving tools

trs is one of several tools in the shell-output-compression space for
AI agents. This page lists the alternatives we're aware of, for folks
evaluating options or migrating between tools.

We don't link to these projects directly, go search for them if you
want to compare. The list is descriptive, not promotional, and we
update it as new tools appear.

## Alternatives we've analyzed

- rtk (Rust Token Killer): another Rust-based CLI proxy for
  shell compression. TOML filter pipeline, SQLite usage tracking,
  dedicated `rtk gain` analytics. Overlaps significantly with trs on
  the core rewrite surface.
- token-optimizer: Node-based compressor, installs as a global
  npm package. Hook integration focused on Claude Code.
- token-saver: early-stage shell wrapper, smaller scope than
  rtk / trs.
... (60 lines, hidden sections: How trs positions itself · Installing alongside another tool)

**safety.md**
# Safety guarantees

trs sits between the user / agent and the underlying tools, so the
worst failure mode we can have is corrupting output or breaking an
exit-code contract. These are the guarantees trs holds to keep
that from happening.

## Command hygiene

- `--no-verify` blocked on `git commit` / `git push`. Pre-commit
  and pre-push hooks are there for a reason, agents that default to
  bypassing them can ship broken code. trs refuses the bypass; users
  who explicitly want it can still invoke git directly (`TRS_SKIP=1`).
- `--json` / `--porcelain` passthrough. When the wrapped tool
  already has a structured mode, trs doesn't re-parse its output, 
  the structured form passes through untouched.
- Exit codes always propagated. If the wrapped command exits 1,
  trs exits 1. Scripts and CI relying on exit codes keep working.
- Never silently change semantics. No rewrites that would change
  what the tool does, only how its output is presented.
... (73 lines, hidden sections: Parser safety · Failure recovery · Install-time safety · See also)

## memory/

**MEMORY.md**
# Memory Index

- [Strict clippy before push](feedback_clippy_before_push.md) — CI uses -D warnings; always run clippy+fmt before git push
- [Use trs raw for bypass](feedback_trs_raw_bypass.md) — use `trs raw <cmd>` not `rtk proxy <cmd>` for raw output
- [Hook migration plan](project_hook_migration.md) — switch from rtk to trs hook after v0.5.10 ships

**feedback_clippy_before_push.md**
---
name: Strict clippy before push
description: Always run cargo clippy -D warnings + cargo fmt before git push in this repo
type: feedback
---

Always run all three before any `git push` in labs-tarscli, in this order:
1. `cargo fmt -- --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`

Why: CI enforces both fmt and clippy -D warnings. Skipping fmt caused a red CI on v0.5.10 (#24 hotfix). Clippy alone is not enough.

How to apply: After any code change, run clippy + fmt before committing or pushing. If clippy gives warnings, fix them — don't use `#[allow(...)]` unless the warning is genuinely a false positive.

**feedback_trs_raw_bypass.md**
---
name: Use trs raw for bypass
description: Use `trs raw <cmd>` not `rtk proxy <cmd>` when raw output is needed
type: feedback
---

When raw (uncompressed) command output is needed in this project, use `trs raw <cmd>`, not `rtk proxy <cmd>`.

Why: trs is the tool we're building; using `rtk proxy` pollutes the stats history with mixed tooling and confuses the optimization analysis. `trs raw` still tracks to trs stats (0% compression), which is the correct behavior.

How to apply: Any time I'd normally reach for `rtk proxy grep`, `rtk proxy trs stats`, etc., use `trs raw grep`, `trs raw trs stats` etc. instead. `TRS_SKIP=1 <cmd>` is the alternative if tracking is also undesired.

**project_hook_migration.md**
---
name: Hook migration plan
description: Switch Claude Code hook from rtk to trs after v0.5.10 ships
type: project
---

Plan to replace rtk with trs as the sole Claude Code hook.

Why: Both rtk and trs hooks are active simultaneously, causing double-wrapping and `rtk proxy` usage that pollutes trs stats. Goal is trs-only workflow.

How to apply: After v0.5.10 is released and installed:
1. Remove `@RTK.md` from `~/.claude/CLAUDE.md`
2. Run `trs init --all --global`
3. Replace any `rtk proxy <cmd>` usage with `trs raw <cmd>`

Do NOT do the switch before v0.5.10 ships — the hook would pin to the older installed binary and miss all the v0.5.10 improvements.

## npm/

**README.md**
# trs: Token-Reducing Shell

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

**package.json**: name: @dpeluche/trs | version: 0.6.18
## npm/bin/

**trs**
#!/bin/sh
# trs: shell wrapper that execs the native binary directly.
# Saves ~25ms vs the previous Node wrapper by skipping the node runtime.
#
# The platform-specific binary is installed by npm as an optionalDependency
# into node_modules/@dpeluche/trs-cli-<os>-<arch>/trs.

set -e
... (72 lines)

**trs.cmd**
@echo off
rem trs launcher for Windows: execs the native binary directly.
rem See bin/trs for the Unix equivalent.
setlocal

set "DIR=%~dp0"
set "PKG=trs-cli-win32-x64"
set "BIN="
... (31 lines)

## npm/platforms/darwin-arm64/

**README.md**
Platform binary package for trs (darwin-arm64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-darwin-arm64 | version: 0.6.18
## npm/platforms/darwin-x64/

**README.md**
Platform binary package for trs (darwin-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-darwin-x64 | version: 0.6.18
## npm/platforms/linux-arm64/

**README.md**
Platform binary package for trs (linux-arm64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-linux-arm64 | version: 0.6.18
## npm/platforms/linux-x64/

**README.md**
Platform binary package for trs (linux-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-linux-x64 | version: 0.6.18
## npm/platforms/win32-x64/

**README.md**
Platform binary package for trs (win32-x64). Install via: npm install -g @dpeluche/trs

**package.json**: name: @dpeluche/trs-cli-win32-x64 | version: 0.6.18
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

**sync-codebase-digest.sh**
#!/usr/bin/env bash

set -euo pipefail

DIGEST="docs/development/codebase-digest.md"

if ! command -v trs >/dev/null 2>&1; then
  if [[ -x "./target/release/trs" ]]; then
... (28 lines)

**sync-version.sh**
#!/bin/bash

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ -n "$1" ]; then
  VERSION="$1"
... (32 lines)

## src/

**ai_tool.rs**
  pub enum AiTool
  pub struct HookSpec
  pub struct AiToolSpec

  impl AiTool
  fn identity(&self) -> &'static AiToolSpec
  pub fn from_str(s: &str) -> Option<Self>
  pub fn name(&self) -> &str
  pub fn all_names() -> String
  pub fn all_tools() -> Vec<Self>
  pub fn target_label(&self) -> &'static str
  pub fn detect_installed(&self) -> bool
  pub fn spec(&self) -> Option<HookSpec>
  mod tests

**ai_tool_tests.rs**
  fn antigravity_aliases_resolve_to_ide()
  fn antigravity_cli_aliases()
  fn antigravity_variants_are_rules_only()
  fn antigravity_target_label_signals_rules_only()
  fn registry_covers_every_variant_and_has_no_dup_aliases()
  fn all_names_is_the_cli_name_list()
  fn devin_cli_is_a_hook_not_rules_only()
  fn output_saver_agents_match_registry()

**audit_docs.rs**
  pub struct DocFile
  pub struct Block
  pub struct DupPair
  pub struct DeadRef
  pub struct InlineBloat
  pub struct SymbolMatch
  pub enum BloatKind
  pub fn run_audit_docs(root: &Path)
  fn discover(root: &Path) -> Vec<DocFile>
  fn collect_markdown_dir(dir: &Path, out: &mut Vec<DocFile>, root: &Path)
  fn load_doc(path: &Path, root: &Path) -> Option<DocFile>
  pub fn estimate_tokens(text: &str) -> usize
  fn find_inline_bloat(docs: &[DocFile]) -> Vec<InlineBloat>
  fn collect_code_fences(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>)
  fn fence_open_lang(line: &str) -> Option<String>
  fn is_fence_close(line: &str) -> bool
  fn collect_large_tables(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>)
  fn is_table_row(line: &str) -> bool
  fn is_table_separator(line: &str) -> bool

**audit_docs_detect.rs**
  pub fn split_into_blocks(content: &str) -> Vec<Block>
  fn flush_block(blocks: &mut Vec<Block>, buf: &mut String, start: usize, end_exclusive: usize)
  pub fn compute_simhash(text: &str) -> u64
  fn fnv1a_64(bytes: &[u8]) -> u64
  pub fn find_near_duplicates(blocks: &[Block]) -> Vec<DupPair>
  pub fn find_dead_refs(docs: &[DocFile], root: &Path) -> Vec<DeadRef>
  fn extract_references(line: &str) -> Vec<String>
  fn looks_like_import_path(s: &str) -> bool
  fn looks_like_local_markdown_link(s: &str) -> bool
  fn ref_resolves(reference: &str, doc_dir: &Path, root: &Path) -> bool

**audit_docs_report.rs**: pub fn render_report( | fn human_tokens(n: usize) -> String
**audit_docs_symbols.rs**
  pub fn extract_fence_symbols(lang: &str, body: &str) -> Vec<String>
  fn is_meaningful_symbol(name: &str) -> bool
  fn extract_js_like_symbol(line: &str) -> Option<String>
  fn extract_python_symbol(line: &str) -> Option<String>
  fn extract_rust_symbol(line: &str) -> Option<String>
  fn extract_go_symbol(line: &str) -> Option<String>
  fn extract_swift_symbol(line: &str) -> Option<String>
  pub fn resolve_symbol_matches(bloat: &mut [InlineBloat], root: &Path)
  fn contains_symbol_definition(content: &str, sym: &str, ext: &str) -> bool
  pub fn last_commit_days_ago(path: &Path, root: &Path) -> Option<u64>

**benchmark.rs**
  struct IterResult
  struct BenchReport
  pub fn run_benchmark(command: &str, args: &[String], repeat: usize, json: bool)
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

**classifier.rs**
  pub use crate::command_registry::keep_ratio
  pub fn full_cmd(cmd: &str, args: &[String]) -> String
  pub use crate::exec::build_command
  pub use crate::classifier_args::preprocess_tail_args
  pub fn classify_command(cmd: &str, args: &[String]) -> Option<ParseCommands>
  mod tests

**classifier_args.rs**
  pub fn preprocess_tail_args(args: &[String]) -> Vec<String>
  pub fn is_after_tail_subcommand(args: &[String], pos: usize) -> bool
  pub fn strip_git_global_opts(args: &[String]) -> Vec<String>
  pub fn unwrap_shell_c(args: &[String]) -> Option<Vec<String>>
  pub fn unwrap_timeout(args: &[String]) -> Option<Vec<String>>
  pub fn has_structured_output_flag(args: &[String]) -> bool

**classifier_exec.rs**
  pub fn execute_and_parse(cmd: &str, args: &[String], ctx: &CommandContext)
  fn failure_is_visible(parsed: &str) -> bool
  fn emit_failure_footer(
  fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String>
  fn write_user_only(path: &std::path::Path, content: &[u8]) -> std::io::Result<()>
  fn is_verbatim_invocation(cmd: &str, args: &[String]) -> bool
  fn generic_compress(input: &str) -> String
  pub fn collapse_repeated_lines(input: &str) -> String
  fn collapse_whitespace(s: &str) -> String

**classifier_tests.rs**
  fn argv(s: &str) -> Vec<String>
  fn ls_files_routes_to_find()
  fn commit_routes_to_parser()
  fn cargo_fmt_routes_to_parser()
  fn bash_c_simple_command_unwraps()
  fn absolute_path_routes_by_basename()
  fn bare_python_linters_and_formatters_route()
  fn timeout_unwraps_inner_command()
  fn bash_c_compound_or_quoted_stays_generic()
  fn git_show_blob_form_is_not_a_diff()

**classifier_transfer.rs**
  pub fn compact_git_transfer(combined: &str, subcmd: &str) -> String
  mod tests
  fn test_push_normal()
  fn test_push_up_to_date()
  fn test_pull_already_up_to_date()
  fn test_pull_fast_forward()
  fn test_fetch_new_branch()
  fn test_push_fatal_error()
  fn test_push_empty_output()
  fn test_push_remote_progress_stripped()
  fn test_push_remote_error_preserved()

**cli.rs**
  pub struct Cli
  pub enum OutputFormat
  pub const fn format_precedence(format: OutputFormat) -> u8

  impl Cli
  pub fn output_format_precedence() -> &'static [OutputFormat]
  pub fn output_format(&self) -> OutputFormat
  pub fn enabled_format_flags(&self) -> Vec<OutputFormat>
  pub fn has_conflicting_format_flags(&self) -> bool
  pub fn current_format_precedence(&self) -> u8

**codex.rs**
  pub fn parse_version(s: &str) -> Option<(u32, u32, u32)>
  pub fn detect_version() -> Option<(u32, u32, u32)>
  pub fn rewrite_hook_supported(version: (u32, u32, u32)) -> bool
  pub fn rewrite_hook_available() -> bool
  mod tests
  fn parses_codex_cli_prefix()
  fn tolerates_prerelease_patch()
  fn rejects_unparseable()
  fn gate_reflects_min_version()

**command_registry.rs**
  pub enum Stderr

  impl Stderr
  fn matches(&self, subcmd: &str) -> bool
  pub struct KeepRatio

  impl KeepRatio
  fn lookup(&self, subcmd: &str) -> f64
  pub struct CommandSpec
  pub fn spec(cmd: &str) -> Option<&'static CommandSpec>
  pub fn keep_ratio(cmd: &str, subcmd: &str) -> f64
  pub fn combine_stderr(cmd: &str, subcmd: &str) -> bool
  pub fn is_verbatim_command(cmd: &str) -> bool
  pub fn is_rewrite_command(cmd: &str) -> bool
  pub fn is_known_binary(cmd: &str) -> bool
  mod tests

**command_registry_tests.rs**
  fn r(cmd: &str, sub: &str) -> f64
  fn keep_ratio_git_subcommands()
  fn keep_ratio_flat_commands()
  fn keep_ratio_package_manager_overrides()
  fn keep_ratio_test_and_run()
  fn keep_ratio_cargo_specific()
  fn keep_ratio_misc_overrides()
  fn keep_ratio_default_for_unknown_and_no_entry_commands()
  fn combine_stderr_always_commands()
  fn combine_stderr_subcommand_scoped()
  fn combine_stderr_never_commands()
  fn is_known_binary_matches_golden_set_exactly()
  fn rewrite_eligibility_matches_legacy_prefixes()
  fn no_duplicate_command_names()
  fn verbatim_commands_are_known_and_never_rewritten()
  fn compressible_commands_stay_out_of_the_verbatim_class()
  fn bunx_dispatches_like_npx()

**commands.rs**
  mod commands_parse
  pub use commands_parse::ParseCommands
  pub enum Commands
  pub enum ReadLevel
  pub enum TestRunner

**commands_parse.rs**
  pub enum ParseCommands

  impl ParseCommands
  pub fn with_file(path: PathBuf) -> Self

**config.rs**
  pub fn config() -> &'static Config
  pub struct Config
  pub struct Hooks
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
  pub fn run(output: Option<&str>)
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

**diff.rs**
  pub fn run_diff(command: &str, args: &[String], json: bool)
  fn dropped_lines(raw: &str, compact: &str) -> Vec<String>
  fn tokens(bytes: usize) -> u64
  fn reduction_pct(raw: usize, compact: usize) -> f64
  fn capture_raw(cmd: &str, args: &[String]) -> Option<String>
  fn capture_compact(cmd: &str, args: &[String]) -> Option<String>
  fn print_human(
  fn print_json(
  fn format_number(n: u64) -> String
  mod tests
  fn dropped_lines_are_raw_minus_compact()
  fn nothing_dropped_when_compact_has_all_lines()
  fn metrics_math()

**discover.rs**
  pub fn run_discover(all_projects: bool, since_days: usize)
  fn scan_directory(
  fn extract_command(line: &str) -> Option<String>
  mod tests
  fn test_extract_command()
  fn test_extract_command_escaped()
  fn test_extract_command_none()

**doctor.rs**
  pub struct Check

  impl Check
  pub fn pass(name: &'static str, detail: impl Into<String>) -> Self
  pub fn warn(name: &'static str, detail: impl Into<String>) -> Self
  pub fn fail(name: &'static str, detail: impl Into<String>) -> Self
  pub fn with_sub(mut self, lines: Vec<String>) -> Self
  pub fn with_hint(mut self, hint: impl Into<String>) -> Self
  pub enum CheckStatus

  impl fmt::Display for CheckStatus
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result

  struct Summary
  impl Summary
  fn from_checks(checks: &[Check]) -> Self
  pub fn run_checks() -> Vec<Check>
  pub fn print_report(checks: &[Check])
  pub fn print_report_json(checks: &[Check])
  mod tests

**doctor_checks.rs**
  pub fn check_codex_hooks_orphan() -> Check
  pub fn check_devin_cli_hook() -> Check
  pub fn check_output_saver_installed() -> Check
  pub fn check_agent_docs_health() -> Check
  fn human_k(n: usize) -> String
  pub fn check_version() -> Check
  pub fn check_path_accessible() -> Check
  pub fn check_dep(cmd: &str, label: &str, required: bool, hint: &str) -> Check
  pub fn check_config_dir() -> Check
  pub fn check_history_writable() -> Check
  pub fn check_stdin_pipeline() -> Check
  pub fn check_hooks_installed() -> Check

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

**exec.rs**
pub(crate) fn build_command(cmd: &str, args: &[String]) -> std::process::Command {
    use std::process::Command;
    #[cfg(windows)]
    {
        if cmd.to_ascii_lowercase().ends_with(".ps1") {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-File", cmd]);
            c.args(args);
... (67 lines)

**fast_find.rs**
  pub fn run(args: &[&str], ctx: &CommandContext)
  fn print_compact(files: &[String], dirs: &[String], _root: &str)
  fn print_raw(files: &[String], dirs: &[String])
  fn print_json(files: &[String], dirs: &[String])
  fn glob_match(pattern: &str, name: &str) -> bool
  fn glob_match_inner(p: &[char], n: &[char]) -> bool
  mod tests
  fn test_glob_match()

**help.rs**
  pub use crate::help_text::*
  pub use crate::help_text_more::*
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

**help_text.rs**: (266 lines)
**help_text_more.rs**: (271 lines)
**init.rs**
  pub struct InstallOpts
  pub use crate::ai_tool::{AiTool, HookSpec
  fn agent_row(sym: char, name: &str, detail: &str)
  pub fn install_hook(tool: &AiTool, opts: InstallOpts, batch: bool) -> bool
  fn install_trs_md_for(tool: &AiTool, quiet: bool)
  pub fn install_all(opts: InstallOpts)
  fn is_trs_in_path() -> bool
  pub fn check_tool(tool: &AiTool) -> bool
  pub fn has_trs_marker(content: &str) -> bool
  pub fn file_has_any_trs_marker(content: &str) -> bool
  fn has_any_trs_marker_at(path_str: &str) -> bool
  fn devin_desktop_present() -> bool
  fn has_any_trs_marker_at_path(path: &Path) -> bool
  pub fn home_dir() -> Result<PathBuf>
  fn check_file_contains_path(path: &Path, needle: &str) -> bool

**init_collision.rs**
  pub struct Collision
  pub enum CollisionKind
  pub fn detect(tool: &AiTool, global: bool) -> Vec<Collision>
  fn target_paths(tool: &AiTool, _global: bool) -> Vec<PathBuf>
  fn scan_json(path: &Path) -> Vec<Collision>
  fn scan_text(path: &Path, depth: usize, visited: &mut HashSet<PathBuf>) -> Vec<Collision>
  fn extract_imports(content: &str, base_file: &Path) -> Vec<PathBuf>
  fn resolve_import(target: &str, base_file: &Path) -> Option<PathBuf>
  fn collect_string_values(val: &serde_json::Value, out: &mut Vec<String>)
  fn truncate(s: &str, max: usize) -> String
  pub fn format_report(tool: &AiTool, collisions: &[Collision]) -> String
  pub fn any_hook_collisions(collisions: &[Collision]) -> bool
  pub fn scrub_file(path: &Path) -> Result<bool>
  pub fn is_json_location(c: &Collision) -> bool
  pub fn is_competitor_hook(val: &serde_json::Value) -> bool
  mod tests

**init_collision_tests.rs**
  fn scan_json_flags_rtk_hook()
  fn scan_json_ignores_trs_hook()
  fn scan_text_flags_rtk_rules()
  fn scan_text_follows_at_imports()
  fn scan_text_breaks_import_cycle()
  fn resolve_import_handles_home_and_relative()
  fn is_competitor_hook_matches_nested()
  fn is_competitor_hook_rejects_trs()

**init_install.rs**
  pub fn install_from_spec(spec: &HookSpec, opts: InstallOpts) -> Result<String>
  pub fn install_codex_agents(opts: InstallOpts) -> Result<String>
  fn write_agents_md_block(path: &Path, opts: InstallOpts) -> Result<String>
  pub fn install_zed_agents(opts: InstallOpts) -> Result<String>
  pub fn install_antigravity_rules(opts: InstallOpts) -> Result<String>
  fn remove_hookspec_at(path: &Path, dry_run: bool) -> Result<()>
  pub fn install_rules(
  fn ensure_parent(path: &Path) -> Result<()>
  fn write_hook(dir: &Path, path: &Path, content: &str, opts: InstallOpts) -> Result<String>
  fn merge_json_hook(
  pub fn contains_trs_rewrite(val: &serde_json::Value) -> bool
  pub fn scrub_legacy_codex_hook(
  mod tests

**init_install_plugins.rs**
  pub fn hermes_home() -> Result<PathBuf>
  pub fn install_openclaw_plugin(opts: InstallOpts) -> Result<String>
  pub fn install_hermes_plugin(opts: InstallOpts) -> Result<String>
  fn write_plugin_file(path: &Path, content: &str, opts: InstallOpts) -> Result<bool>
  fn read_json_or_empty(path: &Path) -> Result<serde_json::Value>
  fn ensure_parent(path: &Path) -> Result<()>
  pub fn merge_openclaw_config(
  pub enum HermesConfigPatch
  pub fn patch_hermes_config(existing: &str) -> HermesConfigPatch
  fn indent_of(line: &str) -> usize
  mod tests
  fn openclaw_merge_fresh_config()
  fn openclaw_merge_preserves_existing_entries()
  fn openclaw_merge_is_idempotent()
  fn openclaw_merge_rejects_non_object_plugins()
  fn hermes_patch_empty_file_writes_full_block()
  fn hermes_patch_appends_to_existing_enabled_list()
  fn hermes_patch_matches_sibling_indentation()
  fn hermes_patch_already_present_is_noop()
  fn hermes_patch_empty_enabled_list_gets_first_item()
  fn hermes_patch_appends_block_when_plugins_key_absent()
  fn hermes_patch_exotic_layouts_fall_back_to_manual()

**init_install_tests.rs**
  fn scrub_legacy_codex_hook_removes_trs_only_event()
  fn scrub_legacy_codex_hook_preserves_user_entries_in_same_event()
  fn scrub_legacy_codex_hook_is_noop_when_clean()
  fn scrub_legacy_codex_hook_dry_run_does_not_write()

**init_show.rs**: pub fn show_status() | pub fn show_status_and_usage()
**init_templates.rs**
  pub use crate::init_templates_plugins::*
  mod tests
  fn opencode_kilo_plugins_are_windows_safe_and_idempotent()

**init_templates_plugins.rs**: (154 lines)
**main.rs**
  mod ai_tool
  mod audit_docs
  mod audit_docs_detect
  mod audit_docs_report
  mod audit_docs_symbols
  mod benchmark
  mod classifier
  mod classifier_args
  mod classifier_exec
  mod classifier_transfer
  mod cli
  mod codex
  mod command_registry
  mod commands
  pub mod config
  mod debug_info
  mod diff
  mod discover
  mod doctor
  mod doctor_checks
  mod exec
  mod formatter
  mod help
  mod help_text
  mod help_text_more
  mod ingest
  mod init
  mod init_collision
  mod init_install
  mod init_install_plugins
  mod init_show
  mod init_templates
  mod init_templates_plugins
  mod output_saver
  mod output_saver_core
  mod parse_out
  mod path_display
  mod process
  mod reducer
  mod rewrite
  mod rewrite_decide
  mod router
  mod schema
  mod text_util
  pub mod tracker
  mod uninstall
  mod upgrade
  pub use cli::format_precedence
  pub use cli::{Cli, OutputFormat
  pub use commands::{Commands, ParseCommands, TestRunner
  mod fast_find
  mod read_intercept
  fn main()
  fn run()
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
  pub fn standalone_file() -> String
  pub fn sentinel_wrapped() -> String
  pub use crate::output_saver_core
  pub fn run(
  fn run_verify(targets: &[&str])
  fn agent_display(id: &str) -> &'static str
  fn run_scan(targets: &[&str])
  fn run_install(targets: &[&str])
  fn run_refresh(targets: &[&str])
  fn run_remove(targets: &[&str])

**output_saver_core.rs**
  enum Target
  pub struct Agent
  fn resolve_target_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Target
  pub enum Status
  pub enum VerifyStatus
  pub fn verify_agent(agent_id: &str) -> VerifyStatus
  fn verify_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> VerifyStatus
  pub fn scan_agent(agent_id: &str) -> Status
  fn scan_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Status
  pub fn install_agent(agent_id: &str) -> Result<String>
  fn install_agent_with_home(
  pub fn remove_agent(agent_id: &str) -> Result<String>
  fn remove_agent_with_home(
  pub fn replace_between(content: &str, start: &str, end: &str, new_block: &str) -> String
  mod tests

**output_saver_core_tests.rs**
  fn replace_between_swaps_segment()
  fn standalone_file_contains_block()
  fn standalone_file_does_not_promote_bypass_mechanisms()
  fn sentinel_wrapped_is_idempotent_on_replace()
  fn scan_unknown_agent_returns_unsupported()
  fn install_and_remove_imported_agent_roundtrip()
  fn install_migrates_legacy_file()
  fn install_inline_file_is_idempotent()
  fn verify_agent_reports_loaded_drifted_and_not_installed()

**parse_out.rs**

  thread_local! {
      static SINK: RefCell<Option<String>> = const { RefCell::new(None) };
  }

  pub(crate) fn emit(s: &str) {
      SINK.with(|sink| {
          let mut slot = sink.borrow_mut();
          match slot.as_mut() {
              Some(buf) => buf.push_str(s),
              None => {
                  let _ = write!(std::io::stdout(), "{}", s);
              }
          }
      });
  }

  pub(crate) fn capture<F: FnOnce()>(f: F) -> String {
      SINK.with(|sink| *sink.borrow_mut() = Some(String::new()));
      f();
      SINK.with(|sink| sink.borrow_mut().take().unwrap_or_default())
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn capture_collects_emits_and_resets() {
          let out = capture(|| {
              emit("hello ");
              emit("world");
          });
          assert_eq!(out, "hello world");
          SINK.with(|s| assert!(s.borrow().is_none()));
      }

      #[test]
      fn empty_capture_is_empty_string() {
          assert_eq!(capture(|| {}), "");
      }
  }

**path_display.rs**

  pub(crate) fn display_path(path: &Path) -> String {
      normalize(&path.to_string_lossy())
  }

  pub(crate) fn tilde(s: &str) -> String {
      let s = normalize(s);
      let home = std::env::var("HOME")
          .or_else(|_| std::env::var("USERPROFILE"))
          .ok()
          .map(|h| normalize(&h));
      match home {
          Some(h) if !h.is_empty() && s.starts_with(&h) => {
              format!("~{}", &s[h.len()..])
          }
          _ => s,
      }
  }

  pub(crate) fn normalize(s: &str) -> String {
      #[cfg(windows)]
      {
          s.replace('\\', "/")
      }
      #[cfg(not(windows))]
      {
          s.to_string()
      }
  }

  #[cfg(test)]
  mod tests {
      use super::normalize;

      #[test]
      fn normalizes_on_windows_only() {
          let out = normalize(r"src\router\handlers");
          #[cfg(windows)]
          assert_eq!(out, "src/router/handlers");
          #[cfg(not(windows))]
          assert_eq!(out, r"src\router\handlers");
          assert_eq!(normalize("src/router/handlers"), "src/router/handlers");
      }

      #[test]
      fn tilde_collapses_home_prefix() {
          use super::tilde;
          let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
          if let Ok(h) = home {
              if !h.is_empty() {
                  assert_eq!(
                      tilde(&format!("{h}/.gemini/settings.json")),
                      "~/.gemini/settings.json"
                  );
              }
          }
          assert_eq!(tilde("/etc/hosts"), "/etc/hosts");
      }
  }

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
  pub(super) fn classify_spawn_error(command: &str, error: io::Error) -> ProcessError
  pub(super) fn capture_output(
  pub(super) fn capture_partial_output(child: &mut std::process::Child) -> (String, String)
  pub(super) trait ChildExt
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

**read_intercept.rs**
  pub fn try_cat(args: &[String], _ctx: &CommandContext) -> bool
  pub fn try_head(args: &[String], _ctx: &CommandContext) -> bool
  pub fn try_sed(args: &[String], _ctx: &CommandContext) -> bool
  fn parse_head_args(args: &[String]) -> Option<(usize, String)>
  fn parse_sed_range(args: &[String]) -> Option<(u32, u32, String)>
  fn filtered_or_raw(filtered: String, raw: String, lang: Language) -> String
  fn parse_range_expr(s: &str) -> Option<(u32, u32)>
  mod tests
  fn parse_range_basic()
  fn parse_range_invalid()
  fn parse_sed_standard()
  fn parse_sed_combined_flag()
  fn parse_sed_no_n_flag()
  fn parse_sed_inplace_edit()
  fn parse_sed_substitution()
  fn parse_head_long_form()
  fn parse_head_short_form()
  fn parse_head_default()

**rewrite.rs**
  pub fn run_rewrite(agent_flag: Option<&str>)
  fn handle_json_protocol(json: &serde_json::Value, agent_flag: Option<&str>)
  fn known_agent_label(s: &str) -> Option<&'static str>

  enum HookEvent
  impl HookEvent
  fn parse(name: &str) -> Self
  fn agent_label(&self) -> &'static str
  fn agent_label_from(&self, has_antigravity_env: bool, trs_agent: Option<&str>) -> &'static str
  fn agent_label_for(&self, has_antigravity_env: bool) -> &'static str
  fn build_hook_response(
  fn cmd_bypasses_trs(cmd: &str) -> bool
  fn tag_with_agent(cmd: &str, agent: &str) -> String
  fn tag_with_agent_for(cmd: &str, agent: &str, is_windows: bool) -> String
  mod tests

**rewrite_decide.rs**
  pub fn maybe_rewrite(cmd: &str) -> Option<String>
  pub(super) fn split_env_prefix(cmd: &str) -> Option<(String, &str)>
  pub fn looks_like_env_assignment(token: &str) -> bool
  fn strip_transparent_prefix(cmd: &str) -> Option<(&str, &str)>
  pub(super) fn strip_word_prefix<'a>(cmd: &'a str, prefix: &str) -> Option<&'a str>
  fn is_simple_command(cmd: &str) -> bool
  fn contains_unquoted(cmd: &str, needle: char) -> bool
  fn find_unquoted(cmd: &str, needle: char) -> Option<usize>
  fn split_and_chain(cmd: &str) -> Vec<&str>
  fn find_unquoted_str(cmd: &str, needle: &str) -> Option<usize>
  pub(super) fn captures_output(cmd: &str) -> bool
  fn split_at_shell_op(s: &str) -> Option<(&str, &str)>
  mod tests

**rewrite_decide_tests.rs**
  fn test_rewrite_git()
  fn test_rewrite_cargo()
  fn test_skip_already_trs()
  fn test_skip_cd()
  fn test_skip_trs_skip_env_var()
  fn test_transparent_prefix_builtins()
  fn test_transparent_prefix_partial_match_not_stripped()
  fn test_transparent_prefix_alone_not_stripped()
  fn test_transparent_prefix_composes_with_env()
  fn test_transparent_prefix_nested()
  fn test_transparent_prefix_venv_runners()
  fn test_npm_run_not_transparent_prefix()
  fn test_transparent_prefix_skips_when_inner_skipped()
  fn test_strip_word_prefix_strict()
  fn test_trs_skip_does_not_match_other_env_vars()
  fn test_env_prefix_stays_in_front()
  fn test_multi_env_prefix()
  fn test_env_prefix_before_unknown_command_still_rewrites()
  fn test_flag_looks_like_assignment_not_matched()
  fn test_split_env_prefix_empty_when_none()
  fn test_stderr_redirects_survive_rewrite()
  fn test_rewrite_pipe_first_segment()
  fn test_rewrite_multi_pipe_first_segment_only()
  fn test_captured_output_is_left_raw()
  fn test_pipes_and_discards_still_rewrite()
  fn test_skip_pipe_when_first_segment_is_skipped()
  fn test_skip_subshells()
  fn test_skip_assignments()
  fn test_skip_empty()
  fn test_rewrite_unknown_command()
  fn test_skip_echo()
  fn test_json_protocol()
  fn test_rewrite_env()
  fn test_skip_shell_builtins()
  fn test_rewrite_cd_chain()
  fn test_rewrite_multi_chain()
  fn test_skip_cd_chain_with_pipe()
  fn test_skip_cd_chain_all_skips()
  fn test_rewrite_inline_scripts_and_generic_clis()
  fn test_node_prefix_does_not_match_nodemon()
  fn test_never_rewrites_inside_heredoc_or_quotes()
  fn test_never_rewrites_multiline_scripts()
  fn test_never_wraps_shell_keywords()
  fn test_simple_commands_still_compress()
  fn test_never_rewrites_compound_or_array_shapes()
  fn test_legitimate_env_prefix_still_wraps()
  fn verbatim_commands_are_left_unwrapped()
  fn verbatim_gate_does_not_swallow_compressible_commands()

**rewrite_tests.rs**
  fn test_cmd_bypasses_trs_detection()
  fn test_cmd_bypasses_trs_disable_alias()
  fn test_cmd_bypasses_env_wrapped()
  fn parse_input(s: &str) -> serde_json::Value
  fn agent_cmd(agent: &str, rest: &str) -> String
  fn test_hook_response_claude_code_format()
  fn test_hook_response_cursor_format()
  fn test_agent_flag_attributes_rewrite()
  fn test_hook_response_unknown_event_fails_open()
  fn test_hook_event_parse_mapping()
  fn test_hook_response_gemini_format()
  fn test_agent_label_antigravity_env_disambiguates_claude_envelope()
  fn test_hook_response_default_is_claude_format()
  fn test_hook_response_no_rewrite_returns_none()
  fn test_hook_response_missing_command_returns_none()
  fn test_hook_response_chain_preserved_across_formats()
  fn tag_with_agent_skips_posix_prefix_on_windows()

**text_util.rs**
pub(crate) fn first_ident(s: &str) -> Option<String> {
    let mut chars = s.char_indices();
    let start = chars.position(|(_, c)| c.is_ascii_alphabetic() || c == '_')?;
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        if c.is_ascii_alphanumeric() || c == '_' {
            end = start + i + c.len_utf8();
        } else {
... (31 lines)

**tracker.rs**
  pub struct HistoryEntry
  fn history_path() -> Option<PathBuf>
  fn dirs_path() -> Option<PathBuf>
  pub fn home_dir() -> Option<PathBuf>
  fn append_history_entry(entry: &HistoryEntry)
  fn maybe_rotate_active(path: &Path, now_ts: u64)
  fn peek_first_entry_ts(path: &Path) -> Option<u64>
  fn month_key_from_ts(ts: u64) -> String
  fn open_user_only(path: &Path) -> std::io::Result<fs::File>
  pub fn log_execution(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64)
  pub fn redact_secrets(cmd: &str) -> String
  pub fn log_bypass(cmd: &str, agent: Option<&str>)
  pub fn read_history() -> Vec<HistoryEntry>
  fn read_jsonl(path: &Path) -> Vec<HistoryEntry>
  fn is_history_archive(path: &Path) -> bool
  pub fn prune_archives(days: u64, dry_run: bool) -> (usize, u64)
  pub fn read_project_history() -> Vec<HistoryEntry>
  pub fn format_bytes_human(bytes: usize) -> String
  mod tests

**tracker_tests.rs**
  fn month_key_from_ts_pads_zero()
  fn month_key_from_ts_handles_december()
  fn is_history_archive_recognizes_monthly_pattern()
  fn maybe_rotate_active_renames_when_month_differs()
  fn maybe_rotate_active_is_noop_in_same_month()
  fn redact_curl_basic_auth()
  fn redact_url_basic_auth()
  fn redact_password_flag()
  fn redact_authorization_header()
  fn redact_token_shapes()
  fn redact_leaves_normal_commands_alone()
  fn test_format_bytes_human()
  fn test_saved_pct_calculation()
  fn test_history_entry_serialization()
  fn test_history_entry_legacy_lines_deserialize()
  fn test_bypass_entry_round_trip()

**uninstall.rs**
  pub struct UninstallOpts
  pub fn run_uninstall(tool: Option<&str>, opts: UninstallOpts)
  fn run_all(opts: UninstallOpts)
  fn run_output_saver_only(opts: UninstallOpts)
  fn print_dry_run_note(opts: UninstallOpts)
  fn run_interactive(opts: UninstallOpts)
  fn uninstall_one(tool: &AiTool, opts: UninstallOpts)
  fn candidate_paths(tool: &AiTool) -> Vec<PathBuf>
  fn scrub_trs_from_json(path: &Path, dry_run: bool) -> Result<Option<String>>
  fn remove_between_sentinels(
  fn delete_plugin_file(path: &Path, dry_run: bool) -> Result<Option<String>>
  fn delete_rules_file(path: &Path, dry_run: bool) -> Result<Option<String>>
  fn remove_output_saver(agent_id: &str, dry_run: bool) -> Result<String>
  fn has_output_saver_installed(agent_id: &str) -> bool
  fn run_output_saver_removal(tool_name: &str, agent_id: &str, dry_run: bool)
  fn has_trs_artifacts(tool: &AiTool) -> bool
  fn output_saver_agent_id(tool: &AiTool) -> Option<&'static str>
  fn is_trs_plugin_dir_file(path: &Path) -> bool
  fn is_json(path: &Path) -> bool
  fn confirm(prompt: &str) -> bool

**upgrade.rs**
  pub enum InstallMethod

  impl InstallMethod
  fn label(&self) -> &'static str
  pub fn run_upgrade(check_only: bool, skip_confirm: bool, binary_only: bool)
  pub fn detect_install_method(exe: Option<&Path>) -> InstallMethod
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

  #[allow(dead_code)]
  pub struct AgentFormatter;

  impl Formatter for AgentFormatter {
      fn name() -> &'static str {
          "agent"
      }

      fn format() -> OutputFormat {
          OutputFormat::Agent
      }
  }

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

  #[allow(dead_code)]
  pub struct CsvFormatter;

  impl Formatter for CsvFormatter {
      fn name() -> &'static str {
          "csv"
      }

      fn format() -> OutputFormat {
          OutputFormat::Csv
      }
  }

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
  mod compact;
  mod compact_schema_git;
  mod compact_schema_output;
  mod csv;
  pub mod helpers;
  mod json;
  mod json_schema;
  mod raw;
  mod tsv;

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

  #[allow(dead_code)]
  pub struct TsvFormatter;

  impl Formatter for TsvFormatter {
      fn name() -> &'static str {
          "tsv"
      }

      fn format() -> OutputFormat {
          OutputFormat::Tsv
      }
  }

## src/ingest/

**collect.rs**
pub(super) fn collect_files(config: &IngestConfig) -> Vec<DigestFile>
pub(super) fn get_changed_files(root: &Path, since: Option<&str>) -> Option<Vec<String>>
pub(super) fn apply_budget(

**collect_compress.rs**
  pub(super) struct CompressResult
  pub(super) fn read_and_compress(path: &Path, level: IngestLevel) -> Option<CompressResult>
  fn has_multiline_python_sig(content: &str) -> bool
  fn join_python_multiline_sigs(content: &str) -> String
  fn extract_signatures(content: &str, ext: &str) -> String
  fn clean_signature(line: &str) -> String

**collect_index.rs**
  pub(super) fn is_module_anchor(lower_name: &str) -> bool
  pub(super) fn extract_module_doc(content: &str, ext: &str) -> Option<String>
  fn contains_manifest_field(s: &str) -> bool
  fn looks_like_config_line(s: &str) -> bool
  fn first_rust_module_doc(content: &str) -> Option<String>
  fn first_python_docstring(content: &str) -> Option<String>
  fn first_jsdoc_summary(content: &str) -> Option<String>
  pub(super) fn extract_symbols(content: &str, ext: &str) -> Vec<String>
  fn symbol_from_rust(line: &str) -> Option<String>
  fn symbol_from_python(line: &str) -> Option<String>
  fn symbol_from_ts(line: &str) -> Option<String>
  fn symbol_from_go(line: &str) -> Option<String>
  fn symbol_from_swift(line: &str) -> Option<String>
  fn symbol_from_java(line: &str) -> Option<String>
  mod tests
  fn captures_visibility_qualified_items()

**collect_manifests.rs**
pub(super) fn extract_data_schema(content: &str, ext: &str) -> String
pub(super) fn summarize_json_value(val: &serde_json::Value, depth: usize) -> String
pub(super) fn compress_toml_manifest(content: &str) -> String
pub(super) fn compress_jupyter_notebook(content: &str) -> String
pub(super) fn compress_package_json(content: &str) -> String

**deps.rs**
  pub(super) struct DepGraph

  impl DepGraph
  pub fn top_central(&self, n: usize) -> Vec<(&str, &Vec<String>)>
  pub fn is_empty(&self) -> bool
  pub(super) fn build_dep_graph(files: &[DigestFile]) -> DepGraph
  fn resolve_imports(
  fn resolve_relative(import: &str, importer_dir: &str, all_paths: &[&str]) -> Option<String>
  fn resolve_by_suffix(suffix: &str, all_paths: &[&str]) -> Option<String>
  fn resolve_by_stem(name: &str, stem_index: &HashMap<String, Vec<&str>>) -> Option<String>
  fn resolve_module_path(import: &str, all_paths: &[&str]) -> Option<String>
  fn normalize_path(base: &str, import: &str) -> String
  pub(super) fn format_dep_summary(graph: &DepGraph) -> String
  fn short_label(rel_path: &str, all_paths: &[&str]) -> String
  pub(super) fn format_dep_full(graph: &DepGraph, project_name: &str) -> String
  mod tests
  fn test_normalize_path()
  fn test_build_dep_graph_empty()
  fn test_dep_summary_skips_singletons()

**deps_extract.rs**
  pub fn extract_raw_imports(content: &str, ext: &str) -> Vec<String>
  fn extract_rust(content: &str) -> Vec<String>
  fn rust_mod_name(t: &str) -> Option<String>
  fn collect_path_refs(line: &str, prefix: &str, out: &mut Vec<String>)
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

**dupes.rs**
  pub(super) struct Dupe
  fn seed(k: usize) -> u64
  fn hash64(x: u64) -> u64
  fn tokenize(body: &str) -> Vec<String>
  fn shingles(tokens: &[String]) -> HashSet<u64>
  fn minhash(set: &HashSet<u64>) -> [u64; K]
  fn jaccard(a: &HashSet<u64>, b: &HashSet<u64>) -> f64

  struct Func
  fn functions(rel: &str, content: &str) -> Vec<Func>
  fn fn_name(line: &str, is_py: bool) -> Option<String>
  fn ident(s: &str) -> String
  fn capture_braces(lines: &[&str], start: usize) -> (String, usize)
  fn capture_py(lines: &[&str], start: usize) -> (String, usize)
  pub(super) fn find_dupes(files: &[&DigestFile]) -> Vec<Dupe>
  fn is_code(rel: &str) -> bool

**format.rs**
  pub(super) fn build_digest(
  fn build_roles_section(files: &[DigestFile]) -> String
  pub fn detect_primary_language(files: &[DigestFile]) -> String
  fn format_file_entry(out: &mut String, name: &str, content: &str)
  pub use super::format_tree::{build_symbol_index, build_tree
  pub fn strip_html_from_markdown(content: &str) -> String
  pub fn format_bytes(n: usize) -> String
  pub(super) fn extract_section(digest: &str, filename: &str) -> Option<String>
  pub(super) fn days_to_date(days: u64) -> (u64, u64, u64)
  pub fn format_tokens(n: usize) -> String
  pub(super) fn format_modified(entry: &std::fs::DirEntry) -> String

**format_html.rs**: pub(super) fn format_html(
**format_html_util.rs**
pub(super) fn is_code(rel: &str) -> bool
pub(super) fn is_entry_module(m: &str) -> bool
pub(super) fn esc(s: &str) -> String
pub(super) fn human(n: usize) -> String
pub(super) fn human_bytes(b: u64) -> String
pub(super) fn json_str(s: &str) -> String
pub(super) fn scan_assets(root: &Path) -> (String, usize, u64)

**format_tree.rs**
  pub fn build_tree(files: &[DigestFile]) -> String
  pub fn build_symbol_index(files: &[DigestFile]) -> String
  fn collect_dir_annotations(files: &[DigestFile]) -> BTreeMap<String, String>

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
  mod dupes
  mod format
  mod format_html
  mod format_html_util
  mod format_tree
  mod meta
  mod mod_html
  mod ollama
  mod purpose
  mod remote
  mod resolve
  mod store
  pub use remote::{is_remote_ref, resolve_remote, TmpMode
  pub use resolve::resolve_project_root
  pub use store::{list_ingests, read_digest
  pub enum IngestLevel

  impl IngestLevel
  pub fn from_str(s: &str) -> Self
  pub struct IngestConfig
  pub struct DigestFile
  pub fn run_ingest(config: &IngestConfig)
  mod tests

**mod_html.rs**: (188 lines)
**ollama.rs**
  pub fn list_ollama_models()
  fn get_ollama_models() -> Option<Vec<(String, String, String)>>
  fn pick_default_model() -> Option<String>
  pub(super) fn ollama_format(digest: &str, model: &str) -> Option<String>

**purpose.rs**
  pub(super) fn module_of(rel: &str) -> String
  pub(super) fn about(files: &[DigestFile]) -> Option<String>
  fn kv_value(content: &str, key: &str) -> Option<String>
  fn json_value(content: &str, key: &str) -> Option<String>
  fn readme_first_para(content: &str) -> Option<String>
  fn is_ordered_item(t: &str) -> bool
  pub(super) fn truncate(s: &str, max: usize) -> String
  pub(super) fn module_edges(files: &[DigestFile]) -> HashMap<(String, String), usize>
  pub(super) fn degrees(
  pub(super) fn core_floor(in_deg: &HashMap<String, usize>) -> usize
  pub(super) fn role_of(in_deg: usize, out_deg: usize, core_floor: usize) -> &'static str
  pub(super) struct RoleInfo
  pub(super) fn roles(files: &[DigestFile], top: usize) -> Vec<RoleInfo>
  mod tests
  fn df(rel: &str, content: &str) -> DigestFile
  fn readme_para_skips_lists_and_blockquotes()
  fn about_prefers_root_readme_over_nested_manifest()
  fn module_of_strips_source_root_and_uses_stem()
  fn role_of_classifies_by_topology()
  fn about_prefers_manifest_description()
  fn about_falls_back_to_readme_paragraph()

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

**resolve.rs**: pub fn resolve_project_root(path: &Path) -> Result<PathBuf> | pub(super) fn suggest_budget(n: usize) -> &'static str
**store.rs**
  struct ListEntry
  pub fn list_ingests()
  pub fn read_digest(name: Option<&str>, project_path: &Path)
  fn ingest_store_dir() -> Option<PathBuf>
  fn get_repo_identity(root: &Path) -> (String, String)
  fn get_repo_name(root: &Path) -> String
  pub(super) fn save_to_store(content: &str, config: &IngestConfig) -> Option<String>
  pub(super) fn digest_path_for(root: &Path) -> Option<PathBuf>
  pub(super) fn stored_head_for(root: &Path) -> Option<String>

**tests.rs**
  fn test_format_tokens()
  fn test_format_bytes()
  fn test_ingest_level_from_str()
  fn test_skip_extensions()
  fn test_skip_files()
  fn test_build_tree()

## src/reducer/

**mod.rs**
  pub mod output
  mod registry
  mod truncation
  mod tests
  pub use output::escape_csv
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
  pub fn escape_csv(value: &str) -> String

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
  pub mod handlers
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

**ansi.rs**
  pub fn strip_ansi_codes(s: &str) -> String
  pub fn strip_emojis(s: &str) -> String
  fn is_emoji(c: char) -> bool
  pub fn sanitize_control_chars(s: &str) -> String

**clean.rs**
  pub struct CleanHandler

  impl CleanHandler
  pub fn read_input(&self, file: &Option<std::path::PathBuf>) -> CommandResult<String>
  pub fn clean_text(&self, text: &str, options: &CleanInput) -> String
  pub fn collapse_blank_lines(&self, text: &str) -> String
  pub fn collapse_repeated_lines(&self, text: &str) -> String
  pub fn format_output(

  impl CommandHandler for CleanHandler
  type Input = CleanInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct CleanInput

**common.rs**
  pub use super::ansi::{sanitize_control_chars, strip_ansi_codes, strip_emojis
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
  mod child_exit
  pub fn set(code: i32)
  pub fn get() -> Option<i32>
  pub use child_exit::set as set_child_exit
  pub fn child_failed() -> bool
  pub fn child_exit_code() -> Option<i32>
  pub fn estimate_tokens(bytes: usize) -> usize
  pub fn escape_csv_field(field: &str) -> String
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
  pub fn format_output_mode(mode: OutputFormat) -> &'static str
  pub fn is_error_line(line: &str) -> bool
  fn has_coded_diagnostic(lower: &str, word: &str) -> bool
  pub fn is_warning_line(line: &str) -> bool
  pub fn output_has_failure_signal(text: &str) -> bool
  pub fn contains_credential(line: &str) -> bool

**err.rs**
  fn is_error_line(line: &str) -> bool
  fn is_error_not_warning(line: &str) -> bool
  pub fn handle_err(command: &str, args: &[String], _ctx: &CommandContext) -> CommandResult

**html2md.rs**
  pub struct Html2mdHandler

  impl Html2mdHandler
  pub fn is_url(input: &str) -> bool
  pub fn fetch_url(&self, url: &str) -> CommandResult<String>
  pub fn read_file(&self, path: &str) -> CommandResult<String>
  pub fn extract_metadata(&self, html: &str, url_or_file: &str) -> serde_json::Value
  pub fn convert_to_markdown(&self, html: &str) -> CommandResult<String>
  pub fn format_output(

  impl CommandHandler for Html2mdHandler
  type Input = Html2mdInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct Html2mdInput

**isclean.rs**
  pub struct IsCleanHandler

  impl IsCleanHandler
  pub fn check_repo_state(check_untracked: bool) -> CommandResult<RepositoryState>
  pub fn format_output(state: &RepositoryState, format: OutputFormat) -> String
  pub fn format_json(state: &RepositoryState) -> String
  pub fn format_compact(state: &RepositoryState) -> String
  pub fn format_raw(state: &RepositoryState) -> String

  impl CommandHandler for IsCleanHandler
  type Input = IsCleanInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct RepositoryState
  pub struct IsCleanInput

**json.rs**
  mod json_query
  pub struct JsonInput
  pub struct JsonHandler

  impl JsonHandler
  pub fn execute(&self, input: &JsonInput, ctx: &CommandContext) -> CommandResult
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
  pub fn resolve_query(
  fn resolve_segments(
  pub fn format_query_result(value: &serde_json::Value) -> String
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
  pub mod ansi;
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
  pub mod stats_coverage;
  pub mod stats_efficiency;
  pub mod stats_render;
  pub mod tail;
  pub mod trim;
  pub mod txt2md;
  pub mod types;

**read.rs**
  pub use super::read_filters
  pub struct ReadInput
  pub enum FilterLevel
  pub struct ReadHandler

  impl ReadHandler
  pub fn execute(&self, input: &ReadInput, ctx: &CommandContext) -> CommandResult
  mod tests

**read_filters.rs**
  pub enum Language
  pub fn detect_language(path: &PathBuf) -> Language
  fn is_comment_line(line: &str, lang: Language) -> bool
  pub fn filter_minimal(content: &str, lang: Language) -> String
  pub fn filter_aggressive(content: &str, lang: Language) -> String
  fn is_import_line(trimmed: &str, lang: Language) -> bool
  fn is_decorator(trimmed: &str, lang: Language) -> bool
  fn is_definition_line(trimmed: &str, lang: Language) -> bool
  fn is_type_or_const(trimmed: &str, lang: Language) -> bool
  pub fn count_braces(line: &str) -> i32
  pub fn apply_line_range<'a>(

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
  pub struct Replacement
  pub struct ReplaceHandler

  impl ReplaceHandler
  pub fn execute_replace(
  pub fn format_output(
  pub fn format_json(
  pub fn format_csv(
  pub fn format_tsv(
  pub fn format_compact(
  pub fn format_raw(
  pub fn truncate_line(line: &str, max_len: usize) -> String
  pub fn escape_tsv_field(field: &str) -> String
  pub fn format_count(count: usize, format: OutputFormat) -> String

  impl CommandHandler for ReplaceHandler
  type Input = ReplaceInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct ReplaceInput

**run.rs**
  pub struct RunHandler

  impl RunHandler
  pub fn format_output(output: &ProcessOutput, format: OutputFormat) -> String
  pub fn escape_tsv_field(field: &str) -> String
  pub fn format_error(error: &ProcessError, format: OutputFormat) -> String

  impl CommandHandler for RunHandler
  type Input = RunInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct RunInput

  impl From<(&String, &Vec<String>, bool, bool, bool, bool, Option<u64>)> for RunInput
  fn from(

**search.rs**
  pub struct SearchHandler

  impl SearchHandler
  pub fn execute_search(&self, input: &SearchInput) -> CommandResult<GrepOutput>

  struct MatchResult
  struct MatchSink
  impl MatchSink
  fn new(matcher: RegexMatcher) -> Self

  impl Sink for MatchSink
  type Error = std::io::Error
  fn matched(
  fn context(
  pub fn format_output(grep_output: &GrepOutput, format: OutputFormat) -> String
  pub fn format_json(grep_output: &GrepOutput) -> String
  pub fn format_csv(grep_output: &GrepOutput) -> String
  pub fn format_tsv(grep_output: &GrepOutput) -> String
  pub fn format_compact(grep_output: &GrepOutput) -> String
  pub fn format_raw(grep_output: &GrepOutput) -> String

  impl CommandHandler for SearchHandler
  type Input = SearchInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct SearchInput

**stats.rs**
  pub fn local_offset() -> time::UtcOffset
  pub fn today_date_label(offset: time::UtcOffset) -> String
  pub fn format_date(ts: u64) -> String
  pub fn format_timestamp(ts: u64, offset: time::UtcOffset) -> String
  pub struct StatsInput
  pub fn handle_stats(input: &StatsInput)
  pub fn normalize_cmd(cmd: &str) -> String

**stats_coverage.rs**
  struct Agg
  impl Agg
  fn avg_in(&self) -> f64
  fn low_pct(&self) -> f64
  pub fn print_coverage(entries: &[HistoryEntry], limit: usize, json: bool, days: u64)
  fn parse_cmd(cmd: &str) -> Option<(String, String)>
  fn looks_like_env_token(tok: &str) -> bool
  type AggMap = std::collections::HashMap<String, Agg>
  type SubMap = std::collections::HashMap<(String, String), Agg>
  fn aggregate(entries: &[HistoryEntry]) -> (AggMap, SubMap)
  fn sample_of(cmd: &str) -> String
  fn classify<'a>(
  fn is_known_binary(name: &str) -> bool
  fn print_human(entries: &[HistoryEntry], binaries: &AggMap, sub_cmds: &SubMap, limit: usize)
  fn print_sub_table(rows: &[((String, String), &Agg)])
  fn print_bin_table(rows: &[(String, &Agg)])
  fn truncate(s: &str, max: usize) -> String
  fn fmt_date(ts: u64) -> String
  fn days_to_ymd(mut days: u64) -> (i32, u32, u32)
  fn is_leap(y: i32) -> bool
  fn print_json(entries: &[HistoryEntry], binaries: &AggMap, sub_cmds: &SubMap, limit: usize)
  mod tests
  fn parse_cmd_basic()
  fn parse_cmd_strips_env_prefix()
  fn parse_cmd_resolves_absolute_path()
  fn parse_cmd_no_subcommand()
  fn looks_like_env_token_basic()
  fn days_to_ymd_known_dates()

**stats_efficiency.rs**

  pub(crate) fn print_recent(entries: &[HistoryEntry]) -> Option<f64> {
      let now_ts = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map(|d| d.as_secs())
          .unwrap_or(0);
      for days in [7u64, 30] {
          if let Some((saved, pct)) = window_totals(entries, now_ts, days) {
              println!(
                  "Last {:<2} days:      {} saved \u{00b7} {:.0}%",
                  days,
                  crate::tracker::format_bytes_human(saved / 4),
                  pct
              );
          }
      }
      efficiency_since(entries, now_ts, 30)
  }

  fn window_totals(entries: &[HistoryEntry], now_ts: u64, days: u64) -> Option<(usize, f64)> {
      let cutoff = now_ts.saturating_sub(days * 86_400);
      let (i, o) = entries
          .iter()
          .filter(|e| e.ts >= cutoff)
          .fold((0usize, 0usize), |(i, o), e| {
              (i + e.in_bytes, o + e.out_bytes)
          });
      (i > 0).then(|| (i.saturating_sub(o), 100.0 * (1.0 - o as f64 / i as f64)))
  }

  pub(crate) fn efficiency_since(entries: &[HistoryEntry], now_ts: u64, days: u64) -> Option<f64> {
      let cutoff = now_ts.saturating_sub(days * 86_400);
      let (i, o) = entries
          .iter()
          .filter(|e| e.ts >= cutoff)
          .fold((0usize, 0usize), |(i, o), e| {
              (i + e.in_bytes, o + e.out_bytes)
          });
      (i > 0).then(|| 100.0 * (1.0 - o as f64 / i as f64))
  }

  pub(crate) fn print_bar(avg_pct: f64, window_days: Option<u64>) {
      let filled = (avg_pct / 5.0).round() as usize;
      let filled = filled.min(20);
      let empty = 20 - filled;
      println!(
          "Efficiency: {}{} {:.0}% ({})",
          "\u{2588}".repeat(filled),
          "\u{2591}".repeat(empty),
          avg_pct,
          match window_days {
              Some(d) => format!("last {}d", d),
              None => "lifetime".to_string(),
          }
      );
  }

**stats_render.rs**
  pub struct CommandAgg

  impl CommandAgg
  pub fn saved(&self) -> usize
  fn avg_reduction_pct(&self) -> f64
  pub fn print_by_command(entries: &[HistoryEntry], limit: usize)
  pub fn print_by_agent(entries: &[HistoryEntry])
  fn format_bypass_cell(bypass_count: usize, total_count: usize) -> String
  pub fn print_summary(entries: &[HistoryEntry], top_limit: usize, window_days: Option<u64>)
  fn today_entries(entries: &[HistoryEntry], offset: time::UtcOffset) -> Vec<&HistoryEntry>
  pub fn print_history(entries: &[HistoryEntry], limit: usize)
  fn display_cmd(cmd: &str) -> String
  pub fn print_json(
  fn truncate_cmd(cmd: &str, max_len: usize) -> String
  mod tests

**stats_render_tests.rs**

  #[test]
  fn format_bypass_cell_zero_is_plain() {
      assert_eq!(format_bypass_cell(0, 100), "0");
      assert_eq!(format_bypass_cell(0, 0), "0");
  }

  #[test]
  fn format_bypass_cell_includes_rate_when_nonzero() {
      assert_eq!(format_bypass_cell(3, 142), "3 (2.1%)");
      assert_eq!(format_bypass_cell(50, 100), "50 (50.0%)");
  }

  #[test]
  fn format_bypass_cell_zero_total_omits_rate() {
      assert_eq!(format_bypass_cell(2, 0), "2");
  }

  #[test]
  fn recent_efficiency_ignores_an_old_outlier() {
      const DAY: u64 = 86_400;
      let now = 100 * DAY;
      let e = |days_ago: u64, inb: usize, outb: usize| HistoryEntry {
          ts: now - days_ago * DAY,
          cmd: "x".into(),
          in_bytes: inb,
          out_bytes: outb,
          saved_pct: 0,
          ms: 0,
          cwd: String::new(),
          agent: None,
          bypass: None,
      };
      let entries = vec![
          e(21, 380_000_000, 370_000_000), // the aws-shaped week: ~3% saved
          e(3, 1_000_000, 150_000),        // recent work: 85% saved
          e(1, 1_000_000, 150_000),
      ];

      use super::super::stats_efficiency::efficiency_since;
      let d7 = efficiency_since(&entries, now, 7).unwrap();
      let d30 = efficiency_since(&entries, now, 30).unwrap();
      assert!(d7 > 80.0, "last 7d should reflect recent work, got {d7}");
      assert!(d30 < 10.0, "30d still contains the outlier, got {d30}");

      assert!(efficiency_since(&entries, now, 0).is_none());
  }

**tail.rs**
  pub struct TailHandler
  pub struct TailLine

  impl TailLine
  fn display(&self) -> &str
  pub struct TailOutput

  impl TailHandler
  pub fn read_tail_lines(&self, input: &TailInput) -> CommandResult<TailOutput>
  fn json_compact(line: &str) -> Option<(String, bool)>
  pub fn is_error_line(line: &str) -> bool
  pub fn stream_tail_lines(
  pub fn format_streaming_line(line: &TailLine, format: OutputFormat) -> String
  pub fn format_output(output: &TailOutput, format: OutputFormat) -> String
  pub fn format_json(output: &TailOutput) -> String
  pub fn format_csv(output: &TailOutput) -> String
  pub fn format_tsv(output: &TailOutput) -> String
  fn format_body_lines(output: &TailOutput) -> String
  pub fn format_agent(output: &TailOutput) -> String
  pub fn format_compact(output: &TailOutput) -> String
  pub fn format_raw(output: &TailOutput) -> String
  pub fn escape_tsv_field(field: &str) -> String

  impl CommandHandler for TailHandler
  type Input = TailInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult
  pub struct TailInput

**trim.rs**
  pub struct TrimInput
  pub struct TrimHandler

  impl TrimHandler
  pub fn read_input(&self, file: &Option<std::path::PathBuf>) -> CommandResult<String>
  pub fn trim_text(&self, text: &str, leading: bool, trailing: bool) -> String
  pub fn format_output(

  impl CommandHandler for TrimHandler
  type Input = TrimInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

## src/router/handlers/parse/

**aws.rs**
  fn prefix_of(target: &str) -> String
  fn progress_line(line: &str) -> Option<(&'static str, String)>

  impl ParseHandler
  pub fn handle_aws(
  mod tests

**aws_tests.rs**

  #[test]
  fn parses_s3_progress_lines() {
      assert_eq!(
          progress_line("delete: s3://bucket/logs/a.log"),
          Some(("delete", "s3://bucket/logs/a.log".to_string()))
      );
      assert_eq!(
          progress_line("copy: s3://src/a.txt to s3://dst/a.txt"),
          Some(("copy", "s3://src/a.txt".to_string()))
      );
      assert_eq!(
          progress_line("Completed 3 file(s) with ~2 file(s) remaining"),
          None
      );
      assert_eq!(progress_line("An error occurred (AccessDenied)"), None);
  }

  #[test]
  fn groups_by_bucket_and_directory() {
      assert_eq!(prefix_of("s3://b/logs/2026/a.log"), "s3://b/logs/2026/");
      assert_eq!(prefix_of("s3://b/top.txt"), "s3://b/");
      assert_eq!(prefix_of("s3://bucket-only"), "s3://bucket-only");
      assert_eq!(prefix_of("./local/file.txt"), "./local/file.txt");
  }

  #[test]
  fn verbs_cover_the_recursive_operations() {
      for verb in ["delete", "upload", "download", "copy", "move"] {
          let line = format!("{}: s3://b/k", verb);
          assert!(progress_line(&line).is_some(), "unhandled verb: {}", verb);
      }
  }

**brew.rs**
  impl ParseHandler
  pub fn handle_brew(
  fn is_progress_bar(line: &str) -> bool
  fn condense_keg_line(line: &str) -> String
  fn format_brew_compact(installed: &[String], errors: &[String], warnings: &[String]) -> String

**bun_format.rs**
  impl ParseHandler
  pub fn format_bun_test(output: &BunTestOutput, format: OutputFormat) -> String
  pub fn format_bun_test_json(output: &BunTestOutput) -> String
  pub fn format_bun_test_compact(output: &BunTestOutput) -> String
  pub fn format_bun_test_raw(output: &BunTestOutput) -> String
  pub fn format_bun_test_agent(output: &BunTestOutput) -> String

**bun_parse.rs**
  impl ParseHandler
  pub fn parse_bun_test(input: &str) -> CommandResult<BunTestOutput>
  pub fn parse_bun_test_line(line: &str, ancestors: &[String]) -> Option<BunTest>
  pub fn parse_bun_duration(s: &str) -> Option<f64>
  pub fn split_bun_test_name_and_duration(s: &str) -> (String, Option<f64>)
  pub fn is_bun_summary_line(line: &str) -> bool
  pub fn parse_bun_summary_line(line: &str, summary: &mut BunTestSummary)
  pub fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary)
  pub fn update_bun_summary_from_tests(output: &mut BunTestOutput)

**extra_cargo_test.rs**: impl ParseHandler | pub fn handle_cargo_test(
**extra_db.rs**
  pub enum DbFormat
  pub struct DbResult

  impl ParseHandler
  pub fn detect_db_format(input: &str) -> Option<DbFormat>
  fn parse_psql(input: &str) -> DbResult
  fn parse_mysql(input: &str) -> DbResult
  fn parse_sqlite(input: &str) -> DbResult
  pub fn parse_db(input: &str) -> DbResult
  pub fn sample_rows(rows: &[Vec<String>]) -> (Vec<&Vec<String>>, usize)
  fn format_row_compact(cells: &[String]) -> String
  fn format_db_compact(result: &DbResult) -> String
  pub fn format_db_json(result: &DbResult) -> String
  fn csv_escape(field: &str) -> String
  pub fn format_db_csv(result: &DbResult) -> String
  pub fn format_db_tsv(result: &DbResult) -> String
  pub fn handle_db(

**extra_download.rs**
  impl ParseHandler
  pub fn handle_download(
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
  pub fn handle_env(

**extra_network.rs**
  impl ParseHandler
  pub fn handle_ping(
  fn format_ping_compact(

**extra_system.rs**
  impl ParseHandler
  pub fn handle_tree(
  pub fn handle_docker_ps(
  pub fn handle_docker_logs(
  pub fn handle_deps(
  pub fn handle_install(
  pub fn handle_build(

  enum Prev
  pub fn handle_wc(

**find.rs**
  impl ParseHandler
  pub fn handle_find(
  pub fn parse_find(input: &str) -> CommandResult<FindOutput>
  pub fn parse_find_error(line: &str) -> FindError
  pub fn extract_extension(path: &str) -> Option<String>
  pub fn calculate_path_depth(path: &str) -> usize
  pub fn format_find(find_output: &FindOutput, format: OutputFormat) -> String
  pub fn format_find_json(find_output: &FindOutput) -> String
  pub fn format_find_compact(find_output: &FindOutput) -> String
  pub fn format_find_raw(find_output: &FindOutput) -> String
  fn common_path_prefix(paths: &[&str]) -> String

**fmt.rs**
  impl ParseHandler
  pub fn handle_fmt(

  struct FmtFile
  fn parse_fmt(input: &str) -> Vec<FmtFile>
  fn format_fmt_compact(files: &[FmtFile], input: &str) -> String
  fn format_fmt_json(files: &[FmtFile]) -> String
  mod tests
  fn test_fmt_basic()
  fn main()
  fn test_fmt_multiple_diffs_same_file()
  fn test_fmt_strips_cwd_prefix()
  fn test_fmt_at_line_variant()
  fn test_fmt_empty_is_clean()
  fn test_fmt_caps_file_list()
  fn test_fmt_unrecognized_passthrough()
  fn test_fmt_json()

**gh_api.rs**
  fn drops(key: &str, parent: &str) -> bool
  fn prune(value: &mut Value, parent: &str)
  pub(super) fn compress_gh_api(input: &str) -> Option<String>

  impl ParseHandler
  pub fn handle_gh_api(
  mod tests

**gh_api_tests.rs**
  fn parsed(s: &str) -> Value
  fn drops_link_boilerplate_but_keeps_html_url()
  fn prunes_nested_objects_and_arrays()
  fn drops_the_pgp_blob_but_keeps_the_verdict()
  fn declines_non_json_bodies()
  fn never_returns_more_than_it_received()
  fn output_is_a_single_line_of_valid_json()

**gh_pr.rs**
  impl ParseHandler
  pub fn handle_gh_pr(
  pub fn handle_gh_issue(
  pub fn handle_gh_pr_view(
  pub fn handle_gh_pr_checks(

  struct Check

**gh_run.rs**
  impl ParseHandler
  pub fn handle_gh_run(
  pub fn handle_gh_run_view(

**git_branch.rs**
  impl ParseHandler
  pub fn handle_git_branch(
  fn render_compact(current: &str, local: &[String], remote: &[String]) -> String
  fn render_group(out: &mut String, branches: &[&String], indent: &str)

**git_commit.rs**
  impl ParseHandler
  pub fn handle_git_commit(

  struct GitCommitResult
  fn parse_git_commit(input: &str) -> GitCommitResult
  fn format_git_commit_compact(r: &GitCommitResult, input: &str) -> String
  fn format_git_commit_json(r: &GitCommitResult) -> String
  mod tests
  fn test_git_commit_basic()
  fn test_git_commit_root_commit()
  fn test_git_commit_detached_head_and_slash_branch()
  fn test_git_commit_no_mode_lines()
  fn test_git_commit_unrecognized_passthrough()
  fn test_git_commit_empty()
  fn test_git_commit_json()

**git_diff.rs**
  impl ParseHandler
  pub fn handle_git_diff(
  pub fn parse_git_diff(input: &str) -> CommandResult<GitDiff>
  fn parse_git_diff_stat(input: &str) -> CommandResult<GitDiff>
  pub fn truncate_diff(diff: &mut GitDiff, max_files: usize)

**git_diff_format.rs**
  impl ParseHandler
  pub fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String
  pub fn format_git_diff_json(diff: &GitDiff) -> String
  fn format_hunk_compressed(hunk: &GitDiffHunk) -> Vec<String>
  fn build_file_summary(diff: &GitDiff) -> String
  pub fn format_git_diff_compact(diff: &GitDiff) -> String
  pub fn format_git_diff_raw(diff: &GitDiff) -> String

**git_log.rs**
  fn apply_truncate(subject: &str, max: Option<usize>) -> String

  impl ParseHandler
  pub fn handle_git_log(
  fn extract_subject(msg: &[String]) -> String
  fn relative_time(date_str: &str) -> String

**git_pull.rs**: impl ParseHandler | pub fn handle_git_pull(
**git_status.rs**
  impl ParseHandler
  pub fn handle_git_status(
  pub fn format_git_status_count(count: usize, format: OutputFormat) -> String
  pub fn parse_git_status(input: &str) -> CommandResult<GitStatus>
  pub fn parse_file_entry(

**git_status_format.rs**
  impl ParseHandler
  pub fn format_git_status(status: &GitStatus, format: OutputFormat) -> String
  pub fn format_git_status_csv(status: &GitStatus) -> String
  pub fn format_git_status_tsv(status: &GitStatus) -> String
  pub fn format_git_status_json(status: &GitStatus) -> String
  pub fn format_git_status_compact(status: &GitStatus) -> String
  fn format_entries_capped(entries: &[GitStatusEntry], max: usize, output: &mut String)
  fn format_entries_listed(entries: &[GitStatusEntry], max: usize, output: &mut String)
  fn format_entries_grouped(entries: &[GitStatusEntry], output: &mut String)
  pub fn format_git_status_raw(status: &GitStatus) -> String
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
  pub fn handle_go_test(

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

**grep.rs**
  impl ParseHandler
  pub fn handle_grep(
  pub fn parse_grep(input: &str) -> CommandResult<GrepOutput>
  pub fn truncate_grep(
  pub fn parse_grep_line(line: &str) -> Option<(String, GrepMatch)>
  fn find_sep_digits(line: &str, sep: u8) -> Option<(String, Option<usize>, String)>
  fn extract_column(s: String) -> (Option<usize>, String)
  fn extract_column_sep(s: String, sep: u8) -> (Option<usize>, String)

**grep_format.rs**
  impl ParseHandler
  pub fn format_grep(grep_output: &GrepOutput, format: OutputFormat) -> String
  pub fn format_grep_json(grep_output: &GrepOutput) -> String
  pub fn format_grep_csv(grep_output: &GrepOutput) -> String
  pub fn format_grep_tsv(grep_output: &GrepOutput) -> String
  pub fn format_grep_compact(grep_output: &GrepOutput) -> String
  pub fn format_grep_raw(grep_output: &GrepOutput) -> String

**jest_format.rs**
  impl ParseHandler
  pub fn format_jest(output: &JestOutput, format: OutputFormat) -> String
  pub fn format_jest_json(output: &JestOutput) -> String
  pub fn format_jest_compact(output: &JestOutput) -> String
  pub fn format_jest_raw(output: &JestOutput) -> String
  pub fn format_jest_agent(output: &JestOutput) -> String

**jest_parse.rs**
  impl ParseHandler
  pub fn parse_jest(input: &str) -> CommandResult<JestOutput>
  pub fn parse_jest_test_line(line: &str) -> Option<JestTest>
  pub fn parse_jest_duration(s: &str) -> Option<f64>
  pub fn parse_jest_summary(line: &str) -> JestSummary
  fn extract_count(text: &str, label: &str) -> usize
  pub fn parse_jest_tests_summary(line: &str, summary: &mut JestSummary)
  pub fn parse_jest_snapshots_summary(line: &str, summary: &mut JestSummary)
  pub fn parse_jest_time_summary(line: &str, summary: &mut JestSummary)

**lint.rs**
  struct LintIssue
  enum LintLevel
  impl ParseHandler
  pub fn handle_lint(
  fn parse_lint_issues(input: &str) -> Vec<LintIssue>
  fn extract_clippy_rule(lines: &[&str], start: usize) -> String
  fn parse_tsc_format(line: &str) -> Option<LintIssue>
  fn parse_colon_format(line: &str) -> Option<LintIssue>
  fn find_eslint_file_context(lines: &[&str], from: usize) -> String
  fn format_lint_compact(issues: &[LintIssue], errors: usize, warnings: usize) -> String
  fn format_lint_json(issues: &[LintIssue], errors: usize, warnings: usize) -> String
  mod tests

**lint_tests.rs**
  fn test_parse_clippy_format()
  fn test_parse_ruff_colon_format()
  fn test_format_compact_clean()
  fn test_format_compact_grouped()
  fn test_parse_tsc_format()
  fn test_tsc_compact_output()
  fn test_format_json()

**logs.rs**
  impl ParseHandler
  pub fn handle_logs(
  pub fn parse_logs(input: &str) -> LogsOutput
  pub fn parse_log_line(line: &str, line_number: usize) -> LogEntry

**logs_format.rs**
  impl ParseHandler
  pub fn format_logs(logs_output: &LogsOutput, format: OutputFormat) -> String
  pub fn format_logs_json(logs_output: &LogsOutput) -> String
  pub fn format_logs_csv(logs_output: &LogsOutput) -> String
  pub fn format_logs_tsv(logs_output: &LogsOutput) -> String
  pub fn format_logs_compact(logs_output: &LogsOutput) -> String
  fn level_indicator(level: LogLevel) -> &'static str
  fn level_name(level: LogLevel) -> &'static str
  fn preview_msg(msg: &str, max: usize) -> String
  fn is_stack_trace_line(entry: &LogEntry) -> bool
  pub fn format_logs_raw(logs_output: &LogsOutput) -> String

**logs_helpers.rs**
  impl ParseHandler
  pub fn extract_timestamp(line: &str) -> Option<String>
  pub fn is_iso8601_timestamp(s: &str) -> bool
  pub fn is_iso8601_space_timestamp(s: &str) -> bool
  pub fn is_slash_date_timestamp(s: &str) -> bool
  pub fn is_syslog_timestamp(s: &str) -> bool
  pub fn is_time_only(s: &str) -> bool
  pub fn detect_log_level(line: &str) -> LogLevel
  pub fn contains_error_keyword(line_upper: &str, keyword: &str) -> bool
  pub fn contains_warning_keyword(line_upper: &str, keyword: &str) -> bool
  pub fn contains_level_marker(line_upper: &str, level: &str) -> bool
  pub fn extract_message(

**logs_json.rs**
  impl ParseHandler
  pub fn try_parse_json_log_line(line: &str, line_number: usize) -> Option<LogEntry>
  fn json_first_str(
  fn json_log_level(obj: &serde_json::Map<String, serde_json::Value>) -> LogLevel
  fn level_from_keyword(up: &str) -> LogLevel
  fn level_from_number(n: u64) -> LogLevel
  mod tests

**logs_json_tests.rs**
  fn pino_numeric_level_and_msg()
  fn string_level_message_logger_appended()
  fn error_field_is_appended()
  fn bunyan_fatal_60()
  fn non_json_returns_none()
  fn arbitrary_json_data_passes_through()
  fn message_only_no_level_is_accepted()
  fn compact_output_drops_json_noise()
  fn verbose_json_logs_compress_hard()
  fn level_drives_parse_logs_counts()

**ls.rs**
  impl ParseHandler
  pub fn handle_ls(
  pub fn parse_ls(input: &str) -> CommandResult<LsOutput>
  pub fn parse_ls_error(line: &str) -> LsError
  pub fn is_long_format_line(line: &str) -> bool
  pub fn parse_long_format_line(line: &str) -> LsEntry
  pub fn detect_entry_type_from_perms(perms: &str) -> LsEntryType
  pub fn detect_entry_type_from_name(name: &str) -> LsEntryType
  pub fn has_file_extension(name: &str) -> bool
  pub fn format_ls(ls_output: &LsOutput, format: OutputFormat) -> String
  pub fn format_ls_json(ls_output: &LsOutput) -> String
  pub fn format_ls_compact(ls_output: &LsOutput) -> String
  pub fn format_ls_raw(ls_output: &LsOutput) -> String

**mod.rs**
  pub mod aws
  pub mod brew
  pub mod bun_format
  pub mod bun_parse
  pub mod extra_cargo_test
  pub mod extra_db
  pub mod extra_download
  pub mod extra_env
  pub mod extra_network
  pub mod extra_system
  pub mod find
  pub mod fmt
  pub mod gh_api
  pub mod gh_pr
  pub mod gh_run
  pub mod git_branch
  pub mod git_commit
  pub mod git_diff
  pub mod git_diff_format
  pub mod git_log
  pub mod git_pull
  pub mod git_status
  pub mod git_status_format
  pub mod go_test
  pub mod grep
  pub mod grep_format
  pub mod jest_format
  pub mod jest_parse
  pub mod lint
  pub mod logs
  pub mod logs_format
  pub mod logs_helpers
  pub mod logs_json
  pub mod ls
  pub mod npm_format
  pub mod npm_parse
  pub mod pnpm_format
  pub mod pnpm_parse
  pub mod ps
  pub mod pytest_format
  pub mod pytest_parse
  pub mod python_traceback
  pub mod sysinfo
  pub mod test
  pub mod vitest_format
  pub mod vitest_parse
  pub struct ParseHandler

  impl ParseHandler
  pub fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String>
  pub fn read_input_raw(file: &Option<std::path::PathBuf>) -> CommandResult<String>
  pub fn json_to_string(value: serde_json::Value) -> String
  pub fn truncate_str(s: &str, max_len: usize) -> String
  pub fn format_human_size(bytes: u64) -> String

  impl CommandHandler for ParseHandler
  type Input = ParseCommands
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**npm_format.rs**
  impl ParseHandler
  pub fn format_npm_test(output: &NpmTestOutput, format: OutputFormat) -> String
  pub fn format_npm_test_json(output: &NpmTestOutput) -> String
  pub fn format_npm_test_compact(output: &NpmTestOutput) -> String
  pub fn format_npm_test_raw(output: &NpmTestOutput) -> String
  pub fn format_npm_test_agent(output: &NpmTestOutput) -> String

**npm_parse.rs**
  impl ParseHandler
  pub fn parse_npm_test(input: &str) -> CommandResult<NpmTestOutput>
  pub fn parse_npm_test_line(line: &str, ancestors: &[String]) -> Option<NpmTest>
  pub fn split_npm_test_name_and_duration(line: &str) -> (String, Option<f64>)
  pub fn parse_npm_duration(s: &str) -> Option<f64>
  pub fn extract_npm_suite_duration(line: &str) -> Option<f64>
  pub fn parse_npm_test_summary_tests(line: &str, summary: &mut NpmTestSummary)
  pub fn parse_npm_test_summary_files(line: &str, summary: &mut NpmTestSummary)
  pub fn parse_npm_test_summary_tests_info(line: &str, summary: &mut NpmTestSummary)
  pub fn parse_npm_test_summary_files_info(line: &str, summary: &mut NpmTestSummary)
  pub fn parse_npm_counts(
  pub fn parse_npm_counts_with_todo(
  pub fn update_npm_summary_from_tests(output: &mut NpmTestOutput)

**pnpm_format.rs**
  impl ParseHandler
  pub fn format_pnpm_test(output: &PnpmTestOutput, format: OutputFormat) -> String
  pub fn format_pnpm_test_json(output: &PnpmTestOutput) -> String
  pub fn format_pnpm_test_compact(output: &PnpmTestOutput) -> String
  pub fn format_pnpm_test_raw(output: &PnpmTestOutput) -> String
  pub fn format_pnpm_test_agent(output: &PnpmTestOutput) -> String

**pnpm_parse.rs**
  impl ParseHandler
  pub fn parse_pnpm_test(input: &str) -> CommandResult<PnpmTestOutput>
  pub fn parse_pnpm_test_line(line: &str, ancestors: &[String]) -> Option<PnpmTest>
  pub fn parse_pnpm_duration(s: &str) -> Option<f64>
  pub fn split_pnpm_test_name_and_duration(s: &str) -> (String, Option<f64>)
  pub fn extract_pnpm_suite_duration(line: &str) -> Option<f64>
  pub fn parse_pnpm_test_summary_tests(line: &str, summary: &mut PnpmTestSummary)
  pub fn parse_pnpm_test_summary_files(line: &str, summary: &mut PnpmTestSummary)
  pub fn parse_pnpm_test_summary_tests_info(line: &str, summary: &mut PnpmTestSummary)
  pub fn parse_pnpm_test_summary_files_info(line: &str, summary: &mut PnpmTestSummary)
  pub fn parse_pnpm_counts(
  pub fn parse_pnpm_counts_with_todo(
  pub fn update_pnpm_summary_from_tests(output: &mut PnpmTestOutput)

**ps.rs**
  struct Proc
  impl ParseHandler
  pub fn handle_ps(
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

**pytest_format.rs**
  impl ParseHandler
  pub fn format_pytest(output: &PytestOutput, format: OutputFormat) -> String
  pub fn format_pytest_json(output: &PytestOutput) -> String
  pub fn format_pytest_compact(output: &PytestOutput) -> String
  pub fn format_pytest_raw(output: &PytestOutput) -> String
  pub fn format_pytest_agent(output: &PytestOutput) -> String

**pytest_parse.rs**
  impl ParseHandler
  pub fn parse_pytest(input: &str) -> CommandResult<PytestOutput>
  fn parse_pytest_quiet_progress(line: &str) -> Vec<TestResult>
  pub fn parse_pytest_test_line(line: &str) -> Option<TestResult>
  pub fn is_pytest_summary_line(line: &str) -> bool
  pub fn parse_pytest_summary(line: &str) -> TestSummary
  fn extract_count(text: &str, label: &str) -> usize

**python_traceback.rs**
  struct Frame
  struct Traceback
  impl ParseHandler
  pub fn handle_python_traceback(
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

**sysinfo.rs**
  fn size_bytes(cell: &str) -> Option<f64>
  fn human(bytes: f64) -> String
  fn short_cmd(cmd: &str) -> String
  fn emit(
  fn compress_du(input: &str) -> Option<String>
  fn compress_lsof(input: &str) -> Option<String>
  fn compress_pgrep(input: &str) -> Option<String>

  impl ParseHandler
  pub fn handle_du(
  pub fn handle_lsof(
  pub fn handle_pgrep(
  mod tests

**sysinfo_tests.rs**
  fn du_sorts_by_size_descending_and_totals()
  fn du_summarizes_the_tail_beyond_the_cap()
  fn du_declines_shapes_it_would_only_make_worse()
  fn lsof_folds_the_descriptors_of_one_process_into_one_row()
  fn lsof_keeps_the_whole_name_cell_including_its_spaces()
  fn lsof_declines_without_the_standard_header()
  fn pgrep_collapses_identical_command_lines()
  fn pgrep_declines_bare_pid_output()
  fn short_cmd_drops_the_argv0_path_but_keeps_the_arguments()
  fn size_bytes_reads_both_human_and_block_forms()

**test.rs**: impl ParseHandler | pub fn handle_test(
**vitest_format.rs**
  impl ParseHandler
  pub fn format_vitest(output: &VitestOutput, format: OutputFormat) -> String
  pub fn format_vitest_json(output: &VitestOutput) -> String
  pub fn format_vitest_compact(output: &VitestOutput) -> String
  pub fn format_vitest_raw(output: &VitestOutput) -> String
  pub fn format_vitest_agent(output: &VitestOutput) -> String

**vitest_parse.rs**
  impl ParseHandler
  pub fn parse_vitest(input: &str) -> CommandResult<VitestOutput>
  pub fn parse_vitest_suite_header(line: &str) -> Option<VitestSuiteInfo>
  pub fn parse_vitest_test_line(line: &str) -> Option<VitestTest>
  pub fn parse_vitest_duration(s: &str) -> Option<f64>
  pub fn parse_vitest_test_files_summary(line: &str) -> VitestSummary
  fn extract_count(text: &str, label: &str) -> usize
  pub fn parse_vitest_tests_summary(line: &str, summary: &mut VitestSummary)

## src/router/handlers/txt2md/

**detect_headings.rs**
  impl Txt2mdHandler
  pub fn is_heading_line(line: &str) -> bool
  pub fn is_numbered_section_heading(line: &str) -> bool
  pub fn strip_numbered_prefix(line: &str) -> Option<&str>
  pub fn starts_with_roman_numeral(s: &str) -> bool
  pub fn is_title_case(line: &str) -> bool
  pub fn is_single_word_section_heading(line: &str, index: usize, lines: &[&str]) -> bool
  pub fn determine_heading_level(line: &str, index: usize, lines: &[&str]) -> usize
  pub fn to_title_case(s: &str) -> String
  pub fn format_heading_text(line: &str) -> String

**detect_lists.rs**
  impl Txt2mdHandler
  pub fn is_unordered_list_item_with_indent(line: &str) -> Option<(char, usize)>
  pub fn is_unordered_list_item(line: &str) -> Option<char>
  pub fn strip_list_prefix(line: &str, prefix: char) -> &str
  pub fn is_ordered_list_item_with_indent(line: &str) -> Option<(u32, usize)>
  pub fn is_ordered_list_item(line: &str) -> bool
  pub fn strip_ordered_prefix(line: &str) -> &str
  pub fn is_list_continuation(line: &str) -> bool
  pub fn is_horizontal_rule(line: &str) -> bool
  pub fn format_inline(text: &str) -> String
  pub fn format_urls(text: &str) -> String

**format.rs**
  impl Txt2mdHandler
  pub fn extract_metadata(
  pub fn normalize_spacing(&self, text: &str) -> String
  pub fn format_output(

**mod.rs**
  pub mod detect_headings
  pub mod detect_lists
  pub mod format
  pub mod parser
  pub struct Txt2mdInput
  pub struct Txt2mdHandler

  impl Txt2mdHandler
  pub fn read_input(&self, input: &Option<std::path::PathBuf>) -> CommandResult<String>

  impl CommandHandler for Txt2mdHandler
  type Input = Txt2mdInput
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**parser.rs**: impl Txt2mdHandler | pub fn convert_to_markdown(&self, text: &str) -> String
## src/router/handlers/types/

**fs.rs**
  pub enum LsEntryType

  impl Default for LsEntryType
  fn default() -> Self
  pub struct LsEntry
  pub struct FindEntry
  pub struct FindError
  pub struct FindOutput
  pub fn is_generated_directory(name: &str) -> bool
  pub struct LsError
  pub struct LsOutput

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

**test_types_core.rs**
  pub enum TestStatus
  pub struct TestResult
  pub struct TestSummary
  pub struct PytestOutput
  pub struct JestOutput
  pub struct JestTestSuite
  pub struct JestTest
  pub enum JestTestStatus
  pub struct JestSummary
  pub struct VitestOutput
  pub struct VitestTestSuite
  pub struct VitestTest
  pub enum VitestTestStatus
  pub struct VitestSummary
  pub struct VitestSuiteInfo

**test_types_runners.rs**
  pub struct NpmTestOutput
  pub struct NpmTestSuite
  pub struct NpmTest
  pub enum NpmTestStatus
  pub struct NpmTestSummary
  pub struct PnpmTestOutput
  pub struct PnpmTestSuite
  pub struct PnpmTest
  pub enum PnpmTestStatus
  pub struct PnpmTestSummary
  pub struct BunTestOutput
  pub struct BunTestSuite
  pub struct BunTest
  pub enum BunTestStatus
  pub struct BunTestSummary

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
*trs ingest v0.7.5 | 581ms | 164.1KB*
