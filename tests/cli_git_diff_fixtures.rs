use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

mod fixtures;
use fixtures::*;

// Git Diff Fixture Tests
// ============================================================

#[test]
fn test_fixture_git_diff_empty() {
    let input = git_diff_empty();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("diff: empty"));
}

#[test]
fn test_fixture_git_diff_modified() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("+2"));
}

#[test]
fn test_fixture_git_diff_added() {
    let input = git_diff_added();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("+5"));
}

#[test]
fn test_fixture_git_diff_deleted() {
    let input = git_diff_deleted();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("- src/deprecated.rs"))
        .stdout(predicate::str::contains("-5"));
}

#[test]
fn test_fixture_git_diff_renamed() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("R"))
        .stdout(predicate::str::contains("old_name.rs"))
        .stdout(predicate::str::contains("new_name.rs"));
}

#[test]
fn test_fixture_git_diff_copied() {
    let input = git_diff_copied();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("C"))
        .stdout(predicate::str::contains("template.rs"))
        .stdout(predicate::str::contains("implementation.rs"));
}

#[test]
fn test_fixture_git_diff_binary() {
    let input = git_diff_binary();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (1)"))
        .stdout(predicate::str::contains("M assets/image.png"));
}

#[test]
fn test_fixture_git_diff_multiple() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (3)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("- src/old.rs"));
}

#[test]
fn test_fixture_git_diff_mixed() {
    let input = git_diff_mixed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (5)"))
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("M src/lib.rs"))
        .stdout(predicate::str::contains("+ src/utils.rs"))
        .stdout(predicate::str::contains("- src/deprecated.rs"));
}

#[test]
fn test_fixture_git_diff_large() {
    let input = git_diff_large();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (10)"))
        .stdout(predicate::str::contains("src/file01.rs"))
        .stdout(predicate::str::contains("src/file10.rs"));
}

#[test]
fn test_fixture_git_diff_long_paths() {
    let input = git_diff_long_paths();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("files (2)"))
        .stdout(predicate::str::contains("nested"));
}

// ============================================================
// Git Diff Fixture JSON Output Tests
// ============================================================

#[test]
fn test_fixture_git_diff_modified_json() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_empty\":false"))
        .stdout(predicate::str::contains("\"total_files\":1"))
        .stdout(predicate::str::contains("\"path\":\"src/main.rs\""))
        .stdout(predicate::str::contains("\"change_type\":\"M\""))
        .stdout(predicate::str::contains("\"additions\":2"));
}

#[test]
fn test_fixture_git_diff_added_json() {
    let input = git_diff_added();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"A\""))
        .stdout(predicate::str::contains("\"path\":\"src/utils.rs\""));
}

#[test]
fn test_fixture_git_diff_deleted_json() {
    let input = git_diff_deleted();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"D\""))
        .stdout(predicate::str::contains("\"path\":\"src/deprecated.rs\""));
}

#[test]
fn test_fixture_git_diff_renamed_json() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"change_type\":\"R\""))
        .stdout(predicate::str::contains("\"new_path\""));
}

#[test]
fn test_fixture_git_diff_binary_json() {
    let input = git_diff_binary();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"is_binary\":true"))
        .stdout(predicate::str::contains("\"path\":\"assets/image.png\""));
}

#[test]
fn test_fixture_git_diff_multiple_json() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_files\":3"))
        .stdout(predicate::str::contains("\"total_additions\":4"))
        .stdout(predicate::str::contains("\"total_deletions\":2"));
}

// ============================================================
// Git Diff Fixture Raw Output Tests
// ============================================================

#[test]
fn test_fixture_git_diff_modified_raw() {
    let input = git_diff_modified();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("M src/main.rs"));
}

#[test]
fn test_fixture_git_diff_multiple_raw() {
    let input = git_diff_multiple();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("M src/main.rs"))
        .stdout(predicate::str::contains("A src/utils.rs"))
        .stdout(predicate::str::contains("D src/old.rs"));
}

#[test]
fn test_fixture_git_diff_renamed_raw() {
    let input = git_diff_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "R src/old_name.rs -> src/new_name.rs",
        ));
}
