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
| Antigravity IDE | programmatic hook (shared) | ✓ | ✓ (`@import`) | `gemini` (see caveat) | global |
| Antigravity CLI (`agy`) | programmatic hook (shared) | ✓ | ✓ (`@import`) | `gemini` (see caveat) | global |
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
  supported as of v0.6.4 (Antigravity moved off its per-project rules
  files and joined the Gemini CLI harness).
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

- **Install mechanism:** programmatic hook into `~/.gemini/settings.json`
  under `hooks.BeforeTool[]`. Both products share the Gemini CLI agent
  harness — same file, same envelope — so installing Gemini, Antigravity
  IDE, or Antigravity CLI all converge on the same hook entry.
  `merge_json_hook` is idempotent on `trs rewrite`, so installing all
  three doesn't double-fire.
- **Aliases:** `trs init antigravity` resolves to the IDE for
  back-compat. Use `trs init antigravity-cli` or `trs init agy` for
  the new terminal CLI explicitly. `trs init --show` lists both rows
  separately so you can see what's detected.
- **Caveat — shared `gemini` attribution.** Because the hook lives in
  Gemini's `settings.json`, every invocation tags as `gemini` in
  `trs stats --by-agent`. We can't disambiguate Gemini vs Antigravity
  IDE vs Antigravity CLI at hook time; same disambiguation roadmap
  item as Droid/Claude.
- **Migration from pre-v0.6.4:** old installs wrote
  `.agent/rules/antigravity-trs-rules.md` per project. The IDE no
  longer reads that path. `trs uninstall antigravity` sweeps it up
  automatically.

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
