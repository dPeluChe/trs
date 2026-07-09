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
    // Devin CLI must be whitelisted — else `--caller devin-cli` silently
    // falls back to `claude` (the bug found in live validation). Regression
    // guard: keep `devin-cli` in `known_agent_label`.
    let out = build_hook_response(&input, Some("devin-cli")).expect("should rewrite");
    assert_eq!(
        out["hookSpecificOutput"]["updatedInput"]["command"],
        serde_json::json!(agent_cmd("devin-cli", "trs git status"))
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
        build_hook_response(&gemini, None).unwrap()["hookSpecificOutput"]["tool_input"]["command"],
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
