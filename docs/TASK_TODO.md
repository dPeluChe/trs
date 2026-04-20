# trs — Roadmap

Binary: `trs` | Language: Rust | Status: **Active development**

---

## Phase 1 — Release & Distribution

- [x] Create first GitHub Release — v0.1.0 shipped; at v0.5.7 now
- [x] npm publish (`@dpeluche/trs`)
- [ ] Homebrew tap (low priority — npm + curl|sh covers 99% of users)
- [ ] Publish to crates.io (`cargo install tars-cli` — currently source-only)
- [ ] Shell completions (bash, zsh, fish)
- [ ] Copilot hook — see Phase 3 "VSCode ecosystem" for the full research scope
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
- [ ] **`git push` compression** — history audit (v0.5.8) shows only
      34-41% reduction on typical `git push origin branch` output.
      Most of the remaining text is the `remote:` progress lines —
      easy to collapse or drop once the push succeeds.
- [ ] **`find` with long paths** — audit shows ~48% reduction where
      the first path arg eats the display width. Parser could
      basename-collapse logged paths the way stats --history now
      does (see `router/handlers/stats.rs::display_cmd`).
- [ ] **`cargo fmt --check` diff output** — only 32% compression on
      failures. The unified diff block has a lot of repeat whitespace
      we could collapse.
- [x] Pipe/redirect first-segment rewrite — shipped in v0.5.6
- [x] Stats header UX overhaul — shipped in v0.5.7
- [x] Brew install/upgrade handler — shipped in v0.5.7
- [x] Ping handler — shipped in v0.5.7
- [x] Swift / xcodebuild routing — shipped in v0.5.7
- [x] **Collision check in `trs init`** — shipped in v0.5.7. Detects
      competing compressor hooks (rtk / token-optimizer) in JSON hook
      files AND rules files (with `@import` following for Claude /
      Gemini). Scans home + project symmetrically. `--replace` scrubs
      competitor hook entries, `--force` installs alongside, default
      aborts with explicit recommendation.
- [x] **`trs output-saver`** — shipped in v0.5.7. Closes the output-side
      gap: installs a compact anti-preamble / anti-narration /
      result-first rules block into each agent's global config.
      Check-first UX, sentinel-wrapped idempotent re-install,
      `--remove` for clean uninstall. 8/9 agents covered
      (Antigravity is per-project only by design).

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
- [ ] **Split `router/handlers/common.rs`** (671 LOC as of v0.5.8). Two
      concerns tangled: ANSI stripping utilities (~50 LOC self-contained)
      and CommandContext / CommandError / CommandStats types. Extract
      ANSI to `router/handlers/util/ansi.rs`; keep the types in common.
      Rest of the large-file audit: most >500 files
      (audit_docs, output_saver, init, rewrite, help, ingest/*) are
      cohesive single features — splitting would fragment them.
      common.rs is the clean win.
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

### VSCode ecosystem (vanilla, not the forks)

Cursor and Windsurf are VSCode forks and already covered. Vanilla
VSCode with its AI extensions is a real gap — it's the most installed
editor on the planet and every major AI-coding integration ships there.

- [ ] **GitHub Copilot / Copilot Chat (VSCode)**. Copilot added Agent
      Mode in late 2025. Check current public API for pre-execution
      hooks: https://code.visualstudio.com/api/extension-guides/ai and
      the copilot-chat extension's announced hook surface. If no hook,
      the fallback is a rules block in `.github/copilot-instructions.md`
      (project-local, no global equivalent we know of).
- [ ] **Continue.dev**. Has a plugin API (`config.ts`, `slashCommands`,
      `contextProviders`). Confirmed in earlier reading to expose
      `streamChat` / `getContextItems` — potentially a prompt-level
      integration. Worth a focused research pass like we did for
      Kilo/OpenCode/Droid.
- [ ] **Cody (Sourcegraph)**. VSCode extension with a context-fetcher
      and custom commands. Check whether commands can prefix shell
      execution or inject system prompts.
- [ ] **Research pass**: decide whether VSCode-base agents warrant
      `trs init vscode-copilot` / `trs init continue` entries or a
      single `trs init vscode` that detects which extension is active.

### Dynamic prompt injection (deferred from output-saver research)

The output-saver file route covers the static case. Two agents expose
prompt-layer hooks we chose not to use — still open for a future
dynamic-rules feature (e.g. per-session rule swapping, A/B testing):

- [ ] **Kilo — `experimental.chat.system.transform`**. Plugin hook that
      mutates the assembled system prompt before the LLM call. The
      `experimental.` prefix means API churn risk; not worth it for a
      static block but useful for dynamic injection.
- [ ] **Droid — `SessionStart` / `UserPromptSubmit`**. Per-session and
      per-turn context injection points. Could power a future feature
      where trs adds just-in-time rules (e.g. "this session is a
      debugging session, prefer terse diffs").

### Output-saver coverage gaps

- [ ] **Windsurf Cascade plugin API research**. We write to
      `~/.codeium/windsurf/memories/global_rules.md` which works, but
      whether Cascade has a programmatic hook equivalent is unconfirmed.
- [ ] **Cursor user-rules programmatic path**. Today we drop an `.mdc`
      file into `~/.cursor/rules/`, which works because Cursor
      auto-loads every file in that directory. Confirm whether Cursor
      also exposes a programmatic API (would allow feature detection
      before install instead of blind write).

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
