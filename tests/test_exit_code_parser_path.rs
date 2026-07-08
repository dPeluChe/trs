//! Exit-code fidelity through the *compressing parser path*
//! (bare `trs <cmd>` → `classifier_exec::execute_and_parse`), as opposed to
//! `trs run <cmd>` which is covered in `test_run_basic.rs`.
//!
//! rtk 0.43.0 shipped fixes for git status/commit reporting `ok` (exit 0)
//! when the underlying command failed (rtk #2494/#2501). trs already
//! propagates `output.status.code()` on every branch of `execute_and_parse`;
//! these tests lock that so a future parser change can't silently swallow a
//! non-zero exit an agent relies on (e.g. `grep` "no match" = 1).

use assert_cmd::Command;
use std::fs;

fn trs() -> Command {
    Command::cargo_bin("trs").unwrap()
}

#[test]
fn grep_no_match_propagates_exit_1() {
    // POSIX grep: 0 = match, 1 = no match, 2 = error. The "no match = 1"
    // convention drives agent control flow, so the compact path must keep it.
    let dir = std::env::temp_dir().join("trs_exit_grep");
    fs::create_dir_all(&dir).unwrap();
    let f = dir.join("f.txt");
    fs::write(&f, "hello\nworld\n").unwrap();

    trs()
        .arg("grep")
        .arg("zzz-no-such-line")
        .arg(&f)
        .assert()
        .code(1);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn grep_match_propagates_success() {
    let dir = std::env::temp_dir().join("trs_exit_grep_ok");
    fs::create_dir_all(&dir).unwrap();
    let f = dir.join("f.txt");
    fs::write(&f, "hello\nworld\n").unwrap();

    trs().arg("grep").arg("hello").arg(&f).assert().success();

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn git_status_outside_repo_fails() {
    // git status outside a repo exits non-zero (128 on standard git). The
    // GitStatus parser must not mask that as success.
    let dir = std::env::temp_dir().join("trs_exit_gitstatus_norepo");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    trs()
        .arg("git")
        .arg("status")
        .current_dir(&dir)
        .assert()
        .failure();

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ls_missing_dir_fails() {
    trs()
        .arg("ls")
        .arg("/no-such-dir-trs-exit-test-xyz")
        .assert()
        .failure();
}
