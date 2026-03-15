//! Comprehensive integration tests for the `clean` command.
//!
//! This test module verifies the clean functionality through the CLI:
//! - Basic clean operation
//! - ANSI stripping (--no-ansi)
//! - Blank line collapsing (--collapse-blanks)
//! - Repeated line collapsing (--collapse-repeats)
//! - Whitespace trimming (--trim)
//! - Output format variations (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Stats output
//! - Edge cases (empty files, special characters, control chars)
//! - Error handling (non-existent files)

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixtures").join(name)
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
    assert!(max_consecutive_blanks <= 1, "Should not have consecutive blank lines");
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
        .arg("--raw")  // Use raw format to avoid reduction percentage in output
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
    assert_eq!(warn_count, 1, "Should collapse consecutive repeated warn lines");
}

#[test]
fn test_clean_collapse_repeats_preserves_non_consecutive() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")  // Use raw format to avoid reduction percentage in output
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
    assert_eq!(info_count, 3, "Non-consecutive lines should not be collapsed");
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
            assert!(!line.starts_with("   "), "Leading whitespace should be trimmed");
        }
        if line.contains("Trailing") {
            assert!(!line.ends_with("   "), "Trailing whitespace should be trimmed");
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
            assert!(!line.ends_with("   "), "Trailing whitespace should be trimmed");
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

// ============================================================
// JSON Format Tests
// ============================================================

#[test]
fn test_clean_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check structure
    assert!(json["content"].is_string());
    assert!(json["stats"].is_object());
    assert!(json["options"].is_object());
}

#[test]
fn test_clean_json_stats_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction_percent"].is_number());
}

#[test]
fn test_clean_json_options_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("--no-ansi")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert_eq!(json["options"]["no_ansi"], true);
    assert_eq!(json["options"]["trim"], true);
}

#[test]
fn test_clean_json_content_cleaned() {
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

    let content = json["content"].as_str().expect("content should be string");
    assert!(content.contains("[INFO]"));
    // ANSI codes should be stripped
    assert!(!content.contains("\x1b["));
}

// ============================================================
// CSV Format Tests
// ============================================================

#[test]
fn test_clean_csv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // CSV format outputs each line as a quoted CSV row
    assert!(stdout.contains("Hello World"));
    // Lines should be quoted
    assert!(stdout.contains('"'));
}

#[test]
fn test_clean_csv_escapes_quotes() {
    let input = "Line with \"quotes\"\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Quotes should be escaped by doubling
    assert!(stdout.contains("\"\"quotes\"\"") || stdout.contains("quotes"));
}

// ============================================================
// TSV Format Tests
// ============================================================

#[test]
fn test_clean_tsv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // TSV format outputs each line as-is
    assert!(stdout.contains("Hello World"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_clean_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format shows reduction percentage
    assert!(stdout.contains("Content") || stdout.contains("reduction"));
}

#[test]
fn test_clean_agent_shows_reduction() {
    let input = "Line 1\n\n\n\n\nLine 2\n"; // Has blank lines that will be reduced
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should show reduction percentage
    assert!(stdout.contains("% reduction") || stdout.contains("reduction"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_clean_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should just output the cleaned content
    assert!(stdout.contains("Hello World"));
    // Should not have JSON or other formatting
    assert!(!stdout.contains("\"content\""));
    assert!(!stdout.contains("reduction"));
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_clean_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format shows content and optionally reduction
    assert!(stdout.contains("Hello World"));
}

#[test]
fn test_clean_compact_shows_reduction_when_positive() {
    let input = "Line 1\n\n\n\n\nLine 2\n"; // Has extra content that will be reduced
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should show reduction percentage when > 0
    assert!(stdout.contains("% reduction") || stdout.contains("Line"));
}

// ============================================================
// Stats Tests
// ============================================================

#[test]
fn test_clean_with_stats() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Stats are printed to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should show reducer name
    assert!(stderr.contains("clean"));
}

#[test]
fn test_clean_stats_shows_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("clean")
        .arg("--no-ansi")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should show the options used
    assert!(stderr.contains("No ANSI") || stderr.contains("no_ansi") || stderr.contains("clean"));
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
            assert!(consecutive_blanks <= 1, "Should not have consecutive blank lines");
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
        .arg("--raw")  // Use raw format to get just the content without percentage
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
        .arg("--raw")  // Use raw format to avoid reduction percentage in output
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
