//! When a parser emits a JSON object, `--json` output carries the child's real
//! verdict and stays a single parseable document.
#![cfg(unix)]

use assert_cmd::Command;

mod support;

fn run_json(program: &str, body: &str) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    support::fake_program(&dir.path().join(program), body);
    Command::cargo_bin("trs")
        .unwrap()
        .env("PATH", support::path_with(dir.path()))
        .args(["--json", program, "test"])
        .output()
        .unwrap()
}

fn json_run(program: &str, body: &str) -> (serde_json::Value, Option<i32>) {
    let out = run_json(program, body);
    (support::document(&out), out.status.code())
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

#[test]
fn every_path_prints_one_document() {
    let (json, code) = json_run("tool", "echo hi; exit 3");
    assert_eq!(code, Some(3));
    assert_eq!(json["output"], "hi\n");
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 3);
    assert!(json["full_output"].is_string());

    let (json, code) = json_run("tool", "echo hi");
    assert_eq!(code, Some(0));
    assert_eq!(json["output"], "hi\n");
    assert_eq!(json["success"], true);

    let (json, code) = json_run("tool", "echo '[1, 2]'");
    assert_eq!(code, Some(0));
    assert_eq!(json, serde_json::json!([1, 2]));
}

#[test]
fn a_failed_git_push_stays_one_document() {
    let (json, code) = json_run("git", "echo 'error: refused' >&2; exit 1");
    assert_eq!(code, Some(1));
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 1);
}

#[test]
fn is_clean_reports_a_failure_once_so_2_to_1_still_parses() {
    let outside = tempfile::tempdir().unwrap();
    let dirty = tempfile::tempdir().unwrap();
    assert!(std::process::Command::new("git")
        .arg("init")
        .current_dir(dirty.path())
        .output()
        .unwrap()
        .status
        .success());
    std::fs::write(dirty.path().join("new.txt"), "x").unwrap();

    for (dir, code) in [(outside.path(), 2), (dirty.path(), 1)] {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .current_dir(dir)
            .env("GIT_CEILING_DIRECTORIES", dir.parent().unwrap())
            .args(["--json", "is-clean"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(code));
        assert!(out.stderr.is_empty(), "{:?}", out.stderr);
        let json = support::document(&out);
        assert_eq!(json["is_clean"], false);
    }
}

fn err_json(script: &str) -> (serde_json::Value, Option<i32>) {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["--json", "err", "sh", "-c", script])
        .output()
        .unwrap();
    assert!(out.stderr.is_empty(), "{:?}", out.stderr);
    (support::document(&out), out.status.code())
}

#[test]
fn err_prints_one_document_with_the_lines_and_the_verdict() {
    let (json, code) = err_json("echo fine; echo 'error: boom'; echo next; exit 3");
    assert_eq!(code, Some(3));
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 3);
    assert_eq!(json["errors"], 1);
    assert_eq!(json["total_lines"], 3);
    assert_eq!(json["lines"][0], "error: boom");
    let raw = json["full_output"].as_str().unwrap();
    assert!(std::fs::read_to_string(raw).unwrap().contains("fine"));
}

#[test]
fn err_keeps_a_failure_no_keyword_caught() {
    let (json, code) = err_json("echo boom; exit 5");
    assert_eq!(code, Some(5));
    assert_eq!(json["success"], false);
    assert_eq!(json["lines"][0], "boom");
}

#[test]
fn err_on_a_clean_run_has_no_lines() {
    let (json, code) = err_json("echo fine");
    assert_eq!(code, Some(0));
    assert_eq!(json["success"], true);
    assert_eq!(json["lines"], serde_json::json!([]));
    assert!(json.get("full_output").is_none());
}

#[test]
fn a_childs_array_keeps_its_shape_and_its_stderr_stream() {
    let out = run_json("zzz_tool", "echo '[1, 2]'; echo careful >&2");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "[1, 2]");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "careful\n");
}

#[test]
fn a_long_stderr_is_cut_and_points_to_the_rest() {
    let (json, code) = json_run(
        "zzz_noisy_tool",
        "echo out; i=0; while [ $i -lt 3000 ]; do echo \"noise line $i\" >&2; i=$((i+1)); done",
    );
    assert_eq!(code, Some(0));
    assert!(json["stderr"].as_str().unwrap().len() < 4000);
    let raw = json["full_output"].as_str().unwrap();
    assert!(std::fs::read_to_string(raw)
        .unwrap()
        .contains("noise line 1500"));
}

fn err_table(flag: &str, script: &str) -> (String, Option<i32>) {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args([flag, "err", "sh", "-c", script])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    (stdout, out.status.code())
}

#[test]
fn err_prints_a_table_with_the_tabular_flags() {
    let script = "echo 'error: a, b'; echo detail; echo 'warning: y'; exit 3";
    let (csv, code) = err_table("--csv", script);
    assert_eq!(code, Some(3));
    assert_eq!(
        csv,
        "kind,line\nerror,\"error: a, b\"\ncontext,detail\nwarning,warning: y\n"
    );
    let (tsv, _) = err_table("--tsv", script);
    assert_eq!(
        tsv,
        "kind\tline\nerror\terror: a, b\ncontext\tdetail\nwarning\twarning: y\n"
    );
    let (clean, code) = err_table("--csv", "echo fine");
    assert_eq!(code, Some(0));
    assert_eq!(clean, "kind,line\n");
    let (unmatched, code) = err_table("--csv", "echo '  no keywords'; exit 2");
    assert_eq!(code, Some(2));
    assert_eq!(unmatched, "kind,line\noutput,  no keywords\n");
}
