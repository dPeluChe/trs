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
fn default_does_not_emit_digest_content() {
    // Default (no --agent): stdout is the saved path, never the digest itself.
    // We assert the *absence* of digest markers rather than the exact path
    // shape — the store-path format varies by platform (e.g. a non-git dir on
    // Windows prints nothing), but it must never be the content.
    let dir = make_project("path");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("# ").not())
        .stdout(predicate::str::contains("## Structure").not());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn agent_large_no_budget_warns_inside_digest() {
    // Agents run `2>/dev/null`, so the stderr budget warning is invisible —
    // it must ride inside the digest. `--warn-at 1` forces the threshold on a
    // tiny project.
    let dir = make_project("warn");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .args(["--agent", "--warn-at", "1"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("# "))
        .stdout(predicate::str::contains("Large digest"))
        .stdout(predicate::str::contains("--budget"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn agent_with_budget_has_no_in_digest_warning() {
    // A budget was set → the digest is already fitted, no nag.
    let dir = make_project("warnbudget");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .args(["--agent", "--warn-at", "1", "--budget", "128k"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Large digest").not());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn agent_warn_disabled_has_no_in_digest_warning() {
    let dir = make_project("warnoff");
    Command::cargo_bin("trs")
        .unwrap()
        .arg("ingest")
        .arg(&dir)
        .args(["--agent", "--warn-at", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Large digest").not());
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
