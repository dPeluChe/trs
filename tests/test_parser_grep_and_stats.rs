//! Integration tests for grep parser.
//!
//! Covers:
//! - Grep parser: empty, simple, multiple files, column, context,
//!   binary, special chars, ripgrep heading

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

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
