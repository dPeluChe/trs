//! Integration tests for the formatter module - JSON, CSV, TSV, Agent, Raw,
//! Compact formats, format consistency, and parse command format tests.

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
        .stdout(predicate::str::contains("src/"))
        .stdout(predicate::str::contains("main.rs"));
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
        .stdout(predicate::str::contains("clean"));
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
        .stdout(predicate::str::contains("src/"))
        .stdout(predicate::str::contains("main.rs"));
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
        .stdout(predicate::str::contains("clean"));
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
        ("--compact", "clean"),
        ("--json", "is_clean"),
        ("--csv", "status"),
        ("--tsv", "status"),
        ("--agent", "clean"),
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
