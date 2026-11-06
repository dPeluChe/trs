//! What `--json` and `--csv` promise on stdout, on every platform. Windows has
//! no `sh`, so `FAILING` picks the script for the host.

use assert_cmd::Command;

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

fn with_failing(flags: &[&str]) -> std::process::Output {
    trs(&[flags, FAILING.as_slice()].concat())
}

fn document(out: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("not one JSON document ({e}): {stdout}"))
}

#[test]
fn a_command_that_never_started_is_a_document_too() {
    let commands: [&[&str]; 4] = [
        &["--json", "run", "zzz_no_such_program"],
        &["--json", "err", "zzz_no_such_program"],
        &["diff", "--json", "zzz_no_such_program"],
        &["benchmark", "--json", "zzz_no_such_program"],
    ];
    for args in commands {
        let out = trs(args);
        assert_eq!(out.status.code(), Some(127), "{args:?}");
        assert!(out.stderr.is_empty(), "{args:?}: {:?}", out.stderr);
        let json = document(&out);
        assert_eq!(json["error"], true);
        assert_eq!(json["success"], false);
        assert_eq!(json["exit_code"], 127);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Command not found"));
    }
}

#[test]
fn a_missing_program_is_one_failed_document() {
    let out = trs(&["--json", "zzz_no_such_program"]);
    assert!(out.stderr.is_empty(), "{:?}", out.stderr);
    let json = document(&out);
    assert_eq!(json["success"], false);
    // Windows starts programs through `cmd`, which fails with exit 1 itself.
    let expected = if cfg!(windows) { 1 } else { 127 };
    assert_eq!(out.status.code(), Some(expected));
    assert_eq!(json["exit_code"], expected);
}

#[test]
fn a_failed_run_is_one_document_with_nothing_on_stderr() {
    for flags in [&["--json", "run"][..], &["--json", "err"]] {
        let out = with_failing(flags);
        assert_eq!(out.status.code(), Some(3), "{flags:?}");
        assert!(out.stderr.is_empty(), "{flags:?}: {:?}", out.stderr);
        document(&out);
    }
}

#[test]
fn the_childs_stderr_stays_inside_the_document() {
    let out = with_failing(&["--json"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stderr.is_empty(), "{:?}", out.stderr);
    let json = document(&out);
    assert_eq!(json["exit_code"], 3);
    assert_eq!(json["success"], false);
    assert!(json["stderr"].as_str().unwrap().contains("bad"));
}

#[test]
fn err_counts_and_tabulates_the_same_lines() {
    let json = document(&with_failing(&["--json", "err"]));
    assert_eq!(json["errors"], 1);
    assert_eq!(json["exit_code"], 3);
    let csv = with_failing(&["--csv", "err"]);
    assert_eq!(csv.status.code(), Some(3));
    let csv = String::from_utf8_lossy(&csv.stdout);
    assert!(csv.starts_with("kind,line"), "{csv}");
    assert!(csv.lines().any(|l| l == "error,error: x"), "{csv}");
}
