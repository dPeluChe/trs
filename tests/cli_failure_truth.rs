//! Truth guarantees for failed commands.
//!
//! A compressed summary may drop detail, but it must never assert something
//! the command's exit status contradicts. Field report: a failing `npm run
//! build` (exit 2) summarized as "build: ok (0 errors, 0 warnings)", and the
//! reporter nearly merged a branch that doesn't compile.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trs_failtruth_{}_{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn failed_command_reports_exit_status_on_stdout() {
    // stdout, not stderr: agents run `2>/dev/null` to keep context clean, so a
    // stderr-only notice disappears exactly when the command failed.
    let dir = scratch("exitcode");
    Command::cargo_bin("trs")
        .unwrap()
        .current_dir(&dir)
        .args(["git", "log"]) // not a repository → non-zero exit
        .assert()
        .failure()
        .stdout(predicate::str::contains("[trs] exit "));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn failed_command_exit_code_is_preserved() {
    let dir = scratch("code");
    let out = Command::cargo_bin("trs")
        .unwrap()
        .current_dir(&dir)
        .args(["git", "log"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "git log outside a repo must fail");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let code = out.status.code().unwrap_or(1);
    assert!(
        stdout.contains(&format!("[trs] exit {}", code)),
        "reported status must match the real exit code; got: {}",
        stdout
    );
    let _ = fs::remove_dir_all(&dir);
}

/// The reported bug, end to end: a build that fails must never summarize as
/// "ok". The verdict comes from the exit status, so it holds even when the
/// tool's error dialect isn't one the text heuristics recognize.
#[test]
fn failed_build_never_summarizes_as_ok() {
    let dir = scratch("build");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname=\"trsbadcrate\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(
        dir.join("src/main.rs"),
        "fn main() { let _x: String = 42; }\n",
    )
    .unwrap();

    Command::cargo_bin("trs")
        .unwrap()
        .current_dir(&dir)
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .args(["cargo", "build"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("build: ok").not())
        .stdout(predicate::str::contains("FAILED"))
        .stdout(predicate::str::contains("exit"));
    let _ = fs::remove_dir_all(&dir);
}

/// stdin must reach the child: `Command::output()` defaults it to null, which
/// made `python3 - <<EOF` run nothing and still exit 0.
#[test]
#[cfg(not(windows))]
fn stdin_is_forwarded_to_the_child() {
    Command::cargo_bin("trs")
        .unwrap()
        .arg("cat")
        .write_stdin("alpha\nbeta\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn tables_keep_the_failure_notice_out_of_their_rows() {
    for (flag, rows) in [("--csv", "line\n\"row\"\n"), ("--tsv", "line\nrow\n")] {
        Command::cargo_bin("trs")
            .unwrap()
            .args([flag, "sh", "-c", "echo row; exit 3"])
            .assert()
            .code(3)
            .stdout(rows)
            .stderr(predicate::str::contains("[trs] exit 3"));
    }
}

#[test]
fn run_says_so_on_stdout_when_the_command_failed() {
    Command::cargo_bin("trs")
        .unwrap()
        .args(["run", "sh", "-c", "exit 3"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("[trs] exit 3"));
}

#[test]
fn a_command_with_no_table_prints_its_lines_as_rows() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "foo 1\nbar\nfoo 2\n").unwrap();
    for args in [&["grep", "-n", "foo", "a.txt"][..], &["ls"][..]] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .current_dir(dir.path())
            .arg("--csv")
            .args(args)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.starts_with("line\n\""), "{args:?}: {stdout}");
        assert!(!stdout.contains("[1 files]"), "{stdout}");
    }
}

#[test]
fn a_text_command_prints_a_line_table() {
    let home = tempfile::tempdir().unwrap();
    for (flag, header) in [("--csv", "line\n\""), ("--tsv", "line\ntrs history")] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .env("TRS_HOME", home.path())
            .args([flag, "history", "--prune", "--dry-run"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).starts_with(header));
    }
    Command::cargo_bin("trs")
        .unwrap()
        .args(["--csv", "parse", "ls"])
        .write_stdin("a.txt\nb.txt\n")
        .assert()
        .stdout("line\n\"a.txt\"\n\"b.txt\"\n");
}
