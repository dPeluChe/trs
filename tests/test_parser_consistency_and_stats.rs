//! Integration tests for parser format consistency, edge cases, and stats.
//!
//! Covers:
//! - Format consistency across all parsers
//! - Edge cases: unicode, empty lines, large input, long paths
//! - Stats flag tests
//! - Git diff stats: raw vs reduced output size comparison

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

// ============================================================
// Parser Format Consistency Tests
// ============================================================

#[test]
fn test_parser_all_formats_git_status() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("main").or(predicate::str::contains("clean")));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"branch\""));

    // Test CSV format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success();

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("main").or(predicate::str::contains("clean")));
}

#[test]
fn test_parser_all_formats_git_diff() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_parser_all_formats_ls() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_parser_all_formats_logs() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("lines:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entries\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stdout(predicate::str::contains("lines:"));
}

#[test]
fn test_parser_all_formats_grep() {
    // Test compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));

    // Test JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));

    // Test agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_multiple_files())
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_parser_handles_unicode() {
    let input = "src/unicode_\u{00f1}ame.rs:42:const greeting = \"Hello \u{4e16}\u{754c}\";";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("unicode"));
}

#[test]
fn test_parser_handles_empty_lines() {
    let input = "\n\nsrc/main.rs:42:fn main() {}\n\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should still parse the one valid line
    assert!(json["files"].as_array().unwrap().len() > 0);
}

#[test]
fn test_parser_large_input() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin(fixtures::grep_large())
        .assert()
        .success()
        .stdout(predicate::str::contains("files"));
}

#[test]
fn test_parser_long_paths() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_long_paths())
        .assert()
        .success()
        .stdout(predicate::str::contains("nested"));
}

// ============================================================
// Stats Flag Tests
// ============================================================

#[test]
fn test_parser_stats_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("git-status")
        .write_stdin(fixtures::git_status_mixed())
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

#[test]
fn test_parser_stats_git_diff() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(fixtures::git_diff_modified())
        .assert()
        .success()
        .stderr(predicate::str::contains("Files changed:"));
}

// ============================================================
// Git Diff Stats: Raw vs Reduced Output Size Comparison Tests
// ============================================================

#[test]
fn test_parser_stats_git_diff_raw_reduction() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert!(output_bytes < input_bytes, "Raw git-diff output should be smaller than input");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_parser_stats_git_diff_json_larger() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    assert!(output_bytes > input_bytes, "JSON git-diff output should be larger than raw input");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_parser_stats_git_diff_compact_reduction() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--compact")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
    assert!(output_bytes < input_bytes, "Compact git-diff output should be smaller than raw input");
}

#[test]
fn test_parser_stats_git_diff_agent_reduction() {
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("--agent")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let input_bytes = extract_bytes(&stderr, "Input bytes:");
    let output_bytes = extract_bytes(&stderr, "Output bytes:");

    assert!(input_bytes.is_some(), "Should have input bytes");
    assert!(output_bytes.is_some(), "Should have output bytes");
    assert_eq!(output_bytes, Some(stdout.len()), "Output bytes should match stdout length");
}

#[test]
fn test_parser_stats_ls() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("ls")
        .write_stdin(fixtures::ls_mixed())
        .assert()
        .success()
        .stderr(predicate::str::contains("Files:"));
}

#[test]
fn test_parser_stats_logs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("logs")
        .write_stdin(fixtures::logs_simple())
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

/// Helper function to extract byte count from stats output
fn extract_bytes(stderr: &str, prefix: &str) -> Option<usize> {
    for line in stderr.lines() {
        if line.contains(prefix) {
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
