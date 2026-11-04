#![cfg(unix)]
//! `--help` / `-h` print usage, not results: it must come back untouched.
//! `gh pr list -h` used to be read as a PR list ("pull requests: 6") and
//! `cargo test -h` as a passing test run ("ok (0 passed)").

use assert_cmd::Command;
use std::path::Path;

mod support;
use support::{fake_program, path_with};

const USAGE: &str = "List pull requests in a repository\n\nUSAGE\n  gh pr list [flags]\n\nFLAGS\n  -L, --limit int   Maximum number of items to fetch\n";

fn run(dir: &Path, args: &[&str]) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .env("PATH", path_with(dir))
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn usage_text_is_not_parsed_as_a_result() {
    let dir = tempfile::tempdir().unwrap();
    for program in ["gh", "cargo"] {
        fake_program(
            &dir.path().join(program),
            &format!("printf '%s' '{}'", USAGE.replace('\'', "'\\''")),
        );
    }
    for args in [
        &["gh", "pr", "list", "-h"][..],
        &["gh", "pr", "list", "--help"],
        &["cargo", "test", "-h"],
        &["cargo", "test", "--help"],
    ] {
        assert_eq!(run(dir.path(), args), USAGE, "{args:?}");
    }
}

#[test]
fn the_hook_does_not_wrap_a_help_request() {
    for cmd in [
        "cargo test --help",
        "gh pr list -h",
        "rg -h",
        "git log -h",
        "FOO=1 gh pr list -h",
        "sudo gh pr list -h",
    ] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .arg("rewrite")
            .write_stdin(cmd)
            .output()
            .unwrap();
        assert!(out.stdout.is_empty(), "{cmd} was wrapped");
    }
    for cmd in [
        "grep -h foo a.txt",
        "ls -lh",
        "git grep -h foo",
        "git -C . grep -h foo",
        "eza -h",
        "rsync -h a b",
    ] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .arg("rewrite")
            .write_stdin(cmd)
            .output()
            .unwrap();
        assert!(!out.stdout.is_empty(), "{cmd} is a real option, not help");
    }
}
