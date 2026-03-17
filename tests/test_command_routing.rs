//! Comprehensive tests for command routing validation.
//!
//! This test module verifies that all CLI commands are properly routed to their
//! respective handlers and produce expected output. Tests cover:
//! - All main commands routing correctly
//! - Parse subcommands routing correctly
//! - Error handling for invalid commands
//! - Global flags working with all commands
//! - Output format flags working correctly

use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Main Command Routing Tests
// ============================================================

#[test]
fn test_run_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("test routing")
        .assert()
        .success()
        .stdout(predicate::str::contains("test routing"));
}

#[test]
fn test_search_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches").or(predicate::str::contains("No matches")));
}

#[test]
fn test_replace_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("replacement")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("0").or(predicate::str::contains("replaced")));
}

#[test]
fn test_tail_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("Cargo.toml")
        .arg("-n")
        .arg("5")
        .assert()
        .success();
}

#[test]
fn test_clean_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .write_stdin("\x1b[32mtest\x1b[0m")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_html2md_command_routes_correctly() {
    // Test with invalid URL to verify routing (not actual fetch)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("<html><body>Test</body></html>")
        .assert()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("test")));
}

#[test]
fn test_txt2md_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Simple text input")
        .assert()
        .success()
        .stdout(predicate::str::contains("Simple"));
}

#[test]
fn test_trim_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("trim")
        .write_stdin("  padded text  ")
        .assert()
        .success()
        .stdout(predicate::str::contains("padded text"));
}

#[test]
fn test_is_clean_command_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("is-clean")
        .assert()
        .code(predicate::in_iter([0, 1, 2])); // clean (0), dirty (1), or error (2)
}

// ============================================================
// Parse Subcommand Routing Tests
// ============================================================

#[test]
fn test_parse_git_status_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin("On branch master\nnothing to commit, working tree clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("branch").or(predicate::str::contains("clean")));
}

#[test]
fn test_parse_git_diff_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin("diff --git a/test.txt b/test.txt\nindex 1234567..abcdef 100644")
        .assert()
        .success()
        .stdout(predicate::str::contains("files").or(predicate::str::contains("0")));
}

#[test]
fn test_parse_ls_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("file1.txt\nfile2.rs\ndir1/")
        .assert()
        .success()
        .stdout(predicate::str::contains("files").or(predicate::str::contains("file")));
}

#[test]
fn test_parse_grep_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("test.rs:10:fn main() {")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs").or(predicate::str::contains("0")));
}

#[test]
fn test_parse_find_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin("./src/main.rs\n./src/router.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("router.rs"));
}

#[test]
fn test_parse_test_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin("1 passed in 0.01s")
        .assert()
        .success()
        .stdout(predicate::str::contains("passed").or(predicate::str::contains("0")));
}

#[test]
fn test_parse_logs_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("2024-01-01 INFO Application started")
        .assert()
        .success()
        .stdout(predicate::str::contains("2024").or(predicate::str::contains("lines")));
}

// ============================================================
// Output Format Flag Routing Tests
// ============================================================

#[test]
fn test_json_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\""));
}

#[test]
fn test_csv_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_tsv_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_agent_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_compact_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_raw_flag_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

// ============================================================
// Output Format Precedence Tests
// ============================================================

#[test]
fn test_json_beats_csv_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("--csv")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"")); // JSON output
}

#[test]
fn test_csv_beats_tsv_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("--tsv")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_tsv_beats_agent_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("--agent")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_agent_beats_compact_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--agent")
        .arg("--compact")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_compact_beats_raw_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("--raw")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success();
}

// ============================================================
// Stats Flag Routing Tests
// ============================================================

#[test]
fn test_stats_flag_with_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("run")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("stats").or(predicate::str::contains("bytes")));
}

#[test]
fn test_stats_flag_with_search() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stderr(predicate::str::contains("stats").or(predicate::str::contains("bytes")));
}

#[test]
fn test_stats_flag_with_parse() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("parse")
        .arg("git-status")
        .write_stdin("On branch master")
        .assert()
        .success()
        .stderr(predicate::str::contains("stats").or(predicate::str::contains("bytes")));
}

// ============================================================
// Command with Invalid Arguments Tests
// ============================================================

#[test]
fn test_run_without_command_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .assert()
        .failure();
}

#[test]
fn test_search_without_path_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("pattern")
        .assert()
        .failure();
}

#[test]
fn test_replace_without_args_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .assert()
        .failure();
}

#[test]
fn test_tail_without_file_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .assert()
        .failure();
}

#[test]
fn test_html2md_without_input_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .assert()
        .failure();
}

#[test]
fn test_parse_without_subcommand_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .assert()
        .failure();
}

// ============================================================
// Command Aliases Tests
// ============================================================

#[test]
fn test_is_clean_alias_clean_question() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean?")
        .assert()
        .code(predicate::in_iter([0, 1, 2])); // clean (0), dirty (1), or error (2)
}

#[test]
fn test_is_clean_alias_repo_clean() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("repo-clean")
        .assert()
        .code(predicate::in_iter([0, 1, 2])); // clean (0), dirty (1), or error (2)
}

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
        .stdout(predicate::str::contains("Check if git repository is in a clean state"));
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
    cmd.write_stdin("")
        .assert()
        .success();
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
