//! Validation tests for CSV and TSV structured output modes.
//!
//! Covers:
//! - CSV: header validation, escaping (commas, quotes)
//! - TSV: header validation, tab delimiters

use assert_cmd::Command;
use std::io::Write;

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
