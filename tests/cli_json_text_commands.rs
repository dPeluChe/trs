//! Commands that print text still give one document under `--json`.

use assert_cmd::Command;

fn trs_json<S: AsRef<std::ffi::OsStr>>(args: &[S]) -> (serde_json::Value, Option<i32>) {
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .env("HOME", home.path())
        .env("TRS_HOME", home.path())
        .current_dir(home.path())
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let doc = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{e}: {stdout}"));
    (doc, out.status.code())
}

#[test]
fn an_admin_failure_is_a_failed_document() {
    let (doc, code) = trs_json(&["init", "no-such-tool"]);
    assert_eq!(code, Some(2));
    assert_eq!(doc["success"], false);
    assert_eq!(doc["exit_code"], 2);
    assert!(doc["stderr"].as_str().unwrap().contains("Unknown tool"));
}

#[test]
fn admin_text_is_wrapped() {
    let (doc, code) = trs_json(&["history", "--prune", "--dry-run"]);
    assert_eq!(code, Some(0));
    assert_eq!(doc["success"], true);
    assert!(doc["output"].as_str().unwrap().contains("trs history"));
}

#[test]
fn ingest_prints_its_path_in_the_document() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}\n").unwrap();
    let digest = dir.path().join("digest.md");
    let (doc, code) = trs_json(&[
        "ingest".as_ref(),
        dir.path().as_os_str(),
        "-o".as_ref(),
        digest.as_os_str(),
    ]);
    assert_eq!(code, Some(0), "{doc}");
    assert!(doc["output"].as_str().unwrap().contains("digest.md"));
    assert!(digest.exists());
}

#[test]
fn a_prompt_with_no_answer_is_a_failure() {
    let (doc, code) = trs_json(&["uninstall", "--all"]);
    assert_eq!(code, Some(1), "{doc}");
    assert!(doc["stderr"].as_str().unwrap().contains("--yes"));
}

#[test]
fn discover_is_wrapped() {
    let (doc, code) = trs_json(&["discover"]);
    assert_eq!(code, Some(0), "{doc}");
    assert!(doc["output"].is_string());
}

#[cfg(unix)]
mod non_utf8 {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn a_program_with_a_non_utf8_argument_is_wrapped() {
        let arg = std::ffi::OsStr::from_bytes(b"a\xff");
        let (doc, code) = trs_json(&["echo".as_ref(), arg]);
        assert_eq!(code, Some(0));
        assert_eq!(doc["success"], true);
        assert!(doc["output"].as_str().unwrap().starts_with('a'));
    }

    #[test]
    fn a_trs_command_with_a_non_utf8_argument_says_so() {
        let arg = std::ffi::OsStr::from_bytes(b"a\xff");
        let (doc, code) = trs_json(&["run".as_ref(), "echo".as_ref(), arg]);
        assert_eq!(code, Some(2));
        assert!(doc["message"].as_str().unwrap().contains("not valid UTF-8"));
    }

    #[test]
    fn flags_before_the_program_are_not_the_program() {
        let arg = std::ffi::OsStr::from_bytes(b"a\xff");
        let out = Command::cargo_bin("trs")
            .unwrap()
            .args(["--csv".as_ref(), "echo".as_ref(), arg])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert!(out.stdout.starts_with(b"a"));
    }
}
