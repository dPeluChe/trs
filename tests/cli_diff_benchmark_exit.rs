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
