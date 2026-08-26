# trs: Roadmap

Binary: `trs` | Language: Rust | Status: **Active development**

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

## Phase 2: New Parsers

- [ ] kubectl (pods, services, deployments, logs)
- [ ] AWS CLI (s3 ls, ec2 describe-instances, cloudwatch)
- [ ] next build / prisma generate
- [ ] playwright test (E2E summaries)
- [ ] Gradle / Maven build output
- [x] `gh run view`: done (v0.5.13): extracts title, conclusion, job counts, annotations, URL
- [ ] `gh issue view`: follow-on from gh pr view

### Improvements to existing parsers

- [ ] Log timestamp normalization (first = t0, rest = relative delta)
- [ ] `git diff` full (not just --stat), reformat unified diff headers
- [ ] **`find` with long paths**: audit shows ~48% reduction where the first path arg eats the display width. Parser could basename-collapse logged paths the way `stats --history` now does.
- [ ] **`cargo fmt --check` diff output**: only 32% compression on failures. The unified diff block has a lot of repeat whitespace we could collapse.
- [ ] **`xcodebuild`** (13.9% compression, 473KB total traffic). Build handler catches errors/warnings but compile-command echoes, swift intermodule dependency checks, and "Write auxiliary file" blocks still bulk up the output.
- [ ] **`awk` / `sed`**, 0-3% compression. Decision: these print arbitrary user data; compressing would risk corrupting what the agent asked for. Leave as passthrough; `TRS_SKIP=1` is the escape hatch.

---

## Phase 2.5: Ideas from competitor analysis

- [x] **Credential preservation scan**: shipped in v0.5.7
- [x] **Multilingual error keywords**: shipped in v0.5.7 (10 locales)
- [x] **Fail-open on errors**: shipped in v0.5.7
- [x] **10% ratio gate**: shipped in v0.5.10
- [x] **Lint rule grouping**: shipped in v0.5.8+, extended with `tsc` in v0.5.10
- [ ] **Read caching**: if the agent reads the same file twice in a session, return the first-read cache. Saves real tokens on multi-turn sessions. Opt-in flag to start (`trs --cache-reads`).
- [ ] **Docs auditor extensions**: recommend section-level split points; detect CLAUDE.md content duplicating README.md; SQL detection in language-less fences.
- [ ] **SQLite metrics** (consider): replace JSONL tracker with SQLite WAL for trending queries.

---

## Phase 3: Agent integration follow-ups

Context: v0.5.6 fixed all 9 supported agents end-to-end.
See [`docs/development/agent-integrations.md`](../development/agent-integrations.md) for the per-agent reference.

- [x] **Split `router/handlers/common.rs`**: shipped in v0.5.10. Extracted ANSI/emoji/control-char utilities to `router/handlers/ansi.rs` (168 LOC). `common.rs` down to 466 LOC.
- [ ] **First-byte dispatch for SKIP_PREFIXES** (`src/rewrite.rs`). Current linear scan of ~20 `starts_with` checks. A first-char dispatch table would shave more than the `has_shell_op` byte-scan did on the non-operator path. Hot path, measurable.
- [ ] **Watch `router/handlers/parse/extra_download.rs`** (463 LOC). Mixes two concepts: HTTP protocol tracer (`curl -v` / `curl -I`) and body-content compressor. Not a hard violation yet, but a clean split would be `extra_download.rs` (protocol) + `http_body.rs` (body / JSON / base64). Revisit if file crosses 500 LOC.
- [ ] **Proactive `.zshenv` check in install.sh**. If `~/.local/bin` is in the user's interactive PATH but NOT in `~/.zshenv`, IDE subshells will still fail.
- [ ] **OpenCode TUI DrizzleError root cause**. Installing our plugin crashed OpenCode's TUI on startup once with a SQLite WAL init error. Couldn't reproduce. If users report it, the plugin file is the likely cause.
- [ ] **`HookEvent::Unknown` variant**. Today unknown `hook_event_name` values default to Claude format. Silent misroute if a 4th client ships its own envelope.
- [ ] **Wrap the whole command instead of editing its text** (`rewrite_decide.rs`). v0.7.4 stopped the corruption by *refusing* every shape it can't parse (heredocs, multi-line, arrays, subshells, keywords), which costs compression on those. Wrapping as `trs sh -c '<cmd>'` would cover them instead of skipping them, but it changes stdin, exit-code and quoting semantics all at once, so it needs its own cycle and a field test. Do not swap a silent corruption for a quieter one.
- [ ] **Research: plain-text hook protocols**. Some clients may pipe the command directly (no JSON envelope). `run_rewrite` handles this via fallback, but no real client uses it yet.

### VSCode ecosystem (vanilla, not the forks)

- [x] **GitHub Copilot (VSCode), researched 2026-06, implementation turnkey,
  gated on live validation.** VS Code agent hooks (preview) speak Claude's
  format: `PreToolUse` + `hookSpecificOutput.updatedInput` +
  `permissionDecision`, exactly what `trs rewrite` already emits. Paths:
  user `~/.copilot/hooks/*.json`, workspace `.github/hooks/*.json`; VS Code
  ALSO reads `~/.claude/settings.json` + project `.claude/settings.json`, so
  users with `trs init claude --global` get de-facto coverage today
  (attributed as `claude`). Matchers are parsed but IGNORED (hook fires on
  every tool), safe: trs no-ops without `tool_input.command` (pinned by
  `test_hook_response_missing_command_returns_none`). Tool input is
  camelCase but the shell field is `command` (unaffected).
  TO SHIP `trs init vscode`: AiTool variant + VSCODE_HOOKS template with
  `TRS_AGENT=vscode trs rewrite` (own attribution), detection (`code` binary /
  `~/.vscode` / Copilot ext dir), uninstall paths, docs row.
  BLOCKER before shipping: validate interactively in VS Code (like Codex
  0.134) that `run_in_terminal` rewrites apply and `updatedInput` doesn't
  drop sibling fields (`isBackground`/`explanation`, merge vs replace is
  undocumented).
  SHIPPED: validated live 2026-06-09 (updatedInput applies cleanly to
  run_in_terminal); `trs init vscode` + TRS_AGENT=vscode attribution landed.
- [ ] **Continue.dev**: has a plugin API (`config.ts`, `slashCommands`, `contextProviders`). Worth a focused research pass like we did for Kilo/OpenCode/Droid.
- [ ] **Cody (Sourcegraph)**: VSCode extension with context-fetcher and custom commands. Check whether commands can prefix shell execution.
- [ ] **Research pass**: decide whether VSCode-base agents warrant `trs init vscode-copilot` / `trs init continue` entries or a single `trs init vscode`.

### Next agents: researched 2026-06-11, turnkey, gated on live validation

- [x] **OpenClaw, implemented, pending live validation** (`trs init
  openclaw`, plugin at `~/.openclaw/plugins/trs/` + config enable in
  `~/.openclaw/openclaw.json`). Plugin SDK (TypeScript):
  `before_tool_call` hook rewrites tool `params` (prepend `trs ` to the exec
  command, idempotency guard like OpenCode), and `resolve_exec_env` injects
  `TRS_AGENT=openclaw` into the exec environment, the cleanest cross-platform
  attribution surface in the catalog. Entry file via
  `definePluginEntry({ id, register(api) { api.on(...) } })`. TO RESOLVE at
  implementation: exact install path (workspace vs managed plugin dirs;
  install also supported via npm/local dir) and whether config registration
  in `plugins.entries.<id>` is needed. Validate live before shipping
  (install OpenClaw, confirm rewrite + attribution).
- [x] **Hermes (NousResearch/hermes-agent), implemented, pending live
  validation** (`trs init hermes`, plugin at
  `~/.hermes/plugins/trs-rewrite/` + `plugins.enabled` patch in
  `~/.hermes/config.yaml`, `HERMES_HOME` honored). Python
  plugin at `~/.hermes/plugins/<name>/` (`__init__.py` + `plugin.yaml`
  manifest listing `pre_tool_call`), registered via `register(ctx)` →
  `ctx.register_hook("pre_tool_call", fn)`; mutate `args` when
  `tool_name == "terminal"`, fail open otherwise. May require a `config.yaml`
  merge (rtk's integration does, reference: their
  `hooks/hermes/rtk-rewrite/__init__.py`, cloned under _repos_2_learn).
  Validate live before shipping.
- [x] **Zed (Agent Panel)**: rules-only via AGENTS.md (native agent has no
  tool hooks, zed#52688); ACP external agents covered transitively. ACP
  interception tracked separately under Research.
- [x] **Devin CLI ("Devin for Terminal", Cognition), 16th agent, real
  programmatic hook** (`trs init devin-cli`, hook merged under the `hooks`
  key of `~/.config/devin/config.json` global / `.devin/config.json` project,
  matcher `exec`, command `trs rewrite --caller devin-cli`; preserves existing
  config; aliases `devin-terminal` / `dcli`). Distinct product from Devin
  Desktop (rules-only). **updatedInput live-validation pending:** docs confirm
  `decision`/`permissionDecision`+`additionalContext` but not
  `hookSpecificOutput.updatedInput`; shipped optimistically (2026-07 research),
  harmless no-op if ignored. Dedicated install needed because Devin's
  `.claude/settings.json` fallback uses matcher `Bash`, which never matches
  Devin's `exec` tool.

### Evaluated: no dedicated integration needed

- [x] **t3code (pingdotgg)**: evaluated; **not applicable for a `trs init` entry.** t3code is a web-GUI *wrapper* that orchestrates other agent CLIs (Codex, Claude, Cursor, OpenCode), not an agent itself. It doesn't run shell or expose its own hook surface. trs attaches transitively at the backend-agent layer (all four already supported), so `trs init <backend>` covers it. Attribution shows the backend agent, not "t3code". Open validation: confirm the hook fires through t3code's Codex *app-server* (JSON-RPC stdio) path, as we validated interactive Codex.

### Competitor intel: headroom (chopratejas)

Context-compression layer (Python+Rust): compresses tool outputs / logs / files / RAG before the LLM, 60-95% fewer tokens. Broader scope than trs (we do terminal-command output via hooks; they do all content via library/proxy/MCP). Ideas worth a look, not adoption:

- [ ] **KV-cache effect of `TRS_AGENT=` prefix**: headroom ships a "CacheAligner" that stabilizes prefixes for provider KV-cache hits. Verify our attribution prefix doesn't bust prefix-caching; if it does, move attribution off the command prefix.
- [ ] **Reversible compression (CCR)**: store original locally, let the agent retrieve on demand. Interesting but conflicts with trs's one-shot simplicity; evaluate only if users ask for lossless recall.
- [ ] **Proxy / MCP distribution**: headroom offers a drop-in proxy and an MCP server. Confirms others ship MCP; feeds the deferred trs-MCP value question.

### Dynamic prompt injection (deferred)

- [ ] **Kilo (`experimental.chat.system.transform`)**: plugin hook that mutates the assembled system prompt. `experimental.` prefix means API churn risk; useful for dynamic injection in a future feature.
- [ ] **Droid (`SessionStart` / `UserPromptSubmit`)**: per-session and per-turn context injection points.

### Output-saver coverage gaps

- [x] **Windsurf → Devin Desktop**: Windsurf rebranded to Devin Desktop (Cognition, 2026-06-02); Cascade replaced by Devin Local (EOL 2026-07-01). Devin Local exposes no shell hook/plugin API → stays rules-only. Done: variant renamed `Devin`, dual target `.devin/rules/trs.md` (frontmatter `trigger: always_on`) + legacy `.windsurfrules`, aliases devin/devin-desktop/windsurf/cascade.
- [x] **Cursor hook surface**: verified against current schema: trs already uses the only rewrite-capable hook (`preToolUse` + `updated_input`). `beforeShellExecution` is allow/deny only, so no migration. No change needed.

### Research before building

- [ ] **trs MCP server (desktop chat apps), value analysis FIRST**: Claude Desktop / ChatGPT-Codex / Gemini desktop don't run a shell; they only execute commands via an MCP server (e.g. Desktop Commander). The only attach surface is shipping a trs MCP with a `run` tool that returns compacted output. OPEN QUESTION (do not build until answered): is it actually valuable, given the user must register the MCP server + grant directory access in the desktop app? Weigh friction vs token savings before any implementation. Framework reference: ECC's capability-surface-selection guide: "Avoid MCP when the job is a one-shot local command" (trs is exactly that for CLI agents → CLI+hooks stays primary); MCP is justified ONLY to reach MCP-only clients (desktop GUI apps), so the whole question reduces to "is reaching desktop-GUI users worth the long-lived-server overhead?"
- [ ] **ACP (Agent Client Protocol) tracking**: JetBrains+Zed open standard (JSON-RPC stdio), now in Devin Local, Kiro, 25+ agents. Editor mediates terminal access. Potential future universal attach surface; investigate whether trs can sit in the ACP terminal path.

---

## Documentation drift (carry-over from v0.5.9)

- [ ] **Designate a source of truth for the agents matrix.** Today the same table lives in `README.md`, `README.es.md`, `docs/index.html`, and `docs/support/agents.md`. Proposed fix: HTML comments pointing at `docs/support/agents.md` as canonical; checklist in `CONTRIBUTING.md`; optional CI diff check.
- [ ] Same drift risk applies to: supported-commands table, built-in tools list, and "8 of 9 agents supported" claim.
- [ ] Decide whether `docs/development/codebase-digest.md` should stay committed or move to a CI-generated release artifact.

### Writing convention: no em dashes

Rule added to the injected output-saver block in #143, then applied to the
tool itself. The convention now lives in `CONTRIBUTING.md` § Writing.

- [x] **CLI micro-copy** (#144): 92 strings across 28 modules. Two traps worth
  remembering: `doctor.rs` wrote the dash as `\u{2014}` so a literal grep
  missed it, and clap renders `///` doc comments into `--help`, so 19 lines
  that look like internal comments are the most-read copy in the tool. Verify
  against the built binary, not the source.
- [x] **Documentation and repo governance** (#145): 628 conversions across the
  docs, plus `README.es.md`, `npm/README.md`, `CONTRIBUTING.md`, `SECURITY.md`,
  the issue/PR templates, the workflows, the hooks and the install scripts.
- [ ] **Internal Rust comments** (~449) and test assertion messages. Being
  cleaned as we touch the files rather than in one sweep, agreed 2026-08-26.
- [ ] **Regenerate `docs/development/codebase-digest.md`** after the next
  release. It is `trs ingest` output, so it picks the cleanup up on its own;
  the ~192 dashes left in it today come from source comments.

Not swept, on purpose: `docs/roadmap/completed/*.md` and `CHANGELOG.md` are
dated records of a past state, and rewriting a log falsifies it. Table cells
holding `—` for "no value" are a glyph, not prose.

### Verbatim commands: known gap

- [ ] A compound shell script (`bash -c "cd x && awk '{print}' f.py"`) still
  reaches generic compression: `is_verbatim_invocation` reads the first token
  of the script only. Covering it means splitting the script on `&&`/`;`/`|`
  and treating the whole thing as verbatim if any segment is, which is a
  bigger change than the bug warranted. Field data says the direct forms are
  what actually run (`awk` 34, `iconv` 10, `cut`, `column`, `xxd`).

### Other drift found while sweeping (2026-08-26)

- [ ] `docs/features/stats.md` § Summary shows a `trs savings:` block in a
  shape the binary no longer prints. The real header is `trs Token Savings`
  followed by `Period:` / `Total commands:` rows.
- [ ] `memory/` is tracked in the repo (4 files, including `MEMORY.md`). It
  reads like agent working notes, it names a competing tool by name, and it
  duplicates state that lives outside the repo. Decide whether it belongs
  here at all.

---

## Phase 2.6: Internal architecture (May 2026 feedback)

Decoupling pass: "the project doesn't need more surface, it needs less internal
coupling and better quality measurement." Command knowledge was spread across
classifier / rewrite_decide / classifier_exec / stats_coverage.

- [x] **Unify command registry**: `src/command_registry.rs` is now the single
  source of truth for per-command facts (aliases, rewrite/known flags,
  keep_ratio, stderr policy). keep_ratio / combine_stderr / is_rewrite_command /
  is_known_binary all read from it. Behavior-preserving, golden-tested.
- [ ] **Extract `ExecutionPipeline`** from `classifier_exec.rs`, pull
  `run_command`, `combine_streams`, `parse_output`, `fallback`, `track`, `tee`
  out of the 184-line `execute_and_parse` (5 returns each duplicating
  track+tee+exit). Pure refactor, lowers blast radius. Next natural step now
  that the registry lands.
- [x] **Quality harness v1**: `tests/quality_harness.rs`: runs fixtures
  through their parsers (15 cases) and asserts signal preservation (error
  codes, failing-file basenames, failure marker) + reports per-case
  compression. Adding a case = one table row. First runs found and fixed 6
  real bugs: all-runners "no tests found" destroying unrecognized output;
  bun is_empty/success ignoring summary counts; bun FAIL-recap header
  dropped; vitest ❯ suites/tests unparsed + summary swallowed by the
  failure-details accumulator; cargo-test dropping panic location/message;
  build dropping rustc `--> file:line` locations.
- [ ] **Quality harness v2 candidates**: suggested-commands preserved;
  xcodebuild / gradle failure cases; raw recoverable.
- [ ] **Per-command config**: `[commands.cargo-test] max_failures = 20,
  preserve_backtraces = true`. Build on the existing `Limits`/`Hooks` config and
  the new registry. Granularity on top of the base config.
- [x] **`trs diff <cmd>`**: shipped in v0.6.12 (`src/diff.rs`). Raw vs compact
  header (bytes/tokens), the compact output, and the lines dropped/collapsed;
  `--json` too. Follow-up idea: feed the dropped-line analysis into a quality
  harness (shares benchmark's raw-vs-compact capture).
- [ ] **Fix environmental doctor tests**: `check_config_dir` /
  `check_history_writable` depend on a writable real `$HOME` (via
  `tracker::home_dir()`); a test must not depend on the real home. Inject a temp
  HOME / path so the suite is hermetic.
- [ ] **File-size cleanup pass** `added: 2026-06-02`, keep reusable files under
  ~500 LOC. DONE in v0.6.12: `init.rs` 711 → 303 (AiTool registry moved to
  `src/ai_tool.rs`). STILL OVER (pre-existing, each needs its own focused split):
  `audit_docs.rs` (1316), `output_saver.rs` (966), `stats.rs` (703), `tracker.rs`
  (673), `commands.rs` (601), `classifier.rs` (596), `main.rs` (555). Plus the
  `ExecutionPipeline` extraction below. Note: the v0.6.x doc-comments were
  reviewed and kept, they explain WHY (gotchas/decisions), not paraphrase.

---

## Phase 3.5: Codex integration (stale, needs rework)

Field finding (May 2026): the Codex integration is out of date in the code.

- [ ] **Re-enable Codex PreToolUse hook for `codex >= 0.129/0.130`.** The repo
  asserts Codex doesn't support `updatedInput`
  (`src/init_templates.rs`, `src/init.rs`, `docs/features/init.md`), but current
  official Codex docs say `PreToolUse` *does* accept `updatedInput.command` with
  `permissionDecision: "allow"` (https://developers.openai.com/codex/hooks).
  Today Codex relies on rules/manual-prefix, not a real hook.
- [ ] **Keep AGENTS.md as fallback + output-saver only** once the hook works.
- [ ] **`doctor`: warn when modern Codex exists without a `trs rewrite` hook.**
  Currently doctor only flags *legacy* orphan entries; it doesn't notice a modern
  Codex that *could* use a hook but has none.
- [ ] **`uninstall`: don't delete valid Codex hooks** by assuming they're legacy.
- [ ] **Output-saver dedup**: a duplicate "Output saver" section was observed in
  `~/.codex/AGENTS.md`, installer should be idempotent / detect existing block.
- [ ] **ChatGPT**: no direct `trs` install path for the ChatGPT desktop app (no
  shell-hook contract). Don't promise "ChatGPT" support except via Codex / Codex
  CLI. Audit any docs that imply otherwise.

---

## Phase 2.5b: Competitive landscape (May 2026)

Not every competitor competes at the same layer.

- Direct rivals to the core: **RTK** (PreToolUse hook, transparent rewrite,
  `gain`/`discover`/`session`, config, tee recovery, very close to trs core,
  https://www.rtk-ai.app/docs/) and **chop** (Claude/Gemini/Codex, 50+ filters,
  `.chop.yml`, SQLite tracking, and `chop diff <cmd>`, https://getchop.run/).
- `trs ingest` competes with **Repomix** (MCP, local/remote pack, grep/read,
  Tree-sitter compression, secret scanning, https://repomix.com/guide/mcp-server),
  not with shell-output compression.
- **Edgee / Tamp** compete one layer up as prompt/API proxies (tool-result
  reduction, tool-surface trimming), https://www.edgee.ai/token-compression,
  https://tamp.dev/whitepaper.pdf.

`trs ingest` is already well-differentiated (`--changed`, `--since-last`,
`--fresh`, `--deps`, `--symbols`, budget-aware packing, HEAD-keyed local cache).
Improvements vs Repomix:

- [ ] **`trs ingest --grep <pattern>`** over saved digests.
- [ ] **Secret scanning before including files** in a digest.
- [ ] **`--audit-loss` mode**: compare full vs signatures (full-vs-compact for
  ingest; mirrors the `trs diff` idea at repo scale).
- [ ] **Docs**: make clear `trs ingest` is *incremental*, not just a repo packer.

---

## Phase 2.7: Ingest upgrade: codebase intelligence (2026-07)

Design study + rationale in `INGEST_RESEARCH.md` (temp doc, gitignored). Key
learning from studying **DeusData/codebase-memory-mcp** (arXiv:2603.27277):
symbol-level dead code needs AST/LSP (they hand-wrote ~15k LOC of per-language
type resolution); but the highest-value pieces run on the **import graph we
already build**, no new deps.

**Shipped this cycle** (PR #109/#111, draft):
- [x] `ingest` captures `pub(crate)`/`pub(super)` symbols (was dropping ~500).
- [x] `trs ingest --html`: self-contained visual report: KPIs, LOC-by-module
  bars (expandable to files), force-directed module dependency graph
  (click-to-pin), oversized files, assets/binaries. `--max-loc N`.
- [x] Validated across Rust / TS / Python / monorepo (module_of groups by
  directory so multi-root layouts don't collapse the graph).

**Next, copy from codebase-memory-mcp (all on the existing graph, zero AST):**
- [x] **Purpose layer.** Port `classify_layer`
  (`store.c:4485`): label each module `entry / api / core / leaf / internal`
  from fan-in/fan-out, with an auto reason ("high fan-in: 42 in, 3 out"). Add
  fan-in **hotspots** and optional **Louvain clustering** (natural subsystems).
  Plus an **"About" block** (README H1 + first para, manifest `description`,
  `//!`/docstring module purpose). Surface in BOTH the md digest and `--html`.
- [x] **Module-level dead code, done right (no more 105 false positives).**
  Apply their trust rules at directory/module granularity: **behavioral root**
  (no inbound imports + has outbound = entry point, keep) → only *no-in + no-out
  = candidate*; whitelist tests/`main`/`lib`/exports first; fail-safe
  (query error ⇒ non-dead). Label clearly as **module-level**.
- [ ] **Symbol/function-level dead code = defer to language tools.** Don't fake
  it without AST. Optionally shell out to `cargo`/`knip`/`vulture` and surface
  their result, or just print a one-liner pointing the user there.
- [x] **Duplicate-function detection.** MinHash+LSH over
  **token-shingles** (normalize identifiers/strings/numbers → placeholders),
  per function; needs only a function-boundary scanner, not a grammar. Threshold
  ~0.8 to catch diverged clones (the npm/pnpm/bun parsers). `SIMILAR_TO` list.
- [ ] **Confidence-weighted dep edges (later).** Adopt their registry model
  (`import_map 0.95 → same_module 0.90 → unique_name 0.75 → …`) to improve graph
  fidelity for multi-language repos.

---

## Phase 4: Analytics & Configuration

- [ ] `trs stats --graph`: ASCII bar chart (30-day view)
- [ ] Version check notification (no auto-update)
- [ ] Consider migrating tracker from JSONL to SQLite (WAL mode, 90-day retention)
- [ ] Command mutation (inject `--porcelain` for more parseable output)
- [ ] Streaming mode for all parsers (not just tail)

---

## Phase 5: Plugin System (future evaluation)

- [ ] TOML filter pipeline
- [ ] Eject system (copy built-in filter to local for customization)
- [ ] Embedded stdlib of filters (compiled into the binary)
- [ ] SemanticDedup (shingle-based cross-block deduplication)

---

## Completed

### v0.5.16

- fix(template): tighter "Shell output" defensive paragraph in `standalone_file()`, directly counters the specific false belief that drove the v0.5.15 field-test failure ("compression podría ocultar detalle que necesito"). New wording: *"the compression is purely presentational: repetition and noise collapsed; signal preserved. There is no detail in raw output that the compressed form hides from you."* Still no mention of `TRS_SKIP=1` or `trs raw` by name; regression test still passes. See `docs/roadmap/completed/2605.md` § Defensive-line iteration for the option-A vs B vs C decision.

### v0.5.15

- fix(template): de-promote bypass to AI agents, `standalone_file()` no longer mentions `trs raw` or `TRS_SKIP=1` to Claude / Gemini / Cursor. Supersedes the v0.5.14 strengthening attempt; agents kept reaching for the escape hatch defensively, so visibility itself was the problem. README headline + `docs/llms.txt` "Configuration" link also demoted. Mechanism still works for humans via `trs --help`. Regression test guards against re-adding bypass mentions to the template.
- feat(stats): bypass telemetry, `tracker::log_bypass` records `TRS_SKIP=` observations from the JSON hook path. `stats --by-agent` adds a BYPASS column (count + rate); `stats --json` exposes top-level `bypass_count`. Lets the user measure whether the prompt-level intervention is reducing bypass per agent.
- refactor(tracker): extract `append_history_entry` private helper to dedupe I/O between `log_execution` and `log_bypass`.
- fix(templates): `CODEX_AGENTS_SECTION` imperative prefix, replaced soft "prefer prefixing" with explicit "you must prefix every shell command with `trs`" since Codex has no pre-execution hook; soft language was getting interpreted as optional and yielding 0% savings. Also refreshed Gemini hook format to the current matcher+nested-hooks shape.
- chore(repo): GitHub description updated from legacy `"Tars - TRS CLI"` to `"Token-Reducing Shell, terminal output compression for AI coding agents"`.
- See `docs/roadmap/completed/2605.md` for full session log + decisions.

### v0.5.14

- fix(grep): two-pass scan fixes path mangling on dashed filenames, `src/my-module/foo.rs:10:content` no longer parsed as `path="src/my"`. New algorithm: prefer `:N:` (match lines) in first pass, fall back to `-N-` (context lines) only if no colon-separated lineno found.
- fix(build): combine stderr for build tools, `cargo build/check`, `make/cmake`, `gcc/g++/clang`, `go build`, `swift build`, `xcodebuild` now merge stderr into parser input; errors and warnings were previously lost (only `cargo clippy` had this). `cargo test` is explicitly excluded.
- fix(template): sync `standalone_file()` with strong `trs raw` guidance, bold DO NOT, explicit "No prefix needed" line, routine commands listed. Next `trs upgrade` distributes the corrected version to all installed agent configs.
- fix(benchmark): `benchmark.sh` always `cd`s to project root regardless of invocation path; `timed_run` now distinguishes real timeout (exit 124) from command error (exit N), tests 6/9/10 no longer falsely show `(timeout)`.

### v0.5.13

- `gh pr diff` → GitDiff parser (~90% reduction)
- `gh pr checks`: new parser: pass/fail/pending summary, only non-passing checks with duration
- `gh run view`: new parser: title, conclusion, job counts, up to 3 annotations, URL
- `npm/pnpm/bun run format` / `format:check` → Lint parser
- `npm/pnpm/bun run lint:strict`, `lint:fix`, etc. → Lint parser (`starts_with("lint")`)
- `poetry run pytest` → Test parser; `poetry run ruff/mypy/pylint/black/flake8/isort` → Lint parser
- `git pull` / `git fetch`: new GitPull parser: strips remote progress noise, keeps branch updates and diff-stat (~85% reduction)
- `trs stats --by-command`: aggregates history by normalized command family, ranked by tokens saved
- `docs/llms.txt`: LLM agent discovery file following llms.txt spec
- `init_collision`: detects `rtk hook` format (rtk v0.37.2+ Windows binary hook)
- Refactor: split `extra_services.rs` (672 LOC) → `gh_pr.rs` + `gh_run.rs`

### v0.5.12

- Fix npm platform packages publishing at wrong version (0.5.9 since v0.5.10), artifact assembly now runs before version sync in release workflow

### v0.5.11

- `git grep` → Grep parser (was falling through to generic compression)
- `npm run` / `pnpm run` / `bun run`: route by script name: `build*` → Build, `test*` → Test, `lint`/`type-check` → Lint
- `npx @biomejs/biome`: package-name form now routes to Lint (short name `biome` already worked)
- Unified `trs.md`, replaces `trs-output-saver.md`; includes input-rewrite section + output-saver rules; migration removes legacy file on next install
- `trs init --global` writes `trs.md` alongside `hooks.json` for Claude Code and Gemini CLI
- Removed `@RTK.md` from `~/.claude/CLAUDE.md` (no longer needed, trs covers everything)

### v0.5.10

- Fast-path intercepts for `cat`, `head`, `sed -n X,Yp`, `filter_minimal` applied before subprocess spawn (10–35% savings)
- `git show`: `git stash show -p`, `stash pop`, `stash apply`, routed to GitDiff parser (~90% reduction)
- `gh pr view`: new GhPrView parser: title, state, author, url, labels, 3-line body preview (~45% reduction)
- `tsc` linter parser: `file(line,col): error TS6133: message` format, grouped by file (~80% reduction); dispatched via `npx tsc`, `pnpm dlx tsc`
- `git push/pull/fetch`: `remote:` progress lines stripped on success (~85% vs previous 34–41%)
- 10% ratio gate in `classifier_exec`, skips parser if `keep_ratio > 0.90`, falls through to generic compression
- Split `common.rs` (671 LOC) → `ansi.rs` (168 LOC) + `common.rs` (466 LOC); all callers unchanged via re-export
- Dropped unused crates: `grep-matcher`, `grep-regex`, `grep-searcher`
- `inject_file_path` free function refactored → `ParseCommands::with_file()` method; `classifier.rs` from 519 → 471 LOC

### v0.5.9

- `trs output-saver`: installs compact anti-preamble / result-first rules block into each agent's global config (8/9 agents)
- Stats header UX overhaul
- Brew install/upgrade handler
- Ping handler
- Swift / xcodebuild routing
- Collision check in `trs init`: detects competing hooks, `--replace` / `--force` / default-abort flow
- Credential preservation scan (`contains_credential`)
- Multilingual error keywords (10 locales)
- Fail-open on errors (`output_has_failure_signal`)
- Lint rule grouping: eslint/ruff/pylint/golangci-lint/cargo clippy grouped by file + rule

### v0.5.8 and earlier

- Pipe/redirect first-segment rewrite (v0.5.6)
- Chain-aware rewrite for `cd X && git Y` (v0.5.5)
- npm publish (`@dpeluche/trs`)
- First GitHub Release (v0.1.0)
