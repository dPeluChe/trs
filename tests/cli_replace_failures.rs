//! A replace that could not finish says so, in the output and the exit code,
//! a target the user named is never skipped as if it held no match, and an
//! edit leaves the rest of the file as written.

use assert_cmd::Command;
use std::path::Path;
use std::process::Output;

mod support;

fn replace(flags: &[&str], path: &Path) -> Output {
    Command::cargo_bin("trs")
        .unwrap()
        .args(flags)
        .args(["replace", path.to_str().unwrap(), "hello", "bye"])
        .output()
        .unwrap()
}

#[test]
fn a_root_under_an_ignored_directory_name_is_searched() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "build/a.txt", "hello\n");
    support::write_file(dir.path(), "build/dist/b.txt", "hello\n");
    let out = replace(&[], &dir.path().join("build"));
    assert_eq!(out.status.code(), Some(0));
    let read = |name| std::fs::read_to_string(dir.path().join(name)).unwrap();
    assert_eq!(read("build/a.txt"), "bye\n");
    assert_eq!(read("build/dist/b.txt"), "hello\n");
}

#[test]
fn a_match_in_a_file_that_is_not_utf8_is_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("latin1.txt"), b"hello caf\xe9\n").unwrap();
    std::fs::write(dir.path().join("binary.bin"), b"\xff\xfe\n").unwrap();
    let out = replace(&[], dir.path());
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2));
    assert!(text.contains("latin1.txt: not UTF-8"), "{text}");
    assert!(!text.contains("binary.bin"), "{text}");
}

#[cfg(unix)]
#[test]
fn a_named_symlink_is_followed() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "real.txt", "hello\n");
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(dir.path().join("real.txt"), &link).unwrap();
    assert_eq!(replace(&[], &link).status.code(), Some(0));
    let real = std::fs::read_to_string(dir.path().join("real.txt")).unwrap();
    assert_eq!(real, "bye\n");
}

#[cfg(unix)]
mod unwritable {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn locked_tree() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().unwrap();
        support::write_file(dir.path(), "ok.txt", "hello\n");
        support::write_file(dir.path(), "locked.txt", "hello\n");
        let locked = dir.path().join("locked.txt");
        let mode = std::fs::Permissions::from_mode(0o444);
        std::fs::set_permissions(&locked, mode).unwrap();
        // Root writes through the mode bits, so there is no failure to see.
        std::fs::OpenOptions::new()
            .write(true)
            .open(&locked)
            .is_err()
            .then_some(dir)
    }

    #[test]
    fn a_failed_write_reports_it_and_keeps_the_rest() {
        let Some(dir) = locked_tree() else { return };
        let out = replace(&[], dir.path());
        let text = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(2));
        assert!(
            text.contains("failed: ") && text.contains("locked.txt"),
            "{text}"
        );
        let ok = std::fs::read_to_string(dir.path().join("ok.txt")).unwrap();
        assert_eq!(ok, "bye\n");
    }

    #[test]
    fn json_carries_the_failure_in_one_document() {
        let Some(dir) = locked_tree() else { return };
        let out = replace(&["--json"], dir.path());
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stderr.is_empty());
        let doc = support::document(&out);
        assert_eq!(doc["success"], false);
        assert_eq!(doc["exit_code"], 2);
        assert_eq!(doc["failed"].as_array().map(Vec::len), Some(1));
        assert_eq!(doc["counts"]["files_affected"], 1);
    }

    #[test]
    fn a_long_list_of_failures_is_capped() {
        let Some(dir) = locked_tree() else { return };
        for i in 0..24 {
            let name = format!("more{i}.txt");
            support::write_file(dir.path(), &name, "hello\n");
            let mode = std::fs::Permissions::from_mode(0o444);
            std::fs::set_permissions(dir.path().join(name), mode).unwrap();
        }
        let out = replace(&[], dir.path());
        let text = String::from_utf8_lossy(&out.stdout);
        assert_eq!(text.matches("failed: ").count(), 21, "{text}");
        assert!(text.contains("failed: 5 more"), "{text}");
    }

    #[test]
    fn a_table_keeps_the_failure_on_stderr() {
        let Some(dir) = locked_tree() else { return };
        let out = replace(&["--csv"], dir.path());
        assert_eq!(out.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&out.stdout).contains("failed: "));
        assert!(String::from_utf8_lossy(&out.stderr).contains("failed: "));
    }
}

#[test]
fn a_crlf_file_keeps_its_line_endings() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    std::fs::write(&file, "foo a\r\nbar\r\nfoo").unwrap();
    Command::cargo_bin("trs")
        .unwrap()
        .args([
            "replace".as_ref(),
            dir.path().as_os_str(),
            "foo".as_ref(),
            "baz".as_ref(),
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read(&file).unwrap(), b"baz a\r\nbar\r\nbaz");
}

#[test]
fn a_pattern_with_a_newline_replaces_across_lines() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    std::fs::write(&file, "x\na\nb\ny\n").unwrap();
    Command::cargo_bin("trs")
        .unwrap()
        .args([
            "replace".as_ref(),
            dir.path().as_os_str(),
            r"a\nb".as_ref(),
            "AB".as_ref(),
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "x\nAB\ny\n");
}
