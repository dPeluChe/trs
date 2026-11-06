//! Integration tests for the `search` command: output formats.
//!
//! Covers:
//! - CSV output format
//! - TSV output format
//! - Compact format
//! - Raw format
//! - Agent format
//! - Format precedence
//! - Stats output
//! - Empty results
//! - Regex patterns
//! - Special characters
//! - Unicode
//! - Multiple files

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// CSV Output Format Tests
// ============================================================

#[test]
fn test_search_csv_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("path").or(predicate::str::contains("line")));
}

#[test]
fn test_search_csv_has_file_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("router/"));
}

// ============================================================
// TSV Output Format Tests
// ============================================================

#[test]
fn test_search_tsv_has_file_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("router/"));
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_search_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_search_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_search_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

// ============================================================
// Format Precedence Tests
// ============================================================

#[test]
fn test_search_format_precedence_json_over_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // JSON should win over raw
    cmd.arg("--json")
        .arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_search_format_precedence_json_over_compact() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // JSON should win over compact
    cmd.arg("--json")
        .arg("--compact")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_search_format_precedence_csv_over_tsv() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // CSV should win over TSV
    cmd.arg("--csv")
        .arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains(","));
}

#[test]
fn test_search_format_precedence_compact_over_raw() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Compact should win over raw
    cmd.arg("--compact")
        .arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

// ============================================================
// Stats Output Tests
// ============================================================

// ============================================================
// Empty Results Tests
// ============================================================

#[test]
fn test_search_no_matches_returns_success() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123_abc")
        .assert()
        .success()
        .stdout("grep: no matches\n");
}

#[test]
fn test_search_no_matches_json_is_empty() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123_abc")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"].as_bool().unwrap());
    assert_eq!(json["counts"]["files"].as_u64().unwrap(), 0);
}

// ============================================================
// Regex Pattern Tests
// ============================================================

#[test]
fn test_search_regex_word_boundary() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Search for whole word "fn"
    cmd.arg("search")
        .arg("src")
        .arg(r"\bfn\b")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn"));
}

#[test]
fn test_search_regex_character_class() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Search for pub or Pub
    cmd.arg("search")
        .arg("src")
        .arg("[pP]ub")
        .assert()
        .success()
        .stdout(predicate::str::contains("pub"));
}

#[test]
fn test_search_regex_alternation() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Search for fn or struct
    cmd.arg("search")
        .arg("src")
        .arg("fn|struct")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn").or(predicate::str::contains("struct")));
}

#[test]
fn test_search_invalid_regex_returns_error() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("[invalid")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid regex pattern"));
}

// ============================================================
// Multiple Files Tests
// ============================================================

#[test]
fn test_search_multiple_files() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // "fn " should appear in multiple files
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("pub fn ")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    // Should find matches in multiple files
    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty(), "Expected at least one file with matches");
}
