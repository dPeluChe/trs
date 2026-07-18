//! `trs ingest --agent` implies `--print`: an agent asking for the digest gets
//! the content on stdout, not just the saved path (unless it writes with `-o`).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// A throwaway project dir, unique per (pid, tag) so parallel tests don't race.
fn make_project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trs_ingest_agent_{}_{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("main.rs"), "fn main() {}\n").unwrap();
    dir
}

#[test]
fn agent_emits_digest_content_to_stdout() {
    let dir = make_project("stdout");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .arg("--agent")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("# "))
        .stdout(predicate::str::contains("## Structure"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn default_emits_saved_path_not_content() {
    let dir = make_project("path");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .assert()
        .success()
        // A saved path, not the markdown digest.
        .stdout(predicate::str::starts_with("# ").not())
        .stdout(predicate::str::is_match(r"\.md\s*$").unwrap());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn agent_with_output_flag_still_returns_path() {
    let dir = make_project("outfile");
    let out = dir.join("digest.md");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .arg("--agent")
        .arg("-o")
        .arg(&out)
        .assert()
        .success()
        // Explicit -o wins: stdout is the path, content went to the file.
        .stdout(predicate::str::starts_with("# ").not())
        .stdout(predicate::str::contains("digest.md"));
    assert!(fs::read_to_string(&out).unwrap().starts_with("# "));
    let _ = fs::remove_dir_all(&dir);
}
