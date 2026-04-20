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
    "swift ",
    "xcodebuild ",
    "ping ",
    "brew ",
    "python ",
    "python3 ",
    "npx ",
    "ps ",
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

/// Which client's hook protocol are we speaking. Each emits a different
/// rewrite envelope; the wire name of the `hook_event_name` field identifies
/// the client. Claude Code is the default when the field is missing or unknown
/// because it's by far the most common client.
#[derive(Clone, Copy)]
enum HookEvent {
    /// Claude Code — `hook_event_name: "PreToolUse"` (capitalized).
    ClaudePreToolUse,
    /// Gemini CLI — `hook_event_name: "BeforeTool"`.
    GeminiBeforeTool,
    /// Cursor — `hook_event_name: "preToolUse"` (lowercase first letter).
    CursorPreToolUse,
}

impl HookEvent {
    fn parse(name: &str) -> Self {
        match name {
            "BeforeTool" => Self::GeminiBeforeTool,
            "preToolUse" => Self::CursorPreToolUse,
            _ => Self::ClaudePreToolUse,
        }
    }
}

/// Build the JSON response for the current hook event, or return `None` to
/// emit nothing (the agent runs the original command unchanged).
///
/// All supported clients share the same input envelope (`tool_input.command`)
/// but expect different output shapes:
///   Claude Code → hookSpecificOutput.updatedInput.command
///   Gemini CLI  → hookSpecificOutput.tool_input.command (+ top-level `decision`)
///   Cursor      → top-level `permission` + top-level `updated_input.command`
fn build_hook_response(json: &serde_json::Value) -> Option<serde_json::Value> {
    let cmd = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())?;
    let rewritten = maybe_rewrite(cmd)?;

    let event = HookEvent::parse(
        json.get("hook_event_name")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );

    let response = match event {
        HookEvent::GeminiBeforeTool => serde_json::json!({
            "systemMessage": "trs auto-rewrite",
            "decision": "allow",
            "hookSpecificOutput": {
                "tool_input": { "command": rewritten }
            }
        }),
        HookEvent::CursorPreToolUse => serde_json::json!({
            "permission": "allow",
            "updated_input": { "command": rewritten }
        }),
        HookEvent::ClaudePreToolUse => serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "trs auto-rewrite",
                "updatedInput": { "command": rewritten }
            }
        }),
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

    // Explicit per-invocation opt-out: `TRS_SKIP=1 git log ...` tells us
    // the caller (user or agent) wants this specific command to bypass
    // trs compression and receive raw output. The env var assignment
    // stays in the command string — the shell will strip it before
    // executing git, so the bypass is transparent downstream. Any value
    // after the `=` works; we only care that the prefix is present.
    if trimmed.starts_with("TRS_SKIP=") {
        return None;
    }

    // Hot-path optimization: almost all incoming commands are plain "git X"
    // with no shell operators. A single byte scan rejects that fast case before
    // we do any of the more expensive `contains(" && ")` / shell-op lookups.
    let has_shell_op = trimmed
        .as_bytes()
        .iter()
        .any(|b| matches!(b, b'&' | b'|' | b'>' | b'<' | b';'));

    // Chain handling: split on " && " and rewrite each segment independently.
    // Checked BEFORE SKIP_PREFIXES so `cd X && git Y` isn't short-circuited by
    // the "cd" skip. Pipes (`|`) and semicolons (`;`) are left to the shell.
    if has_shell_op
        && trimmed.contains(" && ")
        && !trimmed.contains(" | ")
        && !trimmed.contains(" ; ")
    {
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

    // Semicolon chains are too unpredictable (independent commands) — skip.
    if has_shell_op && trimmed.contains(" ; ") {
        return None;
    }

    // Pipe / redirect: rewrite ONLY the first segment and keep the rest
    // untouched. Agents routinely append `| head -N`, `| grep X`, or `> file`
    // to cap / filter output; blocking rewrite on any pipe silently defeated
    // the hook for those very common cases. Rewriting just the command that
    // produces the data preserves shell semantics while still applying trs
    // compression.
    if has_shell_op {
        if let Some((first, rest)) = split_at_shell_op(trimmed) {
            let rewritten = maybe_rewrite(first)?;
            return Some(format!("{}{}", rewritten, rest));
        }
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

/// Split a command at the first shell operator (pipe or redirect) so the left
/// side can be rewritten independently. Longer delimiters (`" >> "`) are
/// tried before their prefixes (`" > "`) so we don't misclassify them.
///
/// Returns `(first_segment, operator_and_rest)` — splicing them back yields
/// the original string.
fn split_at_shell_op(s: &str) -> Option<(&str, &str)> {
    [" | ", " >> ", " > ", " < "]
        .iter()
        .filter_map(|op| s.find(op).map(|pos| (pos, *op)))
        .min_by_key(|(pos, _)| *pos)
        .map(|(pos, _)| (&s[..pos], &s[pos..]))
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
    fn test_skip_trs_skip_env_var() {
        // Explicit per-invocation bypass. The agent adds TRS_SKIP=1 when
        // it genuinely wants raw command output.
        assert_eq!(maybe_rewrite("TRS_SKIP=1 git status"), None);
        assert_eq!(maybe_rewrite("TRS_SKIP=true cargo test"), None);
        assert_eq!(maybe_rewrite("TRS_SKIP= git log"), None); // empty value still signals skip
    }

    #[test]
    fn test_trs_skip_does_not_match_other_env_vars() {
        // Only TRS_SKIP= triggers bypass — other env var prefixes get
        // the normal rewrite treatment so unrelated command invocations
        // (e.g. `RUSTFLAGS=... cargo build`) still benefit from trs.
        // Note: env-var prefix handling beyond skip is not in scope for
        // this guard — we simply don't match on RUSTFLAGS / PATH / etc.
        assert!(maybe_rewrite("RUSTFLAGS=-C git status").is_some());
    }

    #[test]
    fn test_rewrite_pipe_first_segment() {
        // Agents frequently append `| head -N` / `| grep X` — rewrite only
        // the command producing the data, leave the pipeline untouched.
        assert_eq!(
            maybe_rewrite("git log | grep fix"),
            Some("trs git log | grep fix".into())
        );
        assert_eq!(
            maybe_rewrite("find . -name '*.rs' | xargs wc"),
            Some("trs find . -name '*.rs' | xargs wc".into())
        );
        assert_eq!(
            maybe_rewrite("git status | head -3"),
            Some("trs git status | head -3".into())
        );
    }

    #[test]
    fn test_rewrite_multi_pipe_first_segment_only() {
        // Only the first segment gets rewritten; any further pipes are
        // treated as opaque post-processing.
        assert_eq!(
            maybe_rewrite("git log | head -5 | tail -1"),
            Some("trs git log | head -5 | tail -1".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_first_segment() {
        // Redirects follow the same rule — rewrite the producer, keep the
        // redirect target untouched.
        assert_eq!(
            maybe_rewrite("git diff > out.txt"),
            Some("trs git diff > out.txt".into())
        );
        assert_eq!(
            maybe_rewrite("git log >> history.txt"),
            Some("trs git log >> history.txt".into())
        );
    }

    #[test]
    fn test_skip_pipe_when_first_segment_is_skipped() {
        // If the left side is something we'd never rewrite (cat, echo,
        // already-trs), the whole pipeline stays as-is.
        assert_eq!(maybe_rewrite("cat < input.txt"), None);
        assert_eq!(maybe_rewrite("echo hello | xxd"), None);
        assert_eq!(maybe_rewrite("trs git status | head -3"), None);
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
    fn test_hook_response_cursor_format() {
        // Cursor's preToolUse emits a flat envelope with `permission` and
        // `updated_input` at the top level. Lowercase event name distinguishes
        // it from Claude's PreToolUse.
        let input = parse_input(
            r#"{
                "hook_event_name":"preToolUse",
                "tool_name":"Shell",
                "tool_input":{"command":"git status"}
            }"#,
        );
        let out = build_hook_response(&input).expect("should rewrite");
        assert_eq!(out["permission"], serde_json::json!("allow"));
        assert_eq!(
            out["updated_input"]["command"],
            serde_json::json!("trs git status")
        );
        // Cursor format must NOT carry Claude's or Gemini's wrapping.
        assert!(out.get("hookSpecificOutput").is_none());
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
