//! Tests for command routing validation - help text, version flags, stdin processing,
//! multiple command execution, error handling, command-specific flags, and edge cases.
//!
//! This test module verifies that CLI help output, flags, and edge cases work correctly.

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Help Routing Tests
// ============================================================

#[test]
fn test_help_shows_all_commands() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("parse"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("replace"))
        .stdout(predicate::str::contains("tail"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("html2md"))
        .stdout(predicate::str::contains("txt2md"))
        .stdout(predicate::str::contains("trim"))
        .stdout(predicate::str::contains("is-clean"));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Execute a command"));
}

#[test]
fn test_parse_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse structured input"));
}

#[test]
fn test_search_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"));
}

#[test]
fn test_replace_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"));
}

#[test]
fn test_tail_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tail a file"));
}

#[test]
fn test_clean_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean and format"));
}

#[test]
fn test_html2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert HTML to Markdown"));
}

#[test]
fn test_txt2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert plain text to Markdown"));
}

#[test]
fn test_trim_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trim whitespace"));
}

#[test]
fn test_is_clean_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("is-clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Check if git repository is in a clean state",
        ));
}

// ============================================================
// Version Flag Routing Tests
// ============================================================

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("trs"));
}

// ============================================================
// Stdin Processing Routing Tests
// ============================================================

#[test]
fn test_stdin_processing_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("test input line\nanother line")
        .assert()
        .success()
        .stdout(predicate::str::contains("test").or(predicate::str::contains("lines")));
}

#[test]
fn test_stdin_with_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .write_stdin("test input")
        .assert()
        .success();
}

#[test]
fn test_stdin_with_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .write_stdin("test input")
        .assert()
        .success();
}

// ============================================================
// Multiple Command Execution Tests
// ============================================================

#[test]
fn test_run_command_with_multiple_args() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("arg1")
        .arg("arg2")
        .arg("arg3")
        .assert()
        .success()
        .stdout(predicate::str::contains("arg1"))
        .stdout(predicate::str::contains("arg2"))
        .stdout(predicate::str::contains("arg3"));
}

#[test]
fn test_search_with_extension_filter() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn main")
        .arg("-e")
        .arg("rs")
        .assert()
        .success();
}

#[test]
fn test_search_case_insensitive() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("MAIN")
        .arg("--ignore-case")
        .assert()
        .success();
}

#[test]
fn test_replace_dry_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("NONEXISTENT")
        .arg("REPLACEMENT")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_tail_with_line_count() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("Cargo.toml")
        .arg("-n")
        .arg("3")
        .assert()
        .success();
}

// ============================================================
// Error Handling Routing Tests
// ============================================================

#[test]
fn test_run_nonexistent_command() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("nonexistent_command_12345")
        .assert()
        .failure();
}

#[test]
fn test_tail_nonexistent_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("nonexistent_file_12345.txt")
        .assert()
        .failure();
}

#[test]
fn test_html2md_nonexistent_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("nonexistent_file_12345.html")
        .assert()
        .failure();
}

#[test]
fn test_replace_nonexistent_directory() {
    // Replace command succeeds with "No changes made" for non-existent directories
    // since it's designed to be idempotent
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("nonexistent_dir_12345")
        .arg("pattern")
        .arg("replacement")
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes"));
}

// ============================================================
// Command-Specific Flag Routing Tests
// ============================================================

#[test]
fn test_run_with_capture_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test")
        .arg("--capture-stdout")
        .arg("--capture-stderr")
        .assert()
        .success();
}

#[test]
fn test_clean_with_flags() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--collapse-repeats")
        .arg("--trim")
        .write_stdin("  test  ")
        .assert()
        .success();
}

#[test]
fn test_tail_with_errors_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("Cargo.toml")
        .arg("--errors")
        .assert()
        .success();
}

#[test]
fn test_trim_with_leading_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("--leading")
        .write_stdin("  test  ")
        .assert()
        .success();
}

#[test]
fn test_trim_with_trailing_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .arg("--trailing")
        .write_stdin("  test  ")
        .assert()
        .success();
}

#[test]
fn test_html2md_with_metadata_flag() {
    // Create a temp HTML file for testing
    let temp_file = tempfile::NamedTempFile::with_suffix(".html").unwrap();
    std::fs::write(temp_file.path(), "<html><body>Test</body></html>").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(temp_file.path().to_str().unwrap())
        .arg("--metadata")
        .assert()
        .success();
}

// ============================================================
// Parse Command Flag Routing Tests
// ============================================================

#[test]
fn test_parse_git_status_with_count_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .arg("--count")
        .arg("unstaged")
        .write_stdin("M file.txt")
        .assert()
        .success();
}

#[test]
fn test_parse_test_with_runner_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin("1 passed, 0 failed")
        .assert()
        .success();
}

// ============================================================
// Combined Global Flags Tests
// ============================================================

#[test]
fn test_json_and_stats_together() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\""));
}

#[test]
fn test_compact_and_stats_together() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_agent_and_stats_together() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

// ============================================================
// Edge Cases Tests
// ============================================================

#[test]
fn test_empty_stdin() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.write_stdin("").assert().success();
}

#[test]
fn test_run_with_special_characters() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test!@#$%^&*()")
        .assert()
        .success();
}

#[test]
fn test_search_with_regex_pattern() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn\\s+main")
        .assert()
        .success();
}
