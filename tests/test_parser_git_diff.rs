//! Integration tests for git-diff parser.
//!
//! Covers:
//! - Empty diff
//! - Modified, added, deleted, renamed, copied, binary diffs
//! - Multiple files diff
//! - Additions and deletions tracking

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

// ============================================================
// Git Diff Parser Tests
// ============================================================

#[test]
fn test_parse_git_diff_empty() {
    let input = fixtures::git_diff_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""is_empty":true"#));
}

#[test]
fn test_parse_git_diff_modified() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["files"].is_array());
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["change_type"], "M");
    assert!(files[0]["path"].as_str().unwrap().contains("main.rs"));
}

#[test]
fn test_parse_git_diff_added() {
    let input = fixtures::git_diff_added();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert_eq!(files[0]["change_type"], "A");
}

#[test]
fn test_parse_git_diff_deleted() {
    let input = fixtures::git_diff_deleted();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert_eq!(files[0]["change_type"], "D");
}

#[test]
fn test_parse_git_diff_renamed() {
    let input = fixtures::git_diff_renamed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert_eq!(files[0]["change_type"], "R");
    assert!(files[0]["new_path"].is_string());
}

#[test]
fn test_parse_git_diff_copied() {
    let input = fixtures::git_diff_copied();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert_eq!(files[0]["change_type"], "C");
}

#[test]
fn test_parse_git_diff_binary() {
    let input = fixtures::git_diff_binary();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert!(files[0]["is_binary"].as_bool().unwrap());
}

#[test]
fn test_parse_git_diff_multiple() {
    let input = fixtures::git_diff_multiple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert!(files.len() >= 3);
}

#[test]
fn test_parse_git_diff_additions_deletions() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have tracked additions
    assert!(json["total_additions"].as_u64().unwrap() > 0);
}
