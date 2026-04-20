# `trs output-saver` — reduce tokens on the agent's replies

`trs rewrite` (wired up by [`trs init`](init.md)) compresses what
agents **see** — the output of the shell commands they run. Agents
still **emit** verbose replies: preambles ("Sure!"), narration
("Now I will…"), speculative suggestions, hallucinated file paths.

`trs output-saver` installs a short rules block into each supported
agent's global config so those replies come back tighter.

## Quick reference

```bash
trs output-saver                 # read-only scan of all 9 agents
trs output-saver --install       # write to every detected agent
trs output-saver <agent> --install  # scope to one
trs output-saver --remove        # clean uninstall
trs output-saver --print         # dump the block to stdout (pipe-friendly)
```

## What the block says

Six directives, roughly 200 tokens total:

- **No preambles.** Explicit blocklist: "Sure!", "Great question!",
  "Absolutely!", "I'll help you…", "You're absolutely right!".
- **No narration.** Don't announce what's about to happen or recap
  what just happened — the diff / tool output shows it.
- **Result first; explanation only if non-obvious.** State the
  finding, show the fix, stop.
- **Structured output when the data is structured.** Bullets, tables,
  JSON — prose only when the reader is human and the content is
  narrative.
- **Never invent file paths, function names, or API fields.** If
  unknown, return `UNKNOWN` or `null` — guessing costs more tokens
  than asking.
- **One pass.** Don't iterate on passing code, don't refactor or
  polish unless asked.

Plus an explicit user-override clause so the rules never fight a
user's deliberate instructions.

Run `trs output-saver --print` to see the exact text before
installing.

## Coverage matrix

| Agent | Mechanism | Path |
|---|---|---|
| Claude Code | Standalone file + `@import` | `~/.claude/trs-output-saver.md` + line in `~/.claude/CLAUDE.md` |
| Gemini CLI | Standalone file + `@import` | `~/.gemini/trs-output-saver.md` + line in `~/.gemini/GEMINI.md` |
| Cursor | Auto-loaded rules file | `~/.cursor/rules/trs-output-saver.mdc` |
| Codex | Inline with sentinels | `~/.codex/AGENTS.md` |
| Windsurf | Inline with sentinels | `~/.codeium/windsurf/memories/global_rules.md` |
| OpenCode | Inline with sentinels | `~/.config/opencode/AGENTS.md` |
| Kilo Code | Inline with sentinels | `~/.config/kilo/AGENTS.md` |
| Factory Droid | Inline with sentinels | `~/.factory/AGENTS.md` |
| Antigravity | — | not supported globally; use `trs init antigravity` per project |

Codex, OpenCode, Kilo, and Droid are signatories of the
[`AGENTS.md` convention](https://factory.ai/news/agents-md), which is
why they share the same install mechanism.

## How the install is idempotent

Inline installs wrap the block in HTML comment sentinels:

```
<!-- trs:output-saver:start v1 -->
## Output saver — keep replies cheap
…
<!-- trs:output-saver:end -->
```

Running `--install` again detects the sentinels and replaces the
content between them — no duplication, no accidental user-content
loss. The sentinel carries a version tag (`v1`) so we can migrate
block content in future releases without breaking detection.

The `@import` mechanism for Claude/Gemini is naturally idempotent:
re-install overwrites `trs-output-saver.md` and re-adds the import
line only if missing.

## Check-first semantics

A bare `trs output-saver` never writes. It reports what install would
do for each agent and prints the exact commands to commit or remove
the block. This mirrors `trs audit-docs` and `trs doctor` — nothing
destructive happens without an explicit flag.

Sample output:

```
trs output-saver — scan

  + Claude Code  already installed
  . Gemini CLI   not yet installed
  - Cursor       not detected on this system
  + Codex        already installed
  ~ Antigravity  skipped (rules are per-project only — run `trs init antigravity`)

  1 installable, 2 already installed, 1 not detected, 1 unsupported
```

## `--refresh` — pick up template changes without adding new installs

```bash
trs output-saver --refresh
```

Re-installs the block **only** where `trs output-saver` already
reports `AlreadyInstalled`. Agents that never had the block are
skipped silently. This is the mode `trs upgrade` calls automatically
to refresh templates without surprising you by adding rules to agents
you deliberately didn't opt into.

## `--remove` behavior

For `@import` and `RulesDir` installs, `--remove` deletes the
standalone file and strips the import line from the parent config.
For `InlineFile` installs, the content between the sentinels is
removed along with the sentinels themselves — surrounding user
content is preserved exactly.

## Measuring impact

The block doesn't compress existing output; it steers what gets
generated. Exact savings depend on your agent, model, and prompts.
Anecdotally: ~30-40% fewer tokens on simple tasks (where preambles
are proportionally the bulk of replies), less on long reasoning
replies where the compression target is narration and speculation.

The `trs stats` dashboard tracks input-side savings from `trs rewrite`
but cannot measure output-side savings — they happen outside any
process we run. If you want to measure, compare before/after
token-usage numbers in your agent's own dashboard.

## Interaction with `trs init`

They're independent. You can have the input-side hooks installed
(`trs init`) without the output-saver, and vice versa. Both are
additive — no conflicts between them.

## See also

- [`trs init`](init.md) — the input-side compression via hooks.
- [`trs audit-docs`](audit-docs.md) — audit CLAUDE.md / AGENTS.md
  files for bloat; pairs well with output-saver.
- [`trs doctor`](doctor.md) — reports output-saver coverage alongside
  hook coverage.
