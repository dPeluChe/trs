use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

// Stats Output Tests
// ============================================================

#[test]
fn test_replace_stats_shows_reducer() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "test123 test456\n");

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
    create_temp_file(&temp_dir, "test.txt", "fn function fn_main\n");

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
    create_temp_file(&temp_dir, "test.txt", "abc ABC\n");

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
    create_temp_file(&temp_dir, "test.txt", "foo bar baz\n");

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
    create_temp_file(&temp_dir, "test.txt", "Hello world\n");

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
    create_temp_file(&temp_dir, "test.txt", "some-function-name\n");

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
    create_temp_file(&temp_dir, "test.txt", "some_variable_name\n");

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
    create_temp_file(&temp_dir, "test.txt", "test.txt file.txt\n");

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
    create_temp_file(&temp_dir, "test.txt", "a, b, c\n");

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
    create_temp_file(&temp_dir, "file1.txt", "Hello world\n");
    create_temp_file(&temp_dir, "file2.txt", "Hello universe\n");

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
    create_temp_file(&temp_dir, "file1.txt", "Hello world\n");
    create_temp_file(&temp_dir, "file2.txt", "Hello universe\n");

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
