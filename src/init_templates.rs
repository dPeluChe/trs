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

// Plugin/extension templates (TS/Python) live in their own module; re-export
// so existing callers keep importing from init_templates.
pub(crate) use crate::init_templates_plugins::*;

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

// Codex CLI (>= 0.134): same `PreToolUse` envelope as Claude — Codex's shell
// tool is named `Bash` and it honors `hookSpecificOutput.updatedInput.command`
// with `permissionDecision: "allow"` (which `trs rewrite` already emits).
// `--caller codex` attributes the run in history/stats (shell-agnostic; the
// old `TRS_AGENT=codex` env prefix was POSIX-only and stays whitelisted for
// already-installed hooks).
// Merged into the user's existing ~/.codex/hooks.json (notify hooks etc. are
// preserved). Codex gates new hooks behind a one-time trust prompt (`/hooks`).
pub(crate) const CODEX_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite --caller codex"
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
// `--caller droid` attributes runs correctly in stats (previously they were
// indistinguishable from Claude — same envelope, no label).
pub(crate) const DROID_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite --caller droid"
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

// VS Code Copilot agent hooks (preview, validated live 2026-06-09): VS Code
// speaks Claude's PreToolUse envelope and honors
// `hookSpecificOutput.updatedInput.command`. Native path `~/.copilot/hooks/`
// (project: `.github/hooks/`). Matchers are parsed but ignored — the hook
// fires on every tool; trs no-ops without `tool_input.command`, and unknown
// event names fail open. `--caller vscode` attributes runs distinctly from
// Claude Code (VS Code can ALSO load `~/.claude/settings.json` hooks when
// the "Use Claude Hooks" setting is on; rewrite is idempotent so a double
// fire is harmless — first one wins).
pub(crate) const VSCODE_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite --caller vscode"
          }
        ],
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

// Devin CLI ("Devin for Terminal", binary `devin` — distinct from the
// rules-only Devin Desktop / ex-Windsurf). Its hooks speak Claude's
// PreToolUse envelope and it reads `.claude/settings.json` by default, but
// its shell tool is named `exec` (not `Bash`), so the Claude hook's
// `matcher: "Bash"` never fires under Devin — hence a dedicated install.
// Target is `config.json` (global `~/.config/devin/`, project `.devin/`)
// under the `hooks` key; the standalone `.devin/hooks.v1.json` puts events
// at the root with no `hooks` wrapper, which the merge path can't share.
// `--caller devin-cli` attributes runs distinctly in history/stats.
//
// updatedInput: validated live 2026-07-07 — Devin honors
// `hookSpecificOutput.updatedInput` (commands run rewritten as `trs …`),
// even though its docs only mention `decision`/`additionalContext`.
// Attribution requires `devin-cli` in rewrite.rs `known_agent_label`, else
// `--caller devin-cli` falls back to `claude`. Devin also reads `.claude`
// hooks by default (`read_config_from.claude`); set it false so this hook
// wins over the transitive Claude one.
pub(crate) const DEVIN_CLI_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "exec",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite --caller devin-cli"
          }
        ],
        "description": "Route commands through trs for token-optimized output"
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

On **codex-cli >= 0.134** a `PreToolUse` hook in `~/.codex/hooks.json` routes
shell commands through trs automatically (it honors `updatedInput.command`).
`trs init codex --global` installs it; approve it once via `/hooks`. With the
hook active you **don't** need to prefix anything.

If you're on an older Codex build, or the hook isn't trusted yet, fall back to
**prefixing every shell command with `trs`**:

```bash
# Without an active hook, instead of:
git status
cargo test
cargo clippy

# Use:
trs git status
trs cargo test
trs cargo clippy
```

(Prefixing is always safe — trs never double-wraps a command that already
starts with `trs`.) This reduces token consumption by 68-99% without losing
signal. Commands without a dedicated parser still get ANSI stripping and
whitespace collapse (~30-40% reduction "free").

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

// Devin Desktop (ex-Windsurf) directory-rule file: `.devin/rules/trs.md`.
// `trigger: always_on` is the Windsurf-lineage frontmatter Devin reads for
// an always-applied rule (Cursor's `.mdc` uses `alwaysApply`; AGENTS.md needs
// none). Devin Local still has no shell hook, so this rules block — like the
// legacy `.windsurfrules` — is the whole integration.
pub(crate) const DEVIN_RULE: &str = concat!(
    r#"---
trigger: always_on
---

# trs (Token-Reducing Shell) — terminal output optimization

Devin Local does not expose a pre-execution hook, so this rule is the way to
opt into trs. When running shell commands, prefix them with `trs` to get
compact, structured output.

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
agent session — duplicate sections, embedded code/SQL blocks that belong in
their own files, dead references. Every unnecessary token here loads on every
call.

Reference: https://github.com/dPeluChe/trs
"#
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for #53: OpenCode/Kilo on Windows. The POSIX `VAR=value cmd`
    /// prefix breaks in PowerShell/cmd ("not recognized as a cmdlet"), and the
    /// weak guard let retries snowball into
    /// `TRS_AGENT=… trs TRS_AGENT=… trs …`.
    #[test]
    fn opencode_kilo_plugins_are_windows_safe_and_idempotent() {
        for (plugin, agent) in [(OPENCODE_PLUGIN, "opencode"), (KILO_PLUGIN, "kilo")] {
            // No POSIX env-var command prefix.
            assert!(
                !plugin.contains(&format!("TRS_AGENT={agent} trs")),
                "{agent}: plugin still uses the POSIX `TRS_AGENT=… trs` prefix"
            );
            // Attribution moved to the cross-platform shell.env hook.
            assert!(
                plugin.contains("\"shell.env\""),
                "{agent}: missing shell.env hook"
            );
            assert!(
                plugin.contains(&format!("output.env.TRS_AGENT = \"{agent}\"")),
                "{agent}: shell.env does not set TRS_AGENT"
            );
            // Command is just `trs ${cmd}`.
            assert!(
                plugin.contains("`trs ${cmd}`"),
                "{agent}: command is not `trs ${{cmd}}`"
            );
            // Guard skips already-wrapped commands (anti-snowball).
            assert!(
                plugin.contains("cmd.startsWith(\"TRS_AGENT=\")"),
                "{agent}: guard does not skip the legacy TRS_AGENT= prefix"
            );
            assert!(
                plugin.contains("cmd.startsWith(\"trs \")"),
                "{agent}: guard does not skip trs-prefixed commands"
            );
        }
    }
}
