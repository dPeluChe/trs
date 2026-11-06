//! A child killed by a signal has no exit code. trs reports 128+N, as a shell
//! does, on every path that runs a command.
#![cfg(unix)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn a_killed_child_exits_with_128_plus_the_signal() {
    for (signal, code) in [("KILL", 137), ("TERM", 143)] {
        let script = format!("kill -{signal} $$");
        let paths: [&[&str]; 3] = [
            &["sh", "-c", &script],
            &["run", "sh", "-c", &script],
            &["--raw", "sh", "-c", &script],
        ];
        for args in paths {
            Command::cargo_bin("trs")
                .unwrap()
                .args(args)
                .assert()
                .code(code);
        }
    }
}

#[test]
fn the_failure_notice_names_the_real_status() {
    Command::cargo_bin("trs")
        .unwrap()
        .args(["sh", "-c", "kill -KILL $$"])
        .assert()
        .stdout(predicate::str::contains("[trs] exit 137"));
    Command::cargo_bin("trs")
        .unwrap()
        .args(["run", "sh", "-c", "kill -KILL $$"])
        .assert()
        .stderr(predicate::str::contains("exited with code 137"));
}
