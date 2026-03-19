use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Raw Format Tests
// ============================================================

#[test]
fn test_run_command_raw_format() {
    // Raw format should output unprocessed stdout
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("raw_output_test")
        .assert()
        .success()
        .stdout(predicate::str::contains("raw_output_test"));
}

#[test]
fn test_run_command_raw_format_with_stderr() {
    // Raw format should include both stdout and stderr
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("sh")
        .arg("-c")
        .arg("echo stdout_test && echo stderr_test >&2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout_test"))
        .stdout(predicate::str::contains("stderr_test"));
}

#[test]
fn test_parse_git_status_raw_format() {
    // Test raw format for git status parsing
    let status_input = "On branch main\nYour branch is up to date.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // Raw format should show simple status/path pairs
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("? new_file.txt"));
}

#[test]
fn test_parse_git_diff_raw_format() {
    // Test raw format for git diff parsing
    let diff_input = "diff --git a/src/main.rs b/src/main.rs\nindex 1234567..abcdefg 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,5 +1,6 @@\n fn main() {\n-    println!(\"new\");\n+    println!(\"new\");\n }\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(diff_input)
        .assert()
        .success()
        // Raw format should show file with change type
        .stdout(predicate::str::contains("M src/main.rs"));
}

#[test]
fn test_parse_find_raw_format() {
    // Test raw format for find parsing
    let find_input = "./src/main.rs\n./src/lib.rs\n./tests/test.rs\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("find")
        .write_stdin(find_input)
        .assert()
        .success()
        // Raw format should show just the paths
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("lib.rs"))
        .stdout(predicate::str::contains("test.rs"))
        // Should not include metadata like "total:"
        .stdout(predicate::function(|x: &str| !x.contains("total:")));
}

#[test]
fn test_parse_logs_raw_format() {
    // Test raw format for log parsing
    let logs_input = "2024-01-15 10:30:00 INFO Application started\n2024-01-15 10:30:01 ERROR Connection failed\n2024-01-15 10:30:02 INFO Retrying...\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("logs")
        .write_stdin(logs_input)
        .assert()
        .success()
        // Raw format should show just the log lines
        .stdout(predicate::str::contains("Application started"))
        .stdout(predicate::str::contains("Connection failed"))
        .stdout(predicate::str::contains("Retrying..."));
}

#[test]
fn test_parse_pytest_raw_format() {
    // Test raw format for pytest output
    let pytest_input = "tests/test_main.py::test_add PASSED\ntests/test_main.py::test_subtract FAILED\n1 passed, 1 failed in 1.23s\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Raw format should show minimal output
        .stdout(predicate::str::contains("tests/test_main.py::test_add"))
        .stdout(predicate::str::contains(
            "tests/test_main.py::test_subtract",
        ));
}

#[test]
fn test_parse_jest_raw_format() {
    // Test raw format for Jest output
    let jest_input = "PASS src/utils.test.js\n  ✓ should add numbers (5ms)\n  ✓ should subtract numbers (3ms)\n\nTest Suites: 1 passed, 1 total\nTests:       2 passed, 2 total\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
}

#[test]
fn test_parse_vitest_raw_format() {
    // Test raw format for Vitest output
    let vitest_input = " ✓ src/math.test.ts > add (5ms)\n ✓ src/math.test.ts > subtract (3ms)\n\n Test Files  1 passed (1)\n      Tests  2 passed (2)\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
}

#[test]
fn test_parse_npm_test_raw_format() {
    // Test raw format for npm test output
    let npm_input = "\n> project@1.0.0 test\n> jest\n\nPASS src/test.js\n  ✓ test1 (5ms)\n\nTest Suites: 1 passed, 1 total\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
}

#[test]
fn test_raw_format_precedence_over_default() {
    // Test that --raw explicitly sets raw format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_raw_format_lower_precedence_than_json() {
    // Test that JSON has higher precedence than Raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        // JSON format should be used
        .stdout(predicate::str::contains("\"exit_code\""))
        .stdout(predicate::str::contains("\"stdout\""));
}

#[test]
fn test_raw_format_lower_precedence_than_compact() {
    // Test that Compact has higher precedence than Raw
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        // Compact format should be used (just the output)
        .stdout(predicate::str::contains("test"));
}

// ============================================================
// Stdin Input Tests
// ============================================================

#[test]
fn test_stdin_basic_input() {
    // Test reading basic input from stdin without a command
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Hello World")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_stdin_with_trailing_whitespace() {
    // Test that trailing whitespace is trimmed
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Line 1   \nLine 2   ")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_collapses_blank_lines() {
    // Test that multiple blank lines are collapsed into one
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("Line 1\n\n\n\nLine 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_strips_ansi_codes() {
    // Test that ANSI escape codes are stripped
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("\x1b[31mRed Text\x1b[0m Normal Text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Red Text"))
        .stdout(predicate::str::contains("Normal Text"))
        .stdout(predicate::str::contains("\x1b[").not());
}

#[test]
fn test_stdin_with_json_format() {
    // Test JSON output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("Test Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"stats\""))
        .stdout(predicate::str::contains("Test Content"));
}

#[test]
fn test_stdin_with_raw_format() {
    // Test raw output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .write_stdin("Raw Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Raw Content"));
}

#[test]
fn test_stdin_with_csv_format() {
    // Test CSV output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .write_stdin("Line 1\nLine 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"));
}

#[test]
fn test_stdin_with_agent_format() {
    // Test agent output format with stdin input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .write_stdin("Agent Content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content:"))
        .stdout(predicate::str::contains("Agent Content"));
}

// ============================================================
// Malformed Input Handling Tests
// ============================================================

#[test]
fn test_stdin_handles_null_bytes() {
    // Test that null bytes are removed gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("hello\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("helloworld"))
        .stdout(predicate::function(|x: &str| !x.contains('\x00')));
}

#[test]
fn test_stdin_handles_control_characters() {
    // Test that control characters (except newline/tab) are replaced with spaces
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("hello\x01\x02world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_stdin_preserves_newlines_and_tabs() {
    // Test that newlines and tabs are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("line1\nline2\ttabbed")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1\nline2\ttabbed"));
}

#[test]
fn test_stdin_handles_ansi_and_control_chars() {
    // Test combination of ANSI codes and control characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("\x1b[31mhello\x1b[0m\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("helloworld"))
        .stdout(predicate::function(|x: &str| !x.contains("\x1b[31m")));
}

#[test]
fn test_stdin_json_format_with_null_bytes() {
    // Test JSON output handles null bytes correctly
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("hello\x00world")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\":\"helloworld\""));
}

#[test]
fn test_parse_git_status_handles_malformed_input() {
    // Test that malformed git status input is handled gracefully
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("garbage:invalid:data::::here")
        .assert()
        .success(); // Should not crash
}

#[test]
fn test_parse_git_status_with_null_bytes() {
    // Test git status parsing with null bytes in input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("On branch main\x00\nmodified: file.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_parse_git_status_with_up_to_date() {
    // Test git status parsing with "Your branch is up to date" line
    let status_input = "On branch main\nYour branch is up to date with 'origin/main'.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // Should show staged section with count
        .stdout(predicate::str::contains("staged (1)"))
        // Should show untracked section with count
        .stdout(predicate::str::contains("untracked (1)"))
        // Should show the staged file
        .stdout(predicate::str::contains("M src/main.rs"))
        // Should show the untracked file
        .stdout(predicate::str::contains("?? new_file.txt"))
        // Should NOT incorrectly parse "Your branch is up to date" as a file
        .stdout(predicate::function(|x: &str| !x.contains("Yo")));
}

#[test]
fn test_parse_git_status_with_up_to_date_json() {
    // Test git status JSON output with "Your branch is up to date" line
    let status_input = "On branch main\nYour branch is up to date with 'origin/main'.\n\nChanges to be committed:\n  modified:   src/main.rs\n\nUntracked files:\n  new_file.txt\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(status_input)
        .assert()
        .success()
        // JSON should have correct counts
        .stdout(predicate::str::contains("\"staged_count\":1"))
        .stdout(predicate::str::contains("\"untracked_count\":1"))
        // JSON should have correct staged file
        .stdout(predicate::str::contains("\"status\":\"M\""))
        .stdout(predicate::str::contains("\"path\":\"src/main.rs\""))
        // JSON should NOT contain malformed entries
        .stdout(predicate::function(|x: &str| !x.contains("Yo")));
}

#[test]
fn test_parse_logs_with_control_chars() {
    // Test logs parsing with control characters
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("[INFO] Starting\x00\n[ERROR] Failed\x01")
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting"));
}

#[test]
fn test_parse_grep_with_malformed_lines() {
    // Test grep parsing with malformed lines
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("valid:10:match\nmalformed_line\nanother:20:match")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_stdin_empty_input() {
    // Test empty input handling
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("").assert().success();
}

#[test]
fn test_stdin_only_whitespace() {
    // Test whitespace-only input
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("   \n\n   \t  ").assert().success();
}
