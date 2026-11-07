//! `trs diff` and `trs benchmark` report on a command, then exit as it did, so
//! a script around them sees a failure instead of a clean report.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn the_report_prints_and_the_exit_status_is_the_commands() {
    for (tool, report) in [("diff", "raw:"), ("benchmark", "Raw output:")] {
        for (script, code) in [("exit 0", 0), ("exit 3", 3)] {
            Command::cargo_bin("trs")
                .unwrap()
                .args([tool, "sh", "-c", script])
                .assert()
                .code(code)
                .stdout(predicate::str::contains(report));
        }
    }
}

// Windows starts programs through `cmd`, so a missing one is an ordinary run.
#[cfg(unix)]
#[test]
fn a_command_that_never_started_is_127() {
    for tool in ["diff", "benchmark"] {
        Command::cargo_bin("trs")
            .unwrap()
            .args([tool, "zzz_no_such_program"])
            .assert()
            .code(127)
            .stdout("")
            .stderr(predicate::str::contains("Command not found"));
    }
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_run_is_126() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("not-executable");
    std::fs::write(&file, "text").unwrap();
    for tool in [&["diff"][..], &["benchmark"], &[]] {
        Command::cargo_bin("trs")
            .unwrap()
            .args(tool)
            .arg(&file)
            .assert()
            .code(126);
    }
}

#[cfg(unix)]
#[test]
fn diff_fails_when_the_compact_side_cannot_run() {
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("trs");
    std::fs::copy(assert_cmd::cargo::cargo_bin("trs"), &copy).unwrap();

    // The raw side deletes the binary the compact side would run.
    Command::new(&copy)
        .args(["diff", "rm"])
        .arg(&copy)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("smaller").not())
        .stderr(predicate::str::contains("compact side"));
}
