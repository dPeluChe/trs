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

#[cfg(unix)]
#[test]
fn the_comparison_commands_report_a_missing_program_as_a_document() {
    let commands: [&[&str]; 2] = [
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
    let commands: [&[&str]; 3] = [
        &["--json", "zzz_no_such_program"],
        &["--json", "run", "zzz_no_such_program"],
        &["--json", "err", "zzz_no_such_program"],
    ];
    for args in commands {
        let json = failed_document(args, expected);
        if cfg!(unix) {
            assert_not_found(&json);
        }
    }
}

#[cfg(windows)]
#[test]
fn every_command_reaches_a_cmd_shim() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("zzz_shim.cmd"), "@echo shim ran\r\n").unwrap();
    for tool in ["err", "raw", "diff", "benchmark"] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .env("PATH", support::path_with(dir.path()))
            .args([tool, "zzz_shim"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{tool}: {out:?}");
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

#[test]
fn a_usage_error_is_a_document_under_json() {
    let json = failed_document(&["--json", "search"], 2);
    assert_eq!(json["error"], true);
    assert_eq!(json["success"], false);
    assert!(json["message"].as_str().unwrap().contains("<PATH>"));
    let help = trs(&["--json", "--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
    let text = trs(&["search"]);
    assert_eq!(text.status.code(), Some(2));
    assert!(text.stdout.is_empty());
    assert!(String::from_utf8_lossy(&text.stderr).contains("<PATH>"));
}

#[test]
fn a_failed_ingest_write_is_a_document_under_json() {
    let dir = tempfile::tempdir().unwrap();
    support::git(dir.path(), &["init", "-q"]);
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    support::git(dir.path(), &["add", "main.rs"]);
    let blocker = dir.path().join("a_file");
    std::fs::write(&blocker, "").unwrap();
    let ingest = || {
        let mut cmd = Command::cargo_bin("trs").unwrap();
        cmd.current_dir(dir.path()).args(["--json", "ingest", "."]);
        cmd
    };
    let to_output = ingest()
        .arg("--output")
        .arg(blocker.join("digest.md"))
        .output()
        .unwrap();
    let to_store = ingest().env("TRS_HOME", &blocker).output().unwrap();
    for out in [to_output, to_store] {
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        let json = support::document(&out);
        assert!(json["message"].as_str().unwrap().contains("write failed"));
    }
}
