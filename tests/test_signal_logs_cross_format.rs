//! Signal preservation tests: Logs, Cross-Format consistency, and Edge Cases.
//!
//! Validates that:
//! - Log entries, line counts, and levels are preserved
//! - Signals are consistent across JSON, compact, and agent formats
//! - Edge cases (empty input, unicode, long paths) preserve signals correctly

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

// ============================================================
// Logs Signal Preservation Tests
// ============================================================

#[test]
fn test_logs_preserves_entries() {
    // Log entries must be preserved
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Entries should exist
    let entries = json["entries"].as_array().expect("Entries array must exist");
    assert!(!entries.is_empty(), "Entries should not be empty");
}

#[test]
fn test_logs_preserves_line_count() {
    // Line count must be accurate
    let input = fixtures::logs_simple();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let entries_count = json["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    let total_lines = json["counts"]["total_lines"].as_u64().unwrap_or(0) as usize;

    assert_eq!(total_lines, entries_count, "counts.total_lines must match entries count");
}

#[test]
fn test_logs_preserves_level_information() {
    // Log levels (INFO, WARN, ERROR) should be preserved when present
    let input = fixtures::logs_all_levels();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check that some entries have level information
    if let Some(entries) = json["entries"].as_array() {
        let has_levels = entries.iter().any(|e| e["level"].is_string());
        if has_levels {
            // If levels are present, verify they're valid
            for entry in entries {
                if let Some(level) = entry["level"].as_str() {
                    let level_upper = level.to_uppercase();
                    assert!(
                        matches!(level_upper.as_str(), "DEBUG" | "INFO" | "WARN" | "WARNING" | "ERROR" | "FATAL" | "TRACE"),
                        "Level '{}' should be a valid log level",
                        level
                    );
                }
            }
        }
    }
}

// ============================================================
// Cross-Format Signal Preservation Tests
// ============================================================

#[test]
fn test_git_status_signal_consistent_across_formats() {
    // The same signal should be present in all output formats
    let input = fixtures::git_status_mixed();

    // Get JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let json_output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_stdout = String::from_utf8_lossy(&json_output);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).expect("Invalid JSON");

    // Get compact output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let compact_output = cmd
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let compact_stdout = String::from_utf8_lossy(&compact_output);

    // Get agent output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let agent_output = cmd
        .arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let agent_stdout = String::from_utf8_lossy(&agent_output);

    // Branch name should appear in all formats
    let branch = json["branch"].as_str().expect("Branch must be in JSON");
    assert!(compact_stdout.contains(branch), "Branch must be in compact output");
    assert!(agent_stdout.contains(branch), "Branch must be in agent output");

    // File paths should appear in all formats (at least some)
    if let Some(staged) = json["staged"].as_array() {
        if let Some(first_staged) = staged.first() {
            if let Some(path) = first_staged["path"].as_str() {
                // Compact and agent should contain at least part of the path
                let path_part = path.split('/').last().unwrap_or(path);
                assert!(
                    compact_stdout.contains(path) || compact_stdout.contains(path_part),
                    "File path should be in compact output"
                );
            }
        }
    }
}

#[test]
fn test_git_diff_signal_consistent_across_formats() {
    let input = fixtures::git_diff_modified();

    // Get JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let json_output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_stdout = String::from_utf8_lossy(&json_output);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).expect("Invalid JSON");

    // Get compact output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let compact_output = cmd
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let compact_stdout = String::from_utf8_lossy(&compact_output);

    // File paths should appear in both
    if let Some(files) = json["files"].as_array() {
        for file in files {
            if let Some(path) = file["path"].as_str() {
                let path_part = path.split('/').last().unwrap_or(path);
                assert!(
                    compact_stdout.contains(path) || compact_stdout.contains(path_part),
                    "File path '{}' should be in compact output",
                    path
                );
            }
        }
    }
}

#[test]
fn test_ls_signal_consistent_across_formats() {
    let input = fixtures::ls_mixed();

    // Get JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let json_output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_stdout = String::from_utf8_lossy(&json_output);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).expect("Invalid JSON");

    // Get compact output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let compact_output = cmd
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let compact_stdout = String::from_utf8_lossy(&compact_output);

    // File/directory names should appear in both
    if let Some(entries) = json["entries"].as_array() {
        for entry in entries {
            if let Some(name) = entry["name"].as_str() {
                assert!(
                    compact_stdout.contains(name),
                    "Entry name '{}' should be in compact output",
                    name
                );
            }
        }
    }
}

#[test]
fn test_grep_signal_consistent_across_formats() {
    let input = fixtures::grep_simple();

    // Get JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let json_output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_stdout = String::from_utf8_lossy(&json_output);
    let json: serde_json::Value = serde_json::from_str(&json_stdout).expect("Invalid JSON");

    // Get compact output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let compact_output = cmd
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let compact_stdout = String::from_utf8_lossy(&compact_output);

    // File paths should appear in both
    if let Some(files) = json["files"].as_array() {
        for file in files {
            if let Some(path) = file["path"].as_str() {
                let path_part = path.split('/').last().unwrap_or(path);
                assert!(
                    compact_stdout.contains(path) || compact_stdout.contains(path_part),
                    "File path '{}' should be in compact output",
                    path
                );
            }
        }
    }
}

// ============================================================
// Edge Case Signal Preservation Tests
// ============================================================

#[test]
fn test_empty_input_signal_preservation() {
    // Empty input should produce valid output indicating emptiness
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should produce valid JSON even for empty input
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should produce valid JSON for empty input");

    // Should indicate empty/clean state
    assert!(json["is_clean"].as_bool().unwrap_or(true), "Empty input should indicate clean state");
}

#[test]
fn test_unicode_signal_preservation() {
    // Unicode content must be preserved
    let input = "src/unicode_文件.rs:42:const greeting = \"Hello 世界\";";

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

    // Unicode characters should be preserved
    assert!(stdout.contains("文件"), "Unicode file name should be preserved");
    assert!(stdout.contains("世界"), "Unicode content should be preserved");
}

#[test]
fn test_long_paths_signal_preservation() {
    // Long file paths must be fully preserved
    let input = fixtures::git_status_long_paths();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Long paths should be preserved (not truncated)
    if let Some(staged) = json["staged"].as_array() {
        for entry in staged {
            if let Some(path) = entry["path"].as_str() {
                // Path should be complete (not end with ...)
                assert!(!path.ends_with("..."), "Long paths should not be truncated");
            }
        }
    }
}
