//! Droid matches its command lists against the REWRITTEN command and a hook
//! `allow` skips them, so the Droid hook applies them to the original one.

mod support;

use serde_json::{json, Value};
use std::path::Path;
use support::{decision, run_hook, write_file as write};

/// Run `trs rewrite --caller droid` with `home` as `FACTORY_HOME_OVERRIDE`.
fn hook(home: &Path, cwd: &Path, cmd: &str, mode: &str) -> Option<Value> {
    let input = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Execute",
        "permission_mode": mode,
        "tool_input": { "command": cmd, "riskLevel": "low" },
        "cwd": cwd,
    });
    run_hook(
        "droid",
        &input,
        home,
        &[("FACTORY_HOME_OVERRIDE", home)],
        &[],
        None,
    )
}

const USER: &str = r#"{"commandDenylist":["git push"],"commandBlocklist":["deploy"]}"#;

fn setup() -> (tempfile::TempDir, tempfile::TempDir) {
    let (home, cwd) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    write(home.path(), ".factory/settings.json", USER);
    (home, cwd)
}

#[test]
fn denylist_and_blocklist_hits_stop_the_rewrite() {
    let (home, cwd) = setup();
    for cmd in [
        "git push origin main",
        "git status && git push",
        "deploy now",
    ] {
        assert!(
            hook(home.path(), cwd.path(), cmd, "auto-high").is_none(),
            "{cmd}"
        );
    }
}

#[test]
fn a_listed_command_is_rewritten_and_approved_in_any_mode() {
    let (home, cwd) = setup();
    for cmd in ["git status", "git log -3"] {
        let v = hook(home.path(), cwd.path(), cmd, "off").unwrap_or_else(|| panic!("{cmd}"));
        assert_eq!(decision(&v), Some("allow"), "{cmd}");
        let new = v["hookSpecificOutput"]["updatedInput"]["command"]
            .as_str()
            .unwrap();
        assert!(new.contains("trs "), "{cmd}");
    }
}

#[test]
fn an_unlisted_command_is_rewritten_unapproved_only_in_auto_high() {
    let (home, cwd) = setup();
    let cmd = "git branch -D old";
    let v = hook(home.path(), cwd.path(), cmd, "auto-high").unwrap();
    assert_eq!(decision(&v), None);
    assert!(v["hookSpecificOutput"]["updatedInput"]["command"].is_string());
    for mode in ["off", "auto-low", "auto-medium", "spec"] {
        assert!(hook(home.path(), cwd.path(), cmd, mode).is_none(), "{mode}");
    }
}

#[test]
fn unreadable_settings_fail_closed() {
    let (home, cwd) = setup();
    write(home.path(), ".factory/settings.json", "{ not json");
    assert!(hook(home.path(), cwd.path(), "git status", "auto-high").is_none());
}

/// Droid honors a project's legacy command lists without a trust prompt (verified on 1.2.0).
#[test]
fn a_projects_lists_count_without_folder_trust() {
    let (home, project) = setup();
    std::fs::create_dir_all(project.path().join(".git")).unwrap();
    write(
        project.path(),
        ".factory/settings.json",
        r#"{"commandAllowlist":["git branch"],"commandDenylist":["git tag"]}"#,
    );
    let run = |cmd: &str| hook(home.path(), project.path(), cmd, "off");
    assert_eq!(decision(&run("git branch -D old").unwrap()), Some("allow"));
    assert!(run("git tag v1").is_none());
}
