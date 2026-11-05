//! What a compact diff must keep: it drops context, never a fact about the change.

use assert_cmd::Command;

fn compact(diff: &str) -> String {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["parse", "git-diff"])
        .write_stdin(diff)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).unwrap()
}

#[test]
fn a_binary_change_is_not_zero_lines() {
    let out = compact(
        "diff --git a/bin.dat b/bin.dat\nindex 1..2 100644\n\
         Binary files a/bin.dat and b/bin.dat differ\n",
    );
    assert!(out.contains("M bin.dat (binary)"), "{out}");
}

#[test]
fn a_chmod_is_not_zero_lines() {
    let out = compact("diff --git a/run.sh b/run.sh\nold mode 100644\nnew mode 100755\n");
    assert!(
        out.contains("run.sh (+0/-0) mode 100644 -> 100755"),
        "{out}"
    );
}

#[test]
fn hunk_lines_that_look_like_headers_are_changes() {
    let out = compact(
        "diff --git a/q.sql b/q.sql\nindex 1..2 100644\n--- a/q.sql\n+++ b/q.sql\n\
         @@ -1,3 +1,3 @@\n keep\n--- sql comment\n-Binary files a and b differ\n\
         +++ counter\n+Binary files c and d differ\n",
    );
    assert!(out.contains("(+2/-2)"), "{out}");
    assert!(out.contains("--- sql comment"), "{out}");
    assert!(out.contains("+++ counter"), "{out}");
    assert!(out.contains("-Binary files a and b differ"), "{out}");
}

#[test]
fn a_path_with_spaces_is_not_a_rename() {
    let out = compact(
        "diff --git a/my file.txt b/my file.txt\nindex 1..2 100644\n--- a/my file.txt\n\
         +++ b/my file.txt\n@@ -1 +1 @@\n-a\n+b\n",
    );
    assert!(out.contains("M my file.txt (+1/-1)"), "{out}");
}

#[test]
fn a_quoted_path_is_not_a_rename() {
    let out = compact(
        "diff --git \"a/caf\\303\\251 x.txt\" \"b/caf\\303\\251 x.txt\"\nindex 1..2 100644\n\
         @@ -1 +1 @@\n-a\n+b\n",
    );
    assert!(out.contains("M caf\\303\\251 x.txt (+1/-1)"), "{out}");
}

#[test]
fn a_rename_with_spaces_keeps_both_names() {
    let out = compact(
        "diff --git a/old name.txt b/new name.txt\nsimilarity index 90%\n\
         rename from old name.txt\nrename to new name.txt\n",
    );
    assert!(out.contains("R old name.txt -> new name.txt"), "{out}");
}

#[test]
fn a_missing_final_newline_stays_visible() {
    let out = compact(
        "diff --git a/n b/n\nindex 1..2 100644\n--- a/n\n+++ b/n\n@@ -1 +1 @@\n\
         -tail\n\\ No newline at end of file\n+tail2\n\\ No newline at end of file\n",
    );
    assert_eq!(out.matches("No newline at end of file").count(), 2, "{out}");
}

fn file_diff(path: &str, hunks: usize, lines: usize) -> String {
    let mut out =
        format!("diff --git a/{path} b/{path}\nindex 1..2 100644\n--- a/{path}\n+++ b/{path}\n");
    for h in 0..hunks {
        out.push_str(&format!("@@ -{h},1 +{h},{lines} @@ fn hunk_{h}\n"));
        for l in 0..lines {
            out.push_str(&format!("+line {l} of hunk {h}\n"));
        }
    }
    out
}

fn section<'a>(out: &'a str, from: &str, to: &str) -> &'a str {
    out.split(from).nth(1).unwrap().split(to).next().unwrap()
}

#[test]
fn a_large_diff_keeps_code_from_every_file_and_says_what_it_cut() {
    let diff = [
        file_diff("Cargo.lock", 1, 900),
        file_diff("src/huge.rs", 6, 900),
        file_diff("src/small.rs", 1, 3),
    ]
    .concat();
    let out = compact(&diff);
    let small = out.split("M src/small.rs (+3/-0)").nth(1).unwrap();
    assert!(small.contains("+line 2 of hunk 0"), "{out}");
    assert!(!small.contains("omitted"), "{out}");
    let huge = section(&out, "M src/huge.rs", "M src/small.rs");
    assert!(huge.contains("fn hunk_5"), "every hunk gets a share: {out}");
    assert!(huge.contains("lines omitted]"), "{out}");
    let lock = section(&out, "M Cargo.lock", "M src/huge.rs");
    assert!(lock.contains("[trs: 901 lines omitted]"), "{out}");
    assert!(out.len() < 28 * 1024, "{}", out.len());
}

#[test]
fn a_lockfile_prints_when_the_code_leaves_room() {
    let diff = [file_diff("Cargo.lock", 1, 40), file_diff("src/a.rs", 1, 40)].concat();
    let out = compact(&diff);
    assert!(!out.contains("omitted"), "{out}");
    assert!(
        section(&out, "M Cargo.lock", "M src/a.rs").contains("+line 39"),
        "{out}"
    );
}

#[test]
fn a_stat_table_prints_as_a_stat_table() {
    let out = compact(" src/a.rs | 400 ++++\n src/b.rs | 300 ----\n 2 files changed, 400 insertions(+), 300 deletions(-)\n");
    assert!(
        out.contains("src/a.rs | 400") && out.contains("src/b.rs | 300"),
        "{out}"
    );
}
