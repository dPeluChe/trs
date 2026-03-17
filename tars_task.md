# TARS CLI Development Tasks

Binary command: `trs`
Language: Rust
Status: **Active development**

TARS is a CLI toolkit that transforms noisy terminal output into compact and structured signal for developers, automation pipelines, and AI agents.

---

# Completed

## Core Architecture
- [x] Rust CLI with clap 4 (derive mode)
- [x] Binary name: `trs`
- [x] Global flags: `--raw`, `--compact`, `--json`, `--csv`, `--tsv`, `--agent`, `--stats`
- [x] Flag precedence rules (json > csv > tsv > agent > compact > raw)
- [x] Flags work before or after command (`trs --json git status` = `trs git status --json`)
- [x] Command routing system (router + handlers)
- [x] Auto-detect classifier (`trs git status` → git-status parser)
- [x] Process execution with timeout, exit code propagation
- [x] Help system with command-specific help
- [x] Modular architecture (63 files, max 621 LOC)

## Parsers (auto-detect mode: `trs <command>`)
- [x] git status (branch, ahead/behind, staged/unstaged/untracked)
- [x] git diff (files changed, additions/deletions, --stat format)
- [x] git log (grouped by author, relative time, truncated messages)
- [x] git branch (current + local/remote, filters duplicates)
- [x] ls (dirs/files/symlinks, sizes, filters ./.. and noise)
- [x] find (grouped by directory, tree-like output)
- [x] grep (grouped by file, line numbers, excerpts)
- [x] tree (summary with dir/file counts)
- [x] docker ps (container status)
- [x] docker logs (reuses log parser)
- [x] env (grouped by category, filtered noise, PATH summarized)
- [x] Test runners: pytest, jest, vitest, npm, pnpm, bun
- [x] Package managers: npm ls, pip list, cargo tree
- [x] Install output: npm install, pip install
- [x] Build output: cargo build, tsc, gcc, make
- [x] Log/tail parsing with level detection and repeated line collapse

## Built-in Tools
- [x] search (ripgrep-powered, grouped by file, excerpts)
- [x] replace (regex, dry-run, count, directory ignore)
- [x] tail (last N lines, error filter, follow/streaming)
- [x] clean (ANSI strip, blank collapse, trim, repeat collapse)
- [x] html2md (URL or file → Markdown)
- [x] txt2md (heading/list detection, metadata)
- [x] trim (leading/trailing whitespace)
- [x] is-clean (git repo state check)

## Output Formats
- [x] Compact (default, human-readable)
- [x] JSON (structured, with schema version)
- [x] CSV (headers + rows)
- [x] TSV (tab-separated)
- [x] Agent (AI-optimized markdown)
- [x] Raw (passthrough)
- [x] Stats mode (input/output bytes, token estimation, reduction %)

## Testing
- [x] 2,399 tests total, 0 failures, 0 warnings
- [x] 16 test suites (unit + integration)
- [x] Fixtures for git, ls, grep, test runners, logs
- [x] Benchmark script (./scripts/benchmark.sh)

## Distribution
- [x] cargo install --path .
- [x] npm package wrapper (tars-cli)
- [x] GitHub Actions CI (test on ubuntu + macos)
- [x] GitHub Actions Release (5 platform binaries on tag)
- [x] Claude Code skill (/trs command)

---

# Pending / Next Steps

## High Priority
- [ ] Push to GitHub + create first release (v0.1.0)
- [ ] npm publish (after GitHub Release with binaries)
- [ ] Homebrew formula
- [ ] Close remaining gaps vs rtk (git diff reduction)

## Medium Priority
- [ ] MCP Server (expose trs as tool for any AI agent)
- [ ] Claude Code hook (auto-rewrite commands through trs)
- [ ] curl/wget output parsing (HTTP headers, response codes)
- [ ] kubectl output parsing (pods, services, deployments)
- [ ] AWS CLI output parsing

## Low Priority
- [ ] Streaming mode for all parsers (not just tail)
- [ ] Config file support (~/.trs.toml)
- [ ] Plugin system for custom parsers
- [ ] Shell completions (bash, zsh, fish)
- [ ] man page generation
