//! `trs ingest`: a digest that missed something must not read as complete, and
//! what is not content stays silent.

use assert_cmd::Command;
use std::process::Output;

mod support;

/// The run and the digest it wrote. `-o` keeps it out of the shared store.
fn ingest(root: &std::path::Path) -> (Output, String) {
    let out = tempfile::tempdir().unwrap();
    let digest = out.path().join("digest.md");
    let run = Command::cargo_bin("trs")
        .unwrap()
        .args(["ingest", "-o"])
        .arg(&digest)
        .arg(root)
        .output()
        .unwrap();
    (run, std::fs::read_to_string(&digest).unwrap_or_default())
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "ok.rs", "fn main() { println!(\"kept\"); }\n");
    dir
}

#[cfg(unix)]
#[test]
fn a_directory_it_could_not_read_fails_and_keeps_stdout_clean() {
    let dir = project();
    support::write_file(dir.path(), "locked/hidden.rs", "fn hidden() {}\n");
    let locked = dir.path().join("locked");
    let Some((out, digest)) = support::while_locked(&locked, || ingest(dir.path())) else {
        return;
    };

    assert_eq!(out.status.code(), Some(2));
    assert!(digest.contains("kept"), "{digest}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("failed: "), "{stdout}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("failed: ") && stderr.contains("locked"),
        "{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn a_file_it_could_not_read_fails() {
    let dir = project();
    support::write_file(dir.path(), "locked.rs", "fn hidden() {}\n");
    let locked = dir.path().join("locked.rs");
    let Some((out, digest)) = support::while_locked(&locked, || ingest(dir.path())) else {
        return;
    };
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("failed: locked.rs"));
    assert!(digest.contains("incomplete: 1 paths"), "{digest}");
}

#[cfg(unix)]
#[test]
fn a_locked_directory_it_skips_anyway_is_no_failure() {
    let dir = project();
    support::write_file(dir.path(), "node_modules/dep/index.js", "x\n");
    let locked = dir.path().join("node_modules/dep");
    let Some((out, _)) = support::while_locked(&locked, || ingest(dir.path())) else {
        return;
    };
    assert_eq!(out.status.code(), Some(0));
}

#[cfg(unix)]
fn committed_project() -> tempfile::TempDir {
    let dir = project();
    support::write_file(dir.path(), "locked/hidden.rs", "fn hidden() {}\n");
    support::write_file(dir.path(), ".gitignore", ".trs-home\n");
    support::git(dir.path(), &["init", "-q"]);
    support::git(dir.path(), &["add", "."]);
    support::git(dir.path(), &["commit", "-qm", "x"]);
    dir
}

#[cfg(unix)]
fn ingest_in_store(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::cargo_bin("trs")
        .unwrap()
        .current_dir(dir)
        .env("TRS_HOME", dir.join(".trs-home"))
        .args(["ingest", "."])
        .args(args)
        .output()
        .unwrap()
}

#[cfg(unix)]
#[test]
fn what_was_excluded_or_unchanged_is_no_failure() {
    let dir = committed_project();
    let run = |args: &[&str]| ingest_in_store(dir.path(), args);
    let Some(excluded) =
        support::while_locked(&dir.path().join("locked"), || run(&["--exclude", "locked"]))
    else {
        return;
    };
    assert_eq!(excluded.status.code(), Some(0), "{excluded:?}");

    // Untracked, so `git diff` leaves it out. A tracked one reads as modified once locked.
    support::write_file(
        dir.path(),
        "new.rs",
        "fn new() {}
",
    );
    let unchanged = dir.path().join("new.rs");
    let Some(unchanged) = support::while_locked(&unchanged, || run(&["--since", "HEAD"])) else {
        return;
    };
    assert_eq!(unchanged.status.code(), Some(0), "{unchanged:?}");
}

#[cfg(unix)]
#[test]
fn fresh_does_not_reuse_a_partial_digest() {
    let dir = committed_project();
    let run = || ingest_in_store(dir.path(), &["--fresh"]);
    let locked = dir.path().join("locked");
    let Some([first, second]) = support::while_locked(&locked, || [run(), run()]) else {
        return;
    };
    assert_eq!(first.status.code(), Some(2), "{first:?}");
    assert_eq!(second.status.code(), Some(2), "{second:?}");
}

#[cfg(unix)]
#[test]
fn binary_files_and_dangling_links_are_no_failure() {
    let dir = project();
    std::fs::write(dir.path().join("blob.rs"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    std::os::unix::fs::symlink("gone", dir.path().join("dangling.rs")).unwrap();
    std::os::unix::fs::symlink(".", dir.path().join("loop.rs")).unwrap();

    let (out, digest) = ingest(dir.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("failed: "));
    assert!(digest.contains("kept"), "{digest}");
}

#[test]
fn a_root_inside_a_skipped_directory_is_still_read() {
    let parent = tempfile::tempdir().unwrap();
    support::write_file(
        parent.path(),
        "vendor/lib/ok.rs",
        "fn main() { let kept = 1; }\n",
    );
    let (out, digest) = ingest(&parent.path().join("vendor/lib"));
    assert_eq!(out.status.code(), Some(0));
    assert!(digest.contains("kept"), "{digest}");
}
