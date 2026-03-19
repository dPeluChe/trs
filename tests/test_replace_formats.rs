use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

// CSV Output Format Tests
// ============================================================

#[test]
fn test_replace_csv_has_header() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "file,line_number,original,replaced",
        ));
}

#[test]
fn test_replace_csv_has_file_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "file\tline_number\toriginal\treplaced",
        ));
}

#[test]
fn test_replace_tsv_has_file_path() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
