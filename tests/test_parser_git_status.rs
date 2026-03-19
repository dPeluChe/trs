//! Integration tests for git-status parser.
//!
//! Covers:
//! - Clean, staged, unstaged, untracked states
//! - Mixed, ahead, behind, diverged states
//! - Conflict, detached, renamed states
//! - Localization (Spanish, German)
//! - Empty input, porcelain format

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

// ============================================================
// Git Status Parser Tests
// ============================================================

#[test]
fn test_parse_git_status_clean() {
    let input = fixtures::git_status_clean();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("main").or(predicate::str::contains("clean")))
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn test_parse_git_status_clean_json() {
    let input = fixtures::git_status_clean();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert_eq!(json["branch"], "main");
    assert!(json["is_clean"].as_bool().unwrap());
}

#[test]
fn test_parse_git_status_staged() {
    let input = fixtures::git_status_staged();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("staged"));
}

#[test]
fn test_parse_git_status_staged_json() {
    let input = fixtures::git_status_staged();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["staged"].is_array());
    let staged = json["staged"].as_array().unwrap();
    assert!(!staged.is_empty());
}

#[test]
fn test_parse_git_status_unstaged() {
    let input = fixtures::git_status_unstaged();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("unstaged"));
}

#[test]
fn test_parse_git_status_untracked() {
    let input = fixtures::git_status_untracked();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("untracked"));
}

#[test]
fn test_parse_git_status_mixed() {
    let input = fixtures::git_status_mixed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["staged"].as_array().unwrap().len() > 0);
    assert!(json["unstaged"].as_array().unwrap().len() > 0);
    assert!(json["untracked"].as_array().unwrap().len() > 0);
}

#[test]
fn test_parse_git_status_ahead() {
    let input = fixtures::git_status_ahead();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["ahead"].is_number());
    assert!(json["ahead"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_git_status_behind() {
    let input = fixtures::git_status_behind();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["behind"].is_number());
    assert!(json["behind"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_git_status_diverged() {
    let input = fixtures::git_status_diverged();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["ahead"].is_number());
    assert!(json["behind"].is_number());
    assert!(json["ahead"].as_u64().unwrap() > 0);
    assert!(json["behind"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_git_status_conflict() {
    let input = fixtures::git_status_conflict();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["unmerged"].is_array());
    assert!(!json["unmerged"].as_array().unwrap().is_empty());
}

#[test]
fn test_parse_git_status_detached() {
    let input = fixtures::git_status_detached();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["branch"].as_str().unwrap().contains("HEAD detached"));
}

#[test]
fn test_parse_git_status_renamed() {
    let input = fixtures::git_status_renamed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let staged = json["staged"].as_array().unwrap();
    assert!(!staged.is_empty());
}

#[test]
fn test_parse_git_status_spanish() {
    let input = fixtures::git_status_spanish_clean();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Spanish localization should parse branch name correctly
    assert_eq!(json["branch"], "main");
}

#[test]
fn test_parse_git_status_german() {
    let input = fixtures::git_status_german_clean();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["is_clean"].as_bool().unwrap());
}

#[test]
fn test_parse_git_status_empty() {
    let input = fixtures::git_status_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    // Empty input should be parsed as clean with no changes
    assert!(json["is_clean"].as_bool().unwrap());
    assert!(json["staged"].as_array().unwrap().is_empty());
}

#[test]
fn test_parse_git_status_porcelain() {
    let input = fixtures::git_status_porcelain();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have parsed the porcelain format
    assert!(!json["is_clean"].as_bool().unwrap());
}
