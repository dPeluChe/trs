//! The Codex hook must honour exec-policy `.rules` against the ORIGINAL
//! command: Codex matches them against the rewritten one, so a `forbidden`
//! `git push` rule never saw `trs git push`. Verified on codex-cli 0.166.0.

use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::Path;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Run `trs rewrite --caller codex` with `rules` as `$CODEX_HOME/rules/default.rules`.
/// `None` means no output (command left alone).
fn hook(rules: Option<&str>, cmd: &str) -> Option<Value> {
    let home = tempfile::tempdir().unwrap();
    let codex = home.path().join("codex");
    std::fs::create_dir_all(&codex).unwrap();
    if let Some(r) = rules {
        write(&codex, "rules/default.rules", r);
    }
    run(&home, &codex, cmd, home.path())
}

fn run(home: &tempfile::TempDir, codex: &Path, cmd: &str, cwd: &Path) -> Option<Value> {
    let input = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": cmd },
        "cwd": cwd,
    });
    let out = Command::cargo_bin("trs")
        .unwrap()
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("CODEX_HOME", codex)
        .env("TRS_HOME", home.path().join("trs"))
        .args(["rewrite", "--caller", "codex"])
        .write_stdin(input.to_string())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    (!text.trim().is_empty()).then(|| serde_json::from_str(&text).unwrap())
}

const RULES: &str = r#"
prefix_rule(pattern=["git", "push"], decision="forbidden", justification="no push")
prefix_rule(pattern=["git", "commit"], decision="prompt")
prefix_rule(pattern=["cargo", "test"], decision="allow")
"#;

fn assert_rewritten(v: &Value, cmd: &str) {
    assert_eq!(
        v["hookSpecificOutput"]["permissionDecision"].as_str(),
        Some("allow"),
        "{cmd}: Codex needs the allow for the rewrite to apply"
    );
    assert!(
        v["hookSpecificOutput"]["updatedInput"]["command"]
            .as_str()
            .unwrap()
            .contains("trs "),
        "{cmd}"
    );
}

#[test]
fn forbidden_prompt_and_allow_rules_stop_the_rewrite() {
    for cmd in [
        "git push origin main",
        "git commit -m x",
        "cargo test --lib",
        "git status && git push",
        "FOO=1 git push",
    ] {
        assert!(hook(Some(RULES), cmd).is_none(), "{cmd}");
    }
}

#[test]
fn commands_no_rule_matches_are_rewritten_with_the_required_allow() {
    for cmd in ["git status", "cargo build", "git log -3"] {
        let v = hook(Some(RULES), cmd).unwrap_or_else(|| panic!("{cmd}"));
        assert_rewritten(&v, cmd);
    }
    assert_rewritten(&hook(None, "git status").unwrap(), "no rules at all");
}

#[test]
fn an_unreadable_or_unknown_rules_file_leaves_the_command_alone() {
    for bad in [
        "prefix_rule(pattern=[",
        "host_executable(name=\"git\", paths=[\"/usr/bin/git\"])",
    ] {
        assert!(hook(Some(bad), "git status").is_none(), "{bad}");
    }
}

#[test]
fn project_rules_count_only_for_trusted_projects() {
    let home = tempfile::tempdir().unwrap();
    let codex = home.path().join("codex");
    let proj = home.path().join("proj");
    write(&proj, ".codex/rules/p.rules", RULES);
    std::fs::create_dir_all(&codex).unwrap();

    let v = run(&home, &codex, "git push", &proj);
    assert!(v.is_some(), "untrusted project rules are ignored by Codex");

    let key = proj.display().to_string().replace('\\', "\\\\");
    write(
        &codex,
        "config.toml",
        &format!("[projects.\"{key}\"]\ntrust_level = \"trusted\"\n"),
    );
    assert!(run(&home, &codex, "git push", &proj).is_none());
    assert!(run(&home, &codex, "git status", &proj).is_some());
}
