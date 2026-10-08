#![cfg(unix)]
//! Flags the program defines must reach it, and structured output must come
//! back byte for byte. A fake program stands in for gh/git so the test needs
//! no network and no account.

use assert_cmd::Command;
use std::path::Path;

mod support;
use support::fake_program;

fn fake(dir: &Path, name: &str, script: &str) {
    fake_program(&dir.join(name), script);
}

fn trs(dir: &Path, args: &[&str]) -> String {
    let path = format!("{}:{}", dir.display(), std::env::var("PATH").unwrap());
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(args)
        .env("PATH", path)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn gh_gets_its_json_flag_and_its_json_back_whole() {
    let dir = tempfile::tempdir().unwrap();
    // Fails like the real gh does when `--json` is missing; otherwise one long
    // compact line, the shape that looks "dense".
    fake(
        dir.path(),
        "gh",
        r#"case " $* " in *" --json "*) ;; *) echo "unknown command \"$4\" for gh run list" >&2; exit 1;; esac
printf '['
i=0; while [ $i -lt 60 ]; do [ $i -gt 0 ] && printf ','; printf '{"databaseId":%d,"displayTitle":"fix  double  space %d","url":"https://x/%d"}' $i $i $i; i=$((i+1)); done
printf ']\n'"#,
    );
    let out = trs(
        dir.path(),
        &["gh", "run", "list", "--json", "databaseId,url"],
    );
    assert!(out.len() > 300, "{out}");
    let rows: serde_json::Value = serde_json::from_str(&out).expect("not valid JSON");
    assert_eq!(rows.as_array().unwrap().len(), 60);
    assert!(
        out.contains("fix  double  space 7"),
        "spacing inside a value changed"
    );
    assert!(
        !out.contains("[minified") && !out.contains("[trs]"),
        "{out}"
    );
}

#[test]
fn git_log_raw_keeps_its_raw_format() {
    let dir = tempfile::tempdir().unwrap();
    fake(
        dir.path(),
        "git",
        r#"case " $* " in *" --raw "*) echo ":100644 100644 aaaaaaa bbbbbbb M	src/a.rs";; *) echo "commit abc";; esac"#,
    );
    let out = trs(dir.path(), &["git", "log", "--raw", "-1"]);
    assert!(out.contains(":100644 100644 aaaaaaa bbbbbbb M"), "{out}");
}

#[test]
fn json_from_a_program_trs_does_not_know_is_not_cut() {
    let dir = tempfile::tempdir().unwrap();
    fake(
        dir.path(),
        "emitjson",
        r#"printf '{"items":['
i=0; while [ $i -lt 80 ]; do [ $i -gt 0 ] && printf ','; printf '{"id":%d,"state":"open"}' $i; i=$((i+1)); done
printf ']}\n'"#,
    );
    let out = trs(dir.path(), &["emitjson", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).expect("not valid JSON");
    assert_eq!(v["items"].as_array().unwrap().len(), 80);
}

/// `trs --compact gh ...` takes the clap path, which had its own copy of the
/// flag stripping.
#[test]
fn the_clap_path_hands_the_flag_over_too() {
    let dir = tempfile::tempdir().unwrap();
    fake(
        dir.path(),
        "gh",
        r#"case " $* " in *" --json "*) echo '[{"id":1}]';; *) echo "missing --json" >&2; exit 1;; esac"#,
    );
    let out = trs(
        dir.path(),
        &["--compact", "gh", "run", "list", "--json", "id"],
    );
    assert!(out.contains(r#"[{"id":1}]"#), "{out}");
}
