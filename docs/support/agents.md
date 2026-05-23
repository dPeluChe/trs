# Supported AI agents

Ten AI coding agents are supported end-to-end. Each row lists the
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
| Antigravity IDE | rules file only (see [research notes](../development/antigravity-hooks-research.md)) | — | ✓ (`@import`) | `(untagged)` | global |
| Antigravity CLI (`agy`) | rules file only (see [research notes](../development/antigravity-hooks-research.md)) | — | ✓ (`@import`) | `(untagged)` | global |
| Codex CLI | rules file only | — | ✓ (inline block) | `(untagged)` | global + project |
| Windsurf | rules file only | — | ✓ (inline block) | `(untagged)` | global + project |

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
  the anti-preamble / result-first rules block. All ten agents are
  supported. Antigravity 2.0 (IDE + CLI) shares Gemini's
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

### OpenCode

- **Install mechanism:** a plugin template rather than a JSON hook.
  The plugin calls `trs rewrite` with the command before exec.
- **Scope:** global only (OpenCode plugin config is per-install).

### Kilo Code

- **Install mechanism:** plugin template, symmetric to OpenCode.
- **Scope:** global only.

### Factory Droid

- **Caveat — shared Claude envelope.** Droid reuses Claude's
  `hook_event_name: PreToolUse` wire format verbatim, so trs can't
  distinguish the two at rewrite time. Both show up as `claude` in
  `trs stats --by-agent`. To separate them you currently need to
  eyeball the `cwd` paths or the time of day. A disambiguation flag
  is tracked on the roadmap.

### Antigravity IDE + Antigravity CLI (`agy`)

- **Status (v0.6.6).** **Rules-only**, same as Codex/Windsurf. There is
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
  commands). Same posture as Codex/Windsurf. When the user prefixes
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

### Codex CLI, Windsurf

- **Install mechanism:** rules file only — these agents have no
  programmatic hook surface. `trs init` appends a rules block
  recommending manual `trs <cmd>` prefixes; the agent reads the rules
  at session start but there's no enforcement.
- **Attribution:** both show as `(untagged)` in stats since there's
  no programmatic signal to tag commands with an agent.

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
