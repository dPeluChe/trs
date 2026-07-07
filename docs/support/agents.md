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
| Devin CLI | programmatic hook | ✓ | — | `devin-cli` | global + project |
| VS Code Copilot | programmatic hook | ✓ | — | `vscode` | global + project |
| OpenClaw | plugin template | ✓ | — | `openclaw` | global |
| Hermes | plugin template | ✓ | — | `hermes` | global |
| Zed (Agent Panel) | rules file only (AGENTS.md) | — | — | `(untagged)`; ACP external agents show their own label | project |

## Column legend

- **Install method.** *Programmatic hook* means the agent fires trs on
  every tool-use event via a JSON hook config. *Plugin template* means
  the agent loads a plugin/extension that calls trs. *Rules file only*
  means the agent has no programmatic hook surface — the only thing
  we can do is append a Markdown rules block that the model reads at
  session start.
- **Input hook (rewrite).** Whether `trs init` can install a hook that
  rewrites the agent's outbound commands (`git status` → `trs git
  status`). Rules-only agents cannot do this; the model ends up
  running raw commands unless the user prefixes `trs` manually.
- **Output-saver.** Whether `trs output-saver --install` can inject
  the anti-preamble / result-first rules block (Pi, VS Code Copilot,
  OpenClaw, and Hermes are not yet wired). Antigravity 2.0 (IDE + CLI)
  shares Gemini's
  `~/.gemini/GEMINI.md` and `~/.gemini/trs.md` for the output-saver
  side; only the *hooks* are jetski-specific (see Antigravity section
  below).
- **Attribution label.** What `trs stats --by-agent` shows for runs
  triggered by this agent. `(untagged)` means trs has no
  programmatic signal to identify the agent — rules-only agents fall
  here, as do direct shell invocations.
- **Scope.** Whether the integration works globally (`~/.claude/`,
  etc.) or only per-project (`./AGENTS.md`, etc.). Project-only means
  every repo needs its own `trs init` inside it.

## Per-agent detail

### Claude Code

- **Hook wire format:** `hook_event_name: PreToolUse` envelope over
  stdin; trs replies with a modified `command` field.
- **Config path:** `~/.claude/settings.json` (global) or
  `.claude/settings.json` (project).
- **Output-saver:** separate file at `~/.claude/trs-output-saver.md`
  with an `@import` line added to `CLAUDE.md`.
- **Typical install:** `trs init claude --global`.

### Gemini CLI

- **Hook wire format:** `hook_event_name: BeforeTool`; same response
  shape as Claude.
- **Config path:** `~/.gemini/settings.json`.
- **Output-saver:** separate file + `@import`, mirroring Claude.

### Cursor

- **Hook wire format:** `hook_event_name: preToolUse` (camelCase vs
  Claude's PascalCase).
- **Config path:** `~/.cursor/hooks.json`.
- **Output-saver:** `.cursor/rules/trs.mdc` — Cursor auto-loads `.mdc`
  files from the rules dir, no explicit import needed.
- **Why `preToolUse`:** it's the only Cursor hook that can rewrite the
  command (via the `updated_input` field); `beforeShellExecution` can
  only allow/deny, not modify.

### OpenCode

- **Install mechanism:** a plugin template rather than a JSON hook.
  The plugin calls `trs rewrite` with the command before exec.
- **Scope:** global only (OpenCode plugin config is per-install).

### Kilo Code

- **Install mechanism:** plugin template, symmetric to OpenCode.
- **Scope:** global only.

### Pi Coding Agent

- **Install mechanism:** a programmatic extension (TypeScript) with a
  bash `spawnHook` that rewrites the command and sets env before exec —
  same tier as Claude/Cursor/OpenCode/Kilo, not rules-only.
- **Config path:** `~/.pi/agent/extensions/trs.ts` (global) or
  `.pi/extensions/trs.ts` (project).
- **Attribution:** `pi` — the extension's env carries `TRS_AGENT=pi`.
- **Typical install:** `trs init pi` (aliases: `pi`, `pi.dev`,
  `pidev`). Upstream: pi.dev (repo `earendil-works/pi`).

### Factory Droid

- **Attribution.** Droid reuses Claude's `hook_event_name: PreToolUse`
  wire format verbatim; the hook command carries `--caller droid` so
  runs show up as `droid` in `trs stats --by-agent`. Installs from
  before v0.6.16 shared Claude's label — re-run `trs init droid` to
  pick up the labeled hook.

### Antigravity IDE + Antigravity CLI (`agy`)

- **Status (v0.6.6).** **Rules-only**, same as Codex/Devin Desktop. There is
  no programmatic auto-rewriting of Bash commands until Google ships
  user-configurable PreToolHooks upstream. Full investigation:
  [`docs/development/antigravity-hooks-research.md`](../development/antigravity-hooks-research.md).
- **Why.** Empirical testing against agy v1.0.1 showed that user-defined
  entries in `~/.gemini/antigravity-{cli,ide}/hooks.json` load as
  **subagents** (the `name`+`description` fields are required by
  `JSONHookSpec` with jsonschema_description "Used to invoke it via
  `invoke_subagent`"). Bash invocations go through
  `*gemini_coder_go_proto.Step_RunCommand`, which bypasses the
  user-visible PreToolHook pipeline. Five different hook schemas were
  tested with side-channel probes — none fired.
- **Install mechanism.** `trs init antigravity` (or `agy` /
  `antigravity-cli`) appends a sentinel-wrapped rules block to
  `~/.gemini/GEMINI.md`. Both IDE and CLI read that file at session
  start. The block tells the agent to prefix shell commands with
  `trs` when token-optimized output is desired.
- **Output-saver.** Unchanged — `~/.gemini/trs.md` + the `@trs.md`
  import in `~/.gemini/GEMINI.md` keeps the anti-preamble / numeric
  budget rules active for both Antigravity surfaces. This part of the
  integration **does** work because it's LLM-side (prompt context),
  not runtime hooks.
- **Aliases.** `trs init antigravity` → IDE (back-compat).
  `trs init antigravity-cli` or `trs init agy` → CLI. `trs init --show`
  lists both rows separately so you can see what's detected.
- **Attribution.** `(untagged)` in `trs stats --by-agent` — we have no
  programmatic signal (no hook ever fires for Antigravity-launched
  commands). Same posture as Codex/Devin Desktop. When the user prefixes
  `trs git status` manually, those runs also land in `(untagged)`.
- **Migration cleanup.** `trs init` and `trs uninstall` both sweep the
  inert artifacts from previous releases:
  - v0.6.5 `hooks.json` at `~/.gemini/antigravity-{cli,ide}/hooks.json`
    (jetski PreToolUse — never actually fired)
  - v0.6.4 BeforeTool entry in `~/.gemini/settings.json` (aliased to
    Gemini's harness — also never fired)
  - Pre-v0.6.4 `.agent/rules/antigravity-trs-rules.md` per-project
    rules file
- **Re-enabling the hook.** When Google ships user-configurable
  PreToolHooks for Bash, restoring the programmatic integration is a
  mechanical revert of branch `fix/antigravity-rules-only-revert` — see
  the [research doc](../development/antigravity-hooks-research.md#what-unblocks-re-enabling-the-integration).

### Codex CLI

- **Install mechanism:** version-gated. On **codex-cli ≥ 0.134**
  (which implements `hookSpecificOutput.updatedInput.command` in its
  `PreToolUse` hook), `trs init codex --global` merges a real
  `PreToolUse` hook (matcher `"Bash"`, command
  `trs rewrite --caller codex`) into `~/.codex/hooks.json`, preserving
  the user's other hooks. Approve it once via Codex's `/hooks` prompt
  and commands rewrite automatically. On older builds (or an untrusted
  hook) it falls back to a rules block in `~/.codex/AGENTS.md`
  recommending manual `trs <cmd>` prefixes.
- **Caveat:** `codex exec` (non-interactive) doesn't dispatch
  `PreToolUse`; the hook fires in interactive sessions only.
- **Attribution:** `codex` when the hook is active (the hook command
  carries `TRS_AGENT=codex`); the rules-only fallback shows as
  `(untagged)`.

### Devin Desktop (ex-Windsurf)

- **Background:** Devin Desktop is Cognition's 2026-06-02 rebrand of
  Windsurf. Its old agent engine "Cascade" was replaced by "Devin
  Local" (Rust rewrite, subagents, ACP support). Cascade reaches
  end-of-life 2026-07-01.
- **Install mechanism:** rules file only — Devin Local exposes no
  shell hook / plugin API. `trs init` appends a rules block
  recommending manual `trs <cmd>` prefixes; the agent reads the rules
  at session start but there's no enforcement.
- **Target file:** `.devin/rules/trs.md` (a directory-rule file with
  YAML frontmatter `trigger: always_on`) when Devin Desktop is
  detected; legacy `.windsurfrules` (plain file, no frontmatter)
  otherwise. Devin reads both, so trs writes only one to avoid
  double-loading; uninstall removes both.
- **CLI names:** primary `devin`; aliases `devin-desktop`, `windsurf`,
  `cascade` (the last two for back-compat).
- **Attribution:** `(untagged)` in stats since there's no programmatic
  signal to tag commands with an agent.

### Devin CLI

- **Background:** "Devin for Terminal" (Devin CLI, binary `devin`, by
  Cognition) — a distinct product from Devin Desktop. Unlike the
  Desktop rules-only integration, the CLI exposes real programmatic
  `PreToolUse` hooks, so trs wires a deterministic rewrite hook here.
- **Install mechanism:** `trs init devin-cli --global` merges a hook
  into `~/.config/devin/config.json` under the `hooks` key, preserving
  the user's existing config (model, org_id, theme). Devin's shell tool
  is named `exec` (not `Bash`), so the hook matcher is `exec` and the
  hook command is `trs rewrite --caller devin-cli`.
- **Target file:** `~/.config/devin/config.json` (global) or
  `.devin/config.json` (project).
- **CLI names:** primary `devin-cli`; aliases `devin-terminal`, `dcli`.
- **Attribution:** `devin-cli` — the hook command carries
  `--caller devin-cli`.
- **updatedInput — validated live (2026-07-07):** Devin honors
  `hookSpecificOutput.updatedInput`; commands execute rewritten as
  `trs …`. (Devin's docs only document `decision` + `additionalContext`,
  but the rewrite works in practice.)
- **Attribution gotcha:** `--caller devin-cli` only tags correctly when
  `devin-cli` is whitelisted in `known_agent_label` (rewrite.rs);
  otherwise it silently falls back to `claude`. Regression-guarded by a
  test in rewrite.rs.
- **`.claude` interplay:** Devin reads `.claude/settings.json` hooks by
  default (`read_config_from.claude: true`). With `trs init claude`
  present, that transitive Claude hook fires and tags runs `claude` — the
  same de-facto-coverage pattern as VS Code. Set
  `read_config_from.claude: false` in `~/.config/devin/config.json` so the
  dedicated `devin-cli` hook wins and attribution is correct.

### VS Code Copilot

- **Status:** programmatic hook via VS Code's **agent hooks
  (preview)**, which speak Claude Code's `PreToolUse` envelope —
  including the `hookSpecificOutput.updatedInput` rewrite that trs
  relies on. Validated live 2026-06-09.
- **Config path:** `trs init vscode` writes `.github/hooks/trs.json`
  (project) or `~/.copilot/hooks/trs.json` (`--global`). The hook
  command is `trs rewrite --caller vscode`.
- **Prerequisite:** enable VS Code's agent hooks (preview feature)
  for the hook to fire.
- **Claude-settings interplay:** the related setting **"Chat: Use
  Claude Hooks"** makes VS Code *also* load `~/.claude/settings.json`
  hooks — users with `trs init claude --global` get de-facto coverage
  that way, but runs are attributed as `claude`. The dedicated
  `trs init vscode` gives correct `vscode` attribution and works
  without Claude Code installed. If both surfaces fire, the
  double-fire is harmless — `trs rewrite` is idempotent.
- **Matcher caveat:** VS Code parses but **ignores** hook matchers,
  so the hook fires on every tool. Safe in practice: trs no-ops on
  tools without a `command` field, and unknown event names fail open.
- **Fail-closed caveat (version skew):** VS Code **blocks the terminal
  tool** when a hook command errors ("blocked by prehook"). The
  `--caller` flag requires **trs ≥ 0.6.16** — an older binary exits
  with a clap usage error and every shell run gets blocked. Relevant
  when `.github/hooks/trs.json` is committed to a shared repo:
  teammates need trs ≥ 0.6.16, or the hook should use plain
  `trs rewrite` until everyone upgrades.
- **Output-saver:** not yet wired (same posture as Pi).
- **Aliases:** `vscode` (primary), `vs-code`, `copilot`,
  `vscode-copilot`, `code`.

### OpenClaw

- **Status:** shipped pending live validation — install and run
  `git status`, then check `trs stats --by-agent`.
- **Install mechanism:** JS plugin at `~/.openclaw/plugins/trs/`
  (`openclaw.plugin.json` manifest + `index.js`). The plugin's
  `before_tool_call` hook prepends `trs ` to `exec` commands
  (idempotent), and `resolve_exec_env` injects `TRS_AGENT=openclaw`
  into the exec environment — cross-platform attribution, no shell
  prefix.
- **Config enable:** `trs init openclaw` also merges
  `plugins.entries.trs.enabled = true` and the plugin dir into
  `plugins.load.paths` in `~/.openclaw/openclaw.json` (everything
  else preserved; idempotent).
- **Scope:** global only — OpenClaw plugins live under the gateway's
  home dir. Restart the gateway after install:
  `openclaw gateway restart`.
- **Uninstall:** removes the plugin files; the now-inert
  `plugins.entries.trs` config entry can be removed manually.
- **Aliases:** `openclaw` (primary), `claw`.

### Hermes

- **Status:** shipped pending live validation — install and run
  `git status`, then check `trs stats --by-agent`.
- **Install mechanism:** Python plugin at
  `~/.hermes/plugins/trs-rewrite/` (`__init__.py` + `plugin.yaml`
  manifest) for NousResearch's hermes-agent. The `pre_tool_call`
  hook prepends `trs ` to `terminal` tool commands (idempotent,
  fails open) and exports `TRS_AGENT=hermes` for attribution.
- **Config enable:** `trs init hermes` adds `trs-rewrite` to
  `plugins.enabled` in `~/.hermes/config.yaml`. The YAML patch is
  conservative — block-style lists are patched in place; exotic
  layouts (inline arrays, `plugins` without `enabled`) get a manual
  instruction instead.
- **Home override:** the `HERMES_HOME` env var relocates the Hermes
  home dir (default `~/.hermes`) for both install and uninstall.
- **Scope:** global only. Restart Hermes after install.
- **Uninstall:** removes the plugin files; the `trs-rewrite` entry
  in `plugins.enabled` can be removed manually.
- **Aliases:** `hermes` (primary), `hermes-agent`.

### Zed (Agent Panel)

- **Status:** rules-only. Zed's native agent exposes no tool hooks —
  the feature request is open upstream
  ([zed-industries/zed#52688](https://github.com/zed-industries/zed/issues/52688)).
  Until it ships, there is no programmatic rewrite surface.
- **Install mechanism:** Zed's native agent reads the project
  `AGENTS.md` as always-on instructions, so `trs init zed` writes the
  same trs sentinel block Codex uses (shared template, shared
  sentinel scrub on uninstall). Project scope only — `--global`
  prints a note and writes nothing (Zed's global personal-instructions
  location is not yet verified).
- **IMPORTANT — external agents via ACP:** running Claude Code,
  Codex CLI, Gemini CLI, or OpenCode inside Zed's Agent Panel (from
  the ACP registry) runs the real CLIs as ACP servers. Those agents'
  existing trs hooks fire transitively — no extra setup — and
  `trs stats --by-agent` attributes runs to the backend agent
  (`claude`, `codex`, `gemini`, `opencode`), not to Zed.
- **Roadmap:** ACP-level interception (covering the native agent
  programmatically) is tracked separately under Research in
  [`docs/roadmap/TASK_TODO.md`](../roadmap/TASK_TODO.md).
- **Aliases:** `zed` (primary), `zed-ide`.

## Install commands

```bash
trs init --show                       # who has what installed
trs init --all --global               # install for every detected agent
trs init claude --global              # single agent
trs init claude --global --replace    # migrate from rtk / token-optimizer
trs output-saver --install            # output-saver block on every detected agent
trs output-saver --remove             # clean uninstall
```

Before any write, `trs init` runs a **pre-install collision check**:
it scans the target config (following `@imports` for Claude/Gemini)
for existing competing compressor hooks and aborts by default.
`--replace` scrubs the previous compressor's hook cleanly before
installing trs; `--force` installs alongside (risky —
double-compression).

See also:
- [`docs/features/init.md`](../features/init.md) — `trs init`
  reference (flags, hooks, merge behavior).
- [`docs/features/output-saver.md`](../features/output-saver.md) —
  `trs output-saver` reference (sentinels, idempotence, check-first).
- [`docs/development/agent-integrations.md`](../development/agent-integrations.md)
  — internals: per-agent file layout, merge paths, known quirks.
- [`docs/support/other-token-savers.md`](./other-token-savers.md) —
  alternatives trs coexists with.
