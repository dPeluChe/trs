//! The Claude hook must honour the user's permission rules against the
//! ORIGINAL command: Claude Code matches them against the rewritten one and a
//! hook `allow` overrides them, so `deny: Bash(git push:*)` never saw
//! `trs git push`. Verified on a real Claude Code with a rewriting hook.

mod support;

use serde_json::{json, Value};
use support::{decision, run_hook, run_hook_with};

/// Run `trs rewrite` as the Claude hook, with `settings` as the user's
/// `~/.claude/settings.json`. `None` means no output (command left alone).
fn hook(settings: Option<&str>, cmd: &str, caller: &str) -> Option<Value> {
    let home = tempfile::tempdir().unwrap();
    if let Some(s) = settings {
        std::fs::create_dir_all(home.path().join(".claude")).unwrap();
        std::fs::write(home.path().join(".claude/settings.json"), s).unwrap();
    }
    let input = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": cmd },
        "cwd": home.path(),
    });
    run_hook(
        caller,
        &input,
        home.path(),
        &[],
        &["CLAUDE_CONFIG_DIR"],
        None,
    )
}

const RULES: &str = r#"{"permissions":{
  "allow":["Bash(cargo test:*)"],
  "ask":["Bash(git push:*)"],
  "deny":["Bash(rm -rf:*)"]}}"#;

#[test]
fn deny_and_ask_rules_stop_the_rewrite() {
    for cmd in [
        "git push origin main",
        "rm -rf build",
        "git status && rm -rf build",
    ] {
        assert!(hook(Some(RULES), cmd, "claude").is_none(), "{cmd}");
    }
}

#[test]
fn an_explicit_allow_rule_or_a_read_only_command_is_approved() {
    for cmd in ["cargo test --lib", "git status", "cd x && git log -3"] {
        let v = hook(Some(RULES), cmd, "claude").unwrap_or_else(|| panic!("{cmd}"));
        assert_eq!(decision(&v), Some("allow"), "{cmd}");
        assert!(
            v["hookSpecificOutput"]["updatedInput"]["command"]
                .as_str()
                .unwrap()
                .contains("trs "),
            "{cmd}"
        );
    }
}

#[test]
fn everything_else_is_rewritten_but_left_to_claudes_own_prompt() {
    for cmd in ["cargo build", "git branch -D old"] {
        let v = hook(Some(RULES), cmd, "claude").unwrap_or_else(|| panic!("{cmd}"));
        assert_eq!(decision(&v), None, "{cmd} must not be auto-approved");
        assert!(v["hookSpecificOutput"]["updatedInput"]["command"].is_string());
    }
    let v = hook(None, "cargo build", "claude").unwrap();
    assert_eq!(decision(&v), None, "no settings at all");
}

#[test]
fn unreadable_settings_fail_closed() {
    assert!(hook(Some("{ not json"), "git status", "claude").is_none());
}

#[test]
fn other_agents_keep_their_own_protocol() {
    let v = hook(Some(RULES), "cargo build", "codex").unwrap();
    assert_eq!(
        decision(&v),
        Some("allow"),
        "Codex needs the explicit allow"
    );
}

/// Claude ignores a project's `allow` until the folder is trusted; a repo's
/// own settings must not be able to approve commands through the hook.
#[test]
fn project_allow_rules_count_only_in_a_trusted_folder() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join(".claude")).unwrap();
    std::fs::write(
        project.path().join(".claude/settings.json"),
        r#"{"permissions":{"allow":["Bash(cargo build:*)"],"deny":["Bash(git push:*)"]}}"#,
    )
    .unwrap();
    let run = |cmd: &str| -> Option<Value> {
        let input = json!({
            "hook_event_name": "PreToolUse",
            "tool_input": { "command": cmd },
            "cwd": project.path(),
        });
        run_hook(
            "claude",
            &input,
            home.path(),
            &[],
            &["CLAUDE_CONFIG_DIR"],
            None,
        )
    };
    let untrusted = run("cargo build").unwrap();
    assert_eq!(decision(&untrusted), None, "untrusted project allow");
    assert!(run("git push").is_none(), "its deny still applies");

    let trust =
        json!({"projects": {project.path().to_str().unwrap(): {"hasTrustDialogAccepted": true}}});
    std::fs::write(home.path().join(".claude.json"), trust.to_string()).unwrap();
    assert_eq!(decision(&run("cargo build").unwrap()), Some("allow"));
}

/// An install from before `--caller` has no label: Claude's own tool name keeps
/// Claude's gate, an older Droid or Devin hook is told apart by its tool name,
/// and an unknown tool is rewritten but never approved.
#[test]
fn a_legacy_hook_without_caller_is_identified_by_its_tool_name() {
    let home = tempfile::tempdir().unwrap();
    let run = |tool: Option<&str>| -> Option<Value> {
        let mut input = json!({
            "hook_event_name": "PreToolUse",
            "permission_mode": "auto-high",
            "tool_input": { "command": "git status" },
            "cwd": home.path(),
        });
        if let Some(t) = tool {
            input["tool_name"] = json!(t);
        }
        run_hook_with(&[], &input, home.path(), &[], &["CLAUDE_CONFIG_DIR"], None)
    };
    let tagged = |v: &Value| {
        v["hookSpecificOutput"]["updatedInput"]["command"]
            .as_str()
            .unwrap()
            .to_string()
    };
    // Windows shells cannot take a `TRS_AGENT=x` prefix, so there is no tag to read.
    let label_is = |v: &Value, label: &str| {
        cfg!(windows) || tagged(v).starts_with(&format!("TRS_AGENT={label} "))
    };
    for tool in [Some("Bash"), None] {
        let v = run(tool).unwrap();
        assert_eq!(decision(&v), Some("allow"));
        assert!(label_is(&v, "claude"));
    }
    for (tool, label) in [("Execute", "droid"), ("exec", "devin-cli")] {
        let v = run(Some(tool)).unwrap_or_else(|| panic!("{tool}"));
        assert!(label_is(&v, label), "{tool}");
    }
    let v = run(Some("SomethingNew")).unwrap();
    assert_eq!(decision(&v), None, "an unknown tool is not approved");
}
