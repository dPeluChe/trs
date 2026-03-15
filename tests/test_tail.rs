//! Comprehensive integration tests for the `tail` command.
//!
//! This test module verifies the tail functionality through the CLI:
//! - Basic tail operation
//! - Line count options (-n, --lines)
//! - Error filtering (--errors)
//! - Output format variations (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Stats output
//! - Edge cases (empty files, small files, special characters)
//! - Error handling (non-existent files)

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixtures").join(name)
}

// ============================================================
// Basic Tail Tests
// ============================================================

#[test]
fn test_tail_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .stdout(predicate::str::contains("tail_app.log"));
}

#[test]
fn test_tail_returns_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .stdout(predicate::str::contains("[INFO]"));
}

#[test]
fn test_tail_shows_line_numbers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Line numbers should appear as "number:" format
    assert!(stdout.contains(':'));
}

#[test]
fn test_tail_default_10_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain line 20 (last line) since file has 20 lines and we show last 10
    assert!(stdout.contains("Line 20"));
    // Should not contain line 10 (too early)
    assert!(!stdout.contains("Line 10 -"));
}

// ============================================================
// Line Count Options Tests
// ============================================================

#[test]
fn test_tail_with_n_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("-n")
        .arg("5")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain last 5 lines
    assert!(stdout.contains("Line 16"));
    assert!(stdout.contains("Line 20"));
    // Should not contain earlier lines
    assert!(!stdout.contains("Line 14"));
}

#[test]
fn test_tail_with_lines_long_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("--lines")
        .arg("3")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain last 3 lines
    assert!(stdout.contains("Line 18"));
    assert!(stdout.contains("Line 19"));
    assert!(stdout.contains("Line 20"));
    // Should not contain earlier lines
    assert!(!stdout.contains("Line 16"));
}

#[test]
fn test_tail_shorthand_minus_n() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("-5")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should show last 5 lines (same as -n 5)
    assert!(stdout.contains("Line 16"));
    assert!(stdout.contains("Line 20"));
}

#[test]
fn test_tail_n_greater_than_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("-n")
        .arg("100")
        .arg(fixture_path("tail_small.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should show the single line in the file
    assert!(stdout.contains("Single line log"));
}

#[test]
fn test_tail_n_one() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("-n")
        .arg("1")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain only the last line
    assert!(stdout.contains("Line 20"));
    // Should not contain earlier lines
    assert!(!stdout.contains("Line 19"));
}

// ============================================================
// Error Filtering Tests
// ============================================================

#[test]
fn test_tail_errors_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain error lines
    assert!(stdout.contains("ERROR") || stdout.contains("FATAL"));
}

#[test]
fn test_tail_errors_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("-e")
        .arg(fixture_path("tail_mixed.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should only contain error-related lines
    assert!(stdout.contains("ERROR") || stdout.contains("Exception") || stdout.contains("FATAL") || stdout.contains("failed") || stdout.contains("ERR"));
}

#[test]
fn test_tail_errors_no_errors_in_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_small.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should indicate no errors found (or empty output)
    // The single line is INFO, not an error
    assert!(!stdout.contains("[INFO]"));
}

#[test]
fn test_tail_errors_detects_various_error_patterns() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_mixed.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should detect various error patterns
    // File contains: ERROR, Exception, failed, FATAL, ERR
    let error_count = ["ERROR", "Exception", "failed", "FATAL", "ERR"]
        .iter()
        .filter(|&pattern| stdout.contains(pattern))
        .count();
    assert!(error_count >= 3, "Should detect at least 3 error patterns");
}

// ============================================================
// JSON Format Tests
// ============================================================

#[test]
fn test_tail_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check structure
    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
    assert!(json["total_lines"].is_number());
    assert!(json["lines_shown"].is_number());
    assert!(json["filtering_errors"].is_boolean());
}

#[test]
fn test_tail_json_lines_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("-n")
        .arg("5")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let lines = json["lines"].as_array().expect("lines should be array");
    assert!(!lines.is_empty());

    // Check line structure
    for line in lines {
        assert!(line["line_number"].is_number());
        assert!(line["line"].is_string());
        assert!(line["is_error"].is_boolean());
    }
}

#[test]
fn test_tail_json_with_errors_filter() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert_eq!(json["filtering_errors"], true);
    // All lines should be errors
    let lines = json["lines"].as_array().expect("lines should be array");
    for line in lines {
        assert_eq!(line["is_error"], true);
    }
}

// ============================================================
// CSV Format Tests
// ============================================================

#[test]
fn test_tail_csv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header
    assert!(lines[0].contains("line_number"));
    assert!(lines[0].contains("line"));
    assert!(lines[0].contains("is_error"));

    // Should have data rows
    assert!(lines.len() > 1);
}

#[test]
fn test_tail_csv_with_special_chars() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle special characters without crashing
    assert!(stdout.contains("line_number"));
}

// ============================================================
// TSV Format Tests
// ============================================================

#[test]
fn test_tail_tsv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header with tabs
    assert!(lines[0].contains('\t'));
    assert!(lines[0].contains("line_number"));
    assert!(lines[0].contains("is_error"));
}

#[test]
fn test_tail_tsv_escaped_tabs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle tabs in content without crashing
    assert!(stdout.contains("line_number"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_tail_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should show file path
    assert!(stdout.contains("File:"));
    // Agent format should show lines info
    assert!(stdout.contains("Lines:") || stdout.contains("total"));
}

#[test]
fn test_tail_agent_error_indicators() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format uses ❌ for error lines
    // The file contains ERROR and FATAL lines
    assert!(stdout.contains('❌'));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_tail_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show line numbers and content
    assert!(stdout.contains(':'));
    // Should not have headers or fancy formatting
    assert!(!stdout.contains("File:"));
}

#[test]
fn test_tail_raw_no_headers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg("-n")
        .arg("3")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should just be line_number:content
    let lines: Vec<&str> = stdout.lines().collect();
    for line in lines {
        // Each line should start with a number
        let first_char = line.chars().next().unwrap();
        assert!(first_char.is_ascii_digit());
    }
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_tail_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format shows header with file info
    assert!(stdout.contains("Last") || stdout.contains("lines"));
}

#[test]
fn test_tail_compact_error_indicators() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format uses ❌ for error lines
    assert!(stdout.contains('❌'));
}

// ============================================================
// Stats Tests
// ============================================================

#[test]
fn test_tail_with_stats() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Stats are printed to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should show reducer name
    assert!(stderr.contains("tail"));
}

#[test]
fn test_tail_stats_with_errors_filter() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Stats are printed to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should mention filtering
    assert!(stderr.contains("Filtering errors") || stderr.contains("Items filtered"));
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_tail_empty_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_empty.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should indicate file is empty
    assert!(stdout.contains("empty") || stdout.contains("Empty") || stdout.lines().count() <= 2);
}

#[test]
fn test_tail_single_line_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_small.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain the single line
    assert!(stdout.contains("Single line log"));
}

#[test]
fn test_tail_with_unicode() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle unicode properly
    assert!(stdout.contains("café") || stdout.contains("Unicode"));
}

#[test]
fn test_tail_with_quotes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle quotes properly
    assert!(stdout.contains("quotes"));
}

#[test]
fn test_tail_with_json_content() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle JSON content in lines
    assert!(stdout.contains("JSON") || stdout.contains("error"));
}

// ============================================================
// Error Handling Tests
// ============================================================

#[test]
fn test_tail_nonexistent_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/nonexistent/path/to/file.log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Error")));
}

#[test]
fn test_tail_nonexistent_file_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("tail")
        .arg("/nonexistent/path/to/file.log")
        .assert()
        .failure();
}

// ============================================================
// Format Combinations Tests
// ============================================================

#[test]
fn test_tail_json_with_line_count() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("-n")
        .arg("3")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have exactly 3 lines
    let lines = json["lines"].as_array().expect("lines should be array");
    assert_eq!(lines.len(), 3);
    assert_eq!(json["lines_shown"], 3);
}

#[test]
fn test_tail_csv_with_errors() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // All data rows should have is_error = true
    for line in lines.iter().skip(1) {
        // The last field should be true (is_error column)
        assert!(line.ends_with(",true"), "Line should end with ,true: {}", line);
    }
}

#[test]
fn test_tail_tsv_with_line_count() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg("-n")
        .arg("5")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header + 5 data rows = 6 lines
    assert_eq!(lines.len(), 6);
}

// ============================================================
// Existing Fixture Tests
// ============================================================

#[test]
fn test_tail_logs_application() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(fixture_path("logs_application.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("logs_application"));
}

#[test]
fn test_tail_logs_errors_only() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("logs_errors_only.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // All lines in this file are errors
    assert!(stdout.contains("ERROR") || stdout.contains("Exception") || stdout.contains("Failed"));
}

#[test]
fn test_tail_logs_simple() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("tail")
        .arg(fixture_path("logs_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain log entries
    assert!(stdout.contains("INFO") || stdout.contains("ERROR") || stdout.contains("FATAL"));
}
