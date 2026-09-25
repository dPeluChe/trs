//! Repeats and dense lines end to end, on commands present on every CI runner.

use assert_cmd::Command;
use std::path::Path;

fn git(dir: &Path, args: &[&str]) {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
}

#[test]
fn a_log_tail_folds_repeats_to_a_count() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("app.log");
    std::fs::write(&log, "boot\nretrying connection to db\nretrying connection to db\nretrying connection to db\nconnected\n").unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .arg("tail")
        .arg(&log)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(out.contains("retrying connection to db (x3)"), "{out}");
    assert!(out.contains("boot") && out.contains("connected"), "{out}");
}

/// A changed minified line keeps its ends, says what was cut, and the full
/// diff is saved with its path printed.
#[test]
fn a_minified_line_in_a_diff_is_cut_and_recoverable() {
    let repo = tempfile::tempdir().unwrap();
    let p = repo.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "t@example.com"]);
    git(p, &["config", "user.name", "t"]);
    std::fs::write(p.join("m.js"), "x\n").unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "base"]);
    let minified = "function Q(n){return n.match(Nr)||[]}var X,nn=\"4.18.1\",tn=200;".repeat(60);
    std::fs::write(p.join("m.js"), format!("{minified}\n")).unwrap();
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["git", "diff"])
        .current_dir(p)
        .env("HOME", home.path())
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(out.contains("…[minified,"), "dense line not cut:\n{out}");
    let saved = out
        .lines()
        .find_map(|l| l.strip_prefix("[trs] full output: "))
        .unwrap_or_else(|| panic!("no pointer to the full line:\n{out}"));
    assert!(std::fs::read_to_string(saved.trim())
        .unwrap()
        .contains(&minified));
}

#[test]
fn log_lines_that_differ_only_in_time_fold_with_their_range() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("runner.log");
    std::fs::write(
        &log,
        "[runner] 19:56:09 quota full for opencode\n[runner] 19:56:20 quota full for opencode\n[runner] 19:56:31 quota full for opencode\n[runner] 19:57:02 lease granted\n",
    )
    .unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .arg("tail")
        .arg(&log)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.contains("19:56:09 quota full for opencode (x3, until 19:56:31)"),
        "{out}"
    );
    assert!(out.contains("19:57:02 lease granted"), "{out}");
}
