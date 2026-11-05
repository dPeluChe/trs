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
