use assert_cmd::Command;
use predicates::prelude::*;

// JSON Output Tests for Not-Implemented Commands
// ============================================================

#[test]
fn test_search_json_output_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // The output should be valid JSON with the grep_output schema
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema"]["type"], "grep_output");
    assert!(!json["is_empty"].as_bool().unwrap());
    assert!(!json["files"].as_array().unwrap().is_empty());
}

#[test]
fn test_replace_json_output_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // The output should be valid JSON with the replace_output schema
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema"]["type"], "replace_output");
    assert!(json["dry_run"].as_bool().unwrap());
    assert_eq!(json["search_pattern"], "NONEXISTENT_PATTERN_12345");
    assert_eq!(json["replacement"], "new");
    // Verify counts are present
    assert!(json["counts"]["files_affected"].is_number());
    assert!(json["counts"]["total_replacements"].is_number());
}

#[test]
fn test_replace_affected_file_count() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace in dry-run mode with JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Verify the affected file count is correct (2 files with matches)
    assert_eq!(json["counts"]["files_affected"].as_u64().unwrap(), 2);
    // Verify total replacements (2 in file1 + 1 in file2 = 3)
    assert_eq!(json["counts"]["total_replacements"].as_u64().unwrap(), 3);
    // Verify the files array has the correct length
    assert_eq!(json["files"].as_array().unwrap().len(), 2);
}

#[test]
fn test_replace_count_flag() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace with --count flag (default output format)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output just the count (3 total replacements: 2 in file1 + 1 in file2)
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn test_replace_count_flag_json_output() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with known patterns
    fs::write(temp_path.join("file1.txt"), "hello world\nhello again").unwrap();
    fs::write(temp_path.join("file2.txt"), "hello everyone").unwrap();
    fs::write(temp_path.join("file3.txt"), "goodbye").unwrap(); // No match

    // Run replace with --count flag and JSON output
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("replace")
        .arg(temp_path)
        .arg("hello")
        .arg("hi")
        .arg("--dry-run")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Should output just the count in JSON format
    assert_eq!(json["count"].as_u64().unwrap(), 3);
}

#[test]
fn test_replace_count_flag_no_matches() {
    // Test that --count returns 0 when there are no matches
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345_UNIQUE")
        .arg("new")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .arg("--count")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "0");
}

#[test]
fn test_clean_json_output() {
    // Test that clean command produces valid JSON output
    let input = "  hello world  \n\n\n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["content"].is_string());
    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction_percent"].is_number());
}

#[test]
fn test_clean_file_input() {
    // Test clean with file input
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "  line 1  ").unwrap();
    writeln!(file, "\n\n").unwrap();
    writeln!(file, "  line 2  ").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--file")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_clean_file_not_found() {
    // Test clean with non-existent file
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--file")
        .arg("/nonexistent/file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_clean_no_ansi() {
    // Test ANSI code removal
    let input = "\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Red Green"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_csi_sequences() {
    // Test CSI (Control Sequence Introducer) sequences
    let input = "\x1b[1mBold\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bold Underline Reverse"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_multiple_params() {
    // Test ANSI codes with multiple parameters
    let input = "\x1b[1;31;42mBold Red on Green\x1b[0m";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bold Red on Green"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_osc_sequences() {
    // Test OSC (Operating System Command) sequences
    let input = "Title\x1b]0;Window Title\x07Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TitleText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_osc_with_st() {
    // Test OSC sequences with String Terminator (ST)
    let input = "Title\x1b]0;Window Title\x1b\\Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TitleText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_hyperlinks() {
    // Test hyperlink sequences (OSC 8)
    let input = "Click \x1b]8;;http://example.com\x07here\x1b]8;;\x07 now";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Click here now"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b]")));
}

#[test]
fn test_clean_no_ansi_simple_escapes() {
    // Test simple two-character escape sequences
    let input = "Before\x1bcAfter"; // RIS (Reset to Initial State)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("BeforeAfter"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1bc")));
}

#[test]
fn test_clean_no_ansi_cursor_movement() {
    // Test cursor movement sequences
    let input = "Line 1\x1b[2A\x1b[10;20H\x1b[JLine 2";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1Line 2"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b[")));
}

#[test]
fn test_clean_no_ansi_character_sets() {
    // Test character set selection sequences
    let input = "Text\x1b(BMore\x1b)0Text";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("TextMoreText"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b(")));
}

#[test]
fn test_clean_no_ansi_mixed() {
    // Test mixed ANSI sequences
    let input = "\x1b[1;31mError:\x1b[0m \x1b]8;;file:///path\x07/path/to/file\x1b]8;;\x07\x1b[K";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Error: /path/to/file"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b")));
}

#[test]
fn test_clean_no_ansi_real_world() {
    // Test real-world terminal output with various ANSI codes
    let input = "\x1b[?25lHidden cursor\x1b[?25h\x1b[2KProgress: 50%\x1b[0K";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hidden cursorProgress: 50%"))
        .stdout(predicate::function(|s: &str| !s.contains("\x1b")));
}
