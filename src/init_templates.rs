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

// ANTIGRAVITY_HOOKS was REMOVED in v0.6.6 (see commit log for branch
// fix/antigravity-rules-only-revert).
//
// The v0.6.5 jetski PreToolUse integration was reverted after empirical
// validation against agy v1.0.1 showed user-defined `hooks.json` entries
// load as **subagents** (via `invoke_subagent`), NOT as tool-call
// wrappers. The `Step_RunCommand` execution path for Bash bypasses the
// user-visible PreToolHook system entirely — only internal MCP browser
// hooks are wired up upstream.
//
// Full investigation: docs/development/antigravity-hooks-research.md.
// Until Google ships user-configurable PreToolHook for Bash, Antigravity
// (IDE + CLI) is rules-only — see ANTIGRAVITY_RULES_SECTION below.

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
pub(crate) const CODEX_AGENTS_SENTINEL_END: &str = "<!-- trs:codex-rules:end -->";

pub(crate) const CODEX_AGENTS_SECTION: &str = concat!(
    "<!-- trs:codex-rules:start v1 -->\n",
    r#"
## Terminal Output Optimization

This project uses `trs` (Token-Reducing Shell) for token-optimized terminal output.

Codex has had `PreToolUse` hooks since codex-cli 0.117, but the
`updatedInput.command` rewrite trs would need is documented yet **not
implemented in the runtime** — Codex rejects it with "unsupported
updatedInput" (tracking: https://github.com/openai/codex/issues/18491). So
trs cannot transparently rewrite commands from a Codex hook today.
**Prefix every shell command with `trs`** to get compressed output:

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

The output-saver reply-brevity rules are installed separately as their own
sentinel-managed block (run `trs output-saver --install`, which `trs init
codex` also triggers) — kept out of this section so the two never duplicate.

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

/// Sentinel that wraps the Antigravity rules block in `~/.gemini/GEMINI.md`.
/// Lets re-runs and uninstall identify the block without false positives
/// from prose mentions of `trs`.
pub(crate) const ANTIGRAVITY_RULES_SENTINEL_START: &str = "<!-- trs:antigravity-rules:start v1 -->";
pub(crate) const ANTIGRAVITY_RULES_SENTINEL_END: &str = "<!-- trs:antigravity-rules:end -->";

/// Rules block appended to `~/.gemini/GEMINI.md` for both the Antigravity
/// IDE and the Antigravity CLI (`agy`). Both products read this file at
/// session start via the Gemini-style `@import` resolution.
///
/// The block is small on purpose — Antigravity sessions already consume a
/// lot of context. We rely on the existing `@trs.md` import for the
/// full output-saver / response shape rules; this block only documents
/// the manual-prefix recommendation and explains why automatic
/// rewriting is not active (jetski PreTool hooks aren't user-config).
pub(crate) const ANTIGRAVITY_RULES_SECTION: &str = concat!(
    "<!-- trs:antigravity-rules:start v1 -->\n",
    r#"
## Terminal Output Optimization (Antigravity IDE + agy CLI)

This project uses `trs` (Token-Reducing Shell) for token-optimized terminal output.

Antigravity v1.0.1 does **not yet expose user-configurable `PreToolUse`
hooks** for shell commands, so trs cannot rewrite commands automatically
on your behalf. Until Google ships that surface, **prefix every shell
command with `trs`** when you want compressed output:

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

68-99% token reduction with no signal loss. Commands without a dedicated
parser still get ANSI stripping + whitespace collapse (~30-40% "free").

See `docs/development/antigravity-hooks-research.md` in the trs repo
for the investigation that led to this rules-only integration.

<!-- trs:antigravity-rules:end -->
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
