//! `trs stats` against its own history file, so the verdict does not depend
//! on what earlier tests or the user left behind.

use assert_cmd::Command;

fn trs(home: &std::path::Path, args: &[&str]) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .current_dir(home)
        .env("TRS_HOME", home)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn test_stats_without_history_says_so() {
    let home = tempfile::tempdir().unwrap();
    for args in [&["stats"][..], &["stats", "--by-command"]] {
        assert!(trs(home.path(), args).contains("No history yet"));
    }
}

#[test]
fn test_stats_by_command_lists_each_command() {
    let home = tempfile::tempdir().unwrap();
    trs(home.path(), &["ls"]);
    let text = trs(home.path(), &["stats", "--by-command"]);
    for column in ["COMMAND", "CALLS", "SHARE", "SAVED", "ls"] {
        assert!(text.contains(column), "lost {column}: {text}");
    }
}

#[test]
fn test_stats_default_does_not_show_by_command_table() {
    let home = tempfile::tempdir().unwrap();
    trs(home.path(), &["ls"]);
    let text = trs(home.path(), &["stats"]);
    assert!(text.contains("Total commands:"), "{text}");
    assert!(!text.contains("COMMAND"), "{text}");
}
