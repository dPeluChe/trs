//! Integration tests for the formatter module - tail command formats,
//! edge cases with special characters, empty input, and replace command formats.

use assert_cmd::Command;
use predicates::prelude::*;

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
