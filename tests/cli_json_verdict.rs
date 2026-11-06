//! When a parser emits a JSON object, `--json` output carries the child's real
//! verdict and stays a single parseable document.
#![cfg(unix)]

use assert_cmd::Command;

mod support;

fn json_run(program: &str, body: &str) -> (serde_json::Value, Option<i32>) {
    let dir = tempfile::tempdir().unwrap();
    support::fake_program(&dir.path().join(program), body);
    let out = Command::cargo_bin("trs")
        .unwrap()
        .env("PATH", support::path_with(dir.path()))
        .args(["--json", program, "test"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("not one JSON document ({e}): {stdout}"));
    (json, out.status.code())
}

#[test]
fn a_script_trs_cannot_read_still_reports_its_failure() {
    let (json, code) = json_run("npm", "echo my custom failure; exit 3");
    assert_eq!(code, Some(3));
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 3);
    let raw = json["full_output"].as_str().unwrap();
    assert!(std::fs::read_to_string(raw)
        .unwrap()
        .contains("custom failure"));
}

#[test]
fn a_passing_run_is_not_called_failed() {
    let (json, code) = json_run("yarn", "echo all good, nothing for a parser");
    assert_eq!(code, Some(0));
    assert_eq!(json["success"], true);
    assert!(json.get("exit_code").is_none());
}
