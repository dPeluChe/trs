# trs — Roadmap

Binary: `trs` | Language: Rust | Status: **Active development**

---

## Phase 1 — Release & Distribution

- [x] Create first GitHub Release — v0.1.0 shipped
- [x] npm publish (`@dpeluche/trs`)
- [x] Rewrite hook: detect `cd X && git Y` chains — done in v0.5.5
- [x] Pipe/redirect first-segment rewrite — shipped in v0.5.6
- [ ] Homebrew tap (low priority — npm + curl|sh covers 99% of users)
- [ ] Publish to crates.io (`cargo install trs-cli` — currently source-only)
- [ ] Shell completions (bash, zsh, fish)
- [ ] Copilot hook — see Phase 3 "VSCode ecosystem"
- [ ] `trs self-update` command — re-download latest binary from GitHub Releases

---

## Phase 2 — New Parsers

- [ ] kubectl (pods, services, deployments, logs)
- [ ] AWS CLI (s3 ls, ec2 describe-instances, cloudwatch)
- [ ] next build / prisma generate
- [ ] playwright test (E2E summaries)
- [ ] Gradle / Maven build output
- [ ] `gh issue view` / `gh run view` — follow-on from gh pr view (v0.5.10)

### Improvements to existing parsers

- [ ] Log timestamp normalization (first = t0, rest = relative delta)
- [ ] `git diff` full (not just --stat) — reformat unified diff headers
- [ ] **`find` with long paths** — audit shows ~48% reduction where the first path arg eats the display width. Parser could basename-collapse logged paths the way `stats --history` now does.
- [ ] **`cargo fmt --check` diff output** — only 32% compression on failures. The unified diff block has a lot of repeat whitespace we could collapse.
- [ ] **`xcodebuild`** (13.9% compression, 473KB total traffic). Build handler catches errors/warnings but compile-command echoes, swift intermodule dependency checks, and "Write auxiliary file" blocks still bulk up the output.
- [ ] **`awk` / `sed`** — 0-3% compression. Decision: these print arbitrary user data; compressing would risk corrupting what the agent asked for. Leave as passthrough; `TRS_SKIP=1` is the escape hatch.

---

## Phase 2.5 — Ideas from competitor analysis

- [x] **Credential preservation scan** — shipped in v0.5.7
- [x] **Multilingual error keywords** — shipped in v0.5.7 (10 locales)
- [x] **Fail-open on errors** — shipped in v0.5.7
- [x] **10% ratio gate** — shipped in v0.5.10
- [x] **Lint rule grouping** — shipped in v0.5.8+, extended with `tsc` in v0.5.10
- [ ] **Read caching** — if the agent reads the same file twice in a session, return the first-read cache. Saves real tokens on multi-turn sessions. Opt-in flag to start (`trs --cache-reads`).
- [ ] **Docs auditor extensions**: recommend section-level split points; detect CLAUDE.md content duplicating README.md; SQL detection in language-less fences.
- [ ] **SQLite metrics** (consider): replace JSONL tracker with SQLite WAL for trending queries.

---

## Phase 3 — Agent integration follow-ups

Context: v0.5.6 fixed all 9 supported agents end-to-end.
See [`docs/development/agent-integrations.md`](../development/agent-integrations.md) for the per-agent reference.

- [x] **Split `router/handlers/common.rs`** — shipped in v0.5.10. Extracted ANSI/emoji/control-char utilities to `router/handlers/ansi.rs` (168 LOC). `common.rs` down to 466 LOC.
- [ ] **First-byte dispatch for SKIP_PREFIXES** (`src/rewrite.rs`). Current linear scan of ~20 `starts_with` checks. A first-char dispatch table would shave more than the `has_shell_op` byte-scan did on the non-operator path. Hot path — measurable.
- [ ] **Watch `router/handlers/parse/extra_download.rs`** (463 LOC). Mixes two concepts: HTTP protocol tracer (`curl -v` / `curl -I`) and body-content compressor. Not a hard violation yet, but a clean split would be `extra_download.rs` (protocol) + `http_body.rs` (body / JSON / base64). Revisit if file crosses 500 LOC.
- [ ] **Proactive `.zshenv` check in install.sh**. If `~/.local/bin` is in the user's interactive PATH but NOT in `~/.zshenv`, IDE subshells will still fail.
- [ ] **OpenCode TUI DrizzleError root cause**. Installing our plugin crashed OpenCode's TUI on startup once with a SQLite WAL init error. Couldn't reproduce. If users report it, the plugin file is the likely cause.
- [ ] **`HookEvent::Unknown` variant**. Today unknown `hook_event_name` values default to Claude format. Silent misroute if a 4th client ships its own envelope.
- [ ] **Research: plain-text hook protocols**. Some clients may pipe the command directly (no JSON envelope). `run_rewrite` handles this via fallback, but no real client uses it yet.

### VSCode ecosystem (vanilla, not the forks)

- [ ] **GitHub Copilot / Copilot Chat (VSCode)** — check current public API for pre-execution hooks. Fallback: rules block in `.github/copilot-instructions.md`.
- [ ] **Continue.dev** — has a plugin API (`config.ts`, `slashCommands`, `contextProviders`). Worth a focused research pass like we did for Kilo/OpenCode/Droid.
- [ ] **Cody (Sourcegraph)** — VSCode extension with context-fetcher and custom commands. Check whether commands can prefix shell execution.
- [ ] **Research pass**: decide whether VSCode-base agents warrant `trs init vscode-copilot` / `trs init continue` entries or a single `trs init vscode`.

### Dynamic prompt injection (deferred)

- [ ] **Kilo — `experimental.chat.system.transform`** — plugin hook that mutates the assembled system prompt. `experimental.` prefix means API churn risk; useful for dynamic injection in a future feature.
- [ ] **Droid — `SessionStart` / `UserPromptSubmit`** — per-session and per-turn context injection points.

### Output-saver coverage gaps

- [ ] **Windsurf Cascade plugin API research** — confirm whether Cascade has a programmatic hook equivalent.
- [ ] **Cursor user-rules programmatic path** — confirm whether Cursor exposes a programmatic API beyond the `.mdc` file drop.

---

## Documentation drift (carry-over from v0.5.9)

- [ ] **Designate a source of truth for the agents matrix.** Today the same table lives in `README.md`, `README.es.md`, `docs/index.html`, and `docs/support/agents.md`. Proposed fix: HTML comments pointing at `docs/support/agents.md` as canonical; checklist in `CONTRIBUTING.md`; optional CI diff check.
- [ ] Same drift risk applies to: supported-commands table, built-in tools list, and "8 of 9 agents supported" claim.
- [ ] Decide whether `docs/development/codebase-digest.md` should stay committed or move to a CI-generated release artifact.

---

## Phase 4 — Analytics & Configuration

- [ ] `trs stats --graph` — ASCII bar chart (30-day view)
- [ ] Version check notification (no auto-update)
- [ ] Consider migrating tracker from JSONL to SQLite (WAL mode, 90-day retention)
- [ ] Command mutation (inject `--porcelain` for more parseable output)
- [ ] Streaming mode for all parsers (not just tail)

---

## Phase 5 — Plugin System (future evaluation)

- [ ] TOML filter pipeline
- [ ] Eject system (copy built-in filter to local for customization)
- [ ] Embedded stdlib of filters (compiled into the binary)
- [ ] SemanticDedup (shingle-based cross-block deduplication)

---

## Completed

### v0.5.11

- `git grep` → Grep parser (was falling through to generic compression)
- `npm run` / `pnpm run` / `bun run` — route by script name: `build*` → Build, `test*` → Test, `lint`/`type-check` → Lint
- `npx @biomejs/biome` — package-name form now routes to Lint (short name `biome` already worked)
- Unified `trs.md` — replaces `trs-output-saver.md`; includes input-rewrite section + output-saver rules; migration removes legacy file on next install
- `trs init --global` writes `trs.md` alongside `hooks.json` for Claude Code and Gemini CLI
- Removed `@RTK.md` from `~/.claude/CLAUDE.md` (no longer needed — trs covers everything)

### v0.5.10

- Fast-path intercepts for `cat`, `head`, `sed -n X,Yp` — `filter_minimal` applied before subprocess spawn (10–35% savings)
- `git show`, `git stash show -p`, `stash pop`, `stash apply` — routed to GitDiff parser (~90% reduction)
- `gh pr view` — new GhPrView parser: title, state, author, url, labels, 3-line body preview (~45% reduction)
- `tsc` linter parser — `file(line,col): error TS6133: message` format, grouped by file (~80% reduction); dispatched via `npx tsc`, `pnpm dlx tsc`
- `git push/pull/fetch` — `remote:` progress lines stripped on success (~85% vs previous 34–41%)
- 10% ratio gate in `classifier_exec` — skips parser if `keep_ratio > 0.90`, falls through to generic compression
- Split `common.rs` (671 LOC) → `ansi.rs` (168 LOC) + `common.rs` (466 LOC); all callers unchanged via re-export
- Dropped unused crates: `grep-matcher`, `grep-regex`, `grep-searcher`
- `inject_file_path` free function refactored → `ParseCommands::with_file()` method; `classifier.rs` from 519 → 471 LOC

### v0.5.9

- `trs output-saver` — installs compact anti-preamble / result-first rules block into each agent's global config (8/9 agents)
- Stats header UX overhaul
- Brew install/upgrade handler
- Ping handler
- Swift / xcodebuild routing
- Collision check in `trs init` — detects competing hooks, `--replace` / `--force` / default-abort flow
- Credential preservation scan (`contains_credential`)
- Multilingual error keywords (10 locales)
- Fail-open on errors (`output_has_failure_signal`)
- Lint rule grouping — eslint/ruff/pylint/golangci-lint/cargo clippy grouped by file + rule

### v0.5.8 and earlier

- Pipe/redirect first-segment rewrite (v0.5.6)
- Chain-aware rewrite for `cd X && git Y` (v0.5.5)
- npm publish (`@dpeluche/trs`)
- First GitHub Release (v0.1.0)
