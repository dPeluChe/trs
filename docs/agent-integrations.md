# AI Agent Integrations — Reference

How `trs` integrates with each supported AI coding agent. Use this doc when
adding a new agent, debugging a broken integration, or reviewing why a
specific quirk exists.

Last validated: 2026-04-18 against `trs` v0.5.6.

## Integration types

Agents fall into three architectural buckets. Pick the right bucket BEFORE
writing a template — the install mechanism, test strategy, and failure modes
differ.

| Type | How it works | Agents | File written |
|---|---|---|---|
| **Hook (JSON event)** | Agent fires a PreToolUse-style event, hands us the command on stdin, applies our rewrite response | Claude Code, Gemini CLI, Cursor, Factory Droid | `settings.json` / `hooks.json` |
| **Plugin (TypeScript)** | Agent auto-discovers `.ts` plugin files at startup and mutates tool args in-process | OpenCode, Kilo Code | `plugins/trs.ts` |
| **Rules file** | No programmatic interception. Agent reads a rules/instructions file and VOLUNTARILY prefixes `trs` | Codex, Google Antigravity, Windsurf | `AGENTS.md` / `.agent/rules/*.md` / `.windsurfrules` |

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

### Google Antigravity

| | |
|---|---|
| Type | Rules file |
| Config | `.agent/rules/antigravity-trs-rules.md` (project) |
| Template | `ANTIGRAVITY_RULES` |

**Quirks**:
- Antigravity is VS Code-based. It spawns its tool-shell as non-interactive
  zsh, which reads ONLY `~/.zshenv` — NOT `~/.zshrc`. If the user added
  `~/.local/bin` to PATH only in `.zshrc`, Antigravity's agent shell gets
  `command not found: trs`.
- The `install.sh` now recommends `~/.zshenv` for zsh users instead of
  `~/.zshrc`. Fixed in `6f84f62`.
- `ANTIGRAVITY_RULES` includes a fallback pointing to the absolute
  `$HOME/.local/bin/trs` path for users who can't easily change their shell
  config.

### Windsurf

| | |
|---|---|
| Type | Rules file |
| Config | `.windsurfrules` (project root) |
| Template | `WINDSURF_RULES` |

Windsurf Cascade has no pre-execution hook. Works like Codex/Antigravity —
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
- `scripts/install.sh` — `.zshenv` targeting for zsh subshells
