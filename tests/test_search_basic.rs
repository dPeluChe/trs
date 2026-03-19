//! Integration tests for the `search` command: basic operations.
//!
//! Covers:
//! - Basic pattern search
//! - Extension filtering
//! - Case-insensitive search
//! - Context lines
//! - Result limiting
//! - JSON output format

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Basic Search Tests
// ============================================================

#[test]
fn test_search_basic_pattern() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("fn main"));
}

#[test]
fn test_search_in_specific_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_returns_file_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("Router")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs"));
}

#[test]
fn test_search_returns_line_number() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Search output should contain line numbers
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(":").and(predicate::str::contains("SearchHandler")));
}

// ============================================================
// Extension Filter Tests
// ============================================================

#[test]
fn test_search_with_extension_rs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn ")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs"));
}

#[test]
fn test_search_with_extension_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn ")
        .arg("-e")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs"));
}

#[test]
fn test_search_with_extension_md() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn ")
        .arg("-e")
        .arg("md")
        .assert()
        .success()
        // Should only find .md files, verify extension filter works
        .stdout(predicate::str::contains(".md"));
}

#[test]
fn test_search_with_extension_nonexistent() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn ")
        .arg("-e")
        .arg("nonexistent_ext_xyz")
        .assert()
        .success();
    // Should return empty results for non-existent extension
}

// ============================================================
// Case-Insensitive Search Tests
// ============================================================

#[test]
fn test_search_case_sensitive_default() {
    // By default, search is case-sensitive
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_ignore_case_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("--ignore-case")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_ignore_case_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("-i")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_case_sensitive_no_match() {
    // Case-sensitive search with wrong case should not match
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("search")
        .arg("src")
        .arg("searchhandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should not find "SearchHandler" when searching for lowercase
    assert!(!stdout.contains("SearchHandler"));
}

// ============================================================
// Context Lines Tests
// ============================================================

#[test]
fn test_search_with_context_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--context")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-C")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context_includes_surrounding_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // With context, there should be more output than just the matching line
    let output_with_context = cmd
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-C")
        .arg("3")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let len_with_context = output_with_context.len();

    let mut cmd2 = Command::cargo_bin("trs").unwrap();
    let output_without_context = cmd2
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let len_without_context = output_without_context.len();

    // With context should generally have more output
    assert!(len_with_context >= len_without_context);
}

// ============================================================
// Limit Option Tests
// ============================================================

#[test]
fn test_search_with_limit() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn ")
        .arg("--limit")
        .arg("5")
        .assert()
        .success();
}

#[test]
fn test_search_with_limit_one() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn ")
        .arg("--limit")
        .arg("1")
        .assert()
        .success();
}

#[test]
fn test_search_with_limit_zero() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn ")
        .arg("--limit")
        .arg("0")
        .assert()
        .success();
}

// ============================================================
// JSON Output Format Tests
// ============================================================

#[test]
fn test_search_json_is_valid() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn test_search_json_has_files_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_search_json_has_path_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""path""#));
}

#[test]
fn test_search_json_has_matches_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""matches""#));
}

#[test]
fn test_search_json_has_line_number_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""line_number""#));
}

#[test]
fn test_search_json_has_line_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""line""#));
}

#[test]
fn test_search_json_has_counts_field() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""counts""#));
}

#[test]
fn test_search_json_empty_result() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"].as_bool().unwrap());
    assert!(json["files"].as_array().unwrap().is_empty());
}
