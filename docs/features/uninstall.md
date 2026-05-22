# `trs uninstall` — remove trs from agent configs

`trs uninstall` is the inverse of [`trs init`](init.md). It walks every
surface `trs init` (and `trs output-saver`) wrote to and cleans up:
JSON hook entries, plugin files, sentinel-delimited rules blocks, and
the output-saver sidecar / `@import` line.

## Quick reference

```bash
trs uninstall                        # interactive — lists installed agents
trs uninstall <agent>                # one agent (claude, codex, gemini, …)
trs uninstall --all                  # every agent, with confirmation
trs uninstall --all --yes            # no prompt — for scripts / CI
trs uninstall --output-saver         # only the output-saver block
trs uninstall --dry-run              # preview without writing
```

## What gets removed per surface

| Surface | Action |
|---|---|
| JSON hooks (Claude / Gemini / Cursor / Factory Droid / Antigravity IDE+CLI) | scrub entries whose `command` contains `trs rewrite` |
| Plugin files (OpenCode / Kilo) | delete `~/.config/<tool>/plugins/trs.ts` |
| Codex AGENTS.md | remove the sentinel-delimited block; delete the file if it was just our block |
| Codex legacy `hooks.json` | scrub orphan `trs rewrite` entries from `~/.codex/hooks.json` (pre-v0.6.x installs added a PreToolUse hook; Codex versions vary in `updatedInput` support, so orphans break tool calls) |
| Windsurf rules file | delete `.windsurfrules` when it carries the trs marker |
| Legacy Antigravity rules | delete pre-v0.6.4 `.agent/rules/antigravity-trs-rules.md` if present |
| Legacy Antigravity `BeforeTool` | scrub orphan `BeforeTool → trs rewrite` from `~/.gemini/settings.json` (v0.6.4 wrongly aliased Antigravity to the Gemini harness; v0.6.5+ uses jetski `PreToolUse` in `~/.gemini/antigravity-{ide,cli}/hooks.json` instead) |
| Output-saver block (Imported agents) | remove the sidecar `trs.md` and the `@trs.md` import line |
| Output-saver block (inline agents) | remove the `<!-- trs:output-saver:start v1 -->` … `:end -->` block in place |

## Interactive mode

Run `trs uninstall` with no arguments. It scans both global
(`~/.tool/`) and project-local (`./.tool/`) paths and lists only the
agents with trs artifacts present:

```
trs uninstall — interactive

Installed:
  [1] Claude Code          hooks → ~/.claude/settings.json
  [2] Gemini CLI           hooks → ~/.gemini/settings.json
  [3] OpenCode             plugin → .opencode/plugins/trs.ts
  [s] output-saver blocks only (preserve hooks)
  [a] all of the above
  [q] quit

Pick (e.g. 1,3 or 'a'):
```

Type a comma-separated list of indices, `a` for everything,
`s` for output-saver only, or `q` to abort.

## `--dry-run`

Lists every file that would change and the action taken (`would scrub`,
`would delete`, `would remove trs block from`) without touching the
filesystem. Useful when uninstalling on a machine you can't easily
restore.

```bash
$ trs uninstall --all --dry-run --yes
trs would remove from Claude Code:
  - would scrub 1 trs entry/entries from /Users/you/.claude/hooks.json
  - would remove output-saver block (claude)
trs would remove from Codex:
  - would remove trs block from /Users/you/.codex/AGENTS.md
note: dry-run — nothing was written. Re-run without --dry-run to apply.
```

## Preserving user-added hooks

The JSON scrub is scoped to entries whose `command` field references
`trs rewrite`. User-added hooks on the same event (e.g. a `notify.sh`
on `SessionStart`, lint runners, analytics) survive untouched. If your
`~/.claude/settings.json` contained:

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "trs rewrite" }] },
      { "matcher": "*", "hooks": [{ "type": "command", "command": "notify.sh" }] }
    ]
  }
}
```

`trs uninstall claude` leaves only the `notify.sh` entry behind.

## Sentinel-based detection

For Codex's `AGENTS.md` and the inline output-saver block,
`trs uninstall` keys on the start sentinel `<!-- trs:codex-rules:start
v1 -->` / `<!-- trs:output-saver:start v1 -->` (see
`docs/features/init.md` and `docs/features/output-saver.md` for the
exact markers). If you've moved the block somewhere unusual but the
sentinels are intact, uninstall still finds it.

## What `trs uninstall` does NOT do

- **Doesn't uninstall the trs binary.** Drop the binary yourself
  (`rm ~/.local/bin/trs`) or use your package manager.
- **Doesn't touch `~/.trs/`** (history, config, tee logs). Delete it
  manually if you also want to drop telemetry.
- **Doesn't undo `TRS_AGENT=` / `TRS_SKIP=` env vars** baked into shell
  configs. Search your `.zshrc` / `.bashrc` if you added them.

## See also

- [`trs init`](init.md) — install hooks for AI agents.
- [`trs output-saver`](output-saver.md) — the symmetric feature for
  LLM-generated output (its `--remove` is bundled into
  `trs uninstall --output-saver`).
- [`trs doctor`](doctor.md) — health check; reports install coverage.
