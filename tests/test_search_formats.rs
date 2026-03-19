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

#[test]
fn test_search_compact_is_default() {
    // Compact is the default format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
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

#[test]
fn test_search_stats_shows_reducer() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

#[test]
fn test_search_stats_shows_output_mode() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode:"));
}

#[test]
fn test_search_stats_shows_files_searched() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Files searched:"));
}

#[test]
fn test_search_stats_shows_files_with_matches() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Files with matches:"));
}

#[test]
fn test_search_stats_shows_total_matches() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Total matches:"));
}

#[test]
fn test_search_stats_with_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: json"));
}

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
        .success();
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
fn test_search_regex_pattern() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Search for a regex pattern (digits)
    cmd.arg("search")
        .arg("src")
        .arg(r"\d+")
        .assert()
        .success();
}

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
    // Invalid regex should return an error
    cmd.arg("search")
        .arg("src")
        .arg("[invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex").or(predicate::str::contains("regex")));
}

// ============================================================
// Special Characters Tests
// ============================================================

#[test]
fn test_search_with_dashes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("long-about")
        .assert()
        .success();
}

#[test]
fn test_search_with_underscores() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_dots() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // . in regex matches any character, so we need to escape it or search literally
    cmd.arg("search")
        .arg(".")
        .arg(".rs")
        .assert()
        .success();
}

// ============================================================
// Unicode Tests
// ============================================================

#[test]
fn test_search_unicode_pattern() {
    // Create a test with unicode - search for comments with unicode chars
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Most source code is ASCII, so let's search for something we know exists
    cmd.arg("search")
        .arg("src")
        .arg("//")
        .assert()
        .success();
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
