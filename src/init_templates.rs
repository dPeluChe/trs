//! Hook and rules file templates for each supported AI tool.
//!
//! Kept in a separate module so `init.rs` stays focused on dispatch and
//! detection logic. Each template is a single string constant used by
//! `install_from_spec` / `install_rules` / `install_codex`.

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

pub(crate) const CURSOR_HOOKS: &str = r#"{
  "hooks": {
    "beforeShellExecution": [
      {
        "command": "trs rewrite",
        "event": "beforeShellExecution",
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

// OpenCode/Kilo plugin: unconditionally prefix trs, let trs decide
// whether to compress or passthrough. No stale rewrite list needed.
pub(crate) const OPENCODE_PLUGIN: &str = r#"// trs plugin — route commands through trs for token-optimized output
import type { Plugin } from "opencode";

export default function trsPlugin(): Plugin {
  return {
    name: "trs",
    hooks: {
      before_tool_call: async (ctx) => {
        if (ctx.tool === "bash" && ctx.input?.command) {
          const cmd = ctx.input.command;
          // Skip if already using trs or if it's a trs command itself
          if (!cmd.startsWith("trs ") && !cmd.startsWith("cd ")) {
            ctx.input.command = `trs ${cmd}`;
          }
        }
        return ctx;
      },
    },
  };
}
"#;

pub(crate) const CODEX_AGENTS_SECTION: &str = r#"
## Terminal Output Optimization

This project uses `trs` (TARS CLI) for token-optimized terminal output.
When running shell commands, prefer prefixing with `trs` for compact output:

```bash
# Instead of:
git status
cargo test
cargo clippy

# Use:
trs git status
trs cargo test
trs cargo clippy
```

This reduces token consumption by 68-99% without losing signal.
See https://github.com/dPeluChe/trs for details.
"#;

pub(crate) const ANTIGRAVITY_RULES: &str = r#"
# trs (TARS CLI) — terminal output optimization

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

Reference: https://github.com/dPeluChe/trs
"#;

pub(crate) const WINDSURF_RULES: &str = r#"
# trs (TARS CLI) — terminal output optimization

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

Reference: https://github.com/dPeluChe/trs
"#;
