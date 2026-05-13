//! Hook and rules file templates for each supported AI tool.
//!
//! Kept in a separate module so `init.rs` stays focused on dispatch and
//! detection logic. Each template is a single string constant used by
//! `install_from_spec` / `install_rules` / `install_codex`.
//!
//! The Output saver block embedded in Codex/Antigravity/Windsurf rules
//! templates is not duplicated — it expands from
//! `output_saver_block_literal!()` so a change in one place flows to
//! every install target.

use crate::output_saver_block_literal;

pub(crate) const CLAUDE_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite"
          }
        ],
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

// Factory Droid: same envelope as Claude's PreToolUse, but Droid's shell tool
// is named `Execute` (not `Bash`), so the matcher is widened. We use ".*" to
// match any tool — trs rewrite internally skips commands that don't look like
// shell invocations, so the overhead of a per-tool check is negligible.
pub(crate) const DROID_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite"
          }
        ],
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

pub(crate) const GEMINI_HOOKS: &str = r#"{
  "hooks": {
    "BeforeTool": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite",
            "name": "trs-rewrite",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}"#;

// Cursor's `beforeShellExecution` hook can only allow/deny — it cannot
// rewrite the command. The only hook with `updated_input` support is
// `preToolUse`. `matcher: "Shell"` limits the hook to actual shell tool
// invocations instead of every Read/Write/MCP call (observed: Cursor spams
// Read on the terminal transcript file, so skipping those saves meaningful
// subprocess overhead).
pub(crate) const CURSOR_HOOKS: &str = r#"{
  "hooks": {
    "preToolUse": [
      {
        "command": "trs rewrite",
        "matcher": "Shell",
        "description": "Route shell commands through trs for token-optimized output"
      }
    ]
  }
}"#;

// OpenCode/Kilo plugin: unconditionally prefix trs, let trs decide whether
// to compress or passthrough. Uses OpenCode's documented plugin shape:
//   - async function returning a hooks map
//   - hook key `"tool.execute.before"` (string literal, not a property name)
//   - `input.tool === "bash"` to gate shell commands
//   - mutate `output.args.command` in-place
// Reference: https://opencode.ai/docs/plugins/
pub(crate) const OPENCODE_PLUGIN: &str = r#"// trs plugin — route commands through trs for token-optimized output

export const TrsPlugin = async () => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;
      const cmd = output.args?.command;
      if (typeof cmd !== "string") return;
      // Skip if already routed through trs or if it's a cd (dir change).
      if (cmd.startsWith("trs ") || cmd.startsWith("cd ")) return;
      // TRS_AGENT=opencode tells trs history.jsonl who triggered the run.
      // The shell strips the env-var assignment before executing.
      output.args.command = `TRS_AGENT=opencode trs ${cmd}`;
    },
  };
};
"#;

/// Kilo plugin — identical mechanism to OpenCode but tags the
/// downstream trs call as `TRS_AGENT=kilo` so history attribution
/// is accurate across forks.
pub(crate) const KILO_PLUGIN: &str = r#"// trs plugin — route commands through trs for token-optimized output

export const TrsPlugin = async () => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return;
      const cmd = output.args?.command;
      if (typeof cmd !== "string") return;
      if (cmd.startsWith("trs ") || cmd.startsWith("cd ")) return;
      // TRS_AGENT=kilo tells trs history.jsonl who triggered the run.
      output.args.command = `TRS_AGENT=kilo trs ${cmd}`;
    },
  };
};
"#;

/// Sentinel that marks the Codex AGENTS.md block on re-runs. The block's
/// prose uses backtick-wrapped `` `trs` `` which doesn't match the plain
/// `trs (Token-Reducing Shell)` marker — without this sentinel re-runs
/// duplicate the section.
pub(crate) const CODEX_AGENTS_SENTINEL_START: &str = "<!-- trs:codex-rules:start v1 -->";

pub(crate) const CODEX_AGENTS_SECTION: &str = concat!(
    "<!-- trs:codex-rules:start v1 -->\n",
    r#"
## Terminal Output Optimization

This project uses `trs` (Token-Reducing Shell) for token-optimized terminal output.

Codex's PreToolUse hook parses but does not implement `updatedInput`
([codex docs](https://developers.openai.com/codex/hooks)), so trs cannot
rewrite commands automatically from a hook. **Prefix every shell command
with `trs`** to get compressed output:

```bash
# Instead of:
git status
cargo test
cargo clippy

# Always use:
trs git status
trs cargo test
trs cargo clippy
```

This reduces token consumption by 68-99% without losing signal.
Commands without a dedicated parser still get ANSI stripping and whitespace
collapse (~30-40% reduction "free").

"#,
    output_saver_block_literal!(),
    r#"

## Keeping this file lean

Periodically run `trs audit-docs` in this project to surface content that
bloats every agent session: duplicate sections across rules files, embedded
code/SQL/JSON that should live in their own files, references to docs that
no longer exist. The tool also cross-checks whether code snippets here
already have definitions in the source tree — flagging them as "remove and
link" vs "extract to a new file".

See https://github.com/dPeluChe/trs for details.

<!-- trs:codex-rules:end -->
"#
);

pub(crate) const ANTIGRAVITY_RULES: &str = concat!(
    r#"
# trs (Token-Reducing Shell) — terminal output optimization

Antigravity does not support pre-execution hooks, so this rules file is the
only mechanism to opt into trs for this project. When you run shell commands,
prefix them with `trs` to get compact, structured output.

```bash
# Instead of:
git status
cargo test
npm test

# Use:
trs git status
trs cargo test
trs npm test
```

Commands without a dedicated trs parser still get whitespace / ANSI
compression (~30-40% reduction). Pipes and chains are passed through unchanged.

## If `trs: command not found` in Antigravity's shell

Antigravity's tool shell does not always inherit your login PATH. If `trs`
runs from your terminal but fails here, fall back to the absolute binary
path or make sure your PATH is exported for non-login shells:

```bash
# Option 1 — explicit path (always works):
$HOME/.local/bin/trs git status

# Option 2 — add to ~/.profile (read by non-login sh / bash):
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.profile
```

Re-run `trs init antigravity` to pick up future updates to this guidance.

"#,
    output_saver_block_literal!(),
    r#"

## Keeping this file lean

Run `trs audit-docs` periodically to spot content that belongs elsewhere
(duplicate sections, embedded SQL/JSON/code blocks, references to files
that no longer exist). Every unnecessary token here loads on every agent
call.

Reference: https://github.com/dPeluChe/trs
"#
);

pub(crate) const WINDSURF_RULES: &str = concat!(
    r#"
# trs (Token-Reducing Shell) — terminal output optimization

Windsurf Cascade does not expose a pre-execution hook, so this rules file is
the way to opt into trs for this project. When running shell commands, prefix
them with `trs` to get compact, structured output.

```bash
# Instead of:
git status
cargo test
pnpm test

# Use:
trs git status
trs cargo test
trs pnpm test
```

Commands without a dedicated trs parser still get whitespace / ANSI
compression (~30-40% reduction). Pipes and chains are passed through unchanged.

"#,
    output_saver_block_literal!(),
    r#"

## Keeping this file lean

Run `trs audit-docs` periodically to surface content that inflates every
agent session — duplicate sections across rules files, embedded code/SQL
blocks that belong in their own files, dead references. Every unnecessary
token here loads on every call.

Reference: https://github.com/dPeluChe/trs
"#
);
