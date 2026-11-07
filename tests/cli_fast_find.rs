//! `trs find --gitignore`: a walk that missed something must not read as a
//! complete one.

use assert_cmd::Command;

mod support;

fn fast_find(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("trs")
        .unwrap()
        .current_dir(dir)
        .arg("find")
        .arg("--gitignore")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn a_missing_root_fails() {
    let dir = tempfile::tempdir().unwrap();
    let out = fast_find(dir.path(), &["nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot read nope"));

    let out = fast_find(dir.path(), &["nope", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let doc = support::document(&out);
    assert!(doc.to_string().contains("cannot read nope"));
}

#[cfg(unix)]
#[test]
fn a_directory_it_could_not_read_fails_and_keeps_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "ok.rs", "");
    support::write_file(dir.path(), "locked/hidden.rs", "");
    let Some([compact, raw, json]) = support::while_locked(&dir.path().join("locked"), || {
        [
            fast_find(dir.path(), &["."]),
            fast_find(dir.path(), &[".", "--raw"]),
            fast_find(dir.path(), &[".", "--json"]),
        ]
    }) else {
        return;
    };

    for out in [&compact, &raw] {
        assert_eq!(out.status.code(), Some(2));
        let text = String::from_utf8_lossy(&out.stdout);
        for shown in ["ok.rs", "failed: ", "locked"] {
            assert!(text.contains(shown), "{text}");
        }
    }

    assert_eq!(json.status.code(), Some(2));
    let doc = support::document(&json);
    assert_eq!(doc["success"], false);
    assert_eq!(doc["exit_code"], 2);
    assert_eq!(doc["total_files"], 1);
    assert_eq!(doc["failed_total"], 1);
}

#[test]
fn a_complete_walk_succeeds_without_a_failure_field() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "sub/a.rs", "");
    support::write_file(dir.path(), ".gitignore", "skipped.rs\n");
    support::write_file(dir.path(), "skipped.rs", "");
    std::fs::create_dir(dir.path().join(".git")).unwrap();

    let out = fast_find(dir.path(), &[".", "-name", "*.rs", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let doc = support::document(&out);
    assert_eq!(doc["total_files"], 1);
    assert!(doc.get("failed").is_none());

    let out = fast_find(dir.path(), &[".", "-type", "f", "-name", "a*", "--raw"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).lines().count(), 1);
}
