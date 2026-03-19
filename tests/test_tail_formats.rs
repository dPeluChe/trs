//! Integration tests for the `tail` command - stats output, edge cases,
//! error handling, format combinations, and existing fixture tests.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
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
    assert!(stdout.contains("caf\u{00E9}") || stdout.contains("Unicode"));
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
