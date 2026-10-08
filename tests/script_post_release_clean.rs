#![cfg(unix)]
//! `scripts/post-release-clean.sh` deletes build output, so its guards are
//! tested against a throwaway repo with a fake `cargo` and `gh` that only log
//! what they were asked to do.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture {
    repo: tempfile::TempDir,
    bin: tempfile::TempDir,
    home: tempfile::TempDir,
}

fn script() -> String {
    fs::read_to_string("scripts/post-release-clean.sh").unwrap()
}

fn executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

/// A repo at version 9.9.9, optionally tagged, with something in target/.
fn fixture(tagged: bool) -> Fixture {
    let f = Fixture {
        repo: tempfile::tempdir().unwrap(),
        bin: tempfile::tempdir().unwrap(),
        home: tempfile::tempdir().unwrap(),
    };
    let r = f.repo.path();
    fs::write(
        r.join("Cargo.toml"),
        "[package]\nname = \"trs-cli\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    fs::create_dir_all(r.join("scripts")).unwrap();
    let copy = r.join("scripts/post-release-clean.sh");
    fs::write(&copy, script()).unwrap();
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir_all(r.join("target/test-trs-home")).unwrap();
    fs::write(r.join("target/test-trs-home/history.jsonl"), "x").unwrap();
    fs::write(r.join("target/keep.bin"), vec![0u8; 4096]).unwrap();
    git(r, &["init", "-q"]);
    git(r, &["add", "-A"]);
    git(r, &["commit", "-q", "-m", "x"]);
    if tagged {
        git(r, &["tag", "v9.9.9"]);
    }
    // The fakes log every call; `gh` answers with $FAKE_GH.
    executable(
        &f.bin.path().join("cargo"),
        "echo \"cargo $*\" >> \"$CALLS\"",
    );
    executable(
        &f.bin.path().join("gh"),
        "echo \"gh $*\" >> \"$CALLS\"; printf '%s\\n' \"$FAKE_GH\"",
    );
    // Things the script must never touch.
    fs::create_dir_all(f.home.path().join(".cargo/registry")).unwrap();
    fs::write(f.home.path().join(".cargo/registry/crate.crate"), "x").unwrap();
    fs::create_dir_all(f.home.path().join(".trs")).unwrap();
    fs::write(f.home.path().join(".trs/history.jsonl"), "x").unwrap();
    f
}

impl Fixture {
    fn calls(&self) -> String {
        fs::read_to_string(self.repo.path().join("calls.log")).unwrap_or_default()
    }

    fn run(&self, gh_answer: &str, args: &[&str]) -> Output {
        let path = format!(
            "{}:{}",
            self.bin.path().display(),
            std::env::var("PATH").unwrap()
        );
        Command::new("bash")
            .arg(self.repo.path().join("scripts/post-release-clean.sh"))
            .args(args)
            .current_dir(self.repo.path())
            .env("PATH", path)
            .env("HOME", self.home.path())
            .env("CALLS", self.repo.path().join("calls.log"))
            .env("FAKE_GH", gh_answer)
            .output()
            .unwrap()
    }

    fn exists(&self, rel: &str) -> bool {
        PathBuf::from(self.repo.path()).join(rel).exists()
    }
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn nothing_is_deleted_for_a_version_that_was_never_tagged() {
    let f = fixture(false);
    let out = f.run("completed success", &["--yes"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("no tag v9.9.9"), "{}", text(&out));
    assert!(!f.calls().contains("cargo clean"), "{}", f.calls());
    assert!(f.exists("target/test-trs-home"));
}

#[test]
fn nothing_is_deleted_until_the_release_run_succeeded() {
    for answer in [
        "in_progress ",
        "completed failure",
        "completed cancelled",
        "",
    ] {
        let f = fixture(true);
        let out = f.run(answer, &["--yes"]);
        assert!(!out.status.success(), "{answer:?}: {}", text(&out));
        assert!(
            !f.calls().contains("cargo clean"),
            "{answer:?}: {}",
            f.calls()
        );
        assert!(f.exists("target/test-trs-home"), "{answer:?}");
    }
}

#[test]
fn the_release_check_asks_about_this_tag() {
    let f = fixture(true);
    f.run("completed success", &[]);
    assert!(
        f.calls()
            .contains("gh run list --workflow release.yml --branch v9.9.9"),
        "{}",
        f.calls()
    );
}

#[test]
fn a_dry_run_only_asks_cargo_to_dry_run() {
    let f = fixture(true);
    let out = f.run("completed success", &[]);
    assert!(out.status.success(), "{}", text(&out));
    let calls = f.calls();
    assert!(
        calls.contains("cargo clean -p trs-cli --dry-run"),
        "{calls}"
    );
    assert!(
        calls.contains("cargo clean -p trs-cli --release --dry-run"),
        "{calls}"
    );
    for line in calls.lines().filter(|l| l.starts_with("cargo clean")) {
        assert!(line.ends_with("--dry-run"), "{line}");
    }
    assert!(
        f.exists("target/test-trs-home"),
        "a dry run deleted something"
    );
}

#[test]
fn yes_cleans_only_this_crate_and_keeps_the_dependencies() {
    let f = fixture(true);
    let out = f.run("completed success", &["--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    let calls = f.calls();
    assert!(
        calls.lines().any(|l| l == "cargo clean -p trs-cli"),
        "{calls}"
    );
    assert!(
        calls
            .lines()
            .any(|l| l == "cargo clean -p trs-cli --release"),
        "{calls}"
    );
    assert!(
        !calls.lines().any(|l| l.trim_end() == "cargo clean"),
        "{calls}"
    );
    assert!(!f.exists("target/test-trs-home"));
}

#[test]
fn the_deps_flag_asks_for_a_full_clean() {
    let f = fixture(true);
    f.run("completed success", &["--yes", "--deps"]);
    assert!(
        f.calls().lines().any(|l| l.trim_end() == "cargo clean"),
        "{}",
        f.calls()
    );
}

#[test]
fn under_the_threshold_nothing_runs_and_no_release_is_needed() {
    let f = fixture(false);
    let out = f.run("", &["--if-over", "50", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("nothing to do"), "{}", text(&out));
    assert_eq!(f.calls(), "");
}

#[test]
fn over_the_threshold_it_cleans_without_asking_about_a_release() {
    let f = fixture(false);
    // 0 GB: any target/ is over it.
    let out = f.run("", &["--if-over", "0", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(!f.calls().contains("gh "), "{}", f.calls());
    assert!(
        f.calls().contains("cargo clean -p trs-cli"),
        "{}",
        f.calls()
    );
}

#[test]
fn what_is_not_ours_is_never_touched() {
    let f = fixture(true);
    let out = f.run("completed success", &["--yes", "--deps", "--bench-cache"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(f.home.path().join(".cargo/registry/crate.crate").exists());
    assert!(f.home.path().join(".trs/history.jsonl").exists());
    assert!(f.exists(".git"));
}

#[test]
fn it_refuses_to_run_outside_the_trs_repo() {
    let f = fixture(true);
    fs::write(
        f.repo.path().join("Cargo.toml"),
        "[package]\nname = \"other\"\n",
    )
    .unwrap();
    let out = f.run("completed success", &["--yes"]);
    assert!(!out.status.success());
    assert!(f.calls().is_empty(), "{}", f.calls());
}
