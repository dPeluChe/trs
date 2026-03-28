use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_doctor_runs_successfully() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("doctor");
    cmd.assert()
        .stdout(predicate::str::contains("TRS Doctor"))
        .stdout(predicate::str::contains("trs binary"))
        .stdout(predicate::str::contains("passed"));
}

#[test]
fn test_doctor_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.args(["doctor", "--json"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(parsed["checks"].is_array());
    assert!(parsed["summary"]["total"].as_u64().unwrap() >= 8);
    assert!(parsed["summary"].get("healthy").is_some());
}

#[test]
fn test_doctor_shows_check_markers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("doctor");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain pass markers (✓)
    assert!(stdout.contains('\u{2713}'), "expected ✓ markers in output");
}

#[test]
fn test_doctor_shows_sub_details() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("doctor");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show version and path as sub-details
    assert!(stdout.contains("version:"), "expected version sub-detail");
    assert!(stdout.contains("path:"), "expected path sub-detail");
}
