//! Signal preservation tests: Git Diff, LS, and Grep.
//!
//! Validates that essential signals are preserved after reduction:
//! - Git Diff: file paths, change types, line counts, empty status
//! - LS: file names, directory distinction, total count, hidden files
//! - Grep: file paths, line numbers, match content, match counts

use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;

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
        assert!(
            !file["path"].as_str().unwrap().is_empty(),
            "Path must not be empty"
        );
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
    assert!(
        json["total_additions"].is_number(),
        "total_additions must be present"
    );
    assert!(
        json["total_deletions"].is_number(),
        "total_deletions must be present"
    );

    // Each file should have additions/deletions
    if let Some(files) = json["files"].as_array() {
        for file in files {
            assert!(
                file["additions"].is_number(),
                "Each file must have additions count"
            );
            assert!(
                file["deletions"].is_number(),
                "Each file must have deletions count"
            );
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
    assert!(
        json["is_empty"].as_bool().unwrap(),
        "Empty diff must have is_empty=true"
    );
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
    let entries = json["entries"]
        .as_array()
        .expect("Entries array must exist");
    assert!(!entries.is_empty(), "Entries should not be empty");

    // Each entry must have a name
    for entry in entries {
        assert!(entry["name"].is_string(), "Each entry must have a name");
        assert!(
            !entry["name"].as_str().unwrap().is_empty(),
            "Name must not be empty"
        );
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

    assert_eq!(
        total, entries_count,
        "Total count must match entries length"
    );
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
            e["name"]
                .as_str()
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
        });

        if has_hidden {
            // If there are hidden files, at least some should be marked as hidden
            let marked_hidden = entries
                .iter()
                .any(|e| e["is_hidden"].as_bool().unwrap_or(false));
            assert!(
                marked_hidden,
                "Hidden files should be marked with is_hidden=true"
            );
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
                    assert!(
                        m["line_number"].is_number(),
                        "Each match must have a line_number"
                    );
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
                    assert!(
                        !m["line"].as_str().unwrap().is_empty(),
                        "Line content must not be empty"
                    );
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
    assert!(
        json["counts"]["total_matches"].is_number(),
        "counts.total_matches must be present"
    );

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
    assert_eq!(
        reported_total, actual_matches,
        "counts.total_matches must match actual match count"
    );
}
