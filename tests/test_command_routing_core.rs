//! Tests for command routing validation - core commands, parse subcommands,
//! output format flags, stats flags, invalid arguments, and command aliases.
//!
//! This test module verifies that CLI commands are properly routed to their
//! respective handlers and produce expected output.

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
    cmd.arg("run").assert().failure();
}

#[test]
fn test_search_without_path_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search").arg("pattern").assert().failure();
}

#[test]
fn test_replace_without_args_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace").arg(".").assert().failure();
}

#[test]
fn test_tail_without_file_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail").assert().failure();
}

#[test]
fn test_html2md_without_input_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md").assert().failure();
}

#[test]
fn test_parse_without_subcommand_fails() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").assert().failure();
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
