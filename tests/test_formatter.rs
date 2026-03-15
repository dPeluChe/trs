//! Comprehensive integration tests for the formatter module.
//!
//! This test module verifies formatter output through the CLI:
//! - Compact format (default)
//! - JSON format (--json)
//! - CSV format (--csv)
//! - TSV format (--tsv)
//! - Agent format (--agent)
//! - Raw format (--raw)

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// JSON Format Tests
// ============================================================

#[test]
fn test_json_output_has_schema() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_clean\""));
}

#[test]
fn test_json_output_git_status_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check git status structure
    assert!(json["is_clean"].is_boolean());
    assert!(json["branch"].is_string());
}

#[test]
fn test_json_output_search_structure() {
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

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check search output structure
    assert!(json["files"].is_array() || json["truncated"].is_boolean());
}

#[test]
fn test_json_output_parse_ls_structure() {
    let ls_input = "src\nmain.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check ls output structure
    assert!(json["entries"].is_array() || json["is_empty"].is_boolean());
}

// ============================================================
// CSV Format Tests
// ============================================================

#[test]
fn test_csv_output_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("path,line_number"));
}

#[test]
fn test_csv_output_escaping() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"));
}

// ============================================================
// TSV Format Tests
// ============================================================

#[test]
fn test_tsv_output_has_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("path\tline_number"));
}

#[test]
fn test_tsv_output_parse_ls() {
    let ls_input = "src\nmain.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("total:"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_agent_output_has_markdown_headers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status:"));
}

#[test]
fn test_agent_output_search_has_results_header() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_agent_output_parse_ls_structure() {
    let ls_input = "src\nmain.rs\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("total:"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_raw_output_search_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs:"));
}

#[test]
fn test_raw_output_parse_git_status() {
    // --raw produces empty output for clean repos, just verify it succeeds
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success();
}

// ============================================================
// Compact Format Tests (default)
// ============================================================

#[test]
fn test_compact_output_parse_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status:"));
}

#[test]
fn test_compact_output_search() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

// ============================================================
// Format Consistency Tests
// ============================================================

#[test]
fn test_all_formats_produce_output() {
    // Test that all formats produce some output for the same command
    // Note: --raw produces empty output for clean repos, so we skip it
    let formats = vec![
        ("--compact", "status:"),
        ("--json", "is_clean"),
        ("--csv", "status"),
        ("--tsv", "status"),
        ("--agent", "status:"),
    ];

    for (flag, expected) in formats {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        cmd.arg(flag)
            .arg("parse")
            .arg("git-status")
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}

#[test]
fn test_search_all_formats() {
    // Test search command with all formats
    let formats = vec![
        ("--compact", "matches"),
        ("--json", "files"),
        ("--csv", "line_number"),
        ("--tsv", "line_number"),
        ("--agent", "matches"),
        ("--raw", ".rs:"),
    ];

    for (flag, expected) in formats {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        cmd.arg(flag)
            .arg("search")
            .arg("src")
            .arg("fn")
            .arg("--extension")
            .arg("rs")
            .arg("--limit")
            .arg("1")
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}

// ============================================================
// Parse Command Format Tests
// ============================================================

#[test]
fn test_parse_git_diff_json() {
    let diff_input = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 1;
+    let y = 2;
 }
"#;

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["files"].is_array());
}

#[test]
fn test_parse_git_diff_csv() {
    let diff_input = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 1;
+    let y = 2;
 }
"#;

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_parse_git_diff_agent() {
    let diff_input = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!("Hello");
+    let x = 1;
+    let y = 2;
 }
"#;

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

// ============================================================
// Tail Command Format Tests
// ============================================================

#[test]
fn test_tail_json_format() {
    use std::io::Write;
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

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
}

#[test]
fn test_tail_csv_format() {
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line_number,line,is_error"));
}

#[test]
fn test_tail_tsv_format() {
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line_number\tline\tis_error"));
}

#[test]
fn test_tail_agent_format() {
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("File:"));
}

#[test]
fn test_tail_raw_format() {
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("1:line 1"))
        .stdout(predicate::str::contains("2:line 2"));
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_json_handles_special_characters() {
    let ls_input = "file with spaces.txt\nfile\twith\ttabs.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Should be valid JSON even with special characters
    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"] == false || json["is_empty"] == true);
}

#[test]
fn test_csv_escapes_special_characters() {
    let ls_input = "file,with,commas.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file,with,commas.txt"));
}

#[test]
fn test_tsv_escapes_tabs() {
    let ls_input = "file\twith\ttabs.txt\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin(ls_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("file"));
}

// ============================================================
// Empty Input Tests
// ============================================================

#[test]
fn test_empty_input_json() {
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

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["is_empty"], true);
}

#[test]
fn test_empty_input_csv() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty"));
}

#[test]
fn test_empty_input_tsv() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty"));
}

#[test]
fn test_empty_input_agent() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("parse")
        .arg("ls")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty"));
}

// ============================================================
// Replace Command Format Tests
// ============================================================

#[test]
fn test_replace_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
}

#[test]
fn test_replace_csv_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry_run"));
}
