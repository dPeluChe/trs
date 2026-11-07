//! What each `trs search` flag does to the results, on a tree with known
//! content, so a flag that stops working fails instead of matching nothing.

use assert_cmd::Command;
use std::process::Output;

mod support;

fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(
        dir.path(),
        "src/a.rs",
        "fn main() {}\nfn Helper() {}\nlet a = 1;\nlet b = 2;\nmain-thread\nv1.2.3\nv1x2y3\ncafé\n",
    );
    support::write_file(dir.path(), "docs/b.md", "fn main() {}\nHELPER\n");
    dir
}

fn trs(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::cargo_bin("trs")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn each_flag_changes_what_is_found() {
    #[rustfmt::skip]
    let cases: &[(&[&str], &[&str], &[&str])] = &[
        (&["main"], &["a.rs", "b.md", "3 matches"], &[]),
        (&["main", "-e", "rs"], &["a.rs", "main-thread"], &["b.md"]),
        (&["helper"], &["no matches"], &["a.rs", "b.md"]),
        (&["helper", "-i"], &["Helper", "HELPER"], &[]),
        (&["fn \\w+"], &["fn main", "fn Helper"], &["let a"]),
        (&["main", "--limit", "1"], &["1/3 matches", "truncated"], &[]),
        (&["main", "--limit", "0"], &["0/3 matches", "truncated"], &["fn main"]),
        (&["let", "--limit", "1"], &["let a", "1/2 matches"], &["let b"]),
        (&["fn ", "--limit", "2"], &["b.md", "a.rs", "2/3 matches"], &["fn Helper"]),
        (&["let", "--limit", "5"], &["let a", "let b"], &["truncated"]),
        (&["let", "-C", "1"], &["fn Helper", "let a = 1", "main-thread"], &["cafe"]),
        (&["main-thread"], &["main-thread"], &["fn main"]),
        (&["v1\\.2"], &["v1.2.3"], &["v1x2y3"]),
        (&["café"], &["café"], &["fn main"]),
        (&["zzz_absent"], &["no matches"], &["a.rs"]),
        (&["fn ", "-e", "zzz"], &["no matches"], &["a.rs"]),
        (&[""], &["a.rs", "b.md"], &[]),
    ];
    let dir = tree();
    for (args, found, absent) in cases {
        let out = trs(dir.path(), &[&["search", "."], *args].concat());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{args:?}: {stdout}");
        for text in *found {
            assert!(stdout.contains(text), "{args:?} lost {text:?}: {stdout}");
        }
        for text in *absent {
            assert!(!stdout.contains(text), "{args:?} kept {text:?}: {stdout}");
        }
    }
}

#[test]
fn stats_report_what_was_searched() {
    let dir = tree();
    let stats = |flags: &[&str]| {
        let args = [flags, &["--stats", "search", ".", "main"]].concat();
        String::from_utf8_lossy(&trs(dir.path(), &args).stderr).into_owned()
    };
    let compact = stats(&[]);
    for line in [
        "Reducer: search",
        "Output mode: compact",
        "Input bytes:",
        "Output bytes:",
        "Items processed: 3",
        "Files searched: 2",
        "Files with matches: 2",
        "Total matches: 3",
    ] {
        assert!(compact.contains(line), "lost {line:?}: {compact}");
    }
    assert!(stats(&["--json"]).contains("Output mode: json"));
}

#[test]
fn context_lines_are_not_counted_as_matches() {
    let dir = tree();
    let out = trs(dir.path(), &["--json", "search", ".", "Helper", "-C", "1"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let lines = json["files"][0]["matches"].as_array().unwrap().len();
    assert_eq!(lines, 3);
    for field in ["matches", "total_matches", "matches_shown"] {
        assert_eq!(json["counts"][field], 1, "{field}");
    }
}

#[test]
fn parsed_grep_context_is_not_counted_either() {
    let input = "a.rs-1-before\na.rs:2:hit\na.rs-3-after\n";
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["--json", "parse", "grep"])
        .write_stdin(input)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["counts"]["total_matches"], 1);
    assert_eq!(json["counts"]["matches_shown"], 1);
}

#[cfg(unix)]
#[test]
fn a_part_it_could_not_read_fails_and_keeps_the_matches() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), "ok.txt", "needle\n");
    support::write_file(dir.path(), "locked/secret.txt", "needle\n");
    let Some([out, json]) = support::while_locked(&dir.path().join("locked"), || {
        [
            trs(dir.path(), &["search", ".", "needle"]),
            trs(dir.path(), &["--json", "search", ".", "needle"]),
        ]
    }) else {
        return;
    };

    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text.contains("ok.txt") && text.contains("failed: "),
        "{text}"
    );
    let doc = support::document(&json);
    assert_eq!(doc["success"], false);
    assert_eq!(doc["failed_total"], 1);
}

#[test]
fn a_bad_glob_in_a_parent_ignore_file_is_not_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".ignore"), "{bad\n").unwrap();
    support::write_file(dir.path(), "src/a.txt", "needle\n");
    let out = trs(&dir.path().join("src"), &["search", ".", "needle"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn only_git_and_gitignored_directories_are_left_out() {
    let dir = tempfile::tempdir().unwrap();
    support::write_file(dir.path(), ".gitignore", "node_modules/\n");
    support::write_file(dir.path(), ".git/HEAD", "needle\n");
    support::write_file(dir.path(), "node_modules/dep.txt", "needle\n");
    let kept = ["src/a.txt", "vendor/lib.txt", "build/gen.txt"];
    for path in kept {
        support::write_file(dir.path(), path, "needle\n");
    }
    let text =
        String::from_utf8_lossy(&trs(dir.path(), &["search", ".", "needle"]).stdout).into_owned();
    assert!(
        !text.contains("HEAD") && !text.contains("dep.txt"),
        "{text}"
    );
    for path in kept {
        assert!(text.contains(path), "{path} missing: {text}");
    }
}
