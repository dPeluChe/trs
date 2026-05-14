# `trs init` — install hooks for AI agents

`trs init` wires your AI coding agent's shell-execution pipeline through
`trs rewrite` so every command gets compressed automatically. Nine
agents are supported end-to-end. See [`docs/development/agent-integrations.md`](../development/agent-integrations.md)
for the full per-agent reference.

## Quick reference

```bash
trs init --show                      # status of all 9 agents
trs init --all --global              # install for every detected agent
trs init <agent>                     # install for one: claude, gemini, cursor, …
trs init --all --global --dry-run    # preview every file that would change
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
| Codex | Rules append | `AGENTS.md` in repo (or `~/.codex/AGENTS.md` with `--global`) |
| Google Antigravity | Rules file | `.agent/rules/antigravity-trs-rules.md` |
| Windsurf | Rules file | `.windsurfrules` |

Hooks fire deterministically on every shell-tool invocation. Rules
files are probabilistic — they only work because the agent chooses to
follow the guidance in them.

Codex sits in the rules-only column on purpose: its `PreToolUse` hook
schema accepts but doesn't implement `updatedInput`, so trs can't
rewrite shell commands from a hook today. The AGENTS.md block tells
the model to prefix shell commands with `trs` instead.

## Preview with `--dry-run`

`--dry-run` lists every file that would change without writing
anything. Useful before installing globally on a machine that already
has other tooling in those configs.

```bash
trs init --all --global --dry-run
trs init codex --global --dry-run
```

Each line shows the target path and the planned action (`would
create`, `would merge trs hook entries into`, `already configured`).
Re-run without `--dry-run` to apply.

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

## Bypassing the hook for one command

Sometimes you want the agent to get raw command output — to diff
exact bytes, pipe into `sha256sum`, or assert on an unmodified shell
response. Prefix the command with `TRS_SKIP=1` and `trs rewrite`
will pass it through unchanged:

```bash
TRS_SKIP=1 git log --pretty=format:'%H %s'
TRS_SKIP=1 cargo test -- --nocapture
```

The env-var assignment stays in the command string; the shell strips
it before executing the downstream program, so the bypass is
transparent to git / cargo / whatever. Any value after `=` works —
we only check for the `TRS_SKIP=` prefix.

No global toggle: the bypass is always per-invocation. Removing trs
entirely is done via uninstall (below), not via an always-skip flag.

## Agent attribution (`TRS_AGENT`)

When `trs rewrite` or a plugin template rewrites a command, it
prefixes the result with `TRS_AGENT=<label>` so the downstream
`trs <cmd>` execution can record which agent triggered the run.

The shell strips the env-var assignment before executing git / cargo
/ etc. — so the tagging is transparent to downstream programs — and
trs's tracker picks up the label and writes it into
`~/.trs/history.jsonl`. Run `trs stats --by-agent` to see the
breakdown.

Labels per agent:

- `claude` — Claude Code (and Factory Droid, which shares the same
  wire format)
- `gemini` — Gemini CLI
- `cursor` — Cursor
- `opencode` — OpenCode (baked into the plugin template)
- `kilo` — Kilo Code (baked into its plugin template)
- `(untagged)` — rules-only agents (Codex / Antigravity / Windsurf)
  and direct-shell invocations, where no programmatic signal is
  available

If you want to spoof attribution for a specific command (e.g.
testing), prefix manually:

```bash
TRS_AGENT=claude trs git status
```

`TRS_AGENT` is only read by trs — it has no other effect on the
environment.

## Uninstalling

Use [`trs uninstall`](uninstall.md). It's symmetric to `trs init` —
interactive by default, or `--all` / `--tool <x>` / `--output-saver` /
`--dry-run` for scripted removal. JSON hooks get scrubbed (user-added
entries on the same event survive), plugin files are deleted, and the
Codex sentinel block in `AGENTS.md` is removed in place. See the
`trs uninstall` doc for the per-surface details.

## See also

- [`trs uninstall`](uninstall.md) — symmetric removal command.
- [`trs output-saver`](output-saver.md) — the symmetric feature for
  LLM-generated output.
- [`trs doctor`](doctor.md) — health check that reports install
  coverage.
- [`docs/development/agent-integrations.md`](../development/agent-integrations.md) — deep per-
  agent reference (wire formats, protocol quirks, test prompts).
