//! Gemini matches its rules against the REWRITTEN command and ignores a hook
//! `allow`, so the hook applies the user's TOML policies and `tools.allowed`
//! to the original one. Verified on Gemini CLI 0.63 with a rewriting hook.

mod support;

use serde_json::{json, Value};
use std::path::Path;
use support::run_hook;

const DENY: &str = "[[rule]]\ntoolName = \"run_shell_command\"\ncommandPrefix = [\"git push\", \"rm -rf\"]\ndecision = \"deny\"\npriority = 900\n";

/// Run `trs rewrite --caller gemini` with `files` under a temp `GEMINI_CLI_HOME/.gemini`.
fn hook_in(home: &Path, cwd: &Path, cmd: &str, extra_env: &[(&str, &str)]) -> Option<Value> {
    let input = json!({
        "hook_event_name": "BeforeTool",
        "tool_name": "run_shell_command",
        "tool_input": { "command": cmd },
        "cwd": cwd,
    });
    let mut envs = vec![("GEMINI_CLI_HOME", home)];
    envs.extend(extra_env.iter().map(|(k, v)| (*k, Path::new(v))));
    run_hook(
        "gemini",
        &input,
        home,
        &envs,
        &["GEMINI_CLI_TRUST_WORKSPACE", "GEMINI_RESTRICTED_MODE"],
        None,
    )
}

fn setup(settings: Option<&str>, policy: Option<&str>) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".gemini");
    std::fs::create_dir_all(dir.join("policies")).unwrap();
    if let Some(s) = settings {
        std::fs::write(dir.join("settings.json"), s).unwrap();
    }
    if let Some(p) = policy {
        std::fs::write(dir.join("policies/user.toml"), p).unwrap();
    }
    home
}

fn hook(home: &tempfile::TempDir, cmd: &str) -> Option<Value> {
    hook_in(home.path(), home.path(), cmd, &[])
}

fn rewritten(v: &Value) -> &str {
    v["hookSpecificOutput"]["tool_input"]["command"]
        .as_str()
        .unwrap()
}

#[test]
fn toml_deny_stops_the_rewrite() {
    let home = setup(None, Some(DENY));
    for cmd in [
        "git push origin main",
        "FOO=1 git push",
        "cargo build && rm -rf x",
    ] {
        assert!(hook(&home, cmd).is_none(), "{cmd}");
    }
}

#[test]
fn rewrite_prints_no_decision_at_all() {
    let home = setup(None, Some(DENY));
    let v = hook(&home, "cargo build").unwrap();
    assert!(rewritten(&v).contains("trs "));
    assert!(
        v.get("decision").is_none(),
        "Gemini ignores allow; never send one"
    );
}

#[test]
fn narrow_allow_rules_would_deny_the_rewrite_so_it_is_skipped() {
    let narrow = r#"{"tools":{"allowed":["run_shell_command(cargo test)"]}}"#;
    let home = setup(Some(narrow), None);
    assert!(hook(&home, "cargo test").is_none());
    let blanket = r#"{"tools":{"allowed":["run_shell_command"]}}"#;
    assert!(hook(&setup(Some(blanket), None), "cargo test").is_some());
}

#[test]
fn read_only_commands_keep_geminis_own_auto_allow() {
    let home = setup(None, None);
    assert!(hook(&home, "git status").is_none());
    assert!(hook(&home, "cargo build").is_some());
}

#[test]
fn unreadable_rules_fail_closed() {
    assert!(hook(&setup(Some("{ nope"), None), "cargo build").is_none());
    assert!(hook(&setup(None, Some("[[rule")), "cargo build").is_none());
}

/// A repo's own `.gemini/settings.json` must not widen what the hook approves
/// unless Gemini itself trusts the folder.
#[test]
fn project_allow_counts_only_in_a_trusted_folder() {
    let home = setup(
        Some(r#"{"tools":{"allowed":["run_shell_command(cargo test)"]}}"#),
        None,
    );
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join(".gemini")).unwrap();
    std::fs::write(
        project.path().join(".gemini/settings.json"),
        r#"{"tools":{"allowed":["run_shell_command"]}}"#,
    )
    .unwrap();
    let run = |env: &[(&str, &str)]| hook_in(home.path(), project.path(), "cargo build", env);
    assert!(
        run(&[]).is_none(),
        "untrusted: the user's narrow list rules"
    );
    assert!(run(&[("GEMINI_CLI_TRUST_WORKSPACE", "true")]).is_some());
    assert!(run(&[("GEMINI_CLI_TRUST_WORKSPACE", "false")]).is_none());
}

#[test]
fn other_agents_are_not_gated_by_gemini_rules() {
    let home = setup(None, Some(DENY));
    let input = json!({"hook_event_name":"preToolUse","tool_input":{"command":"git push"}});
    let out = run_hook(
        "cursor",
        &input,
        home.path(),
        &[("GEMINI_CLI_HOME", home.path())],
        &[],
        None,
    );
    assert!(out.unwrap().to_string().contains("trs git push"));
}
