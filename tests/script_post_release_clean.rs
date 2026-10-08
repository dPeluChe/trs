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
        // $FAKE_GH is one answer, or several separated by `|`, one per call.
        "echo \"gh $*\" >> \"$CALLS\"; n=$(cat \"$CALLS.n\" 2>/dev/null || echo 0); n=$((n+1)); echo $n > \"$CALLS.n\"; \
         printf '%s\\n' \"$FAKE_GH\" | awk -F'|' -v n=$n '{print (n <= NF) ? $n : $NF}'; exit \"${FAKE_GH_EXIT:-0}\"",
    );
    Fixture { root }
}

impl Fixture {
    fn path(&self, rel: &str) -> PathBuf {
        self.root.path().join(rel)
    }

    fn run(&self, gh_answer: &str, args: &[&str]) -> Output {
        self.run_with(gh_answer, &[], args)
    }

    /// `env` is extra environment: `FAKE_GH_EXIT`, `POST_RELEASE_*`.
    fn run_with(&self, gh_answer: &str, env: &[(&str, &str)], args: &[&str]) -> Output {
        Command::new("bash")
            .arg(self.path("repo/scripts/post-release-clean.sh"))
            .args(args)
            .current_dir(self.path("repo"))
            .env("PATH", path_with(&self.path("bin")))
            .env("HOME", self.path("home"))
            .env("CALLS", self.path("calls.log"))
            .env("FAKE_GH", gh_answer)
            .envs(env.iter().copied())
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

/// One-second polls, so the waiting tests stay quick.
const FAST: [(&str, &str); 2] = [
    ("POST_RELEASE_POLL_SECONDS", "1"),
    ("POST_RELEASE_WAIT_SECONDS", "30"),
];

fn gh_calls(f: &Fixture) -> usize {
    f.calls().iter().filter(|l| l.starts_with("gh ")).count()
}

#[test]
fn wait_follows_the_run_until_it_succeeds_and_then_cleans() {
    let f = fixture(true);
    let out = f.run_with(
        "in_progress |in_progress |completed success",
        &FAST,
        &["--wait", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(gh_calls(&f), 3, "{:?}", f.calls());
    assert!(text(&out).contains("checking again"), "{}", text(&out));
    assert_eq!(f.cargo_clean().len(), 2, "{:?}", f.calls());
}

#[test]
fn wait_gives_up_as_soon_as_the_run_finishes_badly() {
    for last in ["completed failure", "completed cancelled"] {
        let f = fixture(true);
        let answers = format!("in_progress |{last}");
        let out = f.run_with(&answers, &FAST, &["--wait", "--yes"]);
        assert!(!out.status.success(), "{last}: {}", text(&out));
        assert_eq!(gh_calls(&f), 2, "{last}: {:?}", f.calls());
        f.assert_untouched(last);
    }
}

#[test]
fn wait_stops_at_its_limit_and_cleans_nothing() {
    let f = fixture(true);
    let env = [
        ("POST_RELEASE_POLL_SECONDS", "1"),
        ("POST_RELEASE_WAIT_SECONDS", "2"),
    ];
    let out = f.run_with("in_progress ", &env, &["--wait", "--yes"]);
    assert!(!out.status.success());
    assert!(
        text(&out).contains("still 'in_progress '"),
        "{}",
        text(&out)
    );
    f.assert_untouched("timeout");
}

#[test]
fn wait_does_not_wait_forever_for_a_run_that_never_appears() {
    let f = fixture(true);
    let env = [
        ("POST_RELEASE_POLL_SECONDS", "1"),
        ("POST_RELEASE_WAIT_SECONDS", "2"),
    ];
    // gh prints `null null` when the workflow has no run for the tag yet.
    let out = f.run_with("null null", &env, &["--wait", "--yes"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("was the tag pushed"), "{}", text(&out));
    f.assert_untouched("no run");
}

#[test]
fn without_wait_an_unfinished_run_is_refused_with_a_hint() {
    let f = fixture(true);
    let out = f.run("in_progress ", &["--yes"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("--wait"), "{}", text(&out));
    assert_eq!(gh_calls(&f), 1, "it polled without --wait");
}

#[test]
fn a_gh_that_cannot_answer_is_not_read_as_a_release() {
    let f = fixture(true);
    let out = f.run_with("", &[("FAKE_GH_EXIT", "1")], &["--yes"]);
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
