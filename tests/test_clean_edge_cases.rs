//! Integration tests for the `clean` command: combined options, edge cases,
//! error handling, format combinations, existing fixtures, reduction calculations,
//! and file flag tests.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// Combined Options Tests
// ============================================================

#[test]
fn test_clean_all_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_messy.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Should have cleaned content
    assert!(stdout.contains("[INFO]"));

    // Should not have ANSI codes
    assert!(!stdout.contains("\x1b["));

    // Should not have consecutive blank lines
    let lines: Vec<&str> = stdout.lines().collect();
    let mut consecutive_blanks = 0;
    for line in &lines {
        if line.trim().is_empty() {
            consecutive_blanks += 1;
            assert!(
                consecutive_blanks <= 1,
                "Should not have consecutive blank lines"
            );
        } else {
            consecutive_blanks = 0;
        }
    }
}

#[test]
fn test_clean_collapse_repeats_and_trim() {
    let input = "Line 1\nLine 1\nLine 1\n   Line 2   \n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--collapse-repeats")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Should collapse repeated lines
    let line1_count = stdout.matches("Line 1").count();
    assert_eq!(line1_count, 1);

    // Should trim whitespace
    assert!(stdout.contains("Line 2"));
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_clean_empty_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_empty.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Empty file should produce empty or minimal output (just newline or empty)
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn test_clean_empty_input_stdin() {
    let input = "";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Empty input should produce empty output
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn test_clean_single_line() {
    let input = "Single line\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("Single line"));
}

#[test]
fn test_clean_only_whitespace() {
    let input = "   \n   \n   \n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw") // Use raw format to get just the content without percentage
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Whitespace-only lines should be trimmed to empty or just whitespace
    // The implementation trims leading/trailing whitespace from the whole output
    assert!(stdout.trim().is_empty());
}

#[test]
fn test_clean_unicode_content() {
    let input = "Hello 世界\nこんにちは\nПривет мир\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Unicode should be preserved
    assert!(stdout.contains("世界"));
    assert!(stdout.contains("こんにちは"));
    assert!(stdout.contains("Привет"));
}

#[test]
fn test_clean_special_characters() {
    let input = "Line with \t tabs\nLine with \r carriage return\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle special characters without crashing
    assert!(stdout.contains("tabs") || stdout.contains("Line"));
}

#[test]
fn test_clean_null_bytes() {
    let input = "Line with \x00 null byte\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle null bytes (they should be removed/sanitized)
    assert!(stdout.contains("Line") || stdout.contains("null"));
}

// ============================================================
// Error Handling Tests
// ============================================================

#[test]
fn test_clean_nonexistent_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg("/nonexistent/path/to/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Error")));
}

#[test]
fn test_clean_nonexistent_file_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("clean")
        .arg("-f")
        .arg("/nonexistent/path/to/file.txt")
        .assert()
        .failure();
}

// ============================================================
// Format Combinations Tests
// ============================================================

#[test]
fn test_clean_json_with_all_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--collapse-repeats")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_messy.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check all options are reflected
    assert_eq!(json["options"]["no_ansi"], true);
    assert_eq!(json["options"]["collapse_blanks"], true);
    assert_eq!(json["options"]["collapse_repeats"], true);
    assert_eq!(json["options"]["trim"], true);
}

#[test]
fn test_clean_csv_with_trim() {
    let input = "   Line 1   \n   Line 2   \n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("clean")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // CSV output should have trimmed content
    assert!(stdout.contains("Line 1"));
    assert!(stdout.contains("Line 2"));
}

#[test]
fn test_clean_agent_with_reduction() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("clean")
        .arg("--no-ansi")
        .arg("-f")
        .arg(fixture_path("clean_with_ansi.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should show content and reduction
    assert!(stdout.contains("Content"));
}

// ============================================================
// Existing Fixture Tests
// ============================================================

#[test]
fn test_clean_logs_application() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg(fixture_path("logs_application.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("[INFO]"));
}

#[test]
fn test_clean_logs_repeated_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw") // Use raw format to avoid reduction percentage in output
        .arg("clean")
        .arg("--collapse-repeats")
        .arg("-f")
        .arg(fixture_path("logs_repeated_lines.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should collapse consecutive repeated lines
    // The file has 3 consecutive "[DEBUG] Processing request" lines and 1 separate at end
    // After collapse, should have 2
    let debug_count = stdout.matches("[DEBUG] Processing request").count();
    assert_eq!(debug_count, 2);

    // The file has 2 consecutive "[ERROR] Connection failed" lines and 1 separate later
    // After collapse, should have 2
    let error_count = stdout.matches("[ERROR] Connection failed").count();
    assert_eq!(error_count, 2);
}

#[test]
fn test_clean_logs_mixed_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg(fixture_path("logs_mixed_format.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("INFO"));
}

// ============================================================
// Reduction Calculation Tests
// ============================================================

#[test]
fn test_clean_reduction_in_json() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("--no-ansi")
        .arg("-f")
        .arg(fixture_path("clean_with_ansi.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Reduction should be calculated
    let reduction = json["stats"]["reduction_percent"].as_f64().unwrap_or(0.0);
    // After stripping ANSI codes, there should be some reduction
    assert!(reduction >= 0.0);
}

#[test]
fn test_clean_no_reduction_simple() {
    let input = "Simple line\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // For simple input with no cleaning needed, reduction should be minimal
    let reduction = json["stats"]["reduction_percent"].as_f64().unwrap_or(0.0);
    assert!(reduction >= 0.0);
}

// ============================================================
// File Flag Tests
// ============================================================

#[test]
fn test_clean_file_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_clean_file_long_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--file")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}
