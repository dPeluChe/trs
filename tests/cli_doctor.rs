use assert_cmd::Command;
use std::process::Output;

fn doctor(args: &[&str]) -> Output {
    let home = tempfile::tempdir().unwrap();
    Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env("TRS_HOME", home.path())
        .output()
        .unwrap()
}

#[test]
fn test_doctor_runs_successfully() {
    let output = doctor(&["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for text in ["TRS Doctor", "trs binary", "passed"] {
        assert!(stdout.contains(text), "lost {text:?}: {stdout}");
    }
}

#[test]
fn test_doctor_json_output() {
    let output = doctor(&["doctor", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(parsed["checks"].is_array());
    assert!(parsed["summary"]["total"].as_u64().unwrap() >= 8);
    assert!(parsed["summary"].get("healthy").is_some());
}

#[test]
fn test_doctor_shows_check_markers() {
    let output = doctor(&["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains('\u{2713}'), "expected ✓ markers in output");
}

#[test]
fn test_doctor_shows_sub_details() {
    let output = doctor(&["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let version = format!("version: {}", env!("CARGO_PKG_VERSION"));
    assert!(stdout.contains(&version), "expected {version}: {stdout}");
    assert!(stdout.contains("path:"), "expected path sub-detail");
}
