//! Integration tests for ls and logs parsers.
//!
//! Covers:
//! - LS parser: empty, simple, directories, hidden, long format,
//!   symlinks, permissions, generated dirs, special chars
//! - Logs parser: empty, simple, level counts, all levels,
//!   errors only, timestamps, syslog, repeated, recent critical,
//!   compact and CSV formats

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

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
