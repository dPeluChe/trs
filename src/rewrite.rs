//! `trs rewrite` — Hook command rewriter for AI coding tools.
//!
//! Called by PreToolUse/BeforeTool hooks. Reads the command from stdin,
//! decides if it should go through trs, and outputs the rewritten command.
//!
//! Protocol (Claude Code / Gemini CLI):
//!   stdin:  {"tool_name":"Bash","tool_input":{"command":"git status"}}
//!   stdout: (empty = no change, or modified JSON)
//!   exit 0 = allow
//!
//! For commands trs knows how to compress, rewrites "git status" → "trs git status".
//! For commands trs doesn't handle, passes through unchanged.

use std::io::Read;

/// Commands (prefixes) that trs should rewrite.
/// Keep in sync with classifier.rs — but intentionally broad.
/// Unknown commands still get generic compression.
const REWRITE_PREFIXES: &[&str] = &[
    "git ",
    "ls ",
    "ls\n",
    "lsd ",
    "exa ",
    "eza ",
    "tree ",
    "find ",
    "fd ",
    "grep ",
    "rg ",
    "ag ",
    "ack ",
    "tail ",
    "cargo ",
    "npm ",
    "pnpm ",
    "bun ",
    "yarn ",
    "pip ",
    "pip3 ",
    "pytest",
    "jest",
    "vitest",
    "make ",
    "make\n",
    "cmake ",
    "tsc ",
    "gcc ",
    "g++ ",
    "clang ",
    "javac ",
    "docker ",
    "gh ",
    "env\n",
    "env ",
    "printenv",
    "wc ",
    "wget ",
    "curl ",
    "eslint",
    "biome ",
    "ruff ",
    "pylint",
    "golangci-lint",
    "ollama ",
    "kubectl ",
];

/// Commands that should NEVER be rewritten (internal, cd, pipes, etc.)
const SKIP_PREFIXES: &[&str] = &[
    "trs ", "cd ", "echo ", "cat ", "head ", "tail -f", "export ", "source ", ".", "set ",
    "unset ", "alias ", "which ", "type ", "true", "false", "exit", "return",
];

/// Run the rewrite logic. Called from main.rs.
pub(crate) fn run_rewrite() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        // No input or read error — allow unchanged
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        return;
    }

    // Try to parse as JSON (Claude Code / Gemini protocol)
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        handle_json_protocol(&json);
        return;
    }

    // Plain text mode (simple command string)
    if let Some(rewritten) = maybe_rewrite(input) {
        println!("{}", rewritten);
    }
}

/// Handle JSON hook protocol. Prints the response envelope (or nothing when
/// no rewrite is needed). Pure formatting lives in `build_hook_response` for
/// testability.
fn handle_json_protocol(json: &serde_json::Value) {
    if let Some(response) = build_hook_response(json) {
        println!("{}", response);
    }
}

/// Claude Code and Gemini CLI share the same input envelope
/// (`tool_input.command`) but expect different output shapes:
///   Claude Code → hookSpecificOutput.updatedInput.command
///   Gemini CLI  → hookSpecificOutput.tool_input.command (+ top-level `decision`)
/// We dispatch on the `hook_event_name` field: `PreToolUse` = Claude,
/// `BeforeTool` = Gemini. Default to Claude format when ambiguous since
/// that's the most common client.
///
/// Returns `None` when the input has no bash command to rewrite (the hook
/// should emit nothing so the original command runs unchanged).
fn build_hook_response(json: &serde_json::Value) -> Option<serde_json::Value> {
    let cmd = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())?;
    let rewritten = maybe_rewrite(cmd)?;

    let event = json
        .get("hook_event_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let response = if event == "BeforeTool" {
        // Gemini CLI: writes `hookSpecificOutput.tool_input` over the model's args.
        serde_json::json!({
            "systemMessage": "trs auto-rewrite",
            "decision": "allow",
            "hookSpecificOutput": {
                "tool_input": { "command": rewritten }
            }
        })
    } else {
        // Claude Code PreToolUse format (also default for unknown events).
        serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "trs auto-rewrite",
                "updatedInput": { "command": rewritten }
            }
        })
    };
    Some(response)
}

/// Decide if a command should be rewritten to go through trs.
/// Returns Some(rewritten) or None (leave unchanged).
fn maybe_rewrite(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();

    // Never rewrite empty commands
    if trimmed.is_empty() {
        return None;
    }

    // Never rewrite commands that are already using trs
    if trimmed.starts_with("trs ") {
        return None;
    }

    // Chain handling: split on " && " and rewrite each segment independently.
    // Checked BEFORE SKIP_PREFIXES so `cd X && git Y` isn't short-circuited by
    // the "cd" skip. Pipes (`|`) and semicolons (`;`) are left to the shell.
    if trimmed.contains(" && ") && !trimmed.contains(" | ") && !trimmed.contains(" ; ") {
        let segments: Vec<&str> = trimmed.split(" && ").map(str::trim).collect();
        let mut any_changed = false;
        let mut rewritten: Vec<String> = Vec::with_capacity(segments.len());
        for seg in &segments {
            match maybe_rewrite(seg) {
                Some(r) => {
                    any_changed = true;
                    rewritten.push(r);
                }
                None => rewritten.push(seg.to_string()),
            }
        }
        if any_changed {
            return Some(rewritten.join(" && "));
        }
        return None;
    }

    // Never rewrite internal/shell commands
    for skip in SKIP_PREFIXES {
        if trimmed.starts_with(skip) || trimmed == skip.trim() {
            return None;
        }
    }

    // Never rewrite pipes or semicolons (let the shell handle them)
    if trimmed.contains(" | ") || trimmed.contains(" ; ") {
        return None;
    }

    // Never rewrite commands with redirections
    if trimmed.contains(" > ") || trimmed.contains(" >> ") || trimmed.contains(" < ") {
        return None;
    }

    // Never rewrite subshells or command substitution
    if trimmed.contains("$(") || trimmed.contains('`') {
        return None;
    }

    // Block --no-verify on git commands (security: don't let agents skip hooks)
    if (trimmed.starts_with("git commit") || trimmed.starts_with("git push"))
        && (trimmed.contains("--no-verify") || trimmed.contains("-n "))
    {
        eprintln!("[trs] blocked: --no-verify is not allowed (protects pre-commit hooks)");
        std::process::exit(2); // Exit 2 = block in Claude Code hook protocol
    }

    // Check if this is a command trs handles
    for prefix in REWRITE_PREFIXES {
        let p = prefix.trim();
        if trimmed.starts_with(prefix) || trimmed == p {
            return Some(format!("trs {}", trimmed));
        }
    }

    // Unknown command — still rewrite for generic compression
    // (whitespace collapse, ANSI stripping)
    // But skip very short commands or assignment-like patterns
    if trimmed.contains('=') && !trimmed.contains(' ') {
        return None; // VAR=value
    }

    Some(format!("trs {}", trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_git() {
        assert_eq!(maybe_rewrite("git status"), Some("trs git status".into()));
        assert_eq!(maybe_rewrite("git log -5"), Some("trs git log -5".into()));
    }

    #[test]
    fn test_rewrite_cargo() {
        assert_eq!(maybe_rewrite("cargo test"), Some("trs cargo test".into()));
        assert_eq!(
            maybe_rewrite("cargo clippy"),
            Some("trs cargo clippy".into())
        );
    }

    #[test]
    fn test_skip_already_trs() {
        assert_eq!(maybe_rewrite("trs git status"), None);
    }

    #[test]
    fn test_skip_cd() {
        assert_eq!(maybe_rewrite("cd /tmp"), None);
    }

    #[test]
    fn test_skip_pipes() {
        assert_eq!(maybe_rewrite("git log | grep fix"), None);
        assert_eq!(maybe_rewrite("find . -name '*.rs' | xargs wc"), None);
    }

    #[test]
    fn test_skip_redirections() {
        assert_eq!(maybe_rewrite("git diff > out.txt"), None);
        assert_eq!(maybe_rewrite("cat < input.txt"), None);
    }

    #[test]
    fn test_skip_subshells() {
        assert_eq!(maybe_rewrite("echo $(git status)"), None);
    }

    #[test]
    fn test_skip_assignments() {
        assert_eq!(maybe_rewrite("FOO=bar"), None);
    }

    #[test]
    fn test_skip_empty() {
        assert_eq!(maybe_rewrite(""), None);
    }

    #[test]
    fn test_rewrite_unknown_command() {
        // Unknown commands still get generic compression
        assert_eq!(
            maybe_rewrite("terraform plan"),
            Some("trs terraform plan".into())
        );
    }

    #[test]
    fn test_json_protocol() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let json: serde_json::Value = serde_json::from_str(input).unwrap();

        let cmd = json
            .get("tool_input")
            .and_then(|ti| ti.get("command"))
            .and_then(|c| c.as_str())
            .unwrap();
        assert_eq!(maybe_rewrite(cmd), Some("trs git status".into()));
    }

    #[test]
    fn test_skip_echo() {
        assert_eq!(maybe_rewrite("echo hello"), None);
    }

    #[test]
    fn test_rewrite_env() {
        assert_eq!(maybe_rewrite("env"), Some("trs env".into()));
    }

    #[test]
    fn test_skip_shell_builtins() {
        assert_eq!(maybe_rewrite("export PATH=/usr/bin"), None);
        assert_eq!(maybe_rewrite("source .env"), None);
    }

    #[test]
    fn test_rewrite_cd_chain() {
        assert_eq!(
            maybe_rewrite("cd /tmp && git status"),
            Some("cd /tmp && trs git status".into())
        );
        assert_eq!(
            maybe_rewrite("cd /path/to/project && cargo test"),
            Some("cd /path/to/project && trs cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_multi_chain() {
        // All rewritable segments get rewritten independently
        assert_eq!(
            maybe_rewrite("cd /tmp && git status && git log"),
            Some("cd /tmp && trs git status && trs git log".into())
        );
        assert_eq!(
            maybe_rewrite("cargo fmt && cargo clippy"),
            Some("trs cargo fmt && trs cargo clippy".into())
        );
    }

    #[test]
    fn test_skip_cd_chain_with_pipe() {
        // If any segment has a pipe, disable chain handling entirely
        assert_eq!(maybe_rewrite("cd /tmp && git status | less"), None);
    }

    #[test]
    fn test_skip_cd_chain_all_skips() {
        // Only non-rewritable commands — nothing changes
        assert_eq!(maybe_rewrite("cd /tmp && echo hello"), None);
    }

    // ----------------------------------------------------------------
    // Hook-response dispatch: Claude Code vs Gemini CLI
    // ----------------------------------------------------------------

    fn parse_input(s: &str) -> serde_json::Value {
        serde_json::from_str(s).expect("test input must be valid JSON")
    }

    #[test]
    fn test_hook_response_claude_code_format() {
        // Claude Code sends hook_event_name: PreToolUse — we emit Claude's
        // updatedInput envelope.
        let input = parse_input(
            r#"{
                "hook_event_name":"PreToolUse",
                "tool_name":"Bash",
                "tool_input":{"command":"git status"}
            }"#,
        );
        let out = build_hook_response(&input).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["hookEventName"],
            serde_json::json!("PreToolUse")
        );
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!("trs git status")
        );
        // Claude format must NOT carry Gemini's tool_input field.
        assert!(out["hookSpecificOutput"]["tool_input"].is_null());
        // No top-level `decision` in Claude's schema.
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_hook_response_gemini_format() {
        // Gemini CLI sends hook_event_name: BeforeTool — we emit its
        // tool_input envelope with a top-level `decision` field.
        let input = parse_input(
            r#"{
                "hook_event_name":"BeforeTool",
                "tool_name":"run_shell_command",
                "tool_input":{"command":"git status"}
            }"#,
        );
        let out = build_hook_response(&input).expect("should rewrite");
        assert_eq!(out["decision"], serde_json::json!("allow"));
        assert_eq!(
            out["hookSpecificOutput"]["tool_input"]["command"],
            serde_json::json!("trs git status")
        );
        // Gemini format must NOT carry Claude's updatedInput field.
        assert!(out["hookSpecificOutput"]["updatedInput"].is_null());
    }

    #[test]
    fn test_hook_response_default_is_claude_format() {
        // Missing / unknown hook_event_name defaults to Claude's shape so
        // we don't silently break the majority client.
        let input = parse_input(r#"{"tool_input":{"command":"git status"}}"#);
        let out = build_hook_response(&input).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!("trs git status")
        );
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_hook_response_no_rewrite_returns_none() {
        // Commands that don't need rewriting produce no response — the hook
        // is expected to emit nothing so the command runs unchanged.
        let input = parse_input(
            r#"{
                "hook_event_name":"PreToolUse",
                "tool_input":{"command":"echo hello"}
            }"#,
        );
        assert!(build_hook_response(&input).is_none());
    }

    #[test]
    fn test_hook_response_missing_command_returns_none() {
        // Malformed input (no tool_input.command) — emit nothing.
        let input = parse_input(r#"{"hook_event_name":"BeforeTool"}"#);
        assert!(build_hook_response(&input).is_none());
    }

    #[test]
    fn test_hook_response_chain_preserved_across_formats() {
        // The chain-aware rewrite applies identically in both formats.
        let claude = parse_input(
            r#"{
                "hook_event_name":"PreToolUse",
                "tool_input":{"command":"cd /tmp && git status && cargo test"}
            }"#,
        );
        let gemini = parse_input(
            r#"{
                "hook_event_name":"BeforeTool",
                "tool_input":{"command":"cd /tmp && git status && cargo test"}
            }"#,
        );
        let expected = "cd /tmp && trs git status && trs cargo test";
        assert_eq!(
            build_hook_response(&claude).unwrap()["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!(expected)
        );
        assert_eq!(
            build_hook_response(&gemini).unwrap()["hookSpecificOutput"]["tool_input"]["command"],
            serde_json::json!(expected)
        );
    }
}
