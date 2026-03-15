//! Signal preservation tests for TARS CLI.
//!
//! This module validates that essential signals are preserved after reduction.
//! "Signal preservation" means that critical information from the original input
//! is still present and correct in the reduced output, regardless of the output format.
//!
//! # What is Signal Preservation?
//!
//! When TARS reduces output (e.g., git status → compact format), we must ensure:
//! - Branch names are preserved
//! - File paths are preserved
//! - Status codes are preserved
//! - Counts are accurate
//! - Critical metadata is not lost
//!
//! # Test Categories
//!
//! - Git Status: branch, files, status codes, counts
//! - Git Diff: files, change types, additions/deletions
//! - LS: files, directories, counts
//! - Grep: file paths, line numbers, match content
//! - Logs: timestamps, levels, messages

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

// ============================================================
// Git Status Signal Preservation Tests
// ============================================================

#[test]
fn test_git_status_preserves_branch_name() {
    // Branch name is critical signal that must be preserved in all formats
    let input = fixtures::git_status_mixed();

    // Compact format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));

    // JSON format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["branch"], "main", "Branch name must be preserved in JSON");

    // Agent format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_git_status_preserves_file_paths() {
    // File paths are essential signal
    let input = fixtures::git_status_mixed();

    // JSON format - extract and verify all file paths
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Collect all file paths from the JSON output
    let mut all_paths: Vec<String> = Vec::new();
    
    if let Some(staged) = json["staged"].as_array() {
        for entry in staged {
            if let Some(path) = entry["path"].as_str() {
                all_paths.push(path.to_string());
            }
        }
    }
    if let Some(unstaged) = json["unstaged"].as_array() {
        for entry in unstaged {
            if let Some(path) = entry["path"].as_str() {
                all_paths.push(path.to_string());
            }
        }
    }
    if let Some(untracked) = json["untracked"].as_array() {
        for entry in untracked {
            if let Some(path) = entry["path"].as_str() {
                all_paths.push(path.to_string());
            }
        }
    }

    // Verify we have file paths
    assert!(!all_paths.is_empty(), "File paths must be preserved");

    // Compact format should also contain the paths
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // At least one file path should be present in compact output
    assert!(
        all_paths.iter().any(|p| stdout.contains(p)),
        "At least one file path must be preserved in compact output"
    );
}

#[test]
fn test_git_status_preserves_status_codes() {
    // Status codes (M, A, D, etc.) must be preserved
    let input = fixtures::git_status_staged();

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

    // Verify staged entries have status codes
    if let Some(staged) = json["staged"].as_array() {
        for entry in staged {
            assert!(
                entry["status"].is_string(),
                "Status code must be preserved for each file"
            );
            let status = entry["status"].as_str().unwrap();
            // Valid git status codes
            assert!(
                matches!(status, "M" | "A" | "D" | "R" | "C" | "T"),
                "Status code '{}' should be a valid git status code",
                status
            );
        }
    }
}

#[test]
fn test_git_status_preserves_counts() {
    // Counts must be accurate
    let input = fixtures::git_status_mixed();

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

    // Verify count fields exist and have valid values
    // The schema uses staged_count, unstaged_count, untracked_count, unmerged_count
    assert!(json["staged_count"].is_number(), "staged_count must be present");
    assert!(json["unstaged_count"].is_number(), "unstaged_count must be present");
    assert!(json["untracked_count"].is_number(), "untracked_count must be present");
    assert!(json["unmerged_count"].is_number(), "unmerged_count must be present");
    
    let staged_count = json["staged_count"].as_u64().unwrap_or(0) as usize;
    let unstaged_count = json["unstaged_count"].as_u64().unwrap_or(0) as usize;
    let untracked_count = json["untracked_count"].as_u64().unwrap_or(0) as usize;

    // Verify counts match actual array lengths
    let actual_staged = json["staged"].as_array().map(|a| a.len()).unwrap_or(0);
    let actual_unstaged = json["unstaged"].as_array().map(|a| a.len()).unwrap_or(0);
    let actual_untracked = json["untracked"].as_array().map(|a| a.len()).unwrap_or(0);

    assert_eq!(staged_count, actual_staged, "Staged count must match array length");
    assert_eq!(unstaged_count, actual_unstaged, "Unstaged count must match array length");
    assert_eq!(untracked_count, actual_untracked, "Untracked count must match array length");
}

#[test]
fn test_git_status_preserves_ahead_behind() {
    // Ahead/behind counts are important sync status signals
    let input = fixtures::git_status_ahead();

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

    // Ahead count must be preserved
    assert!(json["ahead"].is_number(), "Ahead count must be preserved");
    assert!(json["ahead"].as_u64().unwrap() > 0, "Ahead count should be positive");
}

#[test]
fn test_git_status_preserves_behind() {
    // Behind counts are important sync status signals
    let input = fixtures::git_status_behind();

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

    assert!(json["behind"].is_number(), "Behind count must be preserved");
    assert!(json["behind"].as_u64().unwrap() > 0, "Behind count should be positive");
}

#[test]
fn test_git_status_preserves_clean_status() {
    // Clean/dirty status is critical signal
    let input = fixtures::git_status_clean();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["is_clean"].is_boolean(), "is_clean must be a boolean");
    assert!(json["is_clean"].as_bool().unwrap(), "Clean repo must have is_clean=true");

    // Compact format should indicate clean
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn test_git_status_preserves_dirty_status() {
    let input = fixtures::git_status_mixed();

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

    assert!(json["is_clean"].is_boolean(), "is_clean must be a boolean");
    assert!(!json["is_clean"].as_bool().unwrap(), "Dirty repo must have is_clean=false");
}

// ============================================================
// Git Diff Signal Preservation Tests
// ============================================================

#[test]
fn test_git_diff_preserves_file_paths() {
    // File paths in diffs are essential
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Verify files array exists and has entries
    let files = json["files"].as_array().expect("Files array must exist");
    assert!(!files.is_empty(), "Files array must not be empty");

    // Each file must have a path
    for file in files {
        assert!(file["path"].is_string(), "Each file must have a path");
        assert!(!file["path"].as_str().unwrap().is_empty(), "Path must not be empty");
    }
}

#[test]
fn test_git_diff_preserves_change_types() {
    // Change types (M, A, D, R) must be preserved
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let files = json["files"].as_array().expect("Files array must exist");
    for file in files {
        assert!(
            file["change_type"].is_string(),
            "Each file must have a change_type"
        );
        let change_type = file["change_type"].as_str().unwrap();
        assert!(
            matches!(change_type, "M" | "A" | "D" | "R" | "C" | "T"),
            "Change type '{}' should be valid",
            change_type
        );
    }
}

#[test]
fn test_git_diff_preserves_line_counts() {
    // Addition/deletion counts are important signal
    let input = fixtures::git_diff_modified();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Total additions/deletions should be present
    assert!(json["total_additions"].is_number(), "total_additions must be present");
    assert!(json["total_deletions"].is_number(), "total_deletions must be present");

    // Each file should have additions/deletions
    if let Some(files) = json["files"].as_array() {
        for file in files {
            assert!(file["additions"].is_number(), "Each file must have additions count");
            assert!(file["deletions"].is_number(), "Each file must have deletions count");
        }
    }
}

#[test]
fn test_git_diff_preserves_empty_status() {
    // Empty diff status must be preserved
    let input = fixtures::git_diff_empty();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["is_empty"].is_boolean(), "is_empty must be a boolean");
    assert!(json["is_empty"].as_bool().unwrap(), "Empty diff must have is_empty=true");
}

// ============================================================
// LS Signal Preservation Tests
// ============================================================

#[test]
fn test_ls_preserves_file_names() {
    // File names must be preserved
    let input = fixtures::ls_mixed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Entries array should exist and have content
    let entries = json["entries"].as_array().expect("Entries array must exist");
    assert!(!entries.is_empty(), "Entries should not be empty");

    // Each entry must have a name
    for entry in entries {
        assert!(entry["name"].is_string(), "Each entry must have a name");
        assert!(!entry["name"].as_str().unwrap().is_empty(), "Name must not be empty");
    }
}

#[test]
fn test_ls_preserves_directory_distinction() {
    // Directory vs file distinction must be preserved
    let input = fixtures::ls_mixed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have files and directories arrays
    let files = json["files"].as_array();
    let directories = json["directories"].as_array();

    // At least one should have content (depends on fixture)
    assert!(
        files.is_some() || directories.is_some(),
        "Should have files or directories"
    );

    // Each entry should have type field indicating "file", "directory", or "symlink"
    if let Some(entries) = json["entries"].as_array() {
        for entry in entries {
            assert!(entry["type"].is_string(), "Each entry must have type field");
            let entry_type = entry["type"].as_str().unwrap();
            assert!(
                matches!(entry_type, "file" | "directory" | "symlink"),
                "Entry type '{}' should be 'file', 'directory', or 'symlink'",
                entry_type
            );
        }
    }
}

#[test]
fn test_ls_preserves_total_count() {
    // Total count must match actual entry count
    let input = fixtures::ls_mixed();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Count should match entries length (in counts.total)
    let entries_count = json["entries"].as_array().map(|a| a.len()).unwrap_or(0);
    let total = json["counts"]["total"].as_u64().unwrap_or(0) as usize;

    assert_eq!(total, entries_count, "Total count must match entries length");
}

#[test]
fn test_ls_preserves_hidden_files_flag() {
    // Hidden files should be properly flagged
    let input = fixtures::ls_with_hidden();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check that hidden files are properly identified
    if let Some(entries) = json["entries"].as_array() {
        let has_hidden = entries.iter().any(|e| {
            e["name"].as_str()
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
        });

        if has_hidden {
            // If there are hidden files, at least some should be marked as hidden
            let marked_hidden = entries.iter().any(|e| {
                e["is_hidden"].as_bool().unwrap_or(false)
            });
            assert!(marked_hidden, "Hidden files should be marked with is_hidden=true");
        }
    }
}

// ============================================================
// Grep Signal Preservation Tests
// ============================================================

#[test]
fn test_grep_preserves_file_paths() {
    // File paths in grep results must be preserved
    let input = fixtures::grep_multiple_files();

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

    // Files array should exist
    let files = json["files"].as_array().expect("Files array must exist");
    assert!(!files.is_empty(), "Files array should not be empty");

    // Each file should have a path
    for file in files {
        assert!(file["path"].is_string(), "Each file must have a path");
    }
}

#[test]
fn test_grep_preserves_line_numbers() {
    // Line numbers are critical for locating matches
    let input = fixtures::grep_simple();

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

    // Check that matches have line numbers (field is "line_number")
    if let Some(files) = json["files"].as_array() {
        for file in files {
            if let Some(matches) = file["matches"].as_array() {
                for m in matches {
                    assert!(m["line_number"].is_number(), "Each match must have a line_number");
                }
            }
        }
    }
}

#[test]
fn test_grep_preserves_match_content() {
    // Match content (the actual matched text) must be preserved
    let input = fixtures::grep_simple();

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

    // Check that matches have content (field is "line")
    if let Some(files) = json["files"].as_array() {
        for file in files {
            if let Some(matches) = file["matches"].as_array() {
                for m in matches {
                    assert!(m["line"].is_string(), "Each match must have line content");
                    assert!(!m["line"].as_str().unwrap().is_empty(), "Line content must not be empty");
                }
            }
        }
    }
}

#[test]
fn test_grep_preserves_match_counts() {
    // Match counts must be accurate
    let input = fixtures::grep_multiple_files();

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

    // Total matches count should be present (in counts.total_matches)
    assert!(json["counts"]["total_matches"].is_number(), "counts.total_matches must be present");

    // Verify count matches actual match count
    let mut actual_matches = 0;
    if let Some(files) = json["files"].as_array() {
        for file in files {
            if let Some(matches) = file["matches"].as_array() {
                actual_matches += matches.len();
            }
        }
    }

    let reported_total = json["counts"]["total_matches"].as_u64().unwrap_or(0) as usize;
    assert_eq!(reported_total, actual_matches, "counts.total_matches must match actual match count");
}

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
