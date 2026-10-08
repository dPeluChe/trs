//! Routing by what the command really is: an explicit format on a short
//! output, and `git push` however git is invoked. Uses real git in temp repos.

use assert_cmd::Command;
use std::path::Path;

mod support;
use support::git;

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(
        dir.path(),
        &["commit", "-q", "--allow-empty", "-m", "first"],
    );
    std::fs::write(dir.path().join("a.txt"), "x").unwrap();
    dir
}

fn trs(dir: &Path, args: &[&str]) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn an_explicit_format_applies_to_a_short_output() {
    let dir = repo();
    // A few dozen bytes of `git status`: under the size guard, and the JSON is
    // longer than the text, so two guards used to hand back the plain text.
    let json = trs(dir.path(), &["git", "status", "--json"]);
    let v: serde_json::Value =
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"));
    assert_eq!(v["untracked_count"], 1, "{json}");
    assert_eq!(trs(dir.path(), &["--json", "git", "status"]), json);

    let csv = trs(dir.path(), &["git", "status", "--csv"]);
    assert!(csv.starts_with("status,path,"), "{csv}");
}

#[test]
fn without_a_format_a_short_output_stays_as_git_printed_it() {
    let dir = repo();
    let out = trs(dir.path(), &["git", "status", "-sb"]);
    assert!(out.contains("?? a.txt"), "{out}");
    assert!(!out.contains('{'), "{out}");
}

#[test]
fn push_is_compacted_with_git_global_options_too() {
    let work = repo();
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare"]);
    let url = remote.path().to_string_lossy().into_owned();
    git(work.path(), &["remote", "add", "origin", &url]);
    let elsewhere = tempfile::tempdir().unwrap();
    let work_path = work.path().to_string_lossy().into_owned();

    for (i, args) in [
        vec!["git", "push", "origin", "HEAD:refs/heads/plain"],
        vec![
            "git",
            "-C",
            &work_path,
            "push",
            "origin",
            "HEAD:refs/heads/with-c",
        ],
        vec![
            "git",
            "-c",
            "color.ui=never",
            "push",
            "origin",
            "HEAD:refs/heads/with-config",
        ],
    ]
    .iter()
    .enumerate()
    {
        let dir = if i == 1 {
            elsewhere.path()
        } else {
            work.path()
        };
        let out = trs(dir, args);
        assert!(
            out.starts_with("pushed"),
            "{args:?} was not compacted:\n{out}"
        );
    }
}
