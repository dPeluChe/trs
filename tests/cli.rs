use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TARS CLI"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("trs"));
}

#[test]
fn test_global_flags_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--raw"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--stats"));
}

#[test]
fn test_search_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for patterns"))
        .stdout(predicate::str::contains("--extension"))
        .stdout(predicate::str::contains("--ignore-case"))
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_replace_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Search and replace"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_tail_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tail a file"))
        .stdout(predicate::str::contains("--errors"))
        .stdout(predicate::str::contains("--follow"));
}

#[test]
fn test_clean_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean and format"))
        .stdout(predicate::str::contains("--no-ansi"))
        .stdout(predicate::str::contains("--collapse-blanks"));
}

#[test]
fn test_parse_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse structured input"))
        .stdout(predicate::str::contains("git-status"))
        .stdout(predicate::str::contains("git-diff"))
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("grep"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("logs"));
}

#[test]
fn test_html2md_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert HTML to Markdown"))
        .stdout(predicate::str::contains("--metadata"));
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
fn test_search_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_search_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("/path/to/dir")
        .arg("pattern")
        .arg("--extension")
        .arg("rs")
        .arg("--ignore-case")
        .assert()
        .success();
}

#[test]
fn test_replace_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_replace_dry_run() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("replace")
        .arg(".")
        .arg("old")
        .arg("new")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn test_tail_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/var/log/test.log")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_tail_with_lines() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("tail")
        .arg("/var/log/test.log")
        .arg("--lines")
        .arg("20")
        .assert()
        .success();
}

#[test]
fn test_clean_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_clean_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("clean")
        .arg("--no-ansi")
        .arg("--collapse-blanks")
        .arg("--trim")
        .assert()
        .success();
}

#[test]
fn test_parse_git_status() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_parse_git_diff() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("git-diff").assert().success();
}

#[test]
fn test_parse_ls() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("ls").assert().success();
}

#[test]
fn test_parse_grep() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("grep").assert().success();
}

#[test]
fn test_parse_test() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .assert()
        .success();
}

#[test]
fn test_parse_logs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse").arg("logs").assert().success();
}

#[test]
fn test_html2md_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_txt2md_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_global_json_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Json"));
}

#[test]
fn test_global_csv_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output format: Csv"));
}

#[test]
fn test_global_stats_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("search")
        .arg(".")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Stats: enabled"));
}

#[test]
fn test_run_command_basic() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("run")
        .arg("echo")
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}
