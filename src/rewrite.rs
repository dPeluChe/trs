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

/// Run the rewrite logic. Called from main.rs. `agent_flag` is the
/// `--caller <label>` set by the installing hook template — the
/// shell-agnostic attribution channel (the `TRS_AGENT=x` command prefix is
/// POSIX-only; PowerShell/cmd parse it as a bogus command name).
pub(crate) fn run_rewrite(agent_flag: Option<&str>) {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        return;
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        handle_json_protocol(&json, agent_flag);
        return;
    }

    // Plain text mode (simple command string)
    if let Some(rewritten) = maybe_rewrite(input) {
        println!("{}", rewritten);
    }
}

fn handle_json_protocol(json: &serde_json::Value, agent_flag: Option<&str>) {
    if let Some(response) = build_hook_response(json, agent_flag) {
        println!("{}", response);
    }
}

/// Map a user-supplied label to its canonical static form. Whitelist, not
/// pass-through: only labels we ship templates for can attribute runs.
fn known_agent_label(s: &str) -> Option<&'static str> {
    Some(match s {
        "claude" => "claude",
        "gemini" => "gemini",
        "cursor" => "cursor",
        "codex" => "codex",
        "vscode" => "vscode",
        "droid" => "droid",
        "antigravity" => "antigravity",
        "opencode" => "opencode",
        "kilo" => "kilo",
        "pi" => "pi",
        _ => return None,
    })
}

/// Which client's hook protocol we're speaking. Each emits a different
/// envelope; `hook_event_name` identifies the client. A *missing* field
/// defaults to Claude Code (back-compat — by far the most common), but an
/// explicit name we don't recognize maps to `Unknown` so a new client's
/// envelope is never answered in a shape it may not understand.
#[derive(Clone, Copy, PartialEq)]
enum HookEvent {
    /// Claude Code — `hook_event_name: "PreToolUse"` (capitalized).
    /// Also spoken by Droid, Codex ≥0.134, Antigravity (jetski), and
    /// VS Code Copilot agent hooks.
    ClaudePreToolUse,
    /// Gemini CLI — `hook_event_name: "BeforeTool"`.
    GeminiBeforeTool,
    /// Cursor — `hook_event_name: "preToolUse"` (lowercase first letter).
    CursorPreToolUse,
    /// An explicit `hook_event_name` we don't recognize — a client with
    /// its own envelope. Fail open: no rewrite, original command runs.
    Unknown,
}

impl HookEvent {
    fn parse(name: &str) -> Self {
        match name {
            // "" = field absent: legacy/Claude-compatible callers.
            "PreToolUse" | "" => Self::ClaudePreToolUse,
            "BeforeTool" => Self::GeminiBeforeTool,
            "preToolUse" => Self::CursorPreToolUse,
            _ => Self::Unknown,
        }
    }

    /// Label written into the TRS_AGENT env-var so history.jsonl can
    /// attribute the run. Reads the runtime env to disambiguate Antigravity
    /// from Claude (both speak the same PreToolUse envelope).
    fn agent_label(&self) -> &'static str {
        self.agent_label_from(
            std::env::var_os("ANTIGRAVITY_CONVERSATION_ID").is_some(),
            std::env::var("TRS_AGENT").ok().as_deref(),
        )
    }

    /// Codex and VS Code Copilot share Claude's `PreToolUse` envelope, so we
    /// can't tell them apart from `hook_event_name` alone. Their hook
    /// commands set `TRS_AGENT=<label>`, which we read here to attribute the
    /// run. Whitelist (not pass-through): an arbitrary inherited TRS_AGENT
    /// value must not silently relabel runs.
    fn agent_label_from(&self, has_antigravity_env: bool, trs_agent: Option<&str>) -> &'static str {
        match trs_agent {
            Some("codex") => return "codex",
            Some("vscode") => return "vscode",
            _ => {}
        }
        self.agent_label_for(has_antigravity_env)
    }

    /// Pure version of `agent_label` — env state passed explicitly so tests
    /// don't have to mutate process env (which races with parallel tests).
    ///
    /// Antigravity 2.0 (IDE + CLI/`agy`) speaks `ClaudePreToolUse` because
    /// jetski uses the same envelope. We disambiguate via the
    /// `ANTIGRAVITY_CONVERSATION_ID` env var which agy sets when invoking
    /// hooks — without this, all agy runs would show up as `claude` in
    /// `trs stats --by-agent`.
    fn agent_label_for(&self, has_antigravity_env: bool) -> &'static str {
        match self {
            Self::GeminiBeforeTool => "gemini",
            Self::CursorPreToolUse => "cursor",
            Self::ClaudePreToolUse if has_antigravity_env => "antigravity",
            Self::ClaudePreToolUse => "claude",
            Self::Unknown => "unknown",
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
fn build_hook_response(
    json: &serde_json::Value,
    agent_flag: Option<&str>,
) -> Option<serde_json::Value> {
    let cmd = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())?;
    // `--caller` from the hook template wins over envelope/env inference.
    let flag_label = agent_flag.and_then(known_agent_label);

    let event_name = json
        .get("hook_event_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let event = HookEvent::parse(event_name);

    // Unrecognized client envelope: fail open instead of guessing Claude's
    // response shape — a client that doesn't understand it can hard-fail the
    // tool call (seen with pre-0.134 Codex "unsupported updatedInput"). The
    // original command runs unchanged; the zero-cost history entry surfaces
    // the new client in `stats --by-agent` so we can add real support.
    if event == HookEvent::Unknown {
        // cfg-gated so unit tests exercising this path don't append to the
        // user's real history.jsonl.
        #[cfg(not(test))]
        crate::tracker::log_bypass(cmd, Some("unknown"));
        eprintln!("trs rewrite: unrecognized hook_event_name \"{event_name}\" — passing through");
        return None;
    }

    // Bypass telemetry — log the agent-attributed observation before the
    // short-circuit so `stats --by-agent` can surface per-agent rates.
    if cmd_bypasses_trs(cmd) {
        crate::tracker::log_bypass(cmd, Some(flag_label.unwrap_or_else(|| event.agent_label())));
        return None;
    }

    let rewritten = maybe_rewrite(cmd)?;
    let rewritten = tag_with_agent(
        &rewritten,
        flag_label.unwrap_or_else(|| event.agent_label()),
    );

    let response = match event {
        // Handled above; kept here so the match stays exhaustive if the
        // early return ever moves.
        HookEvent::Unknown => return None,
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
/// execution can attribute the run. POSIX shells strip the leading
/// `VAR=value` assignment before exec; PowerShell/cmd do NOT — they treat
/// `TRS_AGENT=opencode` as a command name (issue #53). So on Windows we emit
/// the bare `trs <cmd>` (attribution is lost there, but the command runs).
fn tag_with_agent(cmd: &str, agent: &str) -> String {
    tag_with_agent_for(cmd, agent, cfg!(windows))
}

/// Pure core of `tag_with_agent` — `is_windows` passed explicitly so both
/// branches are testable on any platform.
fn tag_with_agent_for(cmd: &str, agent: &str, is_windows: bool) -> String {
    if is_windows {
        return cmd.to_string();
    }
    // Chains: an env prefix on the FIRST segment never reaches the later
    // ones (`VAR=x cd a && trs b` runs trs without VAR — observed as
    // untagged history entries), so tag every segment that invokes trs.
    // The " && " delimiter matches what maybe_rewrite joins with.
    // Transparent wrappers (time/nohup/…) propagate env, so front-of-
    // segment stays right for `time trs …`. A literal " trs " inside e.g.
    // an echo argument would gain a harmless env prefix.
    cmd.split(" && ")
        .map(|seg| {
            let t = seg.trim_start();
            let invokes_trs = t.starts_with("trs ") || t.contains(" trs ");
            if invokes_trs && !t.starts_with("TRS_AGENT=") {
                format!("TRS_AGENT={} {}", agent, t)
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" && ")
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

    /// Expected rewritten command for `agent`: POSIX gets the `TRS_AGENT=`
    /// prefix, Windows gets none (PowerShell/cmd can't parse it — see #53).
    fn agent_cmd(agent: &str, rest: &str) -> String {
        if cfg!(windows) {
            rest.to_string()
        } else {
            format!("TRS_AGENT={agent} {rest}")
        }
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
        let out = build_hook_response(&input, None).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["hookEventName"],
            serde_json::json!("PreToolUse")
        );
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!(agent_cmd("claude", "trs git status"))
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
        let out = build_hook_response(&input, None).expect("should rewrite");
        assert_eq!(out["permission"], serde_json::json!("allow"));
        assert_eq!(
            out["updated_input"]["command"],
            serde_json::json!(agent_cmd("cursor", "trs git status"))
        );
        assert!(out.get("hookSpecificOutput").is_none());
        assert!(out.get("decision").is_none());
    }

    #[test]
    fn test_agent_flag_attributes_rewrite() {
        // `--caller droid` from the hook template must label the rewritten
        // command (Windows-safe channel; same envelope as Claude).
        let input = parse_input(
            r#"{
                "hook_event_name":"PreToolUse",
                "tool_input":{"command":"git status"}
            }"#,
        );
        let out = build_hook_response(&input, Some("droid")).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!(agent_cmd("droid", "trs git status"))
        );
        // Unknown flag values are ignored (whitelist) — envelope label wins.
        let out = build_hook_response(&input, Some("not-a-real-agent")).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!(agent_cmd("claude", "trs git status"))
        );
    }

    #[test]
    fn test_hook_response_unknown_event_fails_open() {
        // A 4th client with its own envelope must get NO response (the
        // original command runs unchanged), never a Claude-shaped guess.
        let input = parse_input(
            r#"{
                "hook_event_name":"beforeShellExecution",
                "tool_input":{"command":"git status"}
            }"#,
        );
        assert!(build_hook_response(&input, None).is_none());
    }

    #[test]
    fn test_hook_event_parse_mapping() {
        assert!(matches!(
            HookEvent::parse("PreToolUse"),
            HookEvent::ClaudePreToolUse
        ));
        // Missing field stays Claude for back-compat.
        assert!(matches!(HookEvent::parse(""), HookEvent::ClaudePreToolUse));
        assert!(matches!(
            HookEvent::parse("BeforeTool"),
            HookEvent::GeminiBeforeTool
        ));
        assert!(matches!(
            HookEvent::parse("preToolUse"),
            HookEvent::CursorPreToolUse
        ));
        // Anything explicitly different is a new client, not Claude.
        assert!(matches!(
            HookEvent::parse("PostToolUse"),
            HookEvent::Unknown
        ));
        assert!(matches!(HookEvent::parse("pretooluse"), HookEvent::Unknown));
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
        let out = build_hook_response(&input, None).expect("should rewrite");
        assert_eq!(out["decision"], serde_json::json!("allow"));
        assert_eq!(
            out["hookSpecificOutput"]["tool_input"]["command"],
            serde_json::json!(agent_cmd("gemini", "trs git status"))
        );
        assert!(out["hookSpecificOutput"]["updatedInput"].is_null());
    }

    #[test]
    fn test_agent_label_antigravity_env_disambiguates_claude_envelope() {
        // The wire envelope is shared between Claude and Antigravity (jetski).
        // The agent gets relabeled to "antigravity" only when the env var
        // is present at hook-invocation time.
        assert_eq!(HookEvent::ClaudePreToolUse.agent_label_for(false), "claude");
        // Codex rides the same PreToolUse envelope; TRS_AGENT=codex (set by
        // the codex hook command) attributes it correctly, and wins even if
        // the antigravity env happens to be present.
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_from(false, Some("codex")),
            "codex"
        );
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_from(true, Some("codex")),
            "codex"
        );
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_from(false, None),
            "claude"
        );
        // VS Code Copilot also speaks PreToolUse; its hook sets
        // TRS_AGENT=vscode. Unknown env values must NOT relabel.
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_from(false, Some("vscode")),
            "vscode"
        );
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_from(false, Some("something-else")),
            "claude"
        );
        assert_eq!(
            HookEvent::ClaudePreToolUse.agent_label_for(true),
            "antigravity"
        );
        // Gemini/Cursor labels are unaffected by the agy env var.
        assert_eq!(HookEvent::GeminiBeforeTool.agent_label_for(true), "gemini");
        assert_eq!(HookEvent::CursorPreToolUse.agent_label_for(true), "cursor");
    }

    #[test]
    fn test_hook_response_default_is_claude_format() {
        // Missing / unknown hook_event_name defaults to Claude's shape.
        let input = parse_input(r#"{"tool_input":{"command":"git status"}}"#);
        let out = build_hook_response(&input, None).expect("should rewrite");
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["command"],
            serde_json::json!(agent_cmd("claude", "trs git status"))
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
        assert!(build_hook_response(&input, None).is_none());
    }

    #[test]
    fn test_hook_response_missing_command_returns_none() {
        let input = parse_input(r#"{"hook_event_name":"BeforeTool"}"#);
        assert!(build_hook_response(&input, None).is_none());
    }

    #[test]
    fn test_hook_response_chain_preserved_across_formats() {
        // Chain-aware rewrite applies in every format. Each trs-invoking
        // segment carries its own TRS_AGENT= prefix — an env assignment on
        // the first segment never reaches the later ones (`VAR=x cd a &&
        // trs b` runs trs untagged), and `cd` needs no tag at all.
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
        // Per-segment expectation, platform-aware (Windows emits no prefix).
        let chain = |agent: &str| {
            ["cd /tmp", "trs git status", "trs cargo test"]
                .iter()
                .map(|s| {
                    if s.starts_with("trs ") {
                        agent_cmd(agent, s)
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" && ")
        };
        assert_eq!(
            build_hook_response(&claude, None).unwrap()["hookSpecificOutput"]["updatedInput"]
                ["command"],
            serde_json::json!(chain("claude"))
        );
        assert_eq!(
            build_hook_response(&gemini, None).unwrap()["hookSpecificOutput"]["tool_input"]
                ["command"],
            serde_json::json!(chain("gemini"))
        );
    }

    #[test]
    fn tag_with_agent_skips_posix_prefix_on_windows() {
        // POSIX: env-var prefix (the shell strips it before exec).
        assert_eq!(
            tag_with_agent_for("trs git status", "claude", false),
            "TRS_AGENT=claude trs git status"
        );
        // Windows: no prefix — PowerShell/cmd would treat `TRS_AGENT=claude`
        // as a bogus command name (issue #53).
        assert_eq!(
            tag_with_agent_for("trs git status", "claude", true),
            "trs git status"
        );
        // Already tagged → unchanged on either platform.
        assert_eq!(
            tag_with_agent_for("TRS_AGENT=claude trs git status", "claude", false),
            "TRS_AGENT=claude trs git status"
        );
    }
}
