//! What each `trs search` flag does to the results, on a tree with known
//! content, so a flag that stops working fails instead of matching nothing.

use assert_cmd::Command;

fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(
        dir.path().join("src/a.rs"),
        "fn main() {}\nfn Helper() {}\nlet a = 1;\nlet b = 2;\nmain-thread\nv1.2.3\nv1x2y3\ncafé\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("docs/b.md"), "fn main() {}\nHELPER\n").unwrap();
    dir
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
        (&["let", "-C", "1"], &["fn Helper", "let a = 1", "main-thread"], &["cafe"]),
        (&["main-thread"], &["main-thread"], &["fn main"]),
        (&["v1\\.2"], &["v1.2.3"], &["v1x2y3"]),
        (&["café"], &["café"], &["fn main"]),
    ];
    let dir = tree();
    for (args, found, absent) in cases {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .current_dir(dir.path())
            .args(["search", "."])
            .args(*args)
            .output()
            .unwrap();
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
    let trs = |flags: &[&str]| {
        let out = Command::cargo_bin("trs")
            .unwrap()
            .current_dir(dir.path())
            .args(flags)
            .args(["--stats", "search", ".", "main"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stderr).into_owned()
    };
    let stats = trs(&[]);
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
        assert!(stats.contains(line), "lost {line:?}: {stats}");
    }
    assert!(trs(&["--json"]).contains("Output mode: json"));
}
