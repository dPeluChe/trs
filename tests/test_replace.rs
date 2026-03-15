//! Comprehensive integration tests for the `replace` command.
//!
//! This test module verifies the replace functionality through the CLI:
//! - Basic search and replace
//! - Extension filtering
//! - Dry run mode (preview)
//! - Count only mode
//! - Output format variations (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Stats output
//! - Edge cases (empty results, special characters, invalid regex, etc.)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================
// Helper Functions
// ============================================================

/// Create a temporary file with content for testing.
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

// ============================================================
// Basic Replace Tests
// ============================================================

#[test]
fn test_replace_basic_pattern() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\nHello universe\nHello galaxy\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

#[test]
fn test_replace_with_dry_run_flag() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\nHello universe\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

#[test]
fn test_replace_with_preview_alias() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\nHello universe\n",
    );

    // --preview is an alias for --dry-run
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--preview")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

#[test]
fn test_replace_actually_modifies_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Perform actual replace (no dry-run)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .assert()
        .success()
        .stdout(predicate::str::contains("Replaced"));

    // Verify file was modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Hi world"));
    assert!(!content.contains("Hello world"));
}

#[test]
fn test_replace_multiple_occurrences_in_file() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo bar foo\nfoo baz foo\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("foo")
        .arg("qux")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("4 replacements"));
}

#[test]
fn test_replace_shows_file_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_replace_shows_line_number() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Line 1\nHello world\nLine 3\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2:"));
}

// ============================================================
// Extension Filter Tests
// ============================================================

#[test]
fn test_replace_with_extension_rs() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "fn old_function() {}\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "fn old_function() {}\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_function")
        .arg("new_function")
        .arg("--extension")
        .arg("rs")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("test.txt").not());
}

#[test]
fn test_replace_with_extension_short_flag() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "fn old_function() {}\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "fn old_function() {}\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_function")
        .arg("new_function")
        .arg("-e")
        .arg("rs")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));
}

#[test]
fn test_replace_with_extension_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("-e")
        .arg("nonexistent_ext_xyz")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches"));
}

// ============================================================
// Count Only Mode Tests
// ============================================================

#[test]
fn test_replace_count_flag() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo bar foo\nfoo baz foo\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("foo")
        .arg("qux")
        .arg("--count")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_replace_count_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("nonexistent_pattern_xyz")
        .arg("replacement")
        .arg("--count")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_replace_count_json_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo bar foo\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("foo")
        .arg("qux")
        .arg("--count")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["count"].as_u64().unwrap(), 2);
}

// ============================================================
// JSON Output Format Tests
// ============================================================

#[test]
fn test_replace_json_is_valid() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

#[test]
fn test_replace_json_has_files_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_replace_json_has_path_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""path""#));
}

#[test]
fn test_replace_json_has_matches_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""matches""#));
}

#[test]
fn test_replace_json_has_line_number_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""line_number""#));
}

#[test]
fn test_replace_json_has_original_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""original""#));
}

#[test]
fn test_replace_json_has_replaced_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""replaced""#));
}

#[test]
fn test_replace_json_has_counts_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""counts""#));
}

#[test]
fn test_replace_json_has_dry_run_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""dry_run": true"#));
}

#[test]
fn test_replace_json_has_search_pattern_field() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""search_pattern""#));
}

#[test]
fn test_replace_json_empty_result() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["files"].as_array().unwrap().is_empty());
    assert_eq!(json["counts"]["total_replacements"].as_u64().unwrap(), 0);
}

// ============================================================
// CSV Output Format Tests
// ============================================================

#[test]
fn test_replace_csv_has_header() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("file,line_number,original,replaced"));
}

#[test]
fn test_replace_csv_has_file_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_replace_csv_has_summary() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"));
}

// ============================================================
// TSV Output Format Tests
// ============================================================

#[test]
fn test_replace_tsv_has_header() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("file\tline_number\toriginal\treplaced"));
}

#[test]
fn test_replace_tsv_has_file_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_replace_compact_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

#[test]
fn test_replace_compact_is_default() {
    // Compact is the default format
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

#[test]
fn test_replace_compact_shows_replaced_line() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hi world"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_replace_raw_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello world -> Hi world"));
}

#[test]
fn test_replace_raw_has_summary() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_replace_agent_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

// ============================================================
// Format Precedence Tests
// ============================================================

#[test]
fn test_replace_format_precedence_json_over_raw() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // JSON should win over raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--raw")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_replace_format_precedence_json_over_compact() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // JSON should win over compact
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--compact")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""files""#));
}

#[test]
fn test_replace_format_precedence_csv_over_tsv() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // CSV should win over TSV
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("file,line_number"));
}

#[test]
fn test_replace_format_precedence_compact_over_raw() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Compact should win over raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("--raw")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview:"));
}

// ============================================================
// Stats Output Tests
// ============================================================

#[test]
fn test_replace_stats_shows_reducer() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Reducer:"));
}

#[test]
fn test_replace_stats_shows_output_mode() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode:"));
}

#[test]
fn test_replace_stats_shows_files_affected() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Files affected:"));
}

#[test]
fn test_replace_stats_shows_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run: true"));
}

#[test]
fn test_replace_stats_with_json_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output mode: json"));
}

// ============================================================
// Empty Results Tests
// ============================================================

#[test]
fn test_replace_no_matches_returns_success() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_replace_no_matches_compact_message() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_no_matches_not_dry_run_message() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes made"));
}

// ============================================================
// Regex Pattern Tests
// ============================================================

#[test]
fn test_replace_regex_pattern() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "test123 test456\n",
    );

    // Replace digits with X
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg(r"\d+")
        .arg("X")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 replacements"));
}

#[test]
fn test_replace_regex_word_boundary() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "fn function fn_main\n",
    );

    // Replace whole word "fn" only
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg(r"\bfn\b")
        .arg("FUNC")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_regex_character_class() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "abc ABC\n",
    );

    // Replace lowercase letters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("[a-z]")
        .arg("x")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("3 replacement"));
}

#[test]
fn test_replace_regex_alternation() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo bar baz\n",
    );

    // Replace foo or bar with X
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("foo|bar")
        .arg("X")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 replacement"));
}

#[test]
fn test_replace_invalid_regex_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Invalid regex should return an error
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("[invalid")
        .arg("replacement")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex").or(predicate::str::contains("regex")));
}

// ============================================================
// Special Characters Tests
// ============================================================

#[test]
fn test_replace_with_dashes() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "some-function-name\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("some-function")
        .arg("other-function")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_with_underscores() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "some_variable_name\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("some_variable")
        .arg("other_variable")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_with_dots() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "test.txt file.txt\n",
    );

    // . in regex matches any character, so we need to escape it
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg(r"\.txt")
        .arg(".md")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 replacement"));
}

#[test]
fn test_replace_with_commas() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "a, b, c\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg(", ")
        .arg("; ")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 replacement"));
}

// ============================================================
// Multiple Files Tests
// ============================================================

#[test]
fn test_replace_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "file1.txt",
        "Hello world\n",
    );
    create_temp_file(
        &temp_dir,
        "file2.txt",
        "Hello universe\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 files"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_replace_json_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "file1.txt",
        "Hello world\n",
    );
    create_temp_file(
        &temp_dir,
        "file2.txt",
        "Hello universe\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["counts"]["files_affected"].as_u64().unwrap(), 2);
    assert_eq!(json["counts"]["total_replacements"].as_u64().unwrap(), 2);
}

// ============================================================
// Help and Usage Tests
// ============================================================

#[test]
fn test_replace_help_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"))
        .stdout(predicate::str::contains("PATH"))
        .stdout(predicate::str::contains("SEARCH"))
        .stdout(predicate::str::contains("REPLACE"));
}

#[test]
fn test_replace_help_shows_extension_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--extension"));
}

#[test]
fn test_replace_help_shows_dry_run_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_replace_help_shows_count_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--count"));
}

// ============================================================
// Path Handling Tests
// ============================================================

#[test]
fn test_replace_with_current_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_replace_with_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_file_as_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(&file_path)
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Nonexistent directory should still succeed (empty results)
    cmd.arg("replace")
        .arg("/nonexistent/directory/xyz123")
        .arg("test")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success();
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_replace_empty_search_pattern() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Empty search pattern matches at every position (regex behavior)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _ = cmd
        .arg("replace")
        .arg(temp_dir.path())
        .arg("")
        .arg("X")
        .arg("--dry-run")
        .assert();
}

#[test]
fn test_replace_empty_replacement() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Empty replacement should delete the matched text
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello ")
        .arg("")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_multiline_file() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Line")
        .arg("Row")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("5 replacement"));
}

#[test]
fn test_replace_preserves_file_structure() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file(
        &temp_dir,
        "test.txt",
        "Line 1\nLine 2\nLine 3\n",
    );

    // Perform actual replace
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Line")
        .arg("Row")
        .assert()
        .success();

    // Verify file structure is preserved
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Row 1\n"));
    assert!(content.contains("Row 2\n"));
    assert!(content.contains("Row 3\n"));
}

// ============================================================
// Combined Options Tests
// ============================================================

#[test]
fn test_replace_combined_extension_and_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "old_function();\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "old_function();\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_function")
        .arg("new_function")
        .arg("-e")
        .arg("rs")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("test.txt").not());
}

#[test]
fn test_replace_combined_count_and_extension() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "foo foo foo\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo foo foo foo\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("foo")
        .arg("bar")
        .arg("-e")
        .arg("rs")
        .arg("--count")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_replace_json_with_all_options() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "old_function();\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("old_function")
        .arg("new_function")
        .arg("-e")
        .arg("rs")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
// CSV/TSV Edge Cases Tests
// ============================================================

#[test]
fn test_replace_csv_escapes_commas() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello, world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello, world")
        .arg("hi, there")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("\""));
}

#[test]
fn test_replace_csv_escapes_quotes() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "say \"hello\"\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .assert()
        .success()
        // CSV should escape quotes by doubling them
        .stdout(predicate::str::contains("\"\""));
}

#[test]
fn test_replace_tsv_escapes_tabs() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello\tworld\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello\tworld")
        .arg("hi there")
        .arg("--dry-run")
        .assert()
        .success()
        // TSV should escape tabs
        .stdout(predicate::str::contains("\\t"));
}

#[test]
fn test_replace_tsv_escapes_newlines() {
    let temp_dir = TempDir::new().unwrap();
    // Note: we're searching for a literal \n in the file content
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello\\nworld\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello\\nworld")
        .arg("hi")
        .arg("--dry-run")
        .assert()
        .success();
}

// ============================================================
// Ignored Directories Tests
// ============================================================

#[test]
fn test_replace_ignores_git_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "old_pattern\n",
    );
    
    // Create .git directory with a file
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    create_temp_file(
        &temp_dir,
        ".git/config",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"))
        .stdout(predicate::str::contains(".git").not());
}

#[test]
fn test_replace_ignores_target_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "src.txt",
        "old_pattern\n",
    );
    
    // Create target directory with a file
    fs::create_dir(temp_dir.path().join("target")).unwrap();
    create_temp_file(
        &temp_dir,
        "target/output.txt",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("src.txt"))
        .stdout(predicate::str::contains("target").not());
}

#[test]
fn test_replace_ignores_node_modules_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "index.js",
        "old_pattern\n",
    );
    
    // Create node_modules directory with a file
    fs::create_dir_all(temp_dir.path().join("node_modules/package")).unwrap();
    create_temp_file(
        &temp_dir,
        "node_modules/package/index.js",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.js"))
        .stdout(predicate::str::contains("node_modules").not());
}
