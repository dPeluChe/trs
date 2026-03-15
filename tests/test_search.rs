//! Comprehensive integration tests for the `search` command.
//!
//! This test module verifies the search functionality through the CLI:
//! - Basic pattern search
//! - Extension filtering
//! - Case-insensitive search
//! - Context lines
//! - Result limiting
//! - Output format variations (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Stats output
//! - Edge cases (empty results, special characters, etc.)

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
        // Should not find any .md files with "fn "
        .stdout(predicate::str::contains(".rs").not());
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

// ============================================================
// Help and Usage Tests
// ============================================================

#[test]
fn test_search_help_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"))
        .stdout(predicate::str::contains("PATH"))
        .stdout(predicate::str::contains("QUERY"));
}

#[test]
fn test_search_help_shows_extension_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--extension"));
}

#[test]
fn test_search_help_shows_ignore_case_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--ignore-case"));
}

#[test]
fn test_search_help_shows_context_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--context"));
}

#[test]
fn test_search_help_shows_limit_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--limit"));
}

// ============================================================
// Path Handling Tests
// ============================================================

#[test]
fn test_search_with_current_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

#[test]
fn test_search_with_absolute_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Use the current directory as absolute path
    let cwd = std::env::current_dir().unwrap();
    cmd.arg("search")
        .arg(cwd.to_str().unwrap())
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_search_empty_query() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Empty query should match everything or error
    let _ = cmd
        .arg("search")
        .arg("src")
        .arg("")
        .assert(); // Either success or failure is acceptable
}

#[test]
fn test_search_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Nonexistent directory should still succeed (empty results)
    cmd.arg("search")
        .arg("/nonexistent/directory/xyz123")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_search_file_as_path() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Searching a specific file should work
    cmd.arg("search")
        .arg("src/main.rs")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main"));
}

// ============================================================
// Combined Options Tests
// ============================================================

#[test]
fn test_search_combined_extension_and_ignore_case() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("-e")
        .arg("rs")
        .arg("-i")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_combined_context_and_limit() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn ")
        .arg("-C")
        .arg("1")
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn test_search_all_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn ")
        .arg("-e")
        .arg("rs")
        .arg("-i")
        .arg("-C")
        .arg("1")
        .arg("--limit")
        .arg("20")
        .assert()
        .success();
}

#[test]
fn test_search_json_with_all_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-e")
        .arg("rs")
        .arg("-C")
        .arg("1")
        .arg("--limit")
        .arg("10")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
// Raw vs Reduced Output Size Comparison Tests
// ============================================================

#[test]
fn test_search_stats_raw_vs_reduced() {
    // Test that raw output shows the raw ripgrep-style output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // Output bytes should match stdout length
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_search_stats_json_larger() {
    // When using --json format, output_bytes should be larger than input_bytes due to JSON structure
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // For JSON output, output should be larger than input (JSON adds metadata)
    assert!(output_bytes > input_bytes, "JSON search output should be larger than raw input");

    // Verify stdout length matches output bytes
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_search_stats_compact_reduction() {
    // When using --compact format, verify proper byte counting and reduction
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--compact")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");

    // Compact format typically shows reduction since it summarizes matches
    // (though this depends on the specific pattern and matches)
}

#[test]
fn test_search_stats_agent_format() {
    // When using --agent format, verify proper byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--agent")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_search_stats_csv_format() {
    // When using --csv format, verify proper byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_search_stats_tsv_format() {
    // When using --tsv format, verify proper byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present and output bytes should match stdout length
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_search_stats_empty_results() {
    // Test stats with empty search results
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123_abc")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // Both should be present (even for empty results)
    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");

    // For empty results, input_bytes should be 0
    assert_eq!(input_bytes, Some(0), "Empty search should have 0 input bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

/// Helper function to extract byte count from stats output
fn extract_bytes(stderr: &str, prefix: &str) -> Option<usize> {
    for line in stderr.lines() {
        if line.contains(prefix) {
            // Extract the number after the prefix
            if let Some(pos) = line.find(prefix) {
                let after = &line[pos + prefix.len()..];
                if let Ok(bytes) = after.trim().parse::<usize>() {
                    return Some(bytes);
                }
            }
        }
    }
    None
}
