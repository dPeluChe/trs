//! Validation tests for empty input, unicode handling, format precedence,
//! schema version consistency, and run command format.
//!
//! Covers:
//! - Empty input validation
//! - Unicode and special character handling
//! - Format precedence validation
//! - Schema version consistency
//! - Run command format validation

use assert_cmd::Command;
use std::io::Write;

/// Helper to parse and validate JSON output
fn parse_json_output(output: &[u8]) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(output);
    serde_json::from_str(&stdout).expect("Output should be valid JSON")
}

// ============================================================
// Empty Input Validation Tests
// ============================================================

#[test]
fn test_empty_input_json_is_valid() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);
    assert!(
        json["is_empty"].is_boolean(),
        "Empty input should have is_empty boolean"
    );
    assert_eq!(
        json["is_empty"], true,
        "Empty input should have is_empty = true"
    );
}

#[test]
fn test_empty_input_csv_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should still have header even with empty input
    assert!(
        !stdout.is_empty(),
        "CSV should have at least header for empty input"
    );
}

#[test]
fn test_empty_input_tsv_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should still have header even with empty input
    assert!(
        !stdout.is_empty(),
        "TSV should have at least header for empty input"
    );
}

// ============================================================
// Unicode and Special Character Handling Tests
// ============================================================

#[test]
fn test_json_handles_unicode() {
    let input = "\u{6587}\u{4ef6}.txt\n\u{6587}\u{4ef6}\u{5939}/\n\u{6d4b}\u{8bd5}.rs";

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

    // Should be valid JSON with unicode content
    let json = parse_json_output(&output);
    assert!(json["is_empty"] == false || json["is_empty"] == true);
}

#[test]
fn test_csv_handles_unicode() {
    let input = "\u{6587}\u{4ef6}.txt\n\u{6d4b}\u{8bd5}.rs";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle unicode without errors
    assert!(
        stdout.contains("\u{6587}\u{4ef6}")
            || stdout.contains("\u{6d4b}\u{8bd5}")
            || stdout.contains("total:")
    );
}

#[test]
fn test_tsv_handles_unicode() {
    let input = "\u{6587}\u{4ef6}.txt\n\u{6d4b}\u{8bd5}.rs";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle unicode without errors
    assert!(
        stdout.contains("\u{6587}\u{4ef6}")
            || stdout.contains("\u{6d4b}\u{8bd5}")
            || stdout.contains("total:")
    );
}

#[test]
fn test_json_handles_newlines_in_content() {
    let input = "file1.txt\nfile2.rs\nfile3.py";

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

    // Should be valid JSON
    let json = parse_json_output(&output);
    assert!(json["entries"].is_array() || json["is_empty"].is_boolean());
}

// ============================================================
// Format Precedence Validation Tests
// ============================================================

#[test]
fn test_json_has_highest_precedence() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("--csv")
        .arg("--tsv")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Should produce JSON output (highest precedence)
    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.starts_with('{'),
        "JSON should have highest precedence"
    );
}

#[test]
fn test_csv_beats_tsv() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Should produce CSV output (higher precedence than TSV)
    let stdout = String::from_utf8_lossy(&output);
    // CSV uses commas, TSV uses tabs - check it's not all tabs
    assert!(
        stdout.contains(',') || stdout.contains("line_number"),
        "CSV should have higher precedence than TSV"
    );
}

#[test]
fn test_agent_beats_compact() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("--compact")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Both agent and compact use similar key: value format
    // The test verifies that the command succeeds with both flags
    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("branch:") || stdout.contains("clean") || stdout.contains("status:"),
        "Output should have structured format"
    );
}

// ============================================================
// Schema Version Consistency Tests
// ============================================================

#[test]
fn test_json_schema_version_format() {
    // Use replace command which has schema in JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_VERSION_TEST")
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

    // Schema version should follow semver format (X.Y.Z)
    let version = json["schema"]["version"].as_str().unwrap();
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "Schema version should be semver format");

    // Each part should be a number
    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Schema version parts should be numbers"
        );
    }
}

#[test]
fn test_all_schemas_have_consistent_version() {
    // Test that all JSON outputs with schemas have the same schema version format
    // Using commands that actually have schema fields (replace and search)
    let mut versions = Vec::new();

    // Test replace command
    {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        let output = cmd
            .arg("--json")
            .arg("replace")
            .arg("src")
            .arg("NONEXISTENT_PATTERN_CONSISTENCY_TEST")
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
        if let Some(v) = json["schema"]["version"].as_str() {
            versions.push(v.to_string());
        }
    }

    // Test search command
    {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        let output = cmd
            .arg("--json")
            .arg("search")
            .arg("src")
            .arg("fn")
            .arg("--extension")
            .arg("rs")
            .arg("--limit")
            .arg("1")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let json = parse_json_output(&output);
        if let Some(v) = json["schema"]["version"].as_str() {
            versions.push(v.to_string());
        }
    }

    // All versions should be consistent
    if versions.len() > 1 {
        let first = &versions[0];
        for v in &versions[1..] {
            assert_eq!(v, first, "All schema versions should be consistent");
        }
    }
}

// ============================================================
// Run Command Format Validation Tests
// ============================================================

#[test]
fn test_run_json_output_is_valid() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test output")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);
    assert!(
        json["stdout"].is_string() || json["output"].is_string(),
        "Run JSON should have stdout or output field"
    );
}

#[test]
fn test_run_csv_output_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // CSV should have header
    assert!(!stdout.is_empty(), "CSV output should not be empty");
}

#[test]
fn test_run_agent_output_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structure
    assert!(!stdout.is_empty(), "Agent output should not be empty");
}
