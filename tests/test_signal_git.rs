//! Signal preservation tests: Git Status.
//!
//! Validates that essential signals from git status are preserved after reduction:
//! branch names, file paths, status codes, counts, ahead/behind, clean/dirty status.

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
