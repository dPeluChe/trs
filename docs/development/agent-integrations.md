# AI Agent Integrations — Reference

How `trs` integrates with each supported AI coding agent. Use this doc when
adding a new agent, debugging a broken integration, or reviewing why a
specific quirk exists.

Last validated: 2026-04-19 against `trs` v0.5.7.

## Output-saver matrix

`trs output-saver` installs output-reduction rules into each agent's
global config. Orthogonal to the input-side hook/plugin/rules install
matrix below. Six distinct target paths across three mechanisms:

| Agent | Mechanism | Path |
|---|---|---|
| Claude Code | standalone file + `@import` | `~/.claude/trs-output-saver.md` + line in `~/.claude/CLAUDE.md` |
| Gemini CLI | standalone file + `@import` | `~/.gemini/trs-output-saver.md` + line in `~/.gemini/GEMINI.md` |
| Cursor | auto-loaded rules file | `~/.cursor/rules/trs-output-saver.mdc` |
| Codex | inline with sentinels | `~/.codex/AGENTS.md` |
| Windsurf | inline with sentinels | `~/.codeium/windsurf/memories/global_rules.md` |
| OpenCode | inline with sentinels | `~/.config/opencode/AGENTS.md` |
| Kilo Code | inline with sentinels | `~/.config/kilo/AGENTS.md` |
| Factory Droid | inline with sentinels | `~/.factory/AGENTS.md` |
| Antigravity IDE | `@import` (shared with Gemini) — `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` (output-saver); hooks live separately, see below |
| Antigravity CLI (`agy`) | `@import` (shared with Gemini) — `~/.gemini/trs.md` + line in `~/.gemini/GEMINI.md` (output-saver); hooks live separately, see below |

Inline installs use the sentinels
`<!-- trs:output-saver:start v1 -->` / `<!-- trs:output-saver:end -->`
so a second run replaces the block between them without touching the
surrounding user content.

**AGENTS.md convention:** Codex, OpenCode, Kilo, and Droid all auto-load
the `AGENTS.md` convention (Droid is a signatory of the AGENTS.md
consortium — see https://factory.ai/news/agents-md). That's why those
four converge on the same install mechanism.

**Plugin-level hook note:** OpenCode's plugin API has no prompt-layer
hook (`tool.execute.*` only). Kilo exposes
`experimental.chat.system.transform` and Droid exposes `SessionStart` /
`UserPromptSubmit` — both could inject rules dynamically. For a static
rules block the AGENTS.md file is simpler and less likely to break
across agent updates.

## Integration types

Agents fall into three architectural buckets. Pick the right bucket BEFORE
writing a template — the install mechanism, test strategy, and failure modes
differ.

| Type | How it works | Agents | File written |
|---|---|---|---|
| **Hook (JSON event)** | Agent fires a PreToolUse-style event, hands us the command on stdin, applies our rewrite response | Claude Code, Gemini CLI, Cursor, Factory Droid, Antigravity IDE, Antigravity CLI | `settings.json` / `hooks.json` |
| **Plugin (TypeScript)** | Agent auto-discovers `.ts` plugin files at startup and mutates tool args in-process | OpenCode, Kilo Code | `plugins/trs.ts` |
| **Rules file** | No programmatic interception. Agent reads a rules/instructions file and VOLUNTARILY prefixes `trs` | Codex, Windsurf | `AGENTS.md` / `.windsurfrules` |

Hook and plugin are deterministic (binary: fires or not). Rules-based is
probabilistic — depends on the agent choosing to follow the guidance.

## Wire-format dispatch (hook agents)

All three hook agents send `tool_input.command` on stdin but expect different
output shapes. `src/rewrite.rs::HookEvent` dispatches on the
`hook_event_name` field of the stdin JSON:

| `hook_event_name` | Client | Output envelope |
|---|---|---|
| `PreToolUse` (capital P) | Claude Code | `hookSpecificOutput.updatedInput.command` |
| `BeforeTool` | Gemini CLI | `hookSpecificOutput.tool_input.command` + top-level `decision` |
| `preToolUse` (lowercase p) | Cursor | top-level `permission` + top-level `updated_input.command` |
| *missing / unknown* | fallback | Claude format |

Claude's `PreToolUse` and Cursor's `preToolUse` differ ONLY by case. If you
change the match order or add a case-insensitive comparison, you will break
one of them silently.

Factory Droid uses Claude's Shape verbatim — `hook_event_name: "PreToolUse"`.

## Agent attribution (`TRS_AGENT`)

Added in v0.5.8. When the rewriter or a plugin template emits a
rewritten command, it prefixes it with `TRS_AGENT=<label>` so the
downstream `trs <cmd>` execution can attribute the run.

| Agent | Detection | Label |
|---|---|---|
| Claude Code | `hook_event_name: PreToolUse` | `claude` |
| Gemini CLI | `hook_event_name: BeforeTool` | `gemini` |
| Cursor | `hook_event_name: preToolUse` | `cursor` |
| OpenCode | plugin template bakes the label | `opencode` |
| Kilo Code | separate plugin template bakes the label | `kilo` |
| Factory Droid | same wire format as Claude | `claude` (indistinguishable from Claude) |
| Antigravity IDE / CLI | rules-only as of v0.6.6 ([why](antigravity-hooks-research.md)) | `(untagged)` |
| Codex / Windsurf | rules-only, no programmatic signal | `(untagged)` |

The shell treats leading `VAR=value` as a per-command env override
and strips it before executing the downstream program — so the tag
is transparent to git/cargo/etc. Read via `std::env::var("TRS_AGENT")`
at log time in `tracker::log_execution`. View results with
`trs stats --by-agent`.

**Droid limitation:** Droid's envelope is identical to Claude's at
the wire-format layer. We can't distinguish them without a separate
signal (install-time flag, marker in the hook content, etc.). Both
show up as `claude` in attribution today.

## Per-agent reference

### Claude Code

| | |
|---|---|
| Type | Hook |
| Config | `~/.claude/settings.json` (global) or `hooks/hooks.json` (project) |
| Event | `PreToolUse` with `matcher: "Bash"` |
| Template | `CLAUDE_HOOKS` |
| Test prompt | hook-based |

No known quirks. Canonical client — we treat its format as the default.

### Gemini CLI

| | |
|---|---|
| Type | Hook |
| Config | `~/.gemini/settings.json` |
| Event | `BeforeTool` with `matcher: ".*"` |
| Tool name | `run_shell_command` |
| Template | `GEMINI_HOOKS` |
| Docs | https://geminicli.com/docs/hooks/reference/ |

**Quirk**: emits a different output envelope than Claude. Early installs
copied Claude's shape and silently failed — hook fired, exit 0, command not
rewritten. Fixed in `a662c9d` by dispatching on `hook_event_name`.

**Hallucination caveat**: Gemini's chat mode sometimes fabricates plausible
command output when shell is unavailable. If the reported `trs --version`
shows a version that doesn't exist (e.g. `v0.14.13`), Gemini is inventing —
validate via `trs stats --history` from a real terminal.

### Cursor

| | |
|---|---|
| Type | Hook |
| Config | `~/.cursor/hooks.json` |
| Event | `preToolUse` with `matcher: "Shell"` |
| Tool name | `Shell` |
| Template | `CURSOR_HOOKS` |
| Docs | https://cursor.com/docs/hooks |

**Quirks**:
- `beforeShellExecution` (Cursor's other shell-ish hook) only supports
  allow/deny — no rewrite. DO NOT register there. `preToolUse` is the one
  with `updated_input` support. Fixed in `fb4bafb`.
- Cursor fires `preToolUse` for every tool (Shell, Read, Write, MCP, Task).
  Without `matcher: "Shell"`, the hook ran for hundreds of Read-tool polling
  calls per session — wasted ~4ms subprocess each. Fixed in `e797d8a`.
- Cursor matchers are case-sensitive. `"Shell"` is the Shell tool's exact
  name; `"shell"` doesn't match.

### Factory Droid

| | |
|---|---|
| Type | Hook |
| Config | `~/.factory/settings.json` |
| Event | `PreToolUse` with `matcher: ".*"` |
| Tool name | `Execute` (NOT `Bash`) |
| Template | `DROID_HOOKS` |

**Quirk**: Droid's shell tool is called `Execute`, not `Bash`. Initial
installs copied Claude's template with `matcher: "Bash"` → hook registered
but never matched. Fixed in `af5b783` by widening the matcher to `".*"`.
Our `trs rewrite` internally filters non-shell input, so the wider matcher
is safe.

Output envelope is identical to Claude Code.

### OpenCode

| | |
|---|---|
| Type | Plugin (TypeScript) |
| Config | `~/.config/opencode/plugins/trs.ts` (global) or `.opencode/plugins/trs.ts` (project) |
| Hook key | `"tool.execute.before"` (string literal, not `before_tool_call`) |
| Tool name check | `input.tool === "bash"` |
| Mutation | `output.args.command = "trs ..."` |
| Template | `OPENCODE_PLUGIN` |
| Docs | https://opencode.ai/docs/plugins/ |

**Quirks**:
- Plugin files are auto-discovered — no registration in `opencode.json`
  needed.
- Older docs showed `before_tool_call` hook with `ctx.tool` / `ctx.input`;
  that API is outdated. The current API is `"tool.execute.before"` with
  `(input, output)` arguments. Fixed in `b3d783f`.
- Global path is `~/.config/opencode/plugins/`, NOT `~/.opencode/plugins/`.
  The `~/.opencode/` directory exists for unrelated reasons (bun workspace).
- If the plugin throws during OpenCode startup, the whole TUI crashes with a
  DrizzleError on SQLite WAL init — unrelated-looking stack trace. If users
  report this, the plugin file is the likely cause; delete and retry.

### Kilo Code

| | |
|---|---|
| Type | Plugin (TypeScript) |
| Config | `~/.config/kilo/plugins/trs.ts` (global) or `.kilo/plugins/trs.ts` (project) |
| Template | `OPENCODE_PLUGIN` (shared — same plugin API) |

Kilo mirrors OpenCode's plugin architecture. Same `tool.execute.before`
hook, same auto-discovery. Our install spec treats it as an OpenCode clone.

### Codex

| | |
|---|---|
| Type | Rules file |
| Config | `AGENTS.md` (project root) |
| Template | `CODEX_AGENTS_SECTION` |

No programmatic hook available. We append a section to `AGENTS.md`
instructing Codex to prefix `trs` when the user asks for token-optimized
output. Codex is the most consistent rules-based agent — it picks up the
rule and applies it voluntarily when the prompt mentions optimization.

Validate with the rules-based test prompt (see `docs/agent-test-prompts.md`
section below).

### Antigravity IDE + Antigravity CLI

| | |
|---|---|
| Type | Rules file (was programmatic hook, reverted in v0.6.6) |
| Config (shared) | `~/.gemini/GEMINI.md` (sentinel-wrapped rules block) |
| Template | `ANTIGRAVITY_RULES_SECTION` |
| Tool name | n/a — no programmatic interception |

**v0.6.6 revert.** Antigravity (IDE + CLI/`agy`) is rules-only.
Empirical testing against agy v1.0.1 showed user-defined entries in
`~/.gemini/antigravity-{cli,ide}/hooks.json` load as **subagents**
(via `invoke_subagent`), not as tool-call wrappers. Bash invocations
go through `Step_RunCommand` and bypass the user-visible PreToolHook
pipeline entirely. The full investigation lives at
[`docs/development/antigravity-hooks-research.md`](antigravity-hooks-research.md).

**Detection**:
- IDE: `app_exists("Antigravity")` ∨ `~/.gemini/antigravity-ide/`
  exists ∨ legacy `~/.antigravity/`.
- CLI: `in_path("agy")` ∨ `~/.gemini/antigravity-cli/` exists.

**Attribution**: `(untagged)` in `trs stats --by-agent`. The hook
never fires programmatically, so there is no `TRS_AGENT` signal to
attach. Manual prefixes (`trs git status` typed by the agent or
user) also land in `(untagged)`. Same posture as Codex/Windsurf.

**Migration cleanup**. `trs init` and `trs uninstall` both sweep:
- v0.6.5 hooks.json at `~/.gemini/antigravity-{cli,ide}/hooks.json`
- v0.6.4 BeforeTool entry in `~/.gemini/settings.json`
- Pre-v0.6.4 `.agent/rules/antigravity-trs-rules.md` per-project file

**Re-enable plan.** When Google ships user-configurable PreToolHook
for Bash, restoring the programmatic integration is a mechanical
revert of branch `fix/antigravity-rules-only-revert`. See the
[research doc](antigravity-hooks-research.md#what-unblocks-re-enabling-the-integration)
for the exact checklist.

**Quirks**:
- The desktop IDE is VS Code-based. Older Antigravity 1.x spawned
  its tool-shell as non-interactive zsh and only read `~/.zshenv`;
  this could surface as `command not found: trs` if `~/.local/bin`
  was only added to `.zshrc`. install.sh recommends `~/.zshenv` for
  zsh users (fixed in `6f84f62`, plus an explicit warning added in
  v0.6.6 for users who already have `.local/bin` in `.zshrc` only).
  Antigravity 2.0 inherits this constraint.

### Windsurf

| | |
|---|---|
| Type | Rules file |
| Config | `.windsurfrules` (project root) |
| Template | `WINDSURF_RULES` |

Windsurf Cascade has no pre-execution hook. Works like Codex —
rules-based voluntary adoption.

## Test prompts

### Hook-based / plugin-based agents

```
Estoy validando el hook de trs en este entorno. Ejecuta los comandos
abajo usando tu tool de shell. Pega outputs literales.

CRITICO: Si no tienes acceso a shell, di "no tengo shell" en lugar de
inventar. Si ejecutas algo, no parafrasees — copy-paste literal.

1. `trs --version`
2. `git status`
3. `trs stats --history 2>&1 | tail -5`

Despues responde en 1 frase:
- Que tool usaste para ejecutar los comandos
- Que sabes de trs por tu contexto actual
```

**Pass criteria**:
- `git status` returns the compact `branch\nuntracked (N):\n  ?? file`
  format, NOT the native `On branch X\nYour branch is up to date\n...`
- `trs stats --history` shows a new entry with a timestamp matching the
  test moment — **ground truth**, use this instead of trusting the agent's
  reported output.

### Rules-based agents (Codex, Antigravity, Windsurf)

```
Tu archivo de instrucciones (AGENTS.md / .windsurfrules /
.agent/rules/) debe mencionar `trs`. Haz estos 3 tests sin inventar:

1. AWARENESS — Antes de ejecutar nada: ¿que sabes de trs por tu
   contexto actual? Una frase.

2. TOOL CHECK — Ejecuta `trs --version` y pega el output literal.

3. VOLUNTARY USE — Necesito ver el estado del repositorio git,
   minimizando consumo de tokens. NO ejecutes todavia. ESCRIBE el
   comando que usarias y explica por que.
```

**Pass criteria**:
- (1) describes trs correctly
- (2) returns `trs 0.5.X`
- (3) writes `trs git status` voluntarily (not `git status` raw)

## Debugging a broken integration

If a hook/plugin is installed but commands pass through unchanged, use a
debug wrapper instead of trusting the agent's self-report:

```bash
# 1. Create a logging wrapper
cat > /tmp/trs-hook-debug.sh <<'EOF'
#!/bin/bash
LOG=/tmp/trs-hook-debug.log
TS=$(date '+%H:%M:%S')
INPUT=$(cat)
echo "[$TS] >>> STDIN: $INPUT" >> $LOG
OUTPUT=$(echo "$INPUT" | /path/to/trs rewrite)
echo "[$TS] <<< STDOUT: $OUTPUT" >> $LOG
echo "$OUTPUT"
EOF
chmod +x /tmp/trs-hook-debug.sh

# 2. Point the agent's hook at the wrapper (back up first)
# cp ~/.xyz/settings.json ~/.xyz/settings.json.bak
# edit the config to replace `trs rewrite` with `/tmp/trs-hook-debug.sh`

# 3. Reproduce the failure in the agent (full quit + restart first)

# 4. Inspect
cat /tmp/trs-hook-debug.log
```

The log tells you:
- Empty → hook is not firing at all (config not loaded, matcher mismatch)
- Has STDIN but empty STDOUT → we decided not to rewrite (intentional for
  pipes, trs-prefixed commands, SKIP_PREFIXES)
- Has STDIN and STDOUT → we're responding but the agent is ignoring us
  (format mismatch — compare against the table above)

## Adding a new agent

1. **Classify**: hook, plugin, or rules? Read the agent's hook/plugin
   docs BEFORE writing anything. Cursor's and Gemini's formats both
   looked Claude-compatible at first glance and weren't.
2. **Find the rewrite path**: does the agent support `updated_input` or
   equivalent? If only allow/deny, it can't be a hook integration.
3. **Find the wire identifier**: `hook_event_name`, `tool_name`, or a
   header field the agent includes in its stdin payload.
4. **Write the template** in `src/init_templates.rs`. Don't copy
   another agent's template verbatim — the matcher field and tool name
   must match THIS agent's conventions.
5. **Add a dispatch arm** in `src/rewrite.rs::HookEvent::parse` if the
   output envelope differs from existing clients.
6. **Add the install spec** in `src/init.rs::spec()` with correct
   `global_dir` (often `.config/<agent>/<subdir>`).
7. **Test with the debug wrapper** before declaring success. Self-reports
   from the agent can hallucinate.
8. **Document here** — add an entry to the per-agent table.

## Related files

- `src/init.rs` — install orchestration, `merge_json_hook` (safe merging),
  `install_rules` (for rules-file tools)
- `src/init_templates.rs` — all templates
- `src/rewrite.rs` — hook command rewriter, `HookEvent` dispatch
- `docs/install.sh` — `.zshenv` targeting for zsh subshells
