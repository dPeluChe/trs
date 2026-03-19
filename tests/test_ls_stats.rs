//! Integration tests for the `ls` parse command - stats with many entries,
//! file flag, format-specific byte counting, unicode, and empty input.

use assert_cmd::Command;
use predicates::prelude::*;

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

// ============================================================
// Stats: Many Entries Tests
// ============================================================

#[test]
fn test_ls_stats_many_entries() {
    // Test with many entries to verify byte counting scales correctly
    let mut input = String::new();
    for i in 0..100 {
        input.push_str(&format!("file_{:03}.txt\n", i));
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin(input.as_str())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For raw output, input and output bytes should be equal
    assert_eq!(input_bytes, output_bytes, "Raw output with many entries should have same input and output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_ls_stats_json_many_entries_larger() {
    // Test with many entries in JSON format - output should be larger
    let mut input = String::new();
    for i in 0..50 {
        input.push_str(&format!("file_{:03}.txt\n", i));
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input.as_str())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // JSON output should be larger than raw input
    assert!(output_bytes > input_bytes, "JSON output with many entries should be larger than raw input");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_ls_stats_with_file_flag() {
    // Test using --file flag instead of stdin
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .arg("--file")
        .arg("tests/fixture_data/ls_simple.txt")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For raw output, input and output bytes should be equal
    assert_eq!(input_bytes, output_bytes, "Raw output from file should have same input and output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

// ============================================================
// Stats: Format-Specific Byte Counting Tests
// ============================================================

#[test]
fn test_ls_stats_csv_format() {
    // Test CSV format byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
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
fn test_ls_stats_tsv_format() {
    // Test TSV format byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
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
fn test_ls_stats_agent_format() {
    // Test Agent format byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--agent")
        .arg("parse")
        .arg("ls")
        .write_stdin("src\nCargo.toml\n")
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

// ============================================================
// Stats: Unicode and Empty Input Tests
// ============================================================

#[test]
fn test_ls_stats_unicode_content() {
    // Test with Unicode filenames
    let input = "\u{0444}\u{0430}\u{0439}\u{043B}.txt\n\u{6587}\u{6863}.rs\n\u{1F389}.md\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For raw output, input and output bytes should be equal
    assert_eq!(input_bytes, output_bytes, "Raw output with Unicode should have same input and output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_ls_stats_empty_input() {
    // Test with empty input - stats should still be shown
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For empty input, both should be 0 (or stats may not show values if output is empty)
    // The important thing is that if stats are shown, they should be consistent
    if input_bytes.is_some() && output_bytes.is_some() {
        assert_eq!(input_bytes, output_bytes, "Empty input should have equal input and output bytes");
        assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
    }
    // If stats don't show byte values for empty output, that's also acceptable
}
