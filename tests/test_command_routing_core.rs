//! Tests for command routing validation - core commands, parse subcommands,
//! output format flags, stats flags, invalid arguments, and command aliases.
//!
//! This test module verifies that CLI commands are properly routed to their
//! respective handlers and produce expected output.

use assert_cmd::Command;
use predicates::prelude::*;

mod support;

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
        .stdout(predicate::str::contains("fn main"));
}

#[test]
fn test_replace_command_routes_correctly() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "old\n").unwrap();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(dir.path())
        .arg("old")
        .arg("new")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 replacements in 1 files"));
    let kept = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(kept, "old\n");
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
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("<html><body>Test</body></html>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
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
        .stdout("master\nclean\n");
}

#[test]
fn test_parse_git_diff_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin("diff --git a/test.txt b/test.txt\nindex 1234567..abcdef 100644")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_parse_ls_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("ls")
        .write_stdin("file1.txt\nfile2.rs\ndir1/")
        .assert()
        .success()
        .stdout(predicate::str::contains("[2 files, 1 dirs]"));
}

#[test]
fn test_parse_grep_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("grep")
        .write_stdin("test.rs:10:fn main() {")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs:10:fn main()"));
}

#[test]
fn test_parse_find_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("find")
        .write_stdin("./src/main.rs\n./src/router.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("router"));
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
        .stdout(predicate::str::contains("PASS: 1 tests"));
}

#[test]
fn test_parse_logs_routes_correctly() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("logs")
        .write_stdin("2024-01-01 INFO Application started")
        .assert()
        .success()
        .stdout(predicate::str::contains("lines: 1"));
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
        .stderr(predicate::str::contains("Stats:"));
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
        .stderr(predicate::str::contains("Stats:"));
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
    cmd.arg("search")
        .arg("pattern")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<QUERY>"));
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

#[test]
fn test_format_precedence_picks_the_stronger_flag() {
    let cases: &[(&[&str], &str)] = &[
        (&["--csv", "--tsv"], "command,args,stdout"),
        (&["--tsv", "--agent"], "command\targs\tstdout"),
        (&["--json", "--csv"], "\"command\":\"echo\""),
    ];
    for (flags, header) in cases {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        cmd.args(*flags)
            .args(["run", "echo", "test"])
            .assert()
            .success()
            .stdout(predicate::str::contains(*header));
    }
}

#[test]
fn test_is_clean_exit_code_names_the_repo_state() {
    const ALIASES: [&str; 3] = ["is-clean", "clean?", "repo-clean"];
    let dir = tempfile::tempdir().unwrap();
    let expect = |code: i32| {
        for alias in ALIASES {
            Command::cargo_bin("trs")
                .unwrap()
                .current_dir(dir.path())
                .arg(alias)
                .assert()
                .code(code);
        }
    };
    expect(2);
    support::git(dir.path(), &["init", "-q"]);
    std::fs::write(dir.path().join("a.txt"), "x").unwrap();
    support::git(dir.path(), &["add", "."]);
    support::git(dir.path(), &["commit", "-qm", "init"]);
    expect(0);
    std::fs::write(dir.path().join("a.txt"), "y").unwrap();
    expect(1);
}
