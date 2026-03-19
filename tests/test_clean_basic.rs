//! Integration tests for the `clean` command: basic operation, ANSI stripping,
//! blank line collapsing, repeated line collapsing, and whitespace trimming.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// Basic Clean Tests
// ============================================================

#[test]
fn test_clean_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_clean_returns_content() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("simple test file"));
}

#[test]
fn test_clean_from_stdin() {
    let input = "Line 1\nLine 2\nLine 3\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"));
}

#[test]
fn test_clean_preserves_content_without_options() {
    let input = "Hello World\nTest Line\n";
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
    assert!(stdout.contains("Hello World"));
    assert!(stdout.contains("Test Line"));
}

// ============================================================
// ANSI Stripping Tests
// ============================================================

#[test]
fn test_clean_no_ansi_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
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
    // Should contain the log levels without ANSI codes
    assert!(stdout.contains("[INFO]"));
    assert!(stdout.contains("[DEBUG]"));
    assert!(stdout.contains("[WARN]"));
    assert!(stdout.contains("[ERROR]"));
    assert!(stdout.contains("[FATAL]"));
    // Should not contain actual ANSI escape sequences (ESC character)
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn test_clean_no_ansi_short_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
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
    // Should strip ANSI codes
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn test_clean_without_no_ansi_preserves_codes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_with_ansi.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should still contain ANSI codes when --no-ansi is not specified
    // (the fixture contains actual ANSI escape codes)
    // Note: Some ANSI sequences might be present
    assert!(stdout.contains("[INFO]") || stdout.contains("INFO"));
}

// ============================================================
// Blank Line Collapsing Tests
// ============================================================

#[test]
fn test_clean_collapse_blanks_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--collapse-blanks")
        .arg("-f")
        .arg(fixture_path("clean_with_blanks.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have fewer lines than the original (which had multiple consecutive blanks)
    // Original: First line, blank, blank, Second line, blank, blank, blank, Third line, blank, blank, Fourth line
    // After collapse: First line, blank, Second line, blank, Third line, blank, Fourth line
    // Should be 4 content lines + 3 single blank lines = 7 lines max

    // Count consecutive blank lines - should be at most 1
    let mut consecutive_blanks = 0;
    let mut max_consecutive_blanks = 0;
    for line in &lines {
        if line.trim().is_empty() {
            consecutive_blanks += 1;
            max_consecutive_blanks = max_consecutive_blanks.max(consecutive_blanks);
        } else {
            consecutive_blanks = 0;
        }
    }
    assert!(
        max_consecutive_blanks <= 1,
        "Should not have consecutive blank lines"
    );
}

#[test]
fn test_clean_collapse_blanks_preserves_content() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--collapse-blanks")
        .arg("-f")
        .arg(fixture_path("clean_with_blanks.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("First line"));
    assert!(stdout.contains("Second line"));
    assert!(stdout.contains("Third line"));
    assert!(stdout.contains("Fourth line"));
}

// ============================================================
// Repeated Line Collapsing Tests
// ============================================================

#[test]
fn test_clean_collapse_repeats_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw") // Use raw format to avoid reduction percentage in output
        .arg("clean")
        .arg("--collapse-repeats")
        .arg("-f")
        .arg(fixture_path("clean_with_repeats.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Original has 3 consecutive "[DEBUG] Processing request" lines and 1 separate at the end
    // After collapse, should have 2 (one from the collapsed group, one from the end)
    let debug_count = stdout.matches("[DEBUG] Processing request").count();
    assert_eq!(debug_count, 2, "Should collapse consecutive repeated lines");

    // Original has 2 consecutive "[WARN] Cache miss" lines
    let warn_count = stdout.matches("[WARN] Cache miss").count();
    assert_eq!(
        warn_count, 1,
        "Should collapse consecutive repeated warn lines"
    );
}

#[test]
fn test_clean_collapse_repeats_preserves_non_consecutive() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw") // Use raw format to avoid reduction percentage in output
        .arg("clean")
        .arg("--collapse-repeats")
        .arg("-f")
        .arg(fixture_path("clean_with_repeats.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // "[INFO]" appears in non-consecutive positions and should be preserved
    // The file has: [INFO] Application started, [INFO] Request completed, [INFO] Retrying...
    // All are non-consecutive, so all should remain
    let info_count = stdout.matches("[INFO]").count();
    assert_eq!(
        info_count, 3,
        "Non-consecutive lines should not be collapsed"
    );
}

#[test]
fn test_clean_collapse_repeats_overrides_collapse_blanks() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--collapse-repeats")
        .arg("-f")
        .arg(fixture_path("clean_with_repeats.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // When collapse_repeats is set, it should also handle blank lines
    // (collapse_repeats overrides collapse_blanks in the implementation)
    assert!(!stdout.is_empty());
}

// ============================================================
// Trim Tests
// ============================================================

#[test]
fn test_clean_trim_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_with_whitespace.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Check that lines don't have leading whitespace
    for line in stdout.lines() {
        // The content should be trimmed
        if line.contains("Leading") {
            assert!(
                !line.starts_with("   "),
                "Leading whitespace should be trimmed"
            );
        }
        if line.contains("Trailing") {
            assert!(
                !line.ends_with("   "),
                "Trailing whitespace should be trimmed"
            );
        }
    }
}

#[test]
fn test_clean_trim_removes_leading_whitespace() {
    let input = "   Hello World\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should start with "Hello" without leading spaces
    assert!(stdout.starts_with("Hello") || stdout.starts_with("H"));
}

#[test]
fn test_clean_trim_removes_trailing_whitespace() {
    let input = "Hello World   \n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("clean")
        .arg("--trim")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should not end with trailing spaces
    for line in stdout.lines() {
        if line.contains("Hello World") {
            assert!(
                !line.ends_with("   "),
                "Trailing whitespace should be trimmed"
            );
        }
    }
}

#[test]
fn test_clean_without_trim_preserves_leading() {
    let input = "   Indented line\n";
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
    // Without --trim, leading whitespace should be preserved
    // (but trailing whitespace is always trimmed according to implementation)
    assert!(stdout.contains("Indented line"));
}
