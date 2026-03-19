use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

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
