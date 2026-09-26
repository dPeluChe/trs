//! Two install bugs found through a user whose Copilot CLI never ran trs.

use assert_cmd::Command;
use std::path::Path;

fn trs(home: &Path, args: &[&str]) {
    Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .output()
        .unwrap();
}

/// A project hook in `~/.github/hooks/` reported Copilot as configured, so
/// `init --all --global` (and every `trs upgrade`) skipped the global hook
/// Copilot actually reads.
#[test]
fn a_project_hook_in_home_does_not_stand_in_for_the_global_one() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    std::fs::create_dir_all(h.join(".copilot")).unwrap();
    std::fs::create_dir_all(h.join(".github/hooks")).unwrap();
    std::fs::write(
        h.join(".github/hooks/trs.json"),
        r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"trs rewrite --caller vscode"}]}]}}"#,
    )
    .unwrap();
    trs(h, &["init", "--all", "--global"]);
    let global = h.join(".copilot/hooks/trs.json");
    assert!(global.exists(), "global Copilot hook was not installed");
    assert!(std::fs::read_to_string(global)
        .unwrap()
        .contains("trs rewrite"));
}

/// Antigravity's legacy scrub removed every `trs rewrite` entry from
/// `~/.gemini/settings.json`, which is also where Gemini CLI's hook lives.
#[test]
fn installing_antigravity_keeps_the_gemini_hook() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    std::fs::create_dir_all(h.join(".gemini")).unwrap();
    trs(h, &["init", "gemini", "--global"]);
    let settings = h.join(".gemini/settings.json");
    assert!(
        std::fs::read_to_string(&settings)
            .unwrap()
            .contains("trs rewrite"),
        "gemini hook not installed"
    );
    trs(h, &["init", "antigravity", "--global"]);
    assert!(
        std::fs::read_to_string(&settings)
            .unwrap()
            .contains("trs rewrite"),
        "installing Antigravity removed Gemini's hook"
    );
}
