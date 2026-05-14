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
//! Decision logic (which commands get wrapped) lives in `rewrite_decide.rs`.
//! This module owns the I/O entry point and the per-agent JSON envelopes.

use std::io::Read;

use crate::rewrite_decide::{looks_like_env_assignment, maybe_rewrite};

/// Run the rewrite logic. Called from main.rs.
pub(crate) fn run_rewrite() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        return;
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        handle_json_protocol(&json);
        return;
    }

    // Plain text mode (simple command string)
    if let Some(rewritten) = maybe_rewrite(input) {
        println!("{}", rewritten);
    }
}

fn handle_json_protocol(json: &serde_json::Value) {
    if let Some(response) = build_hook_response(json) {
        println!("{}", response);
    }
}

/// Which client's hook protocol we're speaking. Each emits a different
/// envelope; `hook_event_name` identifies the client. Claude Code is the
/// default when the field is missing/unknown — by far the most common.
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

    /// Label written into the TRS_AGENT env-var so history.jsonl can
    /// attribute the run. Droid shares Claude's wire format, so both land
    /// in `ClaudePreToolUse` and label as "claude".
    fn agent_label(&self) -> &'static str {
        match self {
            Self::GeminiBeforeTool => "gemini",
            Self::CursorPreToolUse => "cursor",
            Self::ClaudePreToolUse => "claude",
        }
    }
}

/// Build the JSON response for the current hook event, or `None` to emit
/// nothing (the agent runs the original command unchanged). All clients
/// share the same input envelope (`tool_input.command`) but expect
/// different output shapes:
///   Claude Code → hookSpecificOutput.updatedInput.command
///   Gemini CLI  → hookSpecificOutput.tool_input.command (+ top-level `decision`)
///   Cursor      → top-level `permission` + top-level `updated_input.command`
fn build_hook_response(json: &serde_json::Value) -> Option<serde_json::Value> {
    let cmd = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())?;

    let event = HookEvent::parse(
        json.get("hook_event_name")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );

    // Bypass telemetry — log the agent-attributed observation before the
    // short-circuit so `stats --by-agent` can surface per-agent rates.
    if cmd_bypasses_trs(cmd) {
        crate::tracker::log_bypass(cmd, Some(event.agent_label()));
        return None;
    }

    let rewritten = maybe_rewrite(cmd)?;
    let rewritten = tag_with_agent(&rewritten, event.agent_label());

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

/// True when `cmd` is bypassing trs via a `TRS_SKIP=` or `TRS_DISABLE=`
/// env-var prefix. Walks leading `NAME=value` tokens because shells
/// accept multiple assignments before the command (`FOO=1 TRS_SKIP=1
/// git status`). Also strips a leading `env [-...] [NAME=value...]`
/// invocation so `env TRS_DISABLE=1 npx tsc` is recognized too.
fn cmd_bypasses_trs(cmd: &str) -> bool {
    let mut rest = cmd.trim_start();
    // Strip a leading `env` invocation: `env [-i|-u VAR|--] [NAME=val...]`
    // (or `/usr/bin/env`, the absolute form the user's shell may resolve
    // to) is functionally equivalent to bare `NAME=val ...` for our
    // bypass detection.
    let env_stripped = rest
        .strip_prefix("env ")
        .or_else(|| rest.strip_prefix("/usr/bin/env "));
    if let Some(after) = env_stripped {
        rest = after.trim_start();
        while let Some(tok) = rest.split_whitespace().next() {
            if !(tok.starts_with('-') || looks_like_env_assignment(tok)) {
                break;
            }
            if tok.starts_with("TRS_SKIP=") || tok.starts_with("TRS_DISABLE=") {
                return true;
            }
            rest = rest[tok.len()..].trim_start();
        }
    }
    loop {
        if rest.starts_with("TRS_SKIP=") || rest.starts_with("TRS_DISABLE=") {
            return true;
        }
        let Some(space_at) = rest.find(char::is_whitespace) else {
            return false;
        };
        let token = &rest[..space_at];
        if !looks_like_env_assignment(token) {
            return false;
        }
        rest = rest[space_at..].trim_start();
    }
}

/// Prefix `cmd` with `TRS_AGENT=<label>` so the downstream `trs <cmd>`
/// execution can attribute the run. The shell strips the leading
/// `VAR=value` assignment before exec, so it's transparent to git/cargo/etc.
fn tag_with_agent(cmd: &str, agent: &str) -> String {
    if cmd.starts_with("TRS_AGENT=") {
        return cmd.to_string();
    }
    format!("TRS_AGENT={} {}", agent, cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_bypasses_trs_detection() {
        assert!(cmd_bypasses_trs("TRS_SKIP=1 git status"));
        assert!(cmd_bypasses_trs("TRS_SKIP=true cargo test"));
        // Hidden behind another env-var assignment — still bypass.
        assert!(cmd_bypasses_trs("FOO=bar TRS_SKIP=1 git log"));
        assert!(cmd_bypasses_trs("A=1 B=2 TRS_SKIP=1 cargo build"));
        assert!(cmd_bypasses_trs("  TRS_SKIP=1 git status"));
        // Negatives.
        assert!(!cmd_bypasses_trs("git status"));
        assert!(!cmd_bypasses_trs("RUSTFLAGS=-C cargo build"));
        assert!(!cmd_bypasses_trs("trs git status"));
        // TRS_SKIP as a literal arg later in the line is not a bypass.
        assert!(!cmd_bypasses_trs("git log --grep TRS_SKIP=1"));
    }

    #[test]
    fn test_cmd_bypasses_trs_disable_alias() {
        // TRS_DISABLE=1 — historically used by users as an ad-hoc
        // disable; recognize as bypass for telemetry parity with
        // TRS_SKIP=1.
        assert!(cmd_bypasses_trs("TRS_DISABLE=1 npx tsc"));
        assert!(cmd_bypasses_trs("FOO=bar TRS_DISABLE=1 cargo build"));
    }

    #[test]
    fn test_cmd_bypasses_env_wrapped() {
        // `env [-flags] VAR=val cmd` is equivalent to `VAR=val cmd`
        // for bypass intent. Recognize the env-wrapped form too.
        assert!(cmd_bypasses_trs("env TRS_DISABLE=1 npx tsc"));
        assert!(cmd_bypasses_trs("/usr/bin/env TRS_SKIP=1 cargo build"));
        assert!(cmd_bypasses_trs("env FOO=bar TRS_DISABLE=1 npx tsc"));
        // env without the bypass marker → not a bypass.
        assert!(!cmd_bypasses_trs("env FOO=bar cargo build"));
    }

    fn parse_input(s: &str) -> serde_json::Value {
        serde_json::from_str(s).expect("test input must be valid JSON")
    }

    #[test]
    fn test_hook_response_claude_code_format() {
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
            serde_json::json!("TRS_AGENT=claude trs git status")
        );
        assert!(out["hookSpecificOutput"]["tool_input"].is_null());
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_hook_response_cursor_format() {
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
            serde_json::json!("TRS_AGENT=cursor trs git status")
        );
        assert!(out.get("hookSpecificOutput").is_none());
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_hook_response_gemini_format() {
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
            serde_json::json!("TRS_AGENT=gemini trs git status")
        );
        assert!(out["hookSpecificOutput"]["updatedInput"].is_null());
    }

    #[test]
    fn test_hook_response_default_is_claude_format() {
        // Missing / unknown hook_event_name defaults to Claude's shape.
        let input = parse_input(r#"{"tool_input":{"command":"git status"}}"#);
        let out = build_hook_response(&input).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!("TRS_AGENT=claude trs git status")
        );
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_hook_response_no_rewrite_returns_none() {
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
        let input = parse_input(r#"{"hook_event_name":"BeforeTool"}"#);
        assert!(build_hook_response(&input).is_none());
    }

    #[test]
    fn test_hook_response_chain_preserved_across_formats() {
        // Chain-aware rewrite applies in every format; build_hook_response
        // prefixes the whole result with TRS_AGENT= once.
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
        assert_eq!(
            build_hook_response(&claude).unwrap()["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!("TRS_AGENT=claude cd /tmp && trs git status && trs cargo test")
        );
        assert_eq!(
            build_hook_response(&gemini).unwrap()["hookSpecificOutput"]["tool_input"]["command"],
            serde_json::json!("TRS_AGENT=gemini cd /tmp && trs git status && trs cargo test")
        );
    }
}
