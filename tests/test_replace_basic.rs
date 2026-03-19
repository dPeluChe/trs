use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

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
