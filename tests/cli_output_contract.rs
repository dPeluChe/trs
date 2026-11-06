//! What `--json` and `--csv` promise on stdout, on every platform. Windows has
//! no `sh`, so `FAILING` picks the script for the host.

use assert_cmd::Command;

mod support;

#[cfg(unix)]
const FAILING: [&str; 3] = ["sh", "-c", "echo error: x; echo bad >&2; exit 3"];
#[cfg(windows)]
const FAILING: [&str; 3] = ["cmd", "/c", "echo error: x& echo bad 1>&2& exit 3"];

fn trs(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}

fn failing(flags: &[&'static str]) -> Vec<&'static str> {
    [flags, FAILING.as_slice()].concat()
}

/// A document on stdout saying `code`, the same exit code, nothing on stderr.
fn failed_document(args: &[&str], code: i32) -> serde_json::Value {
    let out = trs(args);
    assert_eq!(out.status.code(), Some(code), "{args:?}");
    assert!(out.stderr.is_empty(), "{args:?}: {:?}", out.stderr);
    let json = support::document(&out);
    assert_eq!(json["exit_code"], code, "{args:?}");
    json
}

fn assert_not_found(json: &serde_json::Value) {
    assert_eq!(json["error"], true);
    assert_eq!(json["success"], false);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Command not found"));
}

#[test]
fn a_command_that_never_started_is_a_document_too() {
    let commands: [&[&str]; 3] = [
        &["--json", "err", "zzz_no_such_program"],
        &["diff", "--json", "zzz_no_such_program"],
        &["benchmark", "--json", "zzz_no_such_program"],
    ];
    for args in commands {
        assert_not_found(&failed_document(args, 127));
    }
}

#[test]
fn a_missing_program_is_one_failed_document() {
    // Windows starts programs through `cmd`, which fails with exit 1 itself.
    let expected = if cfg!(windows) { 1 } else { 127 };
    let commands: [&[&str]; 2] = [
        &["--json", "zzz_no_such_program"],
        &["--json", "run", "zzz_no_such_program"],
    ];
    for args in commands {
        let json = failed_document(args, expected);
        if cfg!(unix) {
            assert_not_found(&json);
        }
    }
}

#[test]
fn the_childs_stderr_stays_inside_the_document() {
    let json = failed_document(&failing(&["--json"]), 3);
    assert_eq!(json["success"], false);
    assert_eq!(json["stderr"].as_str().unwrap().trim(), "bad");
    let run = failed_document(&failing(&["--json", "run"]), 3);
    assert_eq!(run["stderr"].as_str().unwrap().trim(), "bad");
}

#[test]
fn err_counts_and_tabulates_the_same_lines() {
    let json = failed_document(&failing(&["--json", "err"]), 3);
    assert_eq!(json["errors"], 1);
    let csv = trs(&failing(&["--csv", "err"]));
    assert_eq!(csv.status.code(), Some(3));
    let csv = String::from_utf8_lossy(&csv.stdout);
    assert!(csv.starts_with("kind,line"), "{csv}");
    assert!(csv.lines().any(|l| l == "error,error: x"), "{csv}");
}
