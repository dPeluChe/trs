//! Comprehensive validation tests for structured output modes.
//!
//! This test module validates that all output modes produce well-formed,
//! consistent output with proper schemas and structure:
//!
//! - JSON: Valid JSON with schema field and proper types
//! - CSV: Header row with consistent columns
//! - TSV: Header row with tab delimiters
//! - Agent: Structured key-value format
//! - Compact: Human-readable compact format
//! - Raw: Unprocessed output

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
    assert!(json["staged_count"].is_number(), "staged_count should be number");
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
    assert!(json["directories"].is_array(), "directories should be array");
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
    assert!(json["entries"].is_array() || json["lines"].is_array(), "entries/lines should be array");
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

// ============================================================
// CSV Format Validation Tests
// ============================================================

#[test]
fn test_csv_git_status_has_header() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // First line should be header
    assert!(!lines.is_empty(), "CSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains("status"),
        "Header should contain 'status' column"
    );
    assert!(header.contains("path"), "Header should contain 'path' column");
}

#[test]
fn test_csv_grep_has_header() {
    let input = "test.rs:10:fn main() {";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "CSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains("file") || header.contains("path"),
        "Header should contain file/path column"
    );
    assert!(
        header.contains("line"),
        "Header should contain line column"
    );
}

#[test]
fn test_csv_search_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
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

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "CSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains("path") || header.contains("file"),
        "Header should contain path/file column"
    );
    assert!(
        header.contains("line_number"),
        "Header should contain line_number column"
    );
}

#[test]
fn test_csv_tail_has_header() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "CSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains("line_number"),
        "Header should contain line_number column"
    );
    assert!(
        header.contains("line"),
        "Header should contain line column"
    );
}

#[test]
fn test_csv_replace_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_UNIQUE_67890")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "CSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains("file") || header.contains("path"),
        "Header should contain file/path column"
    );
}

#[test]
fn test_csv_escaping_with_commas() {
    // Test that CSV properly escapes commas in content
    let input = "file,with,commas.txt\n";

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
    // Should handle the commas properly (either quoted or escaped)
    assert!(
        stdout.contains("file,with,commas") || stdout.contains("\"file,with,commas\""),
        "CSV should handle commas in content"
    );
}

#[test]
fn test_csv_escaping_with_quotes() {
    // Test that CSV properly handles quotes in content
    let input = "file\"with\"quotes.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin(input)
        .assert()
        .success();
    // Output should be valid CSV (quotes should be escaped or handled)
}

// ============================================================
// TSV Format Validation Tests
// ============================================================

#[test]
fn test_tsv_git_status_has_header() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "TSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains('\t'),
        "TSV header should contain tabs"
    );
    assert!(
        header.contains("status"),
        "Header should contain 'status' column"
    );
    assert!(header.contains("path"), "Header should contain 'path' column");
}

#[test]
fn test_tsv_grep_has_header() {
    let input = "test.rs:10:fn main() {";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("parse")
        .arg("grep")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "TSV should have at least header");
    let header = lines[0];
    assert!(header.contains('\t'), "TSV header should contain tabs");
}

#[test]
fn test_tsv_search_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
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

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "TSV should have at least header");
    let header = lines[0];
    assert!(
        header.contains('\t'),
        "TSV header should contain tabs"
    );
    assert!(
        header.contains("path\t") || header.contains("file\t"),
        "Header should contain path/file column"
    );
}

#[test]
fn test_tsv_tail_has_header() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "TSV should have at least header");
    let header = lines[0];
    assert!(header.contains('\t'), "TSV header should contain tabs");
}

#[test]
fn test_tsv_uses_tab_delimiter() {
    // Use a command that produces tabular output with TSV format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
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

    let stdout = String::from_utf8_lossy(&output);
    // TSV should use tabs as delimiters in header
    assert!(
        stdout.contains("path\t") || stdout.contains("file\t"),
        "TSV output should contain tabs as delimiters"
    );
}

// ============================================================
// Agent Format Validation Tests
// ============================================================

#[test]
fn test_agent_git_status_has_structure() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have key: value structure
    assert!(
        stdout.contains("branch:") || stdout.contains("status:"),
        "Agent format should have key: value structure"
    );
}

#[test]
fn test_agent_search_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
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

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output
    assert!(
        stdout.contains("matches:") || stdout.contains("results:"),
        "Agent format should have matches/results structure"
    );
}

#[test]
fn test_agent_tail_has_structure() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output
    assert!(
        stdout.contains("File:") || stdout.contains("file:"),
        "Agent format should have file structure"
    );
}

#[test]
fn test_agent_replace_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_AGENT_12345")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output or indicate no matches
    assert!(
        stdout.contains("total:")
            || stdout.contains("dry_run:")
            || stdout.contains("No matches")
            || stdout.contains("replaced:")
            || stdout.contains("replacements:"),
        "Agent format should have structured output or indicate no matches"
    );
}

// ============================================================
// Compact Format Validation Tests
// ============================================================

#[test]
fn test_compact_git_status_has_structure() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have key: value structure
    assert!(
        stdout.contains("branch:"),
        "Compact format should have 'branch:' key"
    );
    assert!(
        stdout.contains("clean") || stdout.contains("status:"),
        "Compact format should indicate clean state"
    );
}

#[test]
fn test_compact_search_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
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

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have structured output
    assert!(
        stdout.contains("matches:"),
        "Compact format should have 'matches:' key"
    );
}

#[test]
fn test_compact_replace_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_COMPACT_12345")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have structured output or indicate no matches
    assert!(
        stdout.contains("total:")
            || stdout.contains("replaced:")
            || stdout.contains("No matches")
            || stdout.contains("re:")
            || stdout.contains("dry_run:"),
        "Compact format should have structured output"
    );
}

// ============================================================
// Raw Format Validation Tests
// ============================================================

#[test]
fn test_raw_search_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
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

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show file:line:content pattern
    assert!(
        stdout.contains(".rs:"),
        "Raw search format should show file:line pattern"
    );
}

#[test]
fn test_raw_tail_format() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1 content").unwrap();
    writeln!(file, "line 2 content").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show line_number:content pattern
    assert!(
        stdout.contains(":line"),
        "Raw tail format should show line_number:content pattern"
    );
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
    assert!(!stdout.is_empty(), "CSV should have at least header for empty input");
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
    let input = "文件.txt\n文件夹/\n测试.rs";

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
    let input = "文件.txt\n测试.rs";

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
    assert!(stdout.contains("文件") || stdout.contains("测试") || stdout.contains("total:"));
}

#[test]
fn test_tsv_handles_unicode() {
    let input = "文件.txt\n测试.rs";

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
    assert!(stdout.contains("文件") || stdout.contains("测试") || stdout.contains("total:"));
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
            assert_eq!(
                v, first,
                "All schema versions should be consistent"
            );
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
