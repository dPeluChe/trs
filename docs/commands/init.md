# `trs init` — install hooks for AI agents

`trs init` wires your AI coding agent's shell-execution pipeline through
`trs rewrite` so every command gets compressed automatically. Nine
agents are supported end-to-end. See [`docs/agent-integrations.md`](../agent-integrations.md)
for the full per-agent reference.

## Quick reference

```bash
trs init --show                      # status of all 9 agents
trs init --all --global              # install for every detected agent
trs init <agent>                     # install for one: claude, gemini, cursor, …
trs init --all --global --force      # refresh templates (see "Refreshing hooks")
trs init <agent> --replace           # migrate cleanly from another compressor
```

## What gets installed where

| Agent | Type | Target |
|---|---|---|
| Claude Code | JSON hook | `~/.claude/settings.json` (or `~/.claude/hooks.json`) |
| Gemini CLI | JSON hook | `~/.gemini/settings.json` |
| Cursor | JSON hook | `~/.cursor/hooks.json` |
| Factory Droid | JSON hook | `~/.factory/settings.json` |
| OpenCode | TS plugin | `~/.config/opencode/plugins/trs.ts` |
| Kilo Code | TS plugin | `~/.config/kilo/plugins/trs.ts` |
| Codex | Rules append | `AGENTS.md` in repo |
| Google Antigravity | Rules file | `.agent/rules/antigravity-trs-rules.md` |
| Windsurf | Rules file | `.windsurfrules` |

Hooks fire deterministically on every shell-tool invocation. Rules
files are probabilistic — they only work because the agent chooses to
follow the guidance in them.

## Collision handling

Before writing, `trs init` scans the target config for hooks installed
by another shell-compression tool. Two competitors are detected today:

- **rtk** (Rust Token Killer): `rtk rewrite`, `rtk proxy`, `rtk git`
- **token-optimizer**: `token-optimizer`, `tokopt`

The scan follows `@imports` inside Claude/Gemini CLAUDE.md /
GEMINI.md up to depth 3, so a competitor installed via
`@RTK.md` gets detected too. It runs symmetrically over home and
project paths regardless of the `--global` flag.

Why this matters: when two compressors both fire on the same command,
the second one parses the first one's compressed output. The results
range from inaccurate token counts to garbled context the agent can't
read. The hook layer reports success either way, so failures are
silent.

Default behaviour on detection: **abort** with a report of every
location flagged and the options below.

### `--replace` (recommended for migrations)

Scrubs the competitor's hook entries from each JSON file we detected,
then installs trs. Rules-file collisions (e.g. a `CLAUDE.md` that
describes rtk in prose) are flagged but not edited — rewriting
someone's markdown is out of scope.

```bash
trs init claude --replace
trs init --all --global --replace
```

### `--force` (not recommended)

Installs trs alongside the competitor. Both hooks stay active. Only
useful if you have a specific reason to double up temporarily (e.g.
shadow-testing). Documented risk: double-compression can corrupt
command output.

### The abort message

```
Found 1 potential collision(s) while preparing Claude Code install:

  ! /Users/you/.claude/RTK.md — references 'RTK - Rust Token Killer'
    file references another compressor (RTK - Rust Token Killer)

Risk: running two shell-compression tools on the same
command can double-compress and corrupt output.

Recommended: migrate to trs.
  trs init <tool> --replace   remove competitor hooks, install trs   [recommended]
  trs init <tool> --force     install alongside                      [risky]
  abort                       (default) fix the collisions manually
```

## Refreshing hooks

Templates evolve between trs releases. When `trs init --all` reports
`N already configured, 0 installed`, your hooks are written with an
older template — fine in most cases, but you might be missing
improvements (e.g. a widened matcher, a richer rules file, a new
agent the template now supports).

Re-run with `--force` to overwrite with the current template. The
JSON merge preserves any user-added hooks on the same event (lint
runners, analytics, notify scripts); only trs's own entries are
replaced.

```bash
trs init --all --global --force
```

## `--global` vs project-local

`--global` writes to the agent's home-dir config (`~/.claude/`,
`~/.gemini/`, …). Project-local installs (no flag) write to the
current working directory (`.gemini/settings.json`, etc.).

Most users want `--global` — it works across every project. Project-
local is useful for repos where you want trs's behavior isolated to
that checkout, or for CI that needs a pinned local config.

## Uninstalling

There's no dedicated uninstaller yet. Manual steps:

- JSON hooks: edit `settings.json` and remove the PreToolUse entry
  whose `command` is `trs rewrite`.
- Plugins: delete `~/.config/opencode/plugins/trs.ts` and
  `~/.config/kilo/plugins/trs.ts`.
- Rules files: delete or edit `.agent/rules/antigravity-trs-rules.md`
  and `.windsurfrules`.

## See also

- [`trs output-saver`](output-saver.md) — the symmetric feature for
  LLM-generated output.
- [`trs doctor`](doctor.md) — health check that reports install
  coverage.
- [`docs/agent-integrations.md`](../agent-integrations.md) — deep per-
  agent reference (wire formats, protocol quirks, test prompts).
