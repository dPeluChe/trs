use assert_cmd::Command;
use predicates::prelude::*;

mod fixtures;
use fixtures::*;

// Git Status Fixture Tests
// ============================================================

#[test]
fn test_fixture_git_status_clean() {
    let input = git_status_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_staged() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (3)"))
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/utils.rs"))
        .stdout(predicate::str::contains("src/old_file.rs"));
}

#[test]
fn test_fixture_git_status_unstaged() {
    let input = git_status_unstaged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("unstaged (3)"))
        .stdout(predicate::str::contains("src/router.rs"))
        .stdout(predicate::str::contains("src/formatter.rs"))
        .stdout(predicate::str::contains("src/deprecated.rs"));
}

#[test]
fn test_fixture_git_status_untracked() {
    let input = git_status_untracked();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("untracked (3)"))
        .stdout(predicate::str::contains("new_feature.rs"))
        .stdout(predicate::str::contains("temp_file.txt"))
        .stdout(predicate::str::contains(".env.local"));
}

#[test]
fn test_fixture_git_status_mixed() {
    let input = git_status_mixed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (2)"))
        .stdout(predicate::str::contains("unstaged (2)"))
        .stdout(predicate::str::contains("untracked (2)"));
}

#[test]
fn test_fixture_git_status_ahead() {
    let input = git_status_ahead();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("feature"))
        .stdout(predicate::str::contains("ahead 3"));
}

#[test]
fn test_fixture_git_status_behind() {
    let input = git_status_behind();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("behind 5"));
}

#[test]
fn test_fixture_git_status_diverged() {
    let input = git_status_diverged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("develop"))
        .stdout(predicate::str::contains("ahead 3"))
        .stdout(predicate::str::contains("behind 5"));
}

#[test]
fn test_fixture_git_status_detached() {
    let input = git_status_detached();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("HEAD detached at abc123"));
}

#[test]
fn test_fixture_git_status_renamed() {
    let input = git_status_renamed();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("R"))
        .stdout(predicate::str::contains("new_name.rs"));
}

#[test]
fn test_fixture_git_status_conflict() {
    let input = git_status_conflict();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("unmerged (3)"))
        .stdout(predicate::str::contains("conflict.rs"));
}

#[test]
fn test_fixture_git_status_porcelain() {
    let input = git_status_porcelain();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/router.rs"))
        .stdout(predicate::str::contains("src/new_file.rs"))
        .stdout(predicate::str::contains("untracked_file.txt"));
}

#[test]
fn test_fixture_git_status_porcelain_v2() {
    let input = git_status_porcelain_v2();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_copied() {
    let input = git_status_copied();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged (1)"))
        .stdout(predicate::str::contains("implementation.rs"));
}

#[test]
fn test_fixture_git_status_typechange() {
    let input = git_status_typechange();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("symlink_to_file"));
}

#[test]
fn test_fixture_git_status_spanish_clean() {
    let input = git_status_spanish_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_german_clean() {
    let input = git_status_german_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_fixture_git_status_empty() {
    let input = git_status_empty();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

#[test]
fn test_fixture_git_status_whitespace_only() {
    let input = git_status_whitespace_only();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success();
}

#[test]
fn test_fixture_git_status_no_branch() {
    let input = git_status_no_branch();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("master"));
}

#[test]
fn test_fixture_git_status_long_paths() {
    let input = git_status_long_paths();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("nested"))
        .stdout(predicate::str::contains("with spaces"));
}

#[test]
fn test_fixture_git_status_all_status_codes() {
    let input = git_status_all_status_codes();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("staged"))
        .stdout(predicate::str::contains("unstaged"))
        .stdout(predicate::str::contains("untracked"))
        .stdout(predicate::str::contains("unmerged"));
}

// ============================================================
// Git Status Fixture JSON Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_clean_json() {
    let input = git_status_clean();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"branch\":\"main\""))
        .stdout(predicate::str::contains("\"is_clean\":true"));
}

#[test]
fn test_fixture_git_status_staged_json() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"staged_count\":3"))
        .stdout(predicate::str::contains("\"status\":\"M\""))
        .stdout(predicate::str::contains("\"status\":\"A\""))
        .stdout(predicate::str::contains("\"status\":\"D\""));
}

#[test]
fn test_fixture_git_status_ahead_json() {
    let input = git_status_ahead();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ahead\":3"));
}

#[test]
fn test_fixture_git_status_behind_json() {
    let input = git_status_behind();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"behind\":5"));
}

#[test]
fn test_fixture_git_status_diverged_json() {
    let input = git_status_diverged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ahead\":3"))
        .stdout(predicate::str::contains("\"behind\":5"));
}

// ============================================================
// Git Status Fixture CSV/TSV Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_staged_csv() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("status,path,new_path,section"))
        .stdout(predicate::str::contains("M,src/main.rs"))
        .stdout(predicate::str::contains("A,src/utils.rs"));
}

#[test]
fn test_fixture_git_status_staged_tsv() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("status\tpath\tnew_path\tsection"))
        .stdout(predicate::str::contains("M\tsrc/main.rs"));
}

// ============================================================
// Git Status Fixture Raw Output Tests
// ============================================================

#[test]
fn test_fixture_git_status_staged_raw() {
    let input = git_status_staged();
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/utils.rs"));
}

// ============================================================
