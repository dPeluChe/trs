//! Comprehensive integration tests for parser commands.
//!
//! This test module verifies the parse subcommands through the CLI:
//! - git-status parser
//! - git-diff parser
//! - ls parser
//! - logs parser
//! - grep parser

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
        .stdout(predicate::str::contains("branch:"))
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("status: clean"));
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

// ============================================================
// LS Parser Tests
// ============================================================

#[test]
fn test_parse_ls_empty() {
    let input = fixtures::ls_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"].as_bool().unwrap());
}

#[test]
fn test_parse_ls_simple() {
    let input = fixtures::ls_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["entries"].is_array());
    let entries = json["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn test_parse_ls_with_directories() {
    let input = fixtures::ls_with_directories();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["directories"].is_array());
    let dirs = json["directories"].as_array().unwrap();
    assert!(!dirs.is_empty());
}

#[test]
fn test_parse_ls_with_hidden() {
    let input = fixtures::ls_with_hidden();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["hidden"].is_array());
    let hidden = json["hidden"].as_array().unwrap();
    assert!(!hidden.is_empty());
}

#[test]
fn test_parse_ls_long_format() {
    let input = fixtures::ls_long_format();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Long format should still parse entries
    assert!(json["entries"].is_array());
}

#[test]
fn test_parse_ls_with_symlinks() {
    let input = fixtures::ls_long_format_with_symlinks();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["symlinks"].is_array());
    let symlinks = json["symlinks"].as_array().unwrap();
    assert!(!symlinks.is_empty());
}

#[test]
fn test_parse_ls_permission_denied() {
    let input = fixtures::ls_permission_denied();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["errors"].is_array());
    let errors = json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
}

#[test]
fn test_parse_ls_generated_dirs() {
    let input = fixtures::ls_generated_dirs();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should detect generated directories
    assert!(json["generated"].is_array());
}

#[test]
fn test_parse_ls_special_chars() {
    let input = fixtures::ls_special_chars();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file with spaces"));
}

// ============================================================
// Logs Parser Tests
// ============================================================

#[test]
fn test_parse_logs_empty() {
    let input = fixtures::logs_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    // Empty input should have zero counts
    assert_eq!(json["counts"]["total_lines"].as_u64().unwrap_or(0), 0);
}

#[test]
fn test_parse_logs_simple() {
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["entries"].is_array());
    let entries = json["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn test_parse_logs_level_counts() {
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["counts"]["info"].as_u64().unwrap() > 0);
    assert!(json["counts"]["error"].as_u64().unwrap() > 0);
    assert!(json["counts"]["warning"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_logs_all_levels() {
    let input = fixtures::logs_all_levels();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["counts"]["debug"].as_u64().unwrap() > 0);
    assert!(json["counts"]["info"].as_u64().unwrap() > 0);
    assert!(json["counts"]["warning"].as_u64().unwrap() > 0);
    assert!(json["counts"]["error"].as_u64().unwrap() > 0);
    assert!(json["counts"]["fatal"].as_u64().unwrap() > 0);
}

#[test]
fn test_parse_logs_errors_only() {
    let input = fixtures::logs_errors_only();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have errors
    assert!(json["counts"]["error"].as_u64().unwrap() > 0);
    // Should not have info
    assert_eq!(json["counts"]["info"].as_u64().unwrap(), 0);
}

#[test]
fn test_parse_logs_with_timestamps() {
    let input = fixtures::logs_iso8601_timestamp();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let entries = json["entries"].as_array().unwrap();
    // First entry should have a timestamp
    assert!(entries[0]["timestamp"].is_string());
}

#[test]
fn test_parse_logs_syslog_format() {
    let input = fixtures::logs_syslog_format();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should parse syslog format
    assert!(json["entries"].as_array().unwrap().len() > 0);
}

#[test]
fn test_parse_logs_repeated_lines() {
    let input = fixtures::logs_repeated_lines();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should track repeated lines
    assert!(json["repeated_lines"].is_array());
}

#[test]
fn test_parse_logs_recent_critical() {
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should track recent critical (error and fatal) entries
    assert!(json["recent_critical"].is_array());
}

#[test]
fn test_parse_logs_compact_format() {
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("lines:"));
}

#[test]
fn test_parse_logs_csv_format() {
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line_number,level,timestamp,message"));
}

// ============================================================
// Grep Parser Tests
// ============================================================

#[test]
fn test_parse_grep_empty() {
    let input = fixtures::grep_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"].as_bool().unwrap());
}

#[test]
fn test_parse_grep_simple() {
    let input = fixtures::grep_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
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
    assert!(!files.is_empty());
}

#[test]
fn test_parse_grep_multiple_files() {
    let input = fixtures::grep_multiple_files();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    assert!(files.len() >= 2);
}

#[test]
fn test_parse_grep_with_column() {
    let input = fixtures::grep_with_column();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().unwrap();
    let matches = files[0]["matches"].as_array().unwrap();
    // Should have column information
    assert!(matches[0]["column"].is_number());
}

#[test]
fn test_parse_grep_context_lines() {
    let input = fixtures::grep_context_lines();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should parse context lines
    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty());
}

#[test]
fn test_parse_grep_binary_file() {
    let input = fixtures::grep_binary_file();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should handle binary file indicator - either as has_binary or as a file entry
    let has_binary = json["has_binary"].as_bool().unwrap_or(false);
    let has_files = json["files"].as_array().map_or(false, |f| !f.is_empty());
    assert!(has_binary || has_files, "Expected binary file indicator or files in output");
}

#[test]
fn test_parse_grep_special_chars() {
    let input = fixtures::grep_special_chars();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file with spaces"));
}

#[test]
fn test_parse_grep_ripgrep_heading() {
    let input = fixtures::grep_ripgrep_heading();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should handle ripgrep heading format
    assert!(json["files"].as_array().unwrap().len() > 0);
}

// ============================================================
// Parser Format Consistency Tests
// ============================================================

#[test]
fn test_parser_all_formats_git_status() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("branch:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"branch\""));

    // Test CSV format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success();

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("branch:"));
}

#[test]
fn test_parser_all_formats_git_diff() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_parser_all_formats_ls() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("total:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("total:"));
}

#[test]
fn test_parser_all_formats_logs() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("lines:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("lines:"));
}

#[test]
fn test_parser_all_formats_grep() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_parser_handles_unicode() {
    let input = "src/unicode_ñame.rs:42:const greeting = \"Hello 世界\";";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("unicode"));
}

#[test]
fn test_parser_handles_empty_lines() {
    let input = "\n\nsrc/main.rs:42:fn main() {}\n\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should still parse the one valid line
    assert!(json["files"].as_array().unwrap().len() > 0);
}

#[test]
fn test_parser_large_input() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_large())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_parser_long_paths() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_long_paths())
        .assert()
        .success()
        .stdout(predicate::str::contains("nested"));
}

// ============================================================
// Stats Flag Tests
// ============================================================

#[test]
fn test_parser_stats_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

#[test]
fn test_parser_stats_git_diff() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stderr(predicate::str::contains("Files changed:"));
}

#[test]
fn test_parser_stats_ls() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stderr(predicate::str::contains("Files:"));
}

#[test]
fn test_parser_stats_logs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}
