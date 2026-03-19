//! Integration tests for the `tail` command - basic operations, line count,
//! error filtering, and output format variations (JSON, CSV, TSV, Agent, Raw, Compact).

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
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
    assert!(
        stdout.contains("ERROR")
            || stdout.contains("Exception")
            || stdout.contains("FATAL")
            || stdout.contains("failed")
            || stdout.contains("ERR")
    );
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
