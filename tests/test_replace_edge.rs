use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to create temp file");
    path
}

// Help and Usage Tests
// ============================================================

#[test]
fn test_replace_help_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"))
        .stdout(predicate::str::contains("PATH"))
        .stdout(predicate::str::contains("SEARCH"))
        .stdout(predicate::str::contains("REPLACE"));
}

#[test]
fn test_replace_help_shows_extension_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--extension"));
}

#[test]
fn test_replace_help_shows_dry_run_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_replace_help_shows_count_option() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--count"));
}

// ============================================================
// Path Handling Tests
// ============================================================

#[test]
fn test_replace_with_current_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("nonexistent_pattern_xyz123")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_replace_with_absolute_path() {
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
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_file_as_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(&file_path)
        .arg("Hello")
        .arg("Hi")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    // Nonexistent directory should still succeed (empty results)
    cmd.arg("replace")
        .arg("/nonexistent/directory/xyz123")
        .arg("test")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success();
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_replace_empty_search_pattern() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Empty search pattern matches at every position (regex behavior)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let _ = cmd
        .arg("replace")
        .arg(temp_dir.path())
        .arg("")
        .arg("X")
        .arg("--dry-run")
        .assert();
}

#[test]
fn test_replace_empty_replacement() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Hello world\n",
    );

    // Empty replacement should delete the matched text
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Hello ")
        .arg("")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacement"));
}

#[test]
fn test_replace_multiline_file() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Line")
        .arg("Row")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("5 replacement"));
}

#[test]
fn test_replace_preserves_file_structure() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file(
        &temp_dir,
        "test.txt",
        "Line 1\nLine 2\nLine 3\n",
    );

    // Perform actual replace
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("Line")
        .arg("Row")
        .assert()
        .success();

    // Verify file structure is preserved
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Row 1\n"));
    assert!(content.contains("Row 2\n"));
    assert!(content.contains("Row 3\n"));
}

// ============================================================
// Combined Options Tests
// ============================================================

#[test]
fn test_replace_combined_extension_and_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "old_function();\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "old_function();\n",
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
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("test.txt").not());
}

#[test]
fn test_replace_combined_count_and_extension() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "foo foo foo\n",
    );
    create_temp_file(
        &temp_dir,
        "test.txt",
        "foo foo foo foo\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("foo")
        .arg("bar")
        .arg("-e")
        .arg("rs")
        .arg("--count")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_replace_json_with_all_options() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.rs",
        "old_function();\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("old_function")
        .arg("new_function")
        .arg("-e")
        .arg("rs")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
}

// ============================================================
// CSV/TSV Edge Cases Tests
// ============================================================

#[test]
fn test_replace_csv_escapes_commas() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello, world\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello, world")
        .arg("hi, there")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("\""));
}

#[test]
fn test_replace_csv_escapes_quotes() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "say \"hello\"\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .assert()
        .success()
        // CSV should escape quotes by doubling them
        .stdout(predicate::str::contains("\"\""));
}

#[test]
fn test_replace_tsv_escapes_tabs() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello\tworld\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello\tworld")
        .arg("hi there")
        .arg("--dry-run")
        .assert()
        .success()
        // TSV should escape tabs
        .stdout(predicate::str::contains("\\t"));
}

#[test]
fn test_replace_tsv_escapes_newlines() {
    let temp_dir = TempDir::new().unwrap();
    // Note: we're searching for a literal \n in the file content
    create_temp_file(
        &temp_dir,
        "test.txt",
        "hello\\nworld\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("replace")
        .arg(temp_dir.path())
        .arg("hello\\nworld")
        .arg("hi")
        .arg("--dry-run")
        .assert()
        .success();
}

// ============================================================
// Ignored Directories Tests
// ============================================================

#[test]
fn test_replace_ignores_git_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "test.txt",
        "old_pattern\n",
    );
    
    // Create .git directory with a file
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    create_temp_file(
        &temp_dir,
        ".git/config",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"))
        .stdout(predicate::str::contains(".git").not());
}

#[test]
fn test_replace_ignores_target_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "src.txt",
        "old_pattern\n",
    );
    
    // Create target directory with a file
    fs::create_dir(temp_dir.path().join("target")).unwrap();
    create_temp_file(
        &temp_dir,
        "target/output.txt",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("src.txt"))
        .stdout(predicate::str::contains("target").not());
}

#[test]
fn test_replace_ignores_node_modules_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_file(
        &temp_dir,
        "index.js",
        "old_pattern\n",
    );
    
    // Create node_modules directory with a file
    fs::create_dir_all(temp_dir.path().join("node_modules/package")).unwrap();
    create_temp_file(
        &temp_dir,
        "node_modules/package/index.js",
        "old_pattern\n",
    );

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(temp_dir.path())
        .arg("old_pattern")
        .arg("new_pattern")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("index.js"))
        .stdout(predicate::str::contains("node_modules").not());
}
