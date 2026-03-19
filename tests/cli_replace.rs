use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_replace_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_dry_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_DRY_RUN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_preview_flag() {
    // Test that --preview flag works as an alias for --dry-run
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_PREVIEW_12345")
        .arg("new")
        .arg("--preview")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_replace_preview_json_output() {
    // Test that --preview flag sets dry_run to true in JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_PREVIEW_JSON_12345")
        .arg("new")
        .arg("--preview")
        .arg("--extension")
        .arg("rs")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
}

#[test]
fn test_tail_basic() {
    // Create a temporary test file
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "line 2").unwrap();
    writeln!(file, "line 3").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_tail_with_lines() {
    // Create a temporary test file with many lines
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .arg("--lines")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("line 16"))
        .stdout(predicate::str::contains("line 20"))
        .stdout(predicate::function(|s: &str| !s.contains("line 15")));
}

#[test]
fn test_tail_with_errors_flag() {
    // Create a temporary test file with errors
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "INFO: process started").unwrap();
    writeln!(file, "ERROR: something went wrong").unwrap();
    writeln!(file, "WARNING: deprecated API").unwrap();
    writeln!(file, "FATAL: critical failure").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg(path)
        .arg("--errors")
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"))
        .stdout(predicate::str::contains("FATAL"))
        .stdout(predicate::function(|s: &str| !s.contains("INFO")))
        .stdout(predicate::function(|s: &str| !s.contains("WARNING")));
}

#[test]
fn test_tail_json_output() {
    // Create a temporary test file
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
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
    assert!(json["total_lines"].is_number());
    assert!(json["lines_shown"].is_number());
    assert!(!json["filtering_errors"].as_bool().unwrap());
}

#[test]
fn test_tail_csv_output() {
    // Create a temporary test file
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
        .stdout(predicate::str::contains("line_number,line,is_error"))
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_tail_tsv_output() {
    // Create a temporary test file
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
        .stdout(predicate::str::contains("line_number\tline\tis_error"))
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_tail_raw_output() {
    // Create a temporary test file
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

#[test]
fn test_tail_agent_output() {
    // Create a temporary test file
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
        .stdout(predicate::str::contains("File:"))
        .stdout(predicate::str::contains("❌"))
        .stdout(predicate::str::contains("ERROR"));
}

#[test]
fn test_tail_file_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/nonexistent/file.log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

// ============================================================
