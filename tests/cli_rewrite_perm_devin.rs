//! Devin CLI matches its `Exec(...)` rules against the REWRITTEN command and a
//! hook `allow` overrides deny and ask, so the hook applies them to the
//! original one. Verified on Devin CLI 3000.11.234 with a rewriting hook.

use assert_cmd::Command;
use serde_json::{json, Value};

/// Run `trs rewrite --caller devin-cli` with `user` as `~/.config/devin/config.json`
/// and `project` as the project's `.devin/config.json`. `None` = command left alone.
fn hook(user: Option<&str>, project: Option<&str>, cmd: &str) -> Option<Value> {
    let home = tempfile::tempdir().unwrap();
    let proj = tempfile::tempdir().unwrap();
    if let Some(s) = user {
        std::fs::create_dir_all(home.path().join(".config/devin")).unwrap();
        std::fs::write(home.path().join(".config/devin/config.json"), s).unwrap();
    }
    if let Some(s) = project {
        std::fs::create_dir_all(proj.path().join(".devin")).unwrap();
        std::fs::write(proj.path().join(".devin/config.json"), s).unwrap();
    }
    let input = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "exec",
        "tool_input": { "command": cmd },
    });
    let out = Command::cargo_bin("trs")
        .unwrap()
        .current_dir(proj.path())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("APPDATA", home.path())
        .env_remove("DEVIN_PROJECT_DIR")
        .env("TRS_HOME", home.path().join("trs"))
        .args(["rewrite", "--caller", "devin-cli"])
        .write_stdin(input.to_string())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    (!text.trim().is_empty()).then(|| serde_json::from_str(&text).unwrap())
}

fn decision(v: &Value) -> Option<&str> {
    v["hookSpecificOutput"]["permissionDecision"].as_str()
}

const RULES: &str = r#"{"permissions":{
  "allow":["Exec(cargo test)","Exec(git status)","Fetch(domain:*)"],
  "ask":["Exec(git push)"],
  "deny":["Exec(rm)"]}}"#;

#[test]
fn deny_and_ask_rules_stop_the_rewrite() {
    for cmd in [
        "git push origin main",
        "rm -rf build",
        "git status && rm -rf build",
    ] {
        assert!(hook(Some(RULES), None, cmd).is_none(), "{cmd}");
    }
}

#[test]
fn a_project_deny_rule_stops_the_rewrite() {
    let project = r#"{"permissions":{"deny":["Exec(git status)"]}}"#;
    assert!(hook(Some(RULES), Some(project), "git status").is_none());
}

#[test]
fn a_command_fully_covered_by_allow_rules_is_approved() {
    for cmd in ["cargo test --lib", "git status", "cargo test && git status"] {
        let v = hook(Some(RULES), None, cmd).unwrap_or_else(|| panic!("{cmd}"));
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
fn a_project_allow_rule_is_not_trusted() {
    let project = r#"{"permissions":{"allow":["Exec(git log)"]}}"#;
    let v = hook(Some(RULES), Some(project), "git log").unwrap();
    assert_eq!(decision(&v), None);
}

#[test]
fn everything_else_is_rewritten_but_left_to_devins_own_prompt() {
    for cmd in ["cargo build", "git log", "cargo test && cargo build"] {
        let v = hook(Some(RULES), None, cmd).unwrap_or_else(|| panic!("{cmd}"));
        assert_eq!(decision(&v), None, "{cmd} must not be auto-approved");
        assert!(v["hookSpecificOutput"]["updatedInput"]["command"].is_string());
    }
    let v = hook(None, None, "git status").unwrap();
    assert_eq!(decision(&v), None, "no config at all");
}

#[test]
fn unreadable_config_fails_closed() {
    assert!(hook(Some("{ not json"), None, "git status").is_none());
}
