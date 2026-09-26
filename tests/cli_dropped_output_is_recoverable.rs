//! A summary that leaves content out must say where the rest is. Parsers mark
//! what they drop; the executor saves the raw output under $HOME/.trs/tee and
//! prints the path, as it already did for failed commands.

use assert_cmd::Command;
use std::path::Path;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn repo_with_changed_lines(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "t@example.com"]);
    git(p, &["config", "user.name", "t"]);
    let body = |tag: &str| {
        (0..n)
            .map(|i| format!("line {i} {tag}\n"))
            .collect::<String>()
    };
    std::fs::write(p.join("a.txt"), body("old")).unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "base"]);
    std::fs::write(p.join("a.txt"), body("new")).unwrap();
    dir
}

fn trs_git_diff(repo: &Path, home: &Path) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["git", "diff"])
        .current_dir(repo)
        .env("HOME", home)
        .env_remove("TRS_HOME")
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_summarized_diff_points_to_the_identical_raw_output() {
    let repo = repo_with_changed_lines(700);
    let home = tempfile::tempdir().unwrap();
    let out = trs_git_diff(repo.path(), home.path());
    let path = out
        .lines()
        .find_map(|l| l.strip_prefix("[trs] full output: "))
        .unwrap_or_else(|| panic!("no pointer to the dropped hunks:\n{out}"));
    let saved = std::fs::read_to_string(path.trim()).unwrap();
    // stdout and stderr are both saved, as the agent saw them; on Windows git
    // adds a CRLF warning on stderr, so the diff is contained, not equal.
    let diff = git(repo.path(), &["--no-pager", "diff"]);
    assert!(
        diff.len() > 10_000 && saved.contains(&diff),
        "saved output lacks the full diff"
    );
}

#[test]
fn a_diff_shown_whole_needs_no_pointer() {
    let repo = repo_with_changed_lines(20);
    let home = tempfile::tempdir().unwrap();
    let out = trs_git_diff(repo.path(), home.path());
    assert!(!out.contains("[trs] full output"), "{out}");
    assert!(
        !home.path().join(".trs").join("tee").exists(),
        "wrote a tee file for nothing"
    );
}

/// Small output, small cut: only the parser's own "I dropped something"
/// signal can produce the pointer here, not the >=90%-of-16KB backstop.
#[test]
fn a_capped_list_points_to_the_rest_even_when_small() {
    let repo = repo_with_changed_lines(3);
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-qm", "second"]);
    for i in 0..60 {
        git(repo.path(), &["branch", &format!("feat/topic-{i:02}")]);
    }
    let home = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["git", "branch"])
        .current_dir(repo.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.contains("more)"),
        "expected the 50-name cap to apply:\n{out}"
    );
    let path = out
        .lines()
        .find_map(|l| l.strip_prefix("[trs] full output: "))
        .unwrap_or_else(|| panic!("capped list gave no way back:\n{out}"));
    let saved = std::fs::read_to_string(path.trim()).unwrap();
    assert!(saved.contains("feat/topic-59"), "{saved}");
}
