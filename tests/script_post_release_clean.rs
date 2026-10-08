#![cfg(unix)]
//! `scripts/post-release-clean.sh` deletes build output, so its guards are
//! tested against a throwaway repo with a fake `cargo` and `gh` that only log
//! what they were asked to do.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

mod support;
use support::{fake_program, git, path_with};

/// One temp dir: `repo/` (version 9.9.9), `bin/` (the fakes), `home/`.
struct Fixture {
    root: tempfile::TempDir,
}

fn fixture(tagged: bool) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let (repo, bin, home) = (
        root.path().join("repo"),
        root.path().join("bin"),
        root.path().join("home"),
    );
    for dir in [
        repo.join("scripts"),
        repo.join("target/test-trs-home"),
        bin.clone(),
        home.join(".cargo/registry"),
        home.join(".trs"),
        home.join(".cache/trs-bench"),
    ] {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"trs-cli\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    fs::copy(
        "scripts/post-release-clean.sh",
        repo.join("scripts/post-release-clean.sh"),
    )
    .unwrap();
    fs::write(repo.join("target/test-trs-home/history.jsonl"), "x").unwrap();
    for keep in [
        home.join(".cargo/registry/crate.crate"),
        home.join(".trs/history.jsonl"),
        home.join(".cache/trs-bench/tool"),
    ] {
        fs::write(keep, "x").unwrap();
    }
    git(&repo, &["init", "-q"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "x"]);
    if tagged {
        git(&repo, &["tag", "v9.9.9"]);
    }
    // The fakes log every call; `gh` answers with $FAKE_GH and exits $FAKE_GH_EXIT.
    fake_program(&bin.join("cargo"), "echo \"cargo $*\" >> \"$CALLS\"");
    fake_program(
        &bin.join("gh"),
        "echo \"gh $*\" >> \"$CALLS\"; printf '%s\\n' \"$FAKE_GH\"; exit \"${FAKE_GH_EXIT:-0}\"",
    );
    Fixture { root }
}

impl Fixture {
    fn path(&self, rel: &str) -> PathBuf {
        self.root.path().join(rel)
    }

    fn run(&self, gh_answer: &str, args: &[&str]) -> Output {
        self.run_with(gh_answer, "0", args)
    }

    fn run_with(&self, gh_answer: &str, gh_exit: &str, args: &[&str]) -> Output {
        Command::new("bash")
            .arg(self.path("repo/scripts/post-release-clean.sh"))
            .args(args)
            .current_dir(self.path("repo"))
            .env("PATH", path_with(&self.path("bin")))
            .env("HOME", self.path("home"))
            .env("CALLS", self.path("calls.log"))
            .env("FAKE_GH", gh_answer)
            .env("FAKE_GH_EXIT", gh_exit)
            .output()
            .unwrap()
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.path("calls.log"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    fn cargo_clean(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|l| l.starts_with("cargo clean"))
            .collect()
    }

    /// Nothing was deleted and cargo was never asked to.
    fn assert_untouched(&self, why: &str) {
        assert!(self.cargo_clean().is_empty(), "{why}: {:?}", self.calls());
        assert!(self.path("repo/target/test-trs-home").exists(), "{why}");
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
fn nothing_is_deleted_unless_this_version_was_released_successfully() {
    // (tag exists, what `gh run list` says)
    let cases = [
        (false, "completed success"),
        (true, "in_progress "),
        (true, "completed failure"),
        (true, "completed cancelled"),
        (true, "null null"),
        (true, ""),
    ];
    for (tagged, answer) in cases {
        let f = fixture(tagged);
        let out = f.run(answer, &["--yes"]);
        assert!(!out.status.success(), "{tagged} {answer:?}: {}", text(&out));
        f.assert_untouched(&format!("{tagged} {answer:?}"));
    }
    let out = fixture(false).run("completed success", &["--yes"]);
    assert!(text(&out).contains("no tag v9.9.9"), "{}", text(&out));
}

#[test]
fn a_gh_that_cannot_answer_is_not_read_as_a_release() {
    let f = fixture(true);
    let out = f.run_with("", "1", &["--yes"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("could not read"), "{}", text(&out));
    f.assert_untouched("gh failed");
}

#[test]
fn a_dry_run_only_asks_cargo_to_dry_run_and_asks_about_this_tag() {
    let f = fixture(true);
    let out = f.run("completed success", &[]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        f.cargo_clean(),
        [
            "cargo clean -p trs-cli --dry-run",
            "cargo clean -p trs-cli --release --dry-run"
        ]
    );
    assert!(
        f.calls()
            .iter()
            .any(|l| l.starts_with("gh run list --workflow release.yml --branch v9.9.9")),
        "{:?}",
        f.calls()
    );
    assert!(
        f.path("repo/target/test-trs-home").exists(),
        "a dry run deleted something"
    );
}

#[test]
fn yes_cleans_only_this_crate_and_keeps_the_dependencies() {
    let f = fixture(true);
    let out = f.run("completed success", &["--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        f.cargo_clean(),
        ["cargo clean -p trs-cli", "cargo clean -p trs-cli --release"]
    );
    assert!(!f.path("repo/target/test-trs-home").exists());
}

#[test]
fn the_deps_flag_asks_for_a_full_clean() {
    let f = fixture(true);
    f.run("completed success", &["--yes", "--deps"]);
    assert_eq!(f.cargo_clean(), ["cargo clean"]);
}

#[test]
fn under_the_threshold_nothing_runs_and_no_release_is_needed() {
    let f = fixture(false);
    let out = f.run("", &["--if-over", "50", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("nothing to do"), "{}", text(&out));
    assert!(f.calls().is_empty(), "{:?}", f.calls());
}

#[test]
fn over_the_threshold_it_cleans_without_asking_about_a_release() {
    let f = fixture(false);
    // 0 GB: any target/ is over it.
    let out = f.run("", &["--if-over", "0", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        !f.calls().iter().any(|l| l.starts_with("gh ")),
        "{:?}",
        f.calls()
    );
    assert_eq!(f.cargo_clean().len(), 2, "{:?}", f.calls());
}

#[test]
fn what_is_not_ours_is_never_touched_and_the_bench_cache_only_on_request() {
    let f = fixture(true);
    f.run("completed success", &["--yes", "--deps"]);
    assert!(
        f.path("home/.cache/trs-bench/tool").exists(),
        "no flag, no deletion"
    );

    f.run("completed success", &["--yes", "--bench-cache"]);
    assert!(!f.path("home/.cache/trs-bench").exists());
    for keep in [
        "home/.cargo/registry/crate.crate",
        "home/.trs/history.jsonl",
        "repo/.git",
    ] {
        assert!(f.path(keep).exists(), "{keep} was removed");
    }
}

#[test]
fn it_refuses_to_run_outside_the_trs_repo() {
    let f = fixture(true);
    fs::write(f.path("repo/Cargo.toml"), "[package]\nname = \"other\"\n").unwrap();
    let out = f.run("completed success", &["--yes"]);
    assert!(!out.status.success());
    assert!(f.calls().is_empty(), "{:?}", f.calls());
}
