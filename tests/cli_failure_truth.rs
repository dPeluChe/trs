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
