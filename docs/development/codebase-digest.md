# labs-tarscli (339 files, 52.3k tokens, rust)

> Token-reducing shell for AI agents: compact terminal output, up to 60% fewer tokens on real agent commands

## Structure

//
  README.md  .gitignore  AGENTS.md  CHANGELOG.md  CLAUDE.md  
  CODE_OF_CONDUCT.md  CONTRIBUTING.md  Cargo.toml  LICENSE  
  README.es.md  SECURITY.md  cliff.toml  lefthook.yml

.cargo/
  config.toml

.devin/rules/
  trs.md

.githooks/
  commit-msg  pre-commit  pre-push

.github/
  PULL_REQUEST_TEMPLATE.md  dependabot.yml

.github/ISSUE_TEMPLATE/
  bug_report.yml  config.yml  feature_request.yml

.github/workflows/
  ci.yml  release.yml

docs/
  CNAME  _config.yml  install.ps1  install.sh  llms.txt  robots.txt

docs/commands/
  audit-docs.md  doctor.md  init.md  output-saver.md  stats.md  
  upgrade.md

docs/development/
  agent-integrations.md  antigravity-hooks-research.md  
  output-cost-findings.md

docs/development/benchmarks/
  README.md  bench_stats.py  benchmark-real.sh  benchmark.sh  
  chain-rewrite.sh  output_saver_ab.py  output_saver_agentic.py  
  output_saver_stale.py  recovery.py  transcript_output.py  truth.py

docs/development/benchmarks/compare/
  compare.py  corpus.txt  fixtures.py  setup.sh

docs/features/
  audit-docs.md  configuration.md  diff.md  doctor.md  formats.md  
  ingest.md  init.md  output-saver.md  stats.md  uninstall.md  
  upgrade.md

docs/roadmap/
  TASK_TODO.md

docs/roadmap/completed/
  2603.md  2604.md  2605.md  2608.md  README.md

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
  docker-smoke.sh  post-release-clean.sh  sync-codebase-digest.sh  
  sync-version.sh

src/
  ai_tool.rs  ai_tool_tests.rs  anchor_guard.rs  audit_docs.rs  
  audit_docs_detect.rs  audit_docs_report.rs  audit_docs_symbols.rs  
  audit_docs_volatile.rs  benchmark.rs  classifier.rs  
  classifier_args.rs  classifier_args_tests.rs  classifier_exec.rs  
  classifier_tests.rs  classifier_transfer.rs  cli.rs  codex.rs  
  command_registry.rs  command_registry_tests.rs  commands.rs  
  commands_parse.rs  commands_parse_with_file.rs  config.rs  
  debug_info.rs  diff.rs  discover.rs  doctor.rs  doctor_checks.rs  
  doctor_hooks.rs  doctor_tests.rs  exec.rs  exec_verdict.rs  
  fast_find.rs  fs_atomic.rs  help.rs  help_tests.rs  help_text.rs  
  help_text_more.rs  init.rs  init_collision.rs  
  init_collision_tests.rs  init_install.rs  init_install_plugins.rs  
  init_install_rules.rs  init_install_rules_tests.rs  
  init_install_tests.rs  init_show.rs  init_templates.rs  
  init_templates_plugins.rs  init_vibe.rs  main.rs  main_args.rs  
  main_ingest.rs  main_tests.rs  main_tests_precedence.rs  
  output_saver.rs  output_saver_core.rs  output_saver_core_tests.rs  
  output_saver_write.rs  parse_out.rs  passthrough_cut.rs  
  path_display.rs  pointer_recall.rs  process.rs  process_helpers.rs  
  process_tests.rs  report.rs  report_tests.rs  rerun_rate.rs  
  rewrite.rs  rewrite_decide.rs  rewrite_decide_capture_tests.rs  
  rewrite_decide_tests.rs  rewrite_gate.rs  rewrite_perm_codex.rs  
  rewrite_perm_cursor.rs  rewrite_perm_devin.rs  rewrite_perm_droid.rs  
  rewrite_perm_droid_match.rs  rewrite_perm_droid_rules.rs  
  rewrite_perm_gemini.rs  rewrite_perm_gemini_policy.rs  
  rewrite_permissions.rs  rewrite_pipe_status.rs  rewrite_rules.rs  
  rewrite_tests.rs  safe_folds.rs  safe_folds_tests.rs  sentinel.rs  
  text_util.rs  tracker.rs  tracker_tests.rs  uninstall.rs  
  uninstall_scrub.rs  upgrade.rs

src/formatter/: Formatter system for trs (Token-Reducing Shell)
  agent.rs  compact.rs  compact_schema_git.rs  
  compact_schema_output.rs  csv.rs  helpers.rs  json.rs  
  json_schema.rs  mod.rs  raw.rs  tsv.rs

src/ingest/: Project digest generator for LLM consumption
  collect.rs  collect_compress.rs  collect_index.rs  
  collect_manifests.rs  deps.rs  deps_extract.rs  dupes.rs  format.rs  
  format_html.rs  format_html_util.rs  format_tree.rs  format_util.rs  
  meta.rs  mod.rs  mod_html.rs  ollama.rs  purpose.rs  remote.rs  
  resolve.rs  signatures.rs  store.rs  tests.rs

src/reducer/: Reducer system for trs (Token-Reducing Shell)
  mod.rs  output.rs  registry.rs  truncation.rs

src/router/: Command routing system for trs (Token-Reducing Shell)
  mod.rs

src/router/handlers/
  ansi.rs  clean.rs  common.rs  common_markers.rs  err.rs  html2md.rs  
  isclean.rs  json.rs  json_query.rs  json_tests.rs  mod.rs  read.rs  
  read_filters.rs  read_tests.rs  replace.rs  run.rs  search.rs  
  stats.rs  stats_coverage.rs  stats_efficiency.rs  stats_render.rs  
  stats_render_tests.rs  tail.rs  trim.rs

src/router/handlers/parse/
  aws.rs  aws_tests.rs  brew.rs  bun_format.rs  bun_parse.rs  
  bun_summary.rs  extra_build.rs  extra_cargo_test.rs  extra_db.rs  
  extra_download.rs  extra_env.rs  extra_network.rs  extra_system.rs  
  find.rs  fmt.rs  gh_api.rs  gh_api_tests.rs  gh_pr.rs  gh_run.rs  
  git_branch.rs  git_commit.rs  git_diff.rs  git_diff_format.rs  
  git_log.rs  git_pull.rs  git_status.rs  git_status_format.rs  
  go_test.rs  gradle.rs  gradle_tests.rs  grep.rs  grep_format.rs  
  jest_format.rs  jest_parse.rs  lint.rs  lint_tests.rs  logs.rs  
  logs_format.rs  logs_helpers.rs  logs_json.rs  logs_json_tests.rs  
  ls.rs  ls_format.rs  maven.rs  maven_tests.rs  mod.rs  node_test.rs  
  npm_format.rs  npm_parse.rs  ollama.rs  ollama_tests.rs  
  pnpm_format.rs  pnpm_parse.rs  pnpm_summary.rs  ps.rs  
  pytest_format.rs  pytest_parse.rs  python_traceback.rs  sysinfo.rs  
  sysinfo_tests.rs  test.rs  vitest_format.rs  vitest_parse.rs

src/router/handlers/txt2md/: Handler for the `txt2md` command - converts plain text to Markdown
  detect_headings.rs  detect_lists.rs  format.rs  mod.rs  parser.rs

src/router/handlers/types/: Shared data structures and types for command handlers
  fs.rs  git.rs  grep_types.rs  logs.rs  mod.rs  test_types_core.rs  
  test_types_runners.rs

src/schema/: Stable JSON schemas for trs (Token-Reducing Shell) reducers
  fs.rs  git.rs  logs.rs  mod.rs  process.rs  search.rs  test.rs  
  tests.rs


## Key Dependencies

  parse_out.rs ← src/classifier_exec.rs, src/main.rs, parse/aws.rs, brew.rs (+33)
  tests.rs ← ai_tool.rs, classifier.rs, classifier_args.rs, command_registry.rs (+29)
  init.rs ← src/doctor_checks.rs, src/doctor_hooks.rs, doctor_tests.rs, src/init_collision.rs (+15)
  src/tracker.rs ← src/classifier_exec.rs, config.rs, debug_info.rs, src/doctor_checks.rs (+13)
  formatter/mod.rs ← exec_verdict.rs, agent.rs, compact.rs, csv.rs (+12)

## Architecture (module roles)

From the import graph (fan-in↓ / fan-out↑), no AST:

- **entry**: roots, nothing imports them (main, CLI) → `main` (0↓/74↑)
- **core**: high fan-in, everything routes through → `ingest` (25↓/5↑), `init` (19↓/10↑), `tracker` (13↓/2↑)
- **leaf**: used by many, import nothing (utils, types) → `text_util` (11↓/0↑)
- **internal**: mid-graph plumbing → `router/handlers` (5↓/18↑), `classifier_exec` (2↓/16↑), `rewrite_gate` (10↓/6↑), `rewrite_permissions` (10↓/5↑), `router/handlers/parse` (3↓/12↑), `classifier` (7↓/5↑), `init_templates` (10↓/2↑), `output_saver` (10↓/2↑) +11 more

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
cargo test: ok (2548 passed, 3 ignored, 98 suites, 13.83s)
# 164 KB → 127 B (99% reduction)

$ trs cargo clippy
lint: 102 issues in 39 files
src/main.rs (3):
  W unused_import 8:23
  W redundant_closure 44:30
  W dead_code 112:8
# 55 KB → 5.5 KB (90% reduction)

... (240 lines, hidden sections: Install · macOS / Linux · npm (all platforms) · Windows (PowerShell) · Quick start · 1. Wire hooks into every detected AI agent (the main path) · 2. See your savings · 3. Walk back when you want: symmetric removal)

**.gitignore**
# Build artifacts
/target/
**/*.rs.bk
*.pdb

# Cargo lock file for binary projects (keep for libraries)
# Uncomment if this becomes a library:
# Cargo.lock
... (77 lines)

## AGENTS.md

# AGENTS.md: trs (Token-Reducing Shell)

## What is trs

A Rust CLI that transforms noisy terminal output into compact, structured signal.
Cuts up to 60% of the tokens AI agents spend on terminal output, measured on real sessions (see docs/development/benchmarks/).

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
  `trs rewrite`; output-saver asks for concise replies, no invented paths or
  APIs, and short code comments. Tests found reply rules save little:
  `docs/development/output-cost-findings.md`.
  Check-first by default (`--install` to commit, `--remove` to undo).
- `trs ingest`: project digest for LLM consumption. Use symbol index,
... (118 lines, hidden sections: Architecture · Key Design Decisions · Development · Testing)

## CHANGELOG.md

# Changelog

All notable changes to trs are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-11-06

### Bug Fixes

- init: Count what each agent's row reports (#233)
- run: Always propagate the exit code (#235)
- run: Read child output while it runs (#236)
- bun: Count suites from what bun prints (#238)
- npm: Stop reporting success when node --test failed (#239)
- json: Let the exit status decide the verdict (#240)
- search: Fail on a path that does not exist (#242)
- grep: Stop counting context lines as matches (#243)
- search: Make --limit cap results, not files (#244)
- exit: Report a killed child as 128+N (#245)
- diff,benchmark: Exit as the measured command did (#246)
- parse: Do not call empty test output a pass (#247)
- json: Print one document on every path (#248)
- exit: Keep failure notices where they can be read (#249)

### Documentation

- Drop cargo install, trs is not on crates.io (#234)

### Testing

- Remove tests that cannot catch a regression (#237)
- Assert what commands print instead of that they ran (#241)

## [0.9.2] - 2026-11-05

### Features

- diff: Share a byte budget across a large diff (#231)

... (756 lines, hidden sections: [0.9.1] - 2026-11-05 · [0.9.0] - 2026-11-05 · [0.8.10] - 2026-10-22 · [0.8.9] - 2026-10-19 · [0.8.8] - 2026-10-08 · [0.8.7] - 2026-10-08 · [0.8.6] - 2026-10-07 · [0.8.5] - 2026-09-26)

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
... (56 lines)

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

... (226 lines, hidden sections: Adding a new parser · Proposing new commands · Project structure · Commit messages · Releasing · Questions?)

**Cargo.toml**
[package]
name = "trs-cli"
version = "0.10.0"
edition = "2021"
description = "Token-reducing shell for AI agents: compact terminal output, up to 60% fewer tokens on real agent commands"
license = "MIT"
repository = "https://github.com/dPeluChe/trs"
homepage = "https://usetrs.dev"
publish = false
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

... (242 lines, hidden sections: 1.4 KB → 336 B (76% reducción) · 164 KB → 127 B (99% reducción) · 55 KB → 5.5 KB (90% reducción) · Instalación · macOS / Linux · npm (todas las plataformas) · Windows (PowerShell) · Inicio rápido)

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

## .cargo/

**config.toml**
# Tests run the real trs binary; point its data dir (history, tee, config)
# at target/ so a test run never writes the developer's ~/.trs.
[env]
TRS_HOME = { value = "target/test-trs-home", relative = true }

## .devin/rules/

**trs.md**
---
trigger: always_on
---

# trs (Token-Reducing Shell): terminal output optimization

Devin Local does not expose a pre-execution hook, so this rule is the way to
opt into trs. When running shell commands, prefix them with `trs` to get
compact, structured output.

# Instead of:
git status
cargo test
pnpm test

# Use:
trs git status
trs cargo test
trs pnpm test

... (79 lines, hidden sections: Output saver: keep replies cheap · Code authoring · Keeping this file lean)

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
**_config.yml**
# The digest quotes source code, and `{{` in it stops the whole Pages build.
exclude:
  - development/codebase-digest.md

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

**llms.txt**: (29 lines)
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
| Cursor | local plugin, always-apply rule | `~/.cursor/plugins/local/trs/` |
| Codex | inline with sentinels | `~/.codex/AGENTS.md` |
... (559 lines, hidden sections: Integration types · Wire-format dispatch (hook agents) · Agent attribution (`TRS_AGENT`) · Per-agent reference · Permissions per agent · Test prompts · Debugging a broken integration · 1. Create a logging wrapper)

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

**output-cost-findings.md**
# What actually moves agent cost: findings from the output-saver tests

Measured 2026-11-04 on Claude Code with sonnet. Read this before changing the
output-saver block or claiming an output-side saving. Per-test numbers and the
public claims live in [the output-saver doc](../features/output-saver.md#measuring-impact).

## The short version

- Reply rules don't pay. No rule block beat the one line "Answer concisely."
  on tokens, cost, turns or correct answers.
- Asking for brevity at all does work: about half the output tokens of no
  instruction on chat-style prompts.
- Output is a small part of the bill. Turns and context size are the lever.
- The model already reads the code. Telling it to verify changed nothing we
  could measure.
- So the block is now 698 bytes: the one line, "never invent", the comment rule.

## Where the money goes

From local transcripts, 105,592 model calls, at price ratios of 1 (input), 1.25
... (117 lines, hidden sections: The tests · What we don't know · How to run the next test · Where to go next)

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

## What we learned

[`output-cost-findings.md`](../output-cost-findings.md) records where agent cost actually goes, what the output-saver tests showed and how to run the next one.

## What's in here

... (41 lines)

**bench_stats.py**


  def boot_diff(before, after, fn=statistics.fmean, resamples=2000, seed=11):
      rng = random.Random(seed)
      diffs = sorted(
          fn(rng.choices(after, k=len(after))) - fn(rng.choices(before, k=len(before)))
          for _ in range(resamples)
      )
      return diffs[int(0.025 * resamples)], diffs[int(0.975 * resamples) - 1]

  def records(path, *needles):
      them, which is most of a 4 GB transcript folder."""
      try:
          lines = open(path, encoding="utf-8", errors="replace")
      except OSError:
          return
      for line in lines:
          if needles and not any(n in line for n in needles):
              continue
          try:
              yield json.loads(line)
          except ValueError:
              continue

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

**output_saver_ab.py**
  def trs_block()
  def markers(text)
  def run_real(prompt, arm_text, model, budget)
  def run_dry(prompt, arm_text, *_)
  def main()

**output_saver_agentic.py**
  def sh(cwd, *cmd)
  def tests_pass(cwd)
  def v_fix(cwd, reply)
  def v_find(cwd, reply)
  def v_flag(cwd, reply)
  def v_rename(cwd, reply)
  def run_one(prompt, arm_text, check, model, effort, budget, repo=REPO, allowed=ALLOWED)
  def summarize(rows, arms)
  def main()
  def work(job)

**output_saver_stale.py**: def _check(true_re) | def surfaced(task, reply)
**recovery.py**
  def segments(cmd)
  def family(cmd)
  def body(result)
  def calls(path, since)
  def recovery(cmd, fam, text, after)
  def tally(session, window, count, stats, first, back)
  def median(values)
  def main()

**transcript_output.py**
  def default_since()
  def messages(projects_dir)
  def report(name, before, after, resamples)
  def main()

**truth.py**
  def find_trs()
  def tokenizer()
  def sh(cmd, stderr=False)
  def hook(trs, cmd)
  def anchors(text)
  def facts(text)
  def docker_rows(raw)
  def facts_lost(found, text)
  def has(kind, a)
  def share(found, text)
  def risky(cut, keep)
  def buried(raw)
  def present(kind, a, text, flat)
  def kept(found, text)
  def naive_tail(raw, budget)
  def main()

## docs/development/benchmarks/compare/

**compare.py**
  def payload(cmd, **extra)
  def updated(out)
  def ts_python()
  def decide(tool, cmd)
  def run(cmd, env, cwd)
  def claw(text)
  def corpus()
  def score(raw, out, found, fact)
  def main()
  def report(rows, pipes, tools)
  def agg(t)

**corpus.txt**
# cwd::command. Shapes drawn from 14 days of real agent sessions (grep, git,
# ls, find, gh, docker, ps, test runners). {repo} is this repo; {fixtures} is
# what setup.sh builds. Git rows move with the repo: compare runs at one commit.
# [docker] rows run only when the daemon answers.
{repo}::grep -n "fn " src/report.rs
{repo}::grep -rn "emit_compressed" src
{repo}::grep -rn "fn main" src
{repo}::grep -rn -A3 "pub(crate) fn" src/report.rs
... (42 lines)

**fixtures.py**
  part of the corpus without depending on any real project's state."""

  root = sys.argv[1]
  for d in ("cargofail/src", "buntest", "pyfail"):
      os.makedirs(os.path.join(root, d), exist_ok=True)

  open(f"{root}/cargofail/Cargo.toml", "w").write(
      '[package]\nname = "cargofail"\nversion = "0.1.0"\nedition = "2021"\n')
  rs = ["fn add(a: i32, b: i32) -> i32 { a + b }", "fn main() { println!(\"{}\", add(1, 2)); }",
        "#[cfg(test)] mod tests { use super::*;"]
  rs += [f"  #[test] fn passes_{i}() {{ assert_eq!(add({i}, 1), {i + 1}); }}" for i in range(60)]
  rs += ['  #[test] fn fails_on_overflow_boundary() { assert_eq!(add(2, 2), 5, "addition drifted"); }',
         '  #[test] fn fails_parsing_config() { "abc".parse::<i32>().unwrap(); }', "}"]
  open(f"{root}/cargofail/src/main.rs", "w").write("\n".join(rs) + "\n")

  ts = ["import { test, expect } from 'bun:test';"]
  ts += [f"test('adds case {i}', () => {{ expect({i}+1).toBe({i + 1}); }});" for i in range(80)]
  ts += ["test('formats currency for MXN locale', () => { expect((1234.5).toFixed(1)).toBe('1,234.50'); });",
         "test('rejects expired token', () => { expect({ ok: true, reason: 'valid' }).toEqual({ ok: false, reason: 'expired' }); });"]
  open(f"{root}/buntest/math.test.ts", "w").write("\n".join(ts) + "\n")

  py = [f"def test_ok_{i}():\n    assert {i} + 1 == {i + 1}\n" for i in range(70)]
  py += ["def test_invoice_total_rounding():\n    total = round(0.1 + 0.2, 2)\n    assert total == 0.31, f'expected 0.31, got {total}'\n",
         "def test_user_lookup_missing():\n    users = {'ana': 1}\n    assert users['beto'] == 2\n"]
  open(f"{root}/pyfail/test_app.py", "w").write("\n".join(py))

**setup.sh**
#!/usr/bin/env bash
set -euo pipefail
R=${TRS_BENCH_REPOS:-$(spark config 2>/dev/null | awk '/repos_root/{print $3}')/github.com}
C=${TRS_BENCH_DIR:-$HOME/.cache/trs-bench}
mkdir -p "$C"
echo "clones: $R"; echo "cache:  $C"

command -v rtk >/dev/null || echo "rtk not on PATH: brew install rtk (skipped otherwise)"
... (26 lines)

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
... (137 lines, hidden sections: What it scans · Duplicate detection (SimHash) · Dead `@imports` · Update stamps · Embedded bloat · Language support for symbol extraction · Integration with `trs doctor` · Philosophy)

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

### Data directory

History, saved raw outputs (`tee/`), ingest digests and the per-user
`config.toml` live in `~/.trs`. Set `TRS_HOME` to use another directory.
... (160 lines, hidden sections: Tunable limits · Real exit status for piped checks · Hook-time wrappers · Bypassing trs for a single command · Inspecting the active config · When to skip the config file · See also)

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
... (120 lines, hidden sections: Reading the report · JSON mode · Typical fixes · See also)

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

Before the command a flag is always trs's: `trs --json git status`.
After the command it is trs's only when trs has a formatter for that
command and the command does not take the flag itself, so
`trs git status --json` and `trs --json git status` are equivalent.
Otherwise the flag belongs to the command and is passed on untouched:
... (153 lines, hidden sections: Structured output is never rewritten · One document under `--json` · Examples · When to use what · Built-in tools vs wrapped commands · See also)

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
`trs rewrite` so every command gets compressed automatically. Eighteen
agents are supported end-to-end. See [`docs/development/agent-integrations.md`](../development/agent-integrations.md)
for the full per-agent reference.

## Quick reference

trs init --show                      # status of all 18 agents
trs init --all --global              # install for every detected agent
trs init <agent>                     # install for one: claude, gemini, cursor, …
trs init --all --global --dry-run    # preview every file that would change
trs init --all --global --force      # refresh templates (see "Refreshing hooks")
trs init <agent> --replace           # migrate cleanly from another compressor

## What gets installed where

| Agent | Type | Target |
|---|---|---|
... (243 lines, hidden sections: Preview with `--dry-run` · Collision handling · Refreshing hooks · `--global` vs project-local · Bypassing the hook for one command · Agent attribution (`TRS_AGENT`) · Uninstalling · See also)

**output-saver.md**
# `trs output-saver`: reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents see, the output of the shell commands they run.

`trs output-saver` installs a short rules block into each supported
agent's global config: answer concisely, never invent paths or API
fields, keep code comments short. It is deliberately small: in
pre-registered tests no longer block beat the line "Answer concisely."
([findings](../development/output-cost-findings.md)).

## Quick reference

trs output-saver                 # read-only scan of all agents
trs output-saver --install       # write to every detected agent
trs output-saver <agent> --install  # scope to one
trs output-saver --verify        # per-agent: block matches current canonical?
trs output-saver --refresh       # re-write the block where it's already present
trs output-saver --remove        # clean uninstall
trs output-saver --print         # dump the block to stdout (pipe-friendly)
... (272 lines, hidden sections: What the block says · Why it is this short · Coverage matrix · How the install is idempotent · Output saver: keep replies cheap · Check-first semantics · `--refresh`: pick up template changes without adding new installs · `--remove` behavior)

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
... (345 lines, hidden sections: Time windows · Summary (default) · History view · `--by-agent`: attribution breakdown · `--by-command`: command family breakdown · `--coverage`: parser-gap analysis · JSON mode · What gets tracked)

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
... (147 lines, hidden sections: Detection logic · Why detection is path-based · Confirmation prompt · Roadmap · What happens after a successful upgrade · Interaction with hooks · See also)

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
- [-] Publish to crates.io: not building it. The installer, npm and release binaries cover every platform, and another channel is more to maintain.
- [ ] Shell completions (bash, zsh, fish)
- [x] Copilot hook, see Phase 3 "VSCode ecosystem"
- [ ] `trs self-update` command, re-download latest binary from GitHub Releases

---

... (656 lines, hidden sections: Phase 2: New Parsers · Phase 2.5: Ideas from competitor analysis · Phase 2.9: Sweep of the learn repos, 2026-11-03 · Exegol (our agent desktop app, github.com/dPeluChe/exegol) · Phase 3: Agent integration follow-ups · Documentation drift (carry-over from v0.5.9) · Phase 2.6: Internal architecture (May 2026 feedback) · Phase 3.5: Codex integration (mostly shipped, re-checked 2026-08-29))

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

**2608.md**
# Agosto 2026: trs development log

---

# 2026-08-26: reglas de escritura, barrido de comandos y auditoria de docs

## Context

Un ciclo largo con tres hilos que resultaron ser el mismo problema visto
desde angulos distintos: lo que trs declara no siempre coincidia con lo
que trs hace. La regla de escritura que inyectaba en cada agente la
incumplia el propio repo; el reporte de cobertura decia 45% donde la
compresion real era 0%; y la bandera `known` afirmaba parsers inexistentes en
un sentido y ocultaba parsers reales en el otro.

## Completed

### Reglas de escritura (#143, #144, #145)

- Se agregaron dos reglas al bloque que `output-saver` instala en cada
... (346 lines, hidden sections: Decisions · Files Changed · 2026-08-28: pase de calidad previo al release · Context · Completed · Decisions · 2026-08-28: cerrar los hallazgos que habian quedado diferidos · Context)

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

Eighteen AI coding agents are supported end-to-end. Each row lists the
install method, which sides of the loop trs touches (input / output),
how `trs stats --by-agent` labels runs from that agent, and the
install scope.

| Agent | Install method | Input hook (rewrite) | Output-saver | Attribution label | Scope |
|---|---|---|---|---|---|
| Claude Code | programmatic hook | ✓ | ✓ (`@import`) | `claude` | global + project |
| Gemini CLI | programmatic hook | ✓ | ✓ (`@import`) | `gemini` | global + project |
| Cursor | programmatic hook | ✓ | ✓ (local plugin) | `cursor` | global + project |
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
   and emits a structured compact form. See the measured numbers per command in [the benchmarks](../development/benchmarks/README.md).
2. Dispatched alias. A different binary with the same semantics
   (e.g. `rg` for `grep`, `eza` for `ls`) gets routed to the same
   parser. No configuration, the dispatcher recognizes the binary
   name.
3. Generic compression. Commands without a parser still get ANSI
   stripping and whitespace collapse. Consecutive repeats fold to
   `line (x3)`, or `(last 2 lines x3)` for a repeated block; each fold is
   undone and compared before it is used, and non-consecutive repeats are
   left alone because their order is information. Log lines equal but
   for their timestamps keep the first line plus `(x3, until 19:56:31)`;
   spacing, symbols and numbers still have to match, because in code they
   are content. A 300+ char line with
   almost no spaces (minified code, base64, a data URI) keeps its head
   and tail around `…[minified, 4184 chars]…`, and the full output is
... (319 lines, hidden sections: Commands with dedicated parsers · Built-in trs tools (not wrappers) · Dispatch mechanisms · Generic compression (the fallback))

**install.md**
# Installing trs

Four install channels. All of them ship the same native binary
(~7-10 MB, zero runtime deps, ~4 ms startup). Pick whichever fits your
existing toolchain.

## Quick list

| Channel | One-liner |
|---|---|
| curl / sh | `curl -fsSL https://usetrs.dev/install.sh \| sh` |
| PowerShell | `irm https://usetrs.dev/install.ps1 \| iex` |
| npm | `npm install -g @dpeluche/trs` |
| Prebuilt binary | [GitHub Releases](https://github.com/dPeluChe/trs/releases) |

The `curl|sh` and `irm|iex` scripts are the recommended default: they
detect arch, download the right prebuilt binary, place it in
`~/.local/bin/` (or `$USERPROFILE\.local\bin\` on Windows), and add
that dir to PATH automatically if it isn't already.

... (138 lines, hidden sections: Platform support · Prebuilt binaries: manual install · Pinning a specific version · Custom install directory · Upgrading · Shadowed installs (multi-channel) · Uninstall · Troubleshooting)

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
- token-optimizer: Python hook script (`bash_hook.py`) shipped as a
  Claude Code plugin, not a global package. Hook integration focused on
  Claude Code.
- token-saver: early-stage shell wrapper, smaller scope than
... (61 lines, hidden sections: How trs positions itself · Installing alongside another tool)

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
... (82 lines, hidden sections: Parser safety · Failure recovery · Permissions · Install-time safety · See also)

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

Up to 60% fewer tokens on real agent commands. What it leaves out, it says where to find.

## Install

npm install -g @dpeluche/trs

## Usage

# Git (compressed output)
trs git status
trs git diff
trs git log --oneline -20

# Test runners
trs cargo test / go test / pytest / jest / vitest / npm test
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
... (71 lines)

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

**post-release-clean.sh**
POLL="${POST_RELEASE_POLL_SECONDS:-15}"
WAIT_MAX="${POST_RELEASE_WAIT_SECONDS:-1800}"
fail()
DRY=--dry-run
BENCH_DIR="${TRS_BENCH_DIR:-$HOME/.cache/trs-bench}"

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
... (37 lines)

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

**anchor_guard.rs**
  pub fn looks_like_hash(h: &str) -> bool
  pub fn anchors_lost(raw: &str, summary: &str) -> bool
  mod tests
  fn raw_listing() -> String
  fn a_summary_that_drops_the_paths_is_flagged()
  fn a_summary_that_keeps_the_paths_is_not()
  fn small_or_anchorless_output_is_never_flagged()
  fn a_short_hash_prefix_counts_as_kept()

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
  pub fn fence_open_lang(line: &str) -> Option<String>
  pub fn is_fence_close(line: &str) -> bool
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

**audit_docs_volatile.rs**
  pub struct VolatileLine
  pub fn find_volatile(docs: &[DocFile]) -> Vec<VolatileLine>
  pub fn print_volatile(found: &[VolatileLine])
  mod tests
  fn doc(content: &str) -> DocFile
  fn stamps_are_found_outside_code_fences_only()

**benchmark.rs**
  struct IterResult
  struct BenchReport
  pub fn run_benchmark(command: &str, args: &[String], repeat: usize, json: bool)
  fn run_once(cmd: &str, args: &[String]) -> Option<IterResult>
  fn run_through_trs(cmd: &str, args: &[String]) -> Option<usize>
  fn print_json(r: &BenchReport)
  fn print_table(r: &BenchReport)
  fn format_number(n: u64) -> String
  mod tests
  fn test_format_number()
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
  fn after_runner_opts(args: &[String]) -> Option<&[String]>
  pub fn unwrap_runner<'a>(cmd: &str, args: &'a [String]) -> Option<&'a [String]>
  pub fn stderr_owner<'a>(name: &'a str, args: &'a [String]) -> (&'a str, String)
  pub fn unwrap_timeout(args: &[String]) -> Option<Vec<String>>
  pub fn is_help_request<S: AsRef<str>>(cmd: &str, args: &[S]) -> bool
  pub fn has_structured_output_flag(args: &[String]) -> bool
  pub fn subcommand(cmd: &str, args: &[String]) -> String

  enum Flag
  fn trs_flag(arg: &str) -> Option<Flag>
  pub fn child_owns_format_flag(cmd: &str, args: &[String], flag: &str) -> bool
  pub fn split_format_flags(
  mod tests

**classifier_args_tests.rs**
  fn argv(args: &[&str]) -> Vec<String>
  fn split(args: &[&str]) -> (Vec<String>, Option<OutputFormat>, bool)
  fn kept(args: &[&str]) -> Vec<String>
  fn a_flag_the_program_defines_goes_to_the_program()
  fn git_raw_is_gits_for_diff_and_log_even_after_global_options()
  fn a_flag_trs_formats_for_is_still_trs()
  fn commands_trs_has_no_formatter_for_keep_every_flag()
  fn nothing_after_a_double_dash_is_trs()
  fn args_without_our_flags_are_untouched()
  fn structured_output_requests_are_recognized()
  fn the_subcommand_skips_git_global_options()

**classifier_exec.rs**
  pub fn execute_and_parse(cmd: &str, args: &[String], ctx: &CommandContext)
  pub fn track(
  fn point_to_raw(
  fn print_raw_path(
  pub fn save_tee_output(cmd: &str, stdout: &str, stderr: &str) -> Option<String>
  fn write_user_only(path: &std::path::Path, content: &[u8]) -> std::io::Result<()>
  fn generic_compress(input: &str) -> String
  fn collapse_whitespace(s: &str) -> String

**classifier_tests.rs**
  fn argv(s: &str) -> Vec<String>
  fn ls_files_routes_to_find()
  fn commit_routes_to_parser()
  fn cargo_fmt_routes_to_parser()
  fn bash_c_simple_command_unwraps()
  fn absolute_path_routes_by_basename()
  fn bare_python_linters_and_formatters_route()
  fn runners_reach_the_parser_of_what_they_run()
  fn prettier_is_routed_however_it_is_launched()
  fn a_runner_takes_the_stderr_policy_of_what_it_runs()
  fn a_runner_with_nothing_to_route_stays_out()
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
  fn caller_selected_fields(cmd: &str, rest: &str) -> bool
  fn git_show_blob(cmd: &str, rest: &str) -> bool
  fn tail_of_a_file(cmd: &str, rest: &str) -> bool
  pub fn is_verbatim_invocation(cmd: &str, rest: &str) -> bool
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
  fn no_duplicate_command_names()
  fn verbatim_commands_are_counted_as_known()
  fn a_caller_selected_field_list_is_left_alone()
  fn verbatim_gate_sees_through_a_shell_wrapper()
  fn compressible_commands_stay_out_of_the_verbatim_class()
  fn bunx_dispatches_like_npx()
  fn git_show_of_a_file_is_verbatim()
  fn tail_is_verbatim_unless_it_reads_a_log()
  fn git_log_with_a_chosen_layout_is_verbatim()
  fn a_command_run_by_path_gets_its_own_policy()
  fn git_diff_and_show_with_a_chosen_layout_are_verbatim()

**commands.rs**
  mod commands_parse
  pub use commands_parse::ParseCommands
  pub enum Commands
  pub enum ReadLevel
  pub enum TestRunner
  pub enum ReportKind

**commands_parse.rs**: pub enum ParseCommands | mod with_file
**commands_parse_with_file.rs**: impl ParseCommands | pub fn with_file(path: PathBuf) -> Self
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
  pub fn build_report() -> String
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
  fn capture_raw(cmd: &str, args: &[String]) -> (String, i32)
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
  pub fn check_rules_blocks() -> Check
  pub fn check_agent_docs_health() -> Check
  fn human_k(n: usize) -> String
  pub fn check_version() -> Check
  pub fn check_path_accessible() -> Check
  pub fn check_dep(cmd: &str, label: &str, required: bool, hint: &str) -> Check
  pub fn check_config_dir() -> Check
  pub fn check_history_writable() -> Check
  pub fn check_stdin_pipeline() -> Check

**doctor_hooks.rs**
  pub fn check_hooks_installed() -> Check
  pub fn check_script_plugins() -> Vec<Check>
  fn script_plugin_check(name: &'static str, content: &str, default_export: bool) -> Check
  pub fn check_host_plugins() -> Vec<Check>
  fn host_plugin_check(name: &'static str, content: &str, enabled: Option<bool>) -> Check
  fn openclaw_enabled(dir: &std::path::Path) -> Option<bool>
  fn hermes_enabled(dir: &std::path::Path) -> Option<bool>
  pub fn hook_has_fired(trs_dir: &std::path::Path) -> bool
  mod tests
  fn an_outdated_script_plugin_is_flagged()
  fn host_plugins_are_checked_for_staleness_and_for_being_enabled()
  fn only_a_labelled_run_counts_as_a_hook_firing()

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
  pub fn build_command(cmd: &str, args: &[String]) -> std::process::Command
  fn platform_command(cmd: &str, args: &[String]) -> std::process::Command
  mod build_command_tests
  fn posix_spawns_directly()
  fn windows_routes_cmd_and_powershell()

**exec_verdict.rs**
  pub fn reconcile_json(
  pub fn json_document(text: &str, code: i32, raw: impl Fn() -> Option<String>) -> String
  pub struct JsonRun<'a>

  impl JsonRun<'_>
  pub fn finish(&self, text: &str, cut: bool) -> !
  pub fn print_spawn_failure(cmd: &str, error: &std::io::Error)
  pub fn notice(format: crate::OutputFormat, line: &str)
  pub fn exit_line(code: i32, raw: Option<&str>) -> String
  pub fn emit_failure_footer(
  pub fn failure_is_visible(parsed: &str) -> bool
  mod tests
  fn text_is_wrapped_with_the_verdict()
  fn a_childs_array_stays_as_written_unless_it_failed()
  fn exit_status_overrides_either_wrong_verdict()
  fn a_failure_always_says_so()
  fn the_childs_own_data_survives_a_clean_exit()
  fn a_matching_verdict_keeps_the_parser_text()
  fn a_success_that_is_not_a_bool_is_left_alone()
  fn only_a_json_object_is_reconciled()

**fast_find.rs**
  pub fn run(args: &[&str], ctx: &CommandContext)
  fn print_compact(files: &[String], dirs: &[String], _root: &str)
  fn print_raw(files: &[String], dirs: &[String])
  fn print_json(files: &[String], dirs: &[String])
  fn glob_match(pattern: &str, name: &str) -> bool
  fn glob_match_inner(p: &[char], n: &[char]) -> bool
  mod tests
  fn test_glob_match()

**fs_atomic.rs**
  pub fn write_atomic(path: &Path, content: &str) -> io::Result<()>
  pub fn write_if_unchanged(path: &Path, base: &str, content: &str) -> io::Result<()>
  mod tests
  fn replaces_content_and_leaves_no_temp_file()
  fn refuses_when_the_file_changed_since_it_was_read()
  fn a_symlink_is_written_through_and_stays_a_symlink()

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
  pub fn install_hook(tool: &AiTool, opts: InstallOpts, batch: bool) -> Outcome
  fn install_trs_md_for(tool: &AiTool, quiet: bool)
  pub fn install_all(opts: InstallOpts)
  pub fn install_finished(opts: InstallOpts)
  fn is_trs_in_path() -> bool
  pub fn check_tool(tool: &AiTool) -> bool
  pub fn check_tool_in_scope(tool: &AiTool, global: bool) -> bool
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
  pub enum Outcome

  impl Outcome
  pub fn from_row(row: &str) -> Self
  pub fn install_from_spec(spec: &HookSpec, opts: InstallOpts) -> Result<String>
  pub fn remove_hookspec_at(path: &Path, dry_run: bool) -> Result<()>
  pub fn ensure_parent(path: &Path) -> Result<()>
  fn write_hook(dir: &Path, path: &Path, content: &str, opts: InstallOpts) -> Result<String>
  pub fn merge_json_hook(
  pub fn contains_trs_rewrite(val: &serde_json::Value) -> bool
  pub fn scrub_legacy_codex_hook(
  mod tests

**init_install_plugins.rs**
  pub fn hermes_home() -> Result<PathBuf>
  pub fn openclaw_home() -> Result<PathBuf>
  pub fn openclaw_plugin_dir(openclaw_home: &Path) -> PathBuf
  pub fn hermes_plugin_dir(hermes_home: &Path) -> PathBuf
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

**init_install_rules.rs**
  pub fn install_codex_agents(opts: InstallOpts) -> Result<String>
  pub fn refresh_sentinel_block(
  fn write_agents_md_block(path: &Path, opts: InstallOpts) -> Result<String>
  pub fn install_zed_agents(opts: InstallOpts) -> Result<String>
  pub fn install_antigravity_rules(opts: InstallOpts) -> Result<String>
  pub fn install_rules(
  mod tests
  pub enum RulesBlock
  fn block_status(path: &Path, start: &str, end: &str, section: &str) -> RulesBlock
  pub fn global_rules_blocks() -> Vec<(&'static str, PathBuf, RulesBlock)>

**init_install_rules_tests.rs**
  fn refresh_sentinel_block_leaves_everything_outside_untouched()
  fn refresh_sentinel_block_refuses_to_pair_markers_from_different_blocks()
  fn refresh_sentinel_block_declines_two_well_formed_blocks()
  fn refresh_sentinel_block_declines_without_a_pair()
  fn sentinel_markers_keep_their_exact_wire_form()
  fn the_trs_marker_catches_the_backticked_spelling()
  fn block_status_tells_the_four_states_apart()
  fn block_status_reports_a_missing_file_as_absent()

**init_install_tests.rs**
  fn scrub_legacy_codex_hook_removes_trs_only_event()
  fn scrub_legacy_codex_hook_preserves_user_entries_in_same_event()
  fn scrub_legacy_codex_hook_is_noop_when_clean()
  fn scrub_legacy_codex_hook_dry_run_does_not_write()

**init_show.rs**: pub fn show_status() | pub fn show_status_and_usage()
**init_templates.rs**
  pub use crate::init_templates_plugins::*
  type = "pre_tool"
  mod tests
  fn opencode_kilo_plugins_are_windows_safe_and_idempotent()
  fn opencode_plugin_loads_on_both_major_versions()

**init_templates_plugins.rs**: (308 lines)
**init_vibe.rs**
  pub fn install_vibe_hook(opts: InstallOpts) -> Result<String>
  fn install_at(path: &Path, opts: InstallOpts) -> Result<String>
  mod tests
  fn opts() -> InstallOpts
  fn the_hook_is_valid_toml_with_the_fields_vibe_reads()
  fn install_keeps_user_hooks_and_remove_restores_them()
  fn a_stale_block_is_refreshed_in_place()
  fn dry_run_writes_nothing()

**main.rs**
  mod ai_tool
  mod anchor_guard
  mod audit_docs
  mod audit_docs_detect
  mod audit_docs_report
  mod audit_docs_symbols
  mod audit_docs_volatile
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
  mod doctor_hooks
  mod exec
  mod exec_verdict
  mod formatter
  mod help
  mod help_text
  mod help_text_more
  mod ingest
  mod main_args
  mod init
  mod init_collision
  mod init_install
  mod init_install_plugins
  mod init_install_rules
  mod init_show
  mod init_templates
  mod init_templates_plugins
  mod init_vibe
  mod main_ingest
  mod output_saver
  mod output_saver_core
  mod output_saver_write
  mod parse_out
  mod passthrough_cut
  mod path_display
  mod pointer_recall
  mod process
  mod reducer
  mod report
  mod rerun_rate
  mod rewrite
  mod rewrite_decide
  mod rewrite_gate
  mod rewrite_perm_codex
  mod rewrite_perm_cursor
  mod rewrite_perm_devin
  mod rewrite_perm_droid
  mod rewrite_perm_gemini
  mod rewrite_permissions
  mod rewrite_pipe_status
  mod rewrite_rules
  mod router
  mod safe_folds
  mod schema
  mod sentinel
  mod text_util
  pub mod tracker
  mod uninstall
  mod uninstall_scrub
  mod upgrade
  pub use cli::format_precedence
  pub use cli::{Cli, OutputFormat
  pub use commands::{Commands, ParseCommands, TestRunner
  mod fast_find
  mod fs_atomic
  fn main()
  fn run_unparsed() -> !
  fn run()
  mod tests

**main_args.rs**
  pub fn is_external_fast_path(args: &[String]) -> bool
  pub fn is_closed_pipe_panic(msg: &str) -> bool
  pub fn parse_token_budget(s: &str) -> usize
  mod tests
  fn only_a_closed_output_pipe_is_quiet()

**main_ingest.rs**: pub fn run(cmd: &Commands, ctx: &CommandContext)
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
  pub enum Target
  pub struct Agent
  pub fn plugin_files(dir: &std::path::Path) -> [(PathBuf, String); 2]
  pub fn is_trs_file(path: &std::path::Path) -> bool
  pub fn rules_dir_content(header: &str) -> String
  pub fn resolve_target_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Target
  pub enum Status
  pub enum VerifyStatus
  pub fn verify_agent(agent_id: &str) -> VerifyStatus
  fn verify_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> VerifyStatus
  pub fn scan_agent(agent_id: &str) -> Status
  fn scan_agent_with_home(agent_id: &str, home: Option<&std::path::Path>) -> Status
  pub fn replace_between(content: &str, start: &str, end: &str, new_block: &str) -> String
  mod tests
  pub use crate::output_saver_write::{install_agent, remove_agent

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
  fn standalone_file_explains_cuts_without_advertising_pipes_as_bypass()
  fn copilot_gets_its_own_instructions_file_with_apply_to()
  fn a_rules_dir_file_is_current_only_if_the_whole_file_matches()
  fn cursor_plugin(home: &std::path::Path) -> std::path::PathBuf
  fn cursor_gets_a_local_plugin_with_an_always_apply_rule()
  fn a_changed_or_missing_plugin_file_is_drift()
  fn installing_moves_trs_old_cursor_file_but_never_a_users()
  fn removing_the_plugin_keeps_what_a_user_added_next_to_it()
  fn the_old_cursor_file_counts_as_installed_so_refresh_migrates_it()
  fn kiro_steering_loads_on_every_chat_and_survives_a_refresh_check()
  fn grok_and_openclaw_keep_the_users_own_rules()
  fn the_block_stays_inside_the_size_the_doc_quotes()

**output_saver_write.rs**
  pub fn install_agent(agent_id: &str) -> Result<String>
  pub fn install_agent_with_home(
  pub fn remove_agent(agent_id: &str) -> Result<String>
  pub fn remove_agent_with_home(

**parse_out.rs**

  thread_local! {
      static SINK: RefCell<Option<String>> = const { RefCell::new(None) };
      static DROPPED: Cell<bool> = const { Cell::new(false) };
  }

  pub(crate) fn mark_dropped() {
      DROPPED.with(|d| d.set(true));
  }

  pub(crate) fn take_dropped() -> bool {
      DROPPED.with(|d| d.replace(false))
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
      DROPPED.with(|d| d.set(false));
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

**passthrough_cut.rs**
  fn needles(line: &str) -> impl Iterator<Item = &str>
  pub fn cut(text: &str, max: usize) -> Option<String>
  pub fn fit<'a>(lines: impl Iterator<Item = &'a str>, budget: usize) -> usize
  fn cut_bytes(text: &str, max: usize) -> String
  mod tests
  fn long(n: usize) -> String
  fn short_text_is_untouched()
  fn the_tail_survives()
  fn an_error_in_the_middle_is_kept()
  fn a_unique_id_in_the_middle_is_kept_and_a_repeated_one_is_not()
  fn an_id_dense_log_still_shrinks_to_the_budget()
  fn multibyte_text_does_not_panic()

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

  pub(crate) fn cwd_prefix() -> String {
      std::env::current_dir()
          .map(|d| dir_prefix(&d))
          .unwrap_or_default()
  }

  pub(crate) fn dir_prefix(dir: &Path) -> String {
      let s = dir.to_string_lossy();
      if s.is_empty() {
          return String::new();
      }
      format!("{s}{}", std::path::MAIN_SEPARATOR)
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

**pointer_recall.rs**

  static PRINTED: AtomicBool = AtomicBool::new(false);

  pub(crate) fn note_pointer() {
      PRINTED.store(true, Ordering::Relaxed);
  }

  pub(crate) fn printed() -> bool {
      PRINTED.load(Ordering::Relaxed)
  }

  pub(crate) fn recall(entries: &[HistoryEntry]) -> (usize, usize) {
      let pointers = entries.iter().filter(|e| e.ptr == Some(true)).count();
      let reads = entries
          .iter()
          .filter(|e| e.bypass.is_none() && e.cmd.contains("/tee/") && e.cmd.contains(".log"))
          .count();
      (pointers, reads)
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      fn entry(cmd: &str, ptr: Option<bool>, bypass: Option<bool>) -> HistoryEntry {
          HistoryEntry {
              cmd: cmd.into(),
              bypass,
              ptr,
              ..Default::default()
          }
      }

      #[test]
      fn counts_pointers_and_shell_reads_of_saved_files() {
          let entries = [
              entry("git show --stat", Some(true), None),
              entry("cargo test", Some(true), None),
              entry("git status", None, None),
              entry("cat /home/u/.trs/tee/17_git_show.log", None, None),
              entry("cat /home/u/.trs/tee/18_x.log", None, Some(true)),
          ];
          assert_eq!(recall(&entries), (2, 1));
      }
  }

**process.rs**
  mod helpers
  pub use helpers::{exit_code_of, exit_on_spawn_failure, spawn_failure
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
  pub fn capture_duration(mut self, capture: bool) -> Self
  pub fn run(&self) -> Result<ProcessOutput, ProcessError>
  pub fn run_checked(&self) -> Result<ProcessOutput, ProcessError>
  pub fn run(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>
  pub fn run_checked(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>
  pub fn run_with_timeout(
  mod tests

**process_helpers.rs**
  pub fn exit_code_of(status: &ExitStatus) -> i32
  pub fn exit_on_spawn_failure(cmd: &str, error: &io::Error) -> !
  pub fn spawn_failure(cmd: &str, error: &io::Error) -> (i32, String)
  pub(super) fn classify_spawn_error(command: &str, error: io::Error) -> ProcessError
  pub(super) struct OutputReaders

  impl OutputReaders
  pub(super) fn start(child: &mut std::process::Child) -> Self
  pub(super) fn finish() -> (String, String)
  fn drain(mut pipe: impl Read + Send + 'static) -> JoinHandle<String>
  fn collect(reader: Option<JoinHandle<String>>) -> String
  pub(super) trait ChildExt
  fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<ExitStatus>>

  impl ChildExt for std::process::Child

**process_tests.rs**
  fn test_process_output_success()
  fn test_process_output_failure()
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
  fn test_run_reports_exit_code_zero()
  fn test_run_no_capture_duration()
  fn test_run_capture_duration_default()
  fn test_process_builder_env_clear()
  fn test_process_output_has_output()
  fn test_run_output_larger_than_the_pipe_buffer_does_not_hang()
  fn test_run_keeps_output_around_invalid_utf8()

**report.rs**
  pub fn scrub(text: &str) -> String
  fn confirm(prompt: &str) -> bool
  fn gh_available() -> bool
  fn write_payload(body: &str, output: Option<&str>, default_name: &str) -> Option<PathBuf>
  fn finish(title: &str, path: Option<PathBuf>, submit: bool)
  pub fn run_coverage(days: Option<u64>, output: Option<&str>, submit: bool)
  pub fn run_bug(summary: Option<&str>, output: Option<&str>, submit: bool)
  mod tests
  pub fn print_hint()

**report_tests.rs**

  #[test]
  fn scrub_rewrites_the_home_directory() {
      let home = std::env::var("HOME").unwrap_or_default();
      if home.is_empty() {
          return;
      }
      let line = format!("ls {}/projects/client-work", home);
      let out = scrub(&line);
      assert!(out.starts_with("ls ~/"), "home not rewritten: {out}");
      assert!(!out.contains(&home), "home still present: {out}");
  }

  #[test]
  fn scrub_cannot_detect_an_internal_hostname_and_does_not_pretend_to() {
      let line = "ssh prod-db-01 sudo cat /etc/nftables.conf";
      assert_eq!(scrub(line), line);
  }

  #[test]
  fn scrub_leaves_ordinary_command_lines_alone() {
      for line in ["cargo test --no-fail-fast", "git status --porcelain"] {
          assert_eq!(scrub(line), line);
      }
  }

  #[test]
  fn every_command_the_install_hint_advertises_exists() {
      use clap::CommandFactory;

      let advertised: Vec<&str> = crate::report::HINT
          .lines()
          .filter_map(|l| l.trim().strip_prefix("trs "))
          .filter_map(|rest| rest.split("  ").next())
          .collect();
      assert!(
          !advertised.is_empty(),
          "hint advertises no commands, the parse stopped matching:\n{}",
          crate::report::HINT
      );

      for path in &advertised {
          let mut node = crate::cli::Cli::command();
          for (depth, name) in path.split_whitespace().enumerate() {
              let Some(sub) = node.find_subcommand(name).cloned() else {
                  panic!(
                      "the install hint advertises `trs {path}`, but `{name}` is not a subcommand \
                       of `trs {}`. Either fix the hint in src/report.rs or add the command.",
                      path.split_whitespace()
                          .take(depth)
                          .collect::<Vec<_>>()
                          .join(" ")
                  )
              };
              node = sub;
          }
      }
  }

**rerun_rate.rs**
  pub struct Rate

  impl Rate
  pub fn pct(&self) -> f64
  pub struct Reruns
  fn bucket<'a>(out: &'a mut Reruns, prev: &HistoryEntry) -> Option<&'a mut Rate>
  pub fn analyze(entries: &[HistoryEntry]) -> Reruns
  pub fn print(r: &Reruns)
  mod tests
  fn run(ts: u64, cwd: &str, cmd: &str, saved_pct: u8, ok: bool) -> HistoryEntry
  fn a_repeat_right_after_a_heavy_cut_is_counted_apart_from_a_light_one()
  fn directories_do_not_mix_and_failures_are_not_a_rerun_of_a_cut()
  fn bypass_entries_are_ignored()

**rewrite.rs**
  pub fn run_rewrite(agent_flag: Option<&str>, segments: bool, json_out: bool)
  fn handle_json_protocol(json: &serde_json::Value, agent_flag: Option<&str>)
  fn copilot_cli_or(label: &'static str) -> &'static str
  fn label_for_host(label: &'static str, copilot_cli: Option<&str>) -> &'static str

  enum HookEvent
  impl HookEvent
  fn parse(name: &str) -> Self
  fn agent_label(&self) -> &'static str
  fn agent_label_from(&self, has_antigravity_env: bool, trs_agent: Option<&str>) -> &'static str
  fn agent_label_for(&self, has_antigravity_env: bool) -> &'static str
  fn build_hook_response(
  fn input_with_command(json: &serde_json::Value, command: String) -> serde_json::Value
  fn git_in_claude_worktree(cwd: Option<&str>, cmd: &str) -> bool
  fn cmd_bypasses_trs(cmd: &str) -> bool
  fn tag_with_agent(cmd: &str, agent: &str) -> String
  fn tag_with_agent_for(cmd: &str, agent: &str, is_windows: bool) -> String
  mod tests

**rewrite_decide.rs**
  pub fn maybe_rewrite(cmd: &str) -> Option<String>
  pub(super) fn split_env_prefix(cmd: &str) -> Option<(String, &str)>
  pub fn looks_like_env_assignment(token: &str) -> bool
  pub fn strip_transparent_prefix(cmd: &str) -> Option<(&str, &str)>
  pub(super) fn strip_word_prefix<'a>(cmd: &'a str, prefix: &str) -> Option<&'a str>
  fn is_simple_command(cmd: &str) -> bool
  fn contains_unquoted(cmd: &str, needle: char) -> bool
  fn find_unquoted(cmd: &str, needle: char) -> Option<usize>
  fn comment_start(cmd: &str) -> Option<usize>
  fn split_and_chain(cmd: &str) -> Vec<&str>
  pub fn unquoted_indices(cmd: &str) -> impl Iterator<Item = usize> + '_
  fn find_unquoted_str(cmd: &str, needle: &str) -> Option<usize>
  fn pipes_output(cmd: &str) -> bool
  pub(super) fn captures_output(cmd: &str) -> bool
  mod tests
  mod capture_tests

**rewrite_decide_capture_tests.rs**

  #[test]
  fn test_piped_output_is_left_raw() {
      for cmd in [
          "git log | grep fix",
          "find . -name '*.rs' | xargs wc",
          "find src -name '*.rs' | wc -l",
          "git diff HEAD~3 | grep -c '^+'",
          "git diff --stat | tail -1",
          "ps aux | grep zellij",
          "gh pr view 1 --json title | python3 -c 'import json,sys; json.load(sys.stdin)'",
          "cargo test 2>&1 | grep 'test result'",
          "git status | head -3",
          "git log | head -5 | tail -1",
          "cargo test 2>&1 | tail -30",
          "git status |& head",
          "cd /tmp && git log | head",
      ] {
          assert_eq!(maybe_rewrite(cmd), None, "must not rewrite: {}", cmd);
      }
  }

  #[test]
  fn test_pipe_characters_that_are_not_pipes_still_rewrite() {
      assert_eq!(
          maybe_rewrite("grep -n \"a|b\" src/main.rs"),
          Some("trs grep -n \"a|b\" src/main.rs".into())
      );
      assert_eq!(
          maybe_rewrite("git log --grep='x|y'"),
          Some("trs git log --grep='x|y'".into())
      );
      assert_eq!(
          maybe_rewrite("cargo test || echo failed"),
          Some("trs cargo test || echo failed".into())
      );
  }

  #[test]
  fn ssh_is_left_alone() {
      assert_eq!(maybe_rewrite("ssh prod 'docker ps'"), None);
      assert_eq!(maybe_rewrite("ssh -o BatchMode=yes host uptime"), None);
      assert!(maybe_rewrite("sshfs host:/ /mnt").is_some());
  }

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
  fn test_captured_output_is_left_raw()
  fn test_discards_still_rewrite()
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
  fn local_programs_are_wrapped_only_when_trs_parses_them()
  fn an_option_after_a_wrapper_stays_with_the_wrapper()

**rewrite_gate.rs**
  pub enum Gate
  type GateFn = fn(cmd: &str, cwd: Option<&str>, mode: Option<&str>) -> Gate
  pub struct AgentPolicy
  fn defer(_: &str, _: Option<&str>, _: Option<&str>) -> Gate
  fn never(_: &str, _: Option<&str>, _: Option<&str>) -> Gate

  impl AgentPolicy
  pub fn policy(label: &str) -> Option<&'static AgentPolicy>
  pub fn by_tool(name: &str) -> Option<&'static AgentPolicy>
  pub fn gate_for(label: &str, cmd: &str, json: &Value) -> Gate
  mod tests
  fn labels_and_tool_names_are_unique()

**rewrite_perm_codex.rs**
  type Pattern = Vec<Vec<String>>
  pub fn gate(cmd: &str, cwd: Option<&str>) -> Gate
  fn gate_with(patterns: &[Pattern], cmd: &str) -> Gate
  fn matches_prefix(pattern: &Pattern, tokens: &[String]) -> bool
  fn tokenize(seg: &str) -> Vec<String>
  fn codex_home() -> Option<PathBuf>
  fn load(home: Option<&Path>, cwd: Option<&Path>) -> Option<Vec<Pattern>>
  fn trusted_projects(config: &Path) -> Option<Vec<PathBuf>>
  fn parse_rules(text: &str) -> Option<Vec<Pattern>>
  fn find_close(s: &str) -> Option<usize>
  fn skip_string(b: &[u8], i: usize) -> Option<usize>
  fn split_top(s: &str) -> Option<Vec<&str>>
  fn parse_call(args: &str) -> Option<Pattern>
  fn parse_pattern(s: &str) -> Option<Pattern>
  fn parse_string(s: &str) -> Option<String>
  mod tests
  fn pats(text: &str) -> Vec<Pattern>
  fn parses_decisions_alternatives_and_comments()
  fn unknown_or_broken_forms_fail_closed()
  fn a_matching_rule_of_any_decision_skips()
  fn matching_is_a_token_prefix_per_segment_after_env()
  fn single_token_and_alternative_patterns()
  fn write(dir: &Path, rel: &str, text: &str)
  fn user_rules_load_and_a_bad_file_fails_closed()
  fn project_rules_need_a_trusted_project()
  fn unreadable_config_fails_closed()
  fn tokenizer_handles_quotes_and_escapes()

**rewrite_perm_cursor.rs**
  pub fn gate(cmd: &str, cwd: Option<&str>) -> Gate
  fn gate_with(deny: &[String], cmd: &str) -> Gate
  fn shell_rule_matches(rule: &str, seg: &str) -> bool
  fn config_dir() -> Option<PathBuf>
  fn deny_rules(config: Option<&Path>, cwd: Option<&str>) -> Option<Vec<String>>
  mod tests
  fn a_deny_rule_on_the_original_stops_the_rewrite()
  fn project_cli_json_adds_deny_rules_and_a_bad_one_fails_closed()

**rewrite_perm_devin.rs**
  pub fn gate(cmd: &str, cwd: Option<&str>) -> Gate
  fn gate_with(rules: &Rules, cmd: &str) -> Gate
  fn config_files(dir: Option<&Path>) -> Vec<(PathBuf, bool)>
  fn load_rules(files: &[(PathBuf, bool)]) -> Option<Rules>
  fn rule_matches(rule: &str, cmd: &str) -> bool
  mod tests
  fn rules(allow: &[&str], ask: &[&str], deny: &[&str]) -> Rules
  fn exec_prefix_matches_whole_words_only()
  fn deny_or_ask_on_any_segment_stops_the_rewrite()
  fn only_commands_fully_covered_by_allow_are_approved()
  fn write(dir: &Path, rel: &str, text: &str)
  fn project_files_add_deny_and_ask_but_not_allow()
  fn unreadable_config_fails_closed()
  fn no_config_defers()

**rewrite_perm_droid.rs**
  mod matcher

  struct Roots
  struct Lists
  pub fn gate(cmd: &str, cwd: Option<&str>, permission_mode: Option<&str>) -> Gate
  fn roots() -> Roots
  fn gate_in(roots: &Roots, cmd: &str, cwd: Option<&str>, mode: Option<&str>) -> Gate
  fn load(roots: &Roots, cwd: Option<&str>) -> Option<Lists>
  fn read(path: &Path) -> Option<serde_json::Value>
  fn merge(lists: &mut Lists, json: &serde_json::Value, project: bool) -> Option<()>
  mod tests
  fn write(dir: &Path, rel: &str, text: &str)
  fn run(home: &Path, cwd: &Path, cmd: &str, mode: Option<&str>) -> Gate
  fn deny_hits_skip_and_allowed_commands_are_approved()
  fn unlisted_commands_defer_only_in_auto_high()
  fn a_project_allow_counts_even_in_an_untrusted_folder()
  fn org_lists_cached_from_the_api_count()
  fn unusable_settings_fail_closed()
  fn permission_rules_decide_like_droid()
  fn a_projects_rule_allow_is_ignored_but_its_ask_counts()

**rewrite_perm_droid_match.rs**
  mod rules
  pub(super) use rules::{parse as parse_rules, restricts, Rule
  fn words(seg: &str) -> Vec<&str>
  fn wrapper_takes_value(wrapper: &str, opt: &str) -> bool
  fn program_index(w: &[&str]) -> Option<usize>
  fn unquote(s: &str) -> &str
  pub(super) fn normalize(seg: &str) -> Option<String>
  pub(super) fn keys(text: &str) -> Vec<String>
  fn has_file_redirect(seg: &str) -> bool
  pub(super) fn all_allowed(cmd: &str, allow: &HashSet<String>, rules: &[Rule]) -> bool
  fn pattern_regex(pat: &str, anchored: bool) -> Option<Regex>
  fn substitutions(text: &str) -> (String, Vec<String>)
  fn candidates(text: &str, depth: usize, out: &mut Vec<String>)
  fn first_match<'a>(cmd: &str, patterns: &'a [String]) -> Option<&'a str>
  pub(super) fn blocked(cmd: &str, block: &[String]) -> bool
  pub(super) fn denied(cmd: &str, deny: &[String]) -> bool
  mod tests
  fn set(items: &[&str]) -> HashSet<String>
  fn pats(items: &[&str]) -> Vec<String>
  fn normalizes_like_droid()
  fn deny_patterns_see_the_original_but_not_the_rewrite()
  fn allow_needs_every_part_listed()

**rewrite_perm_droid_rules.rs**
  pub(in crate::rewrite_perm_droid) struct Rule
  pub(in crate::rewrite_perm_droid) fn parse(container: &Value, project: bool) -> Option<Vec<Rule>>
  fn matches(rule: &Rule, tokens: &[&str]) -> bool
  fn tokens(text: &str) -> Vec<&str>
  pub(in crate::rewrite_perm_droid) fn restricts(cmd: &str, rules: &[Rule]) -> bool
  pub(super) fn allows(seg: &str, rules: &[Rule]) -> bool
  mod tests
  fn rules(list: Value, project: bool) -> Option<Vec<Rule>>
  fn rule(decision: &str, prefix: Value) -> Value
  fn ask_and_block_match_by_ordered_tokens_and_alternatives()
  fn allow_is_strict_and_a_projects_allow_is_dropped()
  fn malformed_rules_are_unreadable()

**rewrite_perm_gemini.rs**
  mod policy
  pub fn gate(cmd: &str, cwd: Option<&str>) -> Gate

  struct TrustEnv
  fn gate_with(rules: &Rules, cmd: &str) -> Gate
  fn load(gem_dir: &Path, cwd: Option<&Path>, env: &TrustEnv) -> Option<Rules>
  fn system_dir() -> PathBuf
  fn read_settings(path: &Path) -> Option<policy::Settings>
  fn trusted(cwd: &Path, enabled: bool, env: &TrustEnv, gem_dir: &Path) -> bool
  mod tests
  fn gem(settings: Option<&str>, policies: &[(&str, &str)]) -> tempfile::TempDir
  fn run(d: &tempfile::TempDir, cwd: Option<&Path>, env: &TrustEnv, cmd: &str) -> Gate
  fn toml_deny_and_ask_stop_the_rewrite_even_behind_env_or_chains()
  fn command_regex_and_opaque_rules()
  fn blanket_deny_skips_and_other_tools_are_ignored()
  fn narrow_allow_skips_unless_it_covers_the_rewrite_or_is_blanket()
  fn blanket_allow_does_not_override_a_deny()
  fn read_only_commands_are_left_alone_unless_allowed_wholesale()
  fn unreadable_files_fail_closed()
  fn project_allow_counts_only_in_a_trusted_folder_and_replaces_the_user_list()
  fn trust_file_kinds()

**rewrite_perm_gemini_policy.rs**
  pub enum Pat

  impl Pat
  pub fn matches(&self, text: &str) -> bool
  pub struct Rules
  pub struct Settings
  pub fn parse_settings(text: &str) -> Option<Settings>
  fn strip_comments(s: &str) -> String
  pub fn add_allowed(rules: &mut Rules, entries: &[String])
  pub fn add_toml(rules: &mut Rules, text: &str) -> Option<()>
  fn names(v: Option<&toml::Value>) -> Vec<String>

**rewrite_permissions.rs**
  pub fn gate(cmd: &str, cwd: Option<&str>) -> Gate
  fn gate_with(rules: &Rules, cmd: &str) -> Gate
  fn load_rules(cwd: Option<&str>) -> Option<Rules>
  fn trusted_dirs() -> Vec<PathBuf>
  fn settings_files(cwd: Option<&str>) -> Vec<(PathBuf, Option<&Path>)>
  pub fn rule_matches(rule: &str, cmd: &str) -> bool
  pub fn glob(pat: &[u8], s: &[u8]) -> bool
  pub fn strip_env(seg: &str) -> &str
  fn skip_runner_opts<'a>(wrapper: &str, mut body: &'a str) -> &'a str
  pub fn strip_wrappers(seg: &str) -> &str
  pub fn read_only(seg: &str) -> bool
  pub fn split_segments(cmd: &str) -> Vec<&str>
  mod tests
  fn rules(allow: &[&str], ask: &[&str], deny: &[&str]) -> Rules
  fn rule_forms_match_like_claude_code()
  fn splits_chains_but_not_quotes_or_redirects()
  fn a_deny_or_ask_rule_on_the_original_command_stops_the_rewrite()
  fn glob_handles_stars_without_blowing_up()
  fn only_explicit_allow_or_read_only_commands_are_approved()
  fn unreadable_settings_fail_closed()
  fn a_rule_sees_the_command_behind_runner_options()

**rewrite_pipe_status.rs**
  pub(super) fn wrap(cmd: &str) -> Option<String>
  fn single_pipe(seg: &str) -> Option<(&str, &str)>
  fn is_status_tool(producer: &str) -> bool
  fn sink_reads_everything(sink: &str) -> bool
  mod tests
  fn wrapped(pipeline: &str) -> String
  fn a_check_piped_to_a_full_reader_gets_the_real_status()
  fn earlier_chain_segments_stay_outside()
  fn only_the_last_statement_changes()
  fn multi_line_text_is_never_mistaken_for_the_last_statement()
  fn a_status_something_else_reads_is_left_alone()
  fn a_sink_that_stops_early_would_report_a_false_failure()
  fn tools_whose_status_is_not_a_verdict_are_left_alone()
  fn anything_that_is_not_one_flat_pipe_is_left_alone()

**rewrite_rules.rs**
  pub struct Rules

  impl Rules
  pub fn decide<'a>(
  pub fn extend_from(&mut self, json: &Value, allow_counts: bool)
  pub fn read_json(path: &Path) -> Option<Option<Value>>
  pub fn string_list(json: &Value, pointer: &str) -> Vec<String>
  pub fn is_missing(e: &std::io::Error) -> bool
  pub fn word_prefix(cmd: &str, prefix: &str) -> bool
  pub fn is_trusted(dir: &Path, trusted: &[PathBuf]) -> bool
  pub const fn reads_host_config() -> bool
  pub fn project_dirs(cwd: &Path, skip: Option<&Path>) -> Vec<PathBuf>

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
  fn copilot_cli_is_told_apart_from_vscode_by_its_env_marker()
  fn a_trailing_comment_is_text_not_commands()
  fn git_inside_a_claude_worktree_is_left_unwrapped()
  fn a_rewrite_keeps_every_other_input_field()
  fn a_cursor_rewrite_keeps_the_working_directory()
  fn grok_running_claudes_hook_gets_no_rewrite()
  fn trae_keeps_its_other_input_fields_and_gets_no_decision()
  fn vibe_is_answered_in_its_own_shape_whatever_its_event_is_called()

**safe_folds.rs**
  fn has_marker(line: &str) -> bool
  pub fn fold_repeats(text: &str) -> String
  fn unfold(folded: &str) -> String
  fn timestamp_re() -> &'static regex::Regex
  pub fn fold_timestamped(text: &str) -> (String, bool)
  pub fn is_json(text: &str) -> bool
  fn dense_kind(line: &str) -> Option<String>
  pub fn elide_dense(text: &str) -> (String, bool)
  mod tests

**safe_folds_tests.rs**
  fn consecutive_repeats_fold_to_a_count()
  fn a_repeated_block_is_kept_once()
  fn order_is_kept_so_non_consecutive_repeats_stay()
  fn every_fold_undoes_to_the_exact_input()
  fn input_that_already_looks_folded_is_left_alone()
  fn folding_never_makes_output_larger()
  fn dense_lines_keep_head_tail_and_say_what_was_cut()
  fn ordinary_long_lines_and_tables_are_not_dense()
  fn eliding_is_safe_on_multibyte_text()
  fn log_lines_equal_but_for_the_time_keep_first_line_count_and_last_time()
  fn iso_timestamps_fold_too()
  fn only_times_may_differ_not_spacing_numbers_or_symbols()
  fn identical_timestamped_lines_are_left_to_the_lossless_fold()
  fn json_is_data_not_a_dense_line()
  fn is_json_wants_one_complete_document()

**sentinel.rs**
macro_rules! trs_sentinels {
    ($start:ident, $end:ident, $name:literal) => {
        pub(crate) const $start: &str = concat!("<!-- trs:", $name, ":start v1 -->");
        pub(crate) const $end: &str = concat!("<!-- trs:", $name, ":end -->");
    };
    ($start:ident, $end:ident, $section:ident, $name:literal, $body:literal) => {
        trs_sentinels!($start, $end, $name);
        pub(crate) const $section: &str = concat!(
... (20 lines)

**text_util.rs**
pub(crate) fn first_ident(s: &str) -> Option<String> {
    let mut chars = s.char_indices();
    let start = chars.position(|(_, c)| c.is_ascii_alphabetic() || c == '_')?;
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        if c.is_ascii_alphanumeric() || c == '_' {
            end = start + i + c.len_utf8();
        } else {
... (35 lines)

**tracker.rs**
  pub struct HistoryEntry

  impl HistoryEntry
  pub fn counts_toward_totals(&self) -> bool
  fn history_path() -> Option<PathBuf>
  fn dirs_path() -> Option<PathBuf>
  pub fn trs_dir() -> Option<PathBuf>
  pub fn home_dir() -> Option<PathBuf>
  fn append_history_entry(entry: &HistoryEntry)
  fn maybe_rotate_active(path: &Path, now_ts: u64)
  fn peek_first_entry_ts(path: &Path) -> Option<u64>
  fn month_key_from_ts(ts: u64) -> String
  fn open_user_only(path: &Path) -> std::io::Result<fs::File>
  pub fn log_execution(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64)
  pub fn log_run(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64, ok: bool)
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
  fn has_trs_artifacts(tool: &AiTool) -> bool

**uninstall_scrub.rs**
  pub fn scrub_trs_from_json(path: &Path, dry_run: bool) -> Result<Option<String>>
  pub fn remove_between_sentinels(
  pub fn delete_plugin_file(path: &Path, dry_run: bool) -> Result<Option<String>>
  pub fn delete_rules_file(path: &Path, dry_run: bool) -> Result<Option<String>>
  pub fn remove_output_saver(agent_id: &str, dry_run: bool) -> Result<String>
  pub fn has_output_saver_installed(agent_id: &str) -> bool
  pub fn run_output_saver_removal(tool_name: &str, agent_id: &str, dry_run: bool)
  pub fn output_saver_agent_id(tool: &AiTool) -> Option<&'static str>
  pub fn is_trs_plugin_dir_file(path: &Path) -> bool
  pub fn is_json(path: &Path) -> bool
  pub fn confirm(prompt: &str) -> bool

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

**collect_compress.rs**: pub(super) struct CompressResult | pub(super) fn read_and_compress(path: &Path, level: IngestLevel) -> Option<CompressResult>
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
  pub use super::format_util::*

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

**format_util.rs**
  pub fn strip_html_from_markdown(content: &str) -> String
  pub fn format_bytes(n: usize) -> String
  pub(super) fn extract_section(digest: &str, filename: &str) -> Option<String>
  pub(super) fn days_to_date(days: u64) -> (u64, u64, u64)
  pub fn format_tokens(n: usize) -> String
  pub(super) fn format_modified(entry: &std::fs::DirEntry) -> String

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
  mod format_util
  mod meta
  mod mod_html
  mod ollama
  mod purpose
  mod remote
  mod resolve
  mod signatures
  mod store
  pub use remote::{is_remote_ref, resolve_remote, TmpMode
  pub use resolve::resolve_project_root
  pub use store::{list_ingests, read_digest
  pub fn is_generated(name: &str) -> bool
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
**signatures.rs**
  pub(super) fn extract_signatures(content: &str, ext: &str) -> String
  fn clean_signature(line: &str) -> String
  fn has_multiline_python_sig(content: &str) -> bool
  fn join_python_multiline_sigs(content: &str) -> String

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
  pub use super::common_markers::*

**common_markers.rs**
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
  pub(crate) mod common_markers;
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
  fn keep_first_results(files: &mut Vec<GrepFile>, limit: usize)
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
  pub fn period_line(entries: &[crate::tracker::HistoryEntry]) -> String
  pub fn format_timestamp(ts: u64, offset: time::UtcOffset) -> String
  pub struct StatsInput
  pub fn handle_stats(input: &StatsInput)
  pub fn normalize_cmd(cmd: &str) -> String

**stats_coverage.rs**
  struct Agg
  impl Agg
  fn avg_in(&self) -> f64
  fn low_pct(&self) -> f64
  pub fn coverage_json_for(
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
  fn coverage_json(
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
  fn format_bypass_cell_zero_is_plain()
  fn format_bypass_cell_includes_rate_when_nonzero()
  fn format_bypass_cell_zero_total_omits_rate()
  fn recent_efficiency_ignores_an_old_outlier()
  fn failed_and_oversized_runs_stay_out_of_the_totals()

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
  fn failure_signal(block: &[&str]) -> String
  pub fn parse_bun_test_line(line: &str, ancestors: &[String]) -> Option<BunTest>
  pub fn parse_bun_duration(s: &str) -> Option<f64>
  pub fn split_bun_test_name_and_duration(s: &str) -> (String, Option<f64>)

**bun_summary.rs**
  impl ParseHandler
  pub fn is_bun_summary_line(line: &str) -> bool
  pub fn parse_bun_summary_line(line: &str, summary: &mut BunTestSummary)
  pub fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary)
  pub fn update_bun_summary_from_tests(output: &mut BunTestOutput)

**extra_build.rs**
  impl ParseHandler
  pub fn handle_build(

  enum Prev
  pub fn handle_wc(

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
  fn compact_ports(cell: &str) -> String

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

  const DROP_ANYWHERE: &[&str] = &["node_id", "gravatar_id", "_links", "url"];
  const KEEP: &str = "html_url";
  const DROP_IN_VERIFICATION: &[&str] = &["payload", "signature"];

  fn drops(key: &str, parent: &str) -> bool {
      if key == KEEP {
          return false;
      }
      key.ends_with("_url")
          || DROP_ANYWHERE.contains(&key)
          || (parent == "verification" && DROP_IN_VERIFICATION.contains(&key))
  }

  fn prune(value: &mut Value, parent: &str) {
      match value {
          Value::Object(map) => {
              map.retain(|k, _| !drops(k, parent));
              for (k, v) in map.iter_mut() {
                  prune(v, k);
              }
          }
          Value::Array(items) => {
              for v in items.iter_mut() {
                  prune(v, parent);
              }
          }
          _ => {}
      }
  }

  pub(super) fn compress_gh_api(input: &str) -> Option<String> {
      let trimmed = input.trim();
      if trimmed.is_empty() {
          return None;
      }
      let mut value: Value = serde_json::from_str(trimmed).ok()?;
      prune(&mut value, "");
      let mut out = serde_json::to_string(&value).ok()?;
      if out.len() >= trimmed.len() {
          return None;
      }
      out.push('\n');
      Some(out)
  }

  impl ParseHandler {
      pub(crate) fn handle_gh_api(
          file: &Option<std::path::PathBuf>,
          ctx: &CommandContext,
      ) -> CommandResult {
          let input = Self::read_input(file)?;
          Self::emit_compressed(&input, compress_gh_api(&input), "gh-api", ctx)
      }
  }

  #[cfg(test)]
  #[path = "gh_api_tests.rs"]
  mod tests;

**gh_api_tests.rs**
  fn parsed(s: &str) -> Value
  fn drops_link_boilerplate_but_keeps_html_url()
  fn prunes_nested_objects_and_arrays()
  fn drops_the_pgp_blob_but_keeps_the_verdict()
  fn declines_non_json_bodies()
  fn never_returns_more_than_it_received()
  fn output_is_a_single_line_of_valid_json()

**gh_pr.rs**
  fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str
  fn shared_state<'a>(states: &[&'a str]) -> Option<&'a str>
  fn shared_suffix(state: Option<&str>) -> String
  fn state_tag(shared: Option<&str>, state: &str) -> String

  impl ParseHandler
  pub fn handle_gh_pr(
  pub fn handle_gh_issue(
  pub fn handle_gh_pr_view(
  pub fn handle_gh_pr_checks(

  struct Check
  fn strip_html_comments(text: &str) -> String

**gh_run.rs**
  impl ParseHandler
  pub fn handle_gh_run(
  pub fn handle_gh_run_view(

**git_branch.rs**
  impl ParseHandler
  pub fn handle_git_branch(
  fn render_compact(current: &str, local: &[String], remote: &[String]) -> String
  fn nest_tails(tails: &[&str]) -> Vec<String>
  fn render_group(out: &mut String, branches: &[&String], indent: &str)
  mod tests
  fn names(v: &[&str]) -> Vec<String>
  fn a_remote_is_a_shadow_only_of_the_same_branch()
  fn shared_prefixes_nest_with_every_name_kept()

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
  fn diff_header_paths(line: &str) -> (String, Option<String>)

  impl ParseHandler
  pub fn handle_git_diff(
  pub fn parse_git_diff(input: &str) -> CommandResult<GitDiff>
  fn parse_git_diff_stat(input: &str) -> CommandResult<GitDiff>
  pub fn truncate_diff(diff: &mut GitDiff, max_files: usize)
  fn commit_header(input: &str) -> Option<String>

**git_diff_format.rs**
  fn shares(sizes: &[usize], budget: usize) -> Vec<usize>
  fn file_budgets(files: &[GitDiffEntry], sizes: &[usize], budget: usize) -> Vec<usize>
  fn hunk_bytes(lines: &[String]) -> usize

  impl ParseHandler
  pub fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String
  pub fn format_git_diff_json(diff: &GitDiff) -> String
  fn format_hunk_compressed(hunk: &GitDiffHunk) -> Vec<String>
  fn file_label(file: &GitDiffEntry) -> String
  pub fn format_git_diff_compact(diff: &GitDiff) -> String
  pub fn format_git_diff_raw(diff: &GitDiff) -> String
  mod tests
  fn shares_give_small_files_everything_and_never_overspend()
  fn entry(path: &str) -> GitDiffEntry
  fn generated_files_only_get_what_code_leaves()

**git_log.rs**
  fn apply_truncate(subject: &str, max: Option<usize>) -> String
  fn with_time(block: &str, time: &str) -> String
  fn condense_body(msg: &[String], hash: &str) -> Vec<String>

  impl ParseHandler
  pub fn handle_git_log(
  fn extract_subject(msg: &[String]) -> String
  fn relative_time(date_str: &str) -> String

**git_pull.rs**: impl ParseHandler | pub fn handle_git_pull(
**git_status.rs**
  fn reports_clean(input: &str) -> bool

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

**gradle.rs**
  struct Gradle
  type Reports = HashMap<(String, String), Option<String>>

  impl ParseHandler
  pub fn handle_gradle(
  fn task_line(t: &str) -> Option<(&str, &str)>
  fn failed_test(t: &str) -> Option<(&str, &str)>
  fn report_message(
  fn read_report(root: &Path, task: &str, class: &str) -> Option<String>
  fn failure_message(xml: &str, method: &str) -> Option<String>
  fn xml_escape(s: &str) -> String
  fn xml_unescape(s: &str) -> String
  fn parse_gradle(input: &str, cwd: &Path) -> Gradle
  fn format_gradle_compact(g: &Gradle) -> String
  mod tests

**gradle_tests.rs**
  fn gradle_and_the_wrapper_route_here()
  fn reads_task_and_test_lines()
  fn the_report_is_looked_up_in_the_failed_tasks_project()
  fn a_passing_test_in_the_report_adds_nothing()

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

**ls_format.rs**
  impl ParseHandler
  pub fn format_ls(ls_output: &LsOutput, format: OutputFormat) -> String
  pub fn format_ls_json(ls_output: &LsOutput) -> String
  pub fn format_ls_compact(ls_output: &LsOutput) -> String
  pub fn format_ls_raw(ls_output: &LsOutput) -> String

**maven.rs**
  struct Maven
  impl ParseHandler
  pub fn handle_maven(

  enum Level
  fn split_level(line: &str) -> (Level, &str)
  fn test_counts(body: &str) -> Option<[usize; 4]>
  fn failed_test(body: &str) -> Option<(String, &'static str)>
  fn quiet_log_re() -> &'static regex::Regex
  pub fn is_jvm_notice(t: &str) -> bool
  fn is_test_chatter(t: &str) -> bool
  fn short_class(fqcn: &str) -> &str
  fn reactor_row(body: &str) -> Option<(String, String)>
  fn parse_maven(input: &str, cwd: &str) -> Maven
  fn format_maven_compact(m: &Maven) -> String
  fn format_maven_json(m: &Maven) -> String
  mod tests

**maven_tests.rs**
  fn parse(input: &str) -> String
  fn mvn_and_the_wrapper_route_here()
  fn surefire_2_failure_headers_are_read_too()
  fn counts_sum_across_modules_and_skip_per_class_lines()
  fn paths_are_shown_relative_to_the_project()
  fn unknown_lines_are_kept()
  fn test_logs_keep_warnings_and_errors()
  fn logs_outside_tests_are_kept()
  fn jvm_notices_are_hidden_anywhere_but_app_warnings_are_not()

**mod.rs**
  pub mod aws
  pub mod brew
  pub mod bun_format
  pub mod bun_parse
  pub mod bun_summary
  pub mod extra_build
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
  pub mod gradle
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
  pub mod ls_format
  pub mod maven
  pub mod node_test
  pub mod npm_format
  pub mod npm_parse
  pub mod ollama
  pub mod pnpm_format
  pub mod pnpm_parse
  pub mod pnpm_summary
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
  pub fn emit_compressed(
  pub fn unread(found_nothing: bool, input: &str) -> bool
  pub fn emit_unparsed(input: &str, name: &str, ctx: &CommandContext) -> CommandResult
  fn finish(input: &str, out: &str, reducer: String, ctx: &CommandContext) -> CommandResult
  pub fn emit_restructured(
  pub fn format_human_size(bytes: u64) -> String

  impl CommandHandler for ParseHandler
  type Input = ParseCommands
  fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult

**node_test.rs**
  struct Failure
  pub struct NodeTestRun

  impl NodeTestRun
  pub fn read(input: &str) -> Option<Self>
  fn failures_of(input: &str, failed: usize) -> Vec<Failure>
  fn file_of(location: &str) -> String
  fn spec_failures(recap: &str) -> Vec<Failure>
  fn tap_failures(input: &str) -> Vec<Failure>

  impl NodeTestRun
  pub fn $method() -> $output

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

**ollama.rs**
  fn columns(header: &str) -> Vec<(String, usize)>
  fn compress_table(input: &str) -> Option<String>
  fn compress_pull(input: &str) -> Option<String>

  impl ParseHandler
  pub fn handle_ollama(
  mod tests

**ollama_tests.rs**
  fn table_drops_the_id_column_and_the_padding()
  fn table_keeps_a_dash_size_rather_than_dropping_the_row()
  fn table_declines_a_header_with_no_rows()
  fn pull_folds_progress_into_layers_and_a_verdict()
  fn pull_reports_an_unfinished_download_as_such()
  fn pull_declines_output_that_is_not_a_pull()

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

**pnpm_summary.rs**
  impl ParseHandler
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
  fn failure_signal(msg: &str) -> Vec<&str>

**pytest_parse.rs**
  impl ParseHandler
  pub fn parse_pytest(input: &str) -> CommandResult<PytestOutput>
  fn attach_failure(tests: &mut [TestResult], name: String, message: &str)
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
  fn short_cmd(cmd: &str) -> String
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
  fn restructuring_output_may_be_longer_than_its_input()

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
... (61 lines)

**grep_types.rs**
#[derive(Debug, Clone, Default)]
pub(crate) struct GrepMatch {
    pub(crate) line_number: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) line: String,
    pub(crate) is_context: bool,
    pub(crate) excerpt: Option<String>,
}
... (48 lines)

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
*trs ingest v0.10.0 | 140ms | 204.6KB*
