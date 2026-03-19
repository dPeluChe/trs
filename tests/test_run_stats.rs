//! Integration tests for the `run` command: stats output size comparison
//! (raw vs reduced output, input/output bytes).

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Stats: Raw vs Reduced Output Size Comparison Tests
// ============================================================

#[test]
fn test_run_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_run_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_run_stats_raw_output_same_size() {
    // When using --raw format, input_bytes should equal output_bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
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
    assert_eq!(input_bytes, output_bytes, "Raw output should have same input and output bytes");

    // Also verify stdout length matches
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_run_stats_json_output_larger_than_raw() {
    // When using --json format, output_bytes should be larger than raw input_bytes
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse input bytes and output bytes from stderr
    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    // For JSON output, output should be larger than input (JSON adds metadata)
    assert!(output_bytes > input_bytes, "JSON output should be larger than raw input");

    // Verify stdout length matches output bytes
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_run_stats_compact_output_size() {
    // When using --compact format, verify proper byte counting
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
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
fn test_run_stats_git_status_comparison() {
    // Test with git status to verify raw vs reduced output comparison
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("run")
        .arg("git")
        .arg("status")
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
    assert_eq!(input_bytes, output_bytes, "Raw git status should have same input and output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_run_stats_git_status_json_larger() {
    // Test git status with JSON format - output should be larger due to JSON structure
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("run")
        .arg("git")
        .arg("status")
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
    assert!(output_bytes > input_bytes, "JSON git status output should be larger than raw input");
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
