# trs — Roadmap

Binary: `trs` | Language: Rust | Status: **Active development**

---

## Phase 1 — Release & Distribution

- [x] Create first GitHub Release — v0.1.0 shipped; at v0.5.6 now
- [x] npm publish (`@dpeluche/trs`)
- [ ] Homebrew tap (low priority — npm + curl|sh covers 99% of users)
- [ ] Publish to crates.io (`cargo install tars-cli` — currently source-only)
- [ ] Shell completions (bash, zsh, fish)
- [ ] Copilot hook (needs research — GitHub Copilot's agent hooks aren't public yet)
- [x] ~~Detect pipe context — skip rewriting find/fd when piped~~ —
      replaced in v0.5.6: rewrite the producer segment and pass the pipe
      through unchanged. `git status | head -3` now becomes
      `trs git status | head -3` instead of being skipped entirely.
- [x] Rewrite hook: detect `cd X && git Y` chains — done in v0.5.5
      (chain-aware per-segment rewrite)
- [ ] `trs self-update` command — re-download latest binary from GitHub
      Releases to avoid re-running `curl | sh`. ~30 LOC.

---

## Phase 2 — New Parsers

- [ ] kubectl (pods, services, deployments, logs)
- [ ] AWS CLI (s3 ls, ec2 describe-instances, cloudwatch)
- [ ] gh pr view / gh issue view (detail view, not just list)
- [ ] next build / prisma generate
- [ ] playwright test (E2E summaries)
- [ ] Gradle / Maven build output

### Improvements to existing parsers
- [ ] Log timestamp normalization (first = t0, rest = relative delta)
- [ ] `git diff` full (not just --stat) — reformat unified diff headers
- [x] Pipe/redirect first-segment rewrite — shipped in v0.5.6
- [x] Stats header UX overhaul — shipped in v0.5.7
- [x] Brew install/upgrade handler — shipped in v0.5.7
- [x] Ping handler — shipped in v0.5.7
- [x] Swift / xcodebuild routing — shipped in v0.5.7

---

## Phase 2.5 — Ideas from competitor analysis (token-optimizer)

Researched https://github.com/alexgreensh/token-optimizer for ideas.
Status of each candidate after v0.5.7:

- [x] **Credential preservation scan** — shipped in v0.5.7.
      `common::contains_credential` + new "preserved" bucket in
      handle_build. Covers AWS/GitHub/Stripe/JWT/URL basic-auth/PEM.
- [x] **Multilingual error keywords** — shipped in v0.5.7. 10 locales
      (en/de/fr/es/pt/it/ru/zh-simp/zh-trad/ja/ko) in
      `common::is_error_line` + `is_warning_line`.
- [x] **Fail-open on errors** — shipped in v0.5.7.
      `common::output_has_failure_signal` guards handle_build and
      handle_brew (can extend to more handlers as feedback arrives).
- [ ] **10% ratio gate**: if `compressed_bytes / input_bytes > 0.90`, skip
      the compression and emit raw. Guards against handlers that succeed
      but barely improve things. Apply once in `classifier_exec` after
      the handler returns.
- [ ] **Lint rule grouping**: for eslint/ruff/pylint/golangci-lint output,
      group `file:line:col - rule (source)` entries by file and rule.
      Reduces N file-repeated lines to "src/foo.rs (3): W unused_import
      8:23, 12:5, 45:7".
- [ ] **Read caching** (newly-identified from fleet-auditor deep-dive).
      If the agent reads the same file twice in a session, return the
      first-read cache instead of re-reading. Saves real tokens on
      multi-turn sessions where the agent inspects the same file
      repeatedly. Opt-in flag to start (`trs --cache-reads`).
- [ ] **Docs auditor extensions** (from v0.5.7 dog-fooding):
      - recommend section-level split points (largest H2/H3 by tokens)
      - detect CLAUDE.md content that duplicates README.md
      - SQL / query detection in language-less fences (pure text that
        reads like SQL)
- [ ] **SQLite metrics** (consider): token-optimizer uses a SQLite
      `compression_events` table instead of JSONL. Enables trending
      queries (`WHERE feature='git-status' AND quality_preserved=0`).
      Medium effort, good-to-have.

## Phase 3 — Agent integration follow-ups

Context: v0.5.6 fixed all 9 supported agents end-to-end. See
[`docs/agent-integrations.md`](./agent-integrations.md) for the full
per-agent reference. Outstanding items:

- [ ] **First-byte dispatch for SKIP_PREFIXES** (`src/rewrite.rs`). Current
      linear scan of ~20 `starts_with` checks. A first-char dispatch table
      would shave more than the `has_shell_op` byte-scan did on the
      non-operator path. Hot path — measurable.
- [ ] **Proactive `.zshenv` check in install.sh**. If `~/.local/bin` is in
      the user's interactive PATH (installer's $PATH) but NOT referenced
      in `~/.zshenv`, IDE subshells will still fail. Could detect and
      warn, or offer to add the line.
- [ ] **OpenCode TUI DrizzleError root cause**. Installing our plugin
      crashed OpenCode's TUI on startup once with a SQLite WAL init
      error. Couldn't reproduce on subsequent runs. If users report it,
      the plugin file is the likely cause — delete to recover.
- [ ] **`HookEvent::Unknown` variant**. Today unknown `hook_event_name`
      values default to Claude format. Silent misroute if a 4th client
      ever ships with its own envelope. Explicit `Unknown` variant logged
      to stderr would make the failure obvious.
- [ ] **Research: plain-text hook protocols**. Some clients may pipe the
      command directly (no JSON envelope). `run_rewrite` already handles
      this via the fallback plain-text path, but no real client uses it
      yet. Worth confirming no agent silently supports it and we're not
      emitting JSON into their stdin chain.

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
