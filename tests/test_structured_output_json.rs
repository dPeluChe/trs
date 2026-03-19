//! Validation tests for JSON structured output mode.
//!
//! Covers:
//! - JSON schema validation for all command types
//! - Valid JSON structure for git-status, git-diff, ls, grep,
//!   find, logs, replace, search, tail

use assert_cmd::Command;
use std::io::Write;

// ============================================================
// JSON Schema Validation Tests
// ============================================================

/// Helper to parse and validate JSON output
fn parse_json_output(output: &[u8]) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(output);
    serde_json::from_str(&stdout).expect("Output should be valid JSON")
}

/// Validate that JSON has schema with version and type
fn validate_json_schema(json: &serde_json::Value, expected_type: &str) {
    assert!(json["schema"].is_object(), "JSON should have schema object");
    assert!(
        json["schema"]["version"].is_string(),
        "Schema should have version string"
    );
    assert_eq!(
        json["schema"]["type"], expected_type,
        "Schema type should match expected"
    );
}

#[test]
fn test_json_git_status_has_valid_schema() {
    let input = "On branch main\nnothing to commit, working tree clean";

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

    let json = parse_json_output(&output);

    // Validate schema structure
    assert!(json["branch"].is_string(), "branch should be string");
    assert!(json["is_clean"].is_boolean(), "is_clean should be boolean");
    assert!(json["staged"].is_array(), "staged should be array");
    assert!(json["unstaged"].is_array(), "unstaged should be array");
    assert!(json["untracked"].is_array(), "untracked should be array");
    assert!(json["unmerged"].is_array(), "unmerged should be array");

    // Validate counts are numbers
    assert!(
        json["staged_count"].is_number(),
        "staged_count should be number"
    );
    assert!(
        json["unstaged_count"].is_number(),
        "unstaged_count should be number"
    );
    assert!(
        json["untracked_count"].is_number(),
        "untracked_count should be number"
    );
    assert!(
        json["unmerged_count"].is_number(),
        "unmerged_count should be number"
    );
}

#[test]
fn test_json_git_diff_has_valid_schema() {
    let input = "diff --git a/test.txt b/test.txt\nindex 1234567..abcdef 100644\n--- a/test.txt\n+++ b/test.txt\n@@ -1 +1 @@\n-old\n+new";

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

    let json = parse_json_output(&output);

    // Validate git-diff structure (no schema field in current implementation)
    assert!(json["is_empty"].is_boolean(), "is_empty should be boolean");
    assert!(json["files"].is_array(), "files should be array");

    // Validate file entries have proper structure
    if let Some(files) = json["files"].as_array() {
        if !files.is_empty() {
            let file = &files[0];
            assert!(file["path"].is_string(), "file path should be string");
            assert!(
                file["change_type"].is_string(),
                "change_type should be string"
            );
            assert!(file["additions"].is_number(), "additions should be number");
            assert!(file["deletions"].is_number(), "deletions should be number");
        }
    }
}

#[test]
fn test_json_ls_has_valid_schema() {
    let input = "src/\nmain.rs\nCargo.toml";

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

    let json = parse_json_output(&output);

    // Validate ls output structure (no schema field in parse commands)
    assert!(json["is_empty"].is_boolean(), "is_empty should be boolean");
    assert!(json["entries"].is_array(), "entries should be array");
}

#[test]
fn test_json_grep_has_valid_schema() {
    let input = "test.rs:10:fn main() {\nother.rs:20:fn test()";

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

    let json = parse_json_output(&output);

    // Validate grep parse output structure (no schema field)
    assert!(json["is_empty"].is_boolean(), "is_empty should be boolean");
    assert!(json["files"].is_array(), "files should be array");
}

#[test]
fn test_json_find_has_valid_schema() {
    let input = "./src/main.rs\n./tests/test.rs\n./target/debug";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("find")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);

    // Validate find parse output structure (no schema field)
    assert!(json["is_empty"].is_boolean(), "is_empty should be boolean");
    assert!(json["files"].is_array(), "files should be array");
    assert!(
        json["directories"].is_array(),
        "directories should be array"
    );
}

#[test]
fn test_json_logs_has_valid_schema() {
    let input = "2024-01-01 INFO Application started\n2024-01-01 ERROR Connection failed";

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

    let json = parse_json_output(&output);

    // Validate logs parse output structure (no schema field)
    assert!(
        json["entries"].is_array() || json["lines"].is_array(),
        "entries/lines should be array"
    );
    assert!(json["counts"].is_object(), "counts should be object");
}

#[test]
fn test_json_replace_has_valid_schema() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_UNIQUE_12345")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);

    // Validate replace output has schema and proper structure
    validate_json_schema(&json, "replace_output");
    assert!(json["dry_run"].is_boolean(), "dry_run should be boolean");
    assert!(json["files"].is_array(), "files should be array");
    assert!(json["counts"].is_object(), "counts should be object");
}

#[test]
fn test_json_search_has_valid_schema() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("5")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);

    // Validate schema structure
    validate_json_schema(&json, "grep_output");
    assert!(json["files"].is_array(), "files should be array");
    assert!(
        json["truncated"].is_boolean() || json["is_empty"].is_boolean(),
        "should have truncated or is_empty"
    );
}

#[test]
fn test_json_tail_has_valid_schema() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);

    // Validate tail output structure (no schema field in tail command)
    assert!(json["file"].is_string(), "file should be string");
    assert!(json["lines"].is_array(), "lines should be array");
}
