use assert_cmd::Command;
use predicates::prelude::*;

// Trim Command Tests
// ============================================================

#[test]
fn test_trim_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trim whitespace"))
        .stdout(predicate::str::contains("--leading"))
        .stdout(predicate::str::contains("--trailing"));
}

#[test]
fn test_trim_basic() {
    // Test basic whitespace trimming (both sides)
    let input = "  hello world  \n\tfoo bar\t\n   baz   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"))
        .stdout(predicate::str::contains("foo bar"))
        .stdout(predicate::str::contains("baz"));
}

#[test]
fn test_trim_default_mode() {
    // Test that default mode trims both leading and trailing
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should trim both sides
    assert!(stdout.contains("hello"));
    assert!(!stdout.contains("  hello"));
    assert!(!stdout.contains("hello  "));
    assert!(stdout.contains("mode: both"));
}

#[test]
fn test_trim_leading_only() {
    // Test trimming leading whitespace only
    let input = "  hello  \n\tworld\t";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--leading")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain "hello  " (trailing preserved) and "world\t" (trailing preserved)
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
    assert!(stdout.contains("mode: leading"));
}

#[test]
fn test_trim_trailing_only() {
    // Test trimming trailing whitespace only
    let input = "  hello  \n\tworld\t";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Should contain "  hello" (leading preserved) and "\tworld" (leading preserved)
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
    assert!(stdout.contains("mode: trailing"));
}

#[test]
fn test_trim_both_flags() {
    // Test with both --leading and --trailing (should be equivalent to default)
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("trim")
        .arg("--leading")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("mode: both"));
}

#[test]
fn test_trim_file_input() {
    // Test trim with file input
    use std::io::Write;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "  line 1  ").unwrap();
    writeln!(file, "\tline 2\t").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("-f")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_trim_file_not_found() {
    // Test trim with non-existent file
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("-f")
        .arg("/nonexistent/path/file.txt")
        .assert()
        .failure();
}

#[test]
fn test_trim_json_output() {
    // Test JSON output format
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["content"], "hello");
    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction"].is_number());
    assert_eq!(json["options"]["leading"], false);
    assert_eq!(json["options"]["trailing"], false);
}

#[test]
fn test_trim_csv_output() {
    // Test CSV output format
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line"))
        .stdout(predicate::str::contains("\"line 1\""))
        .stdout(predicate::str::contains("\"line 2\""));
}

#[test]
fn test_trim_tsv_output() {
    // Test TSV output format
    let input = "  line 1  \n  line 2  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"));
}

#[test]
fn test_trim_agent_output() {
    // Test agent format output
    let input = "  hello  \n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"))
        .stdout(predicate::str::contains("Stats:"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

#[test]
fn test_trim_raw_output() {
    // Test raw output format
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Raw output should just be the trimmed content, no stats
    assert_eq!(stdout.trim(), "hello");
}

#[test]
fn test_trim_empty_input() {
    // Test with empty input
    let input = "";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim").write_stdin(input).assert().success();
}

#[test]
fn test_trim_whitespace_only() {
    // Test with whitespace-only input
    let input = "   \n\t\t\n   ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // All lines should be empty after trimming
    for line in stdout.lines() {
        if !line.contains("% reduction") && !line.contains("mode:") {
            assert!(line.trim().is_empty());
        }
    }
}

#[test]
fn test_trim_no_reduction() {
    // Test with input that has no whitespace to trim
    let input = "hello\nworld";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd.arg("trim").write_stdin(input).assert().success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
}

#[test]
fn test_trim_mixed_whitespace() {
    // Test with various whitespace types
    let input = "  spaces  \n\ttabs\t\n \t mixed \t ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("spaces"))
        .stdout(predicate::str::contains("tabs"))
        .stdout(predicate::str::contains("mixed"));
}

#[test]
fn test_trim_preserves_empty_lines() {
    // Test that empty lines are preserved
    let input = "  hello  \n\n  world  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("trim")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "hello");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "world");
}

#[test]
fn test_trim_json_with_leading_flag() {
    // Test JSON output with --leading flag
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .arg("--leading")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["options"]["leading"], true);
    assert_eq!(json["options"]["trailing"], false);
}

#[test]
fn test_trim_json_with_trailing_flag() {
    // Test JSON output with --trailing flag
    let input = "  hello  ";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("trim")
        .arg("--trailing")
        .write_stdin(input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["options"]["leading"], false);
    assert_eq!(json["options"]["trailing"], true);
}

// ============================================================
