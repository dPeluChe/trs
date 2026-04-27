use assert_cmd::Command;
use predicates::prelude::*;

// gh pr diff routes to git-diff parser

#[test]
fn test_gh_pr_diff_produces_diff_output() {
    // gh pr diff output looks like a standard unified diff
    let input = "diff --git a/src/main.rs b/src/main.rs\n\
index abc1234..def5678 100644\n\
--- a/src/main.rs\n\
+++ b/src/main.rs\n\
@@ -1,5 +1,6 @@\n\
 fn main() {\n\
+    println!(\"hello\");\n\
     let x = 1;\n\
 }\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("+1"));
}

#[test]
fn test_gh_pr_diff_json_format() {
    let input = "diff --git a/README.md b/README.md\n\
index 0000000..1111111 100644\n\
--- a/README.md\n\
+++ b/README.md\n\
@@ -1,3 +1,4 @@\n\
 # Title\n\
+New line\n\
 content\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("git-diff")
        .arg("--json")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\"").or(predicate::str::contains("README.md")));
}

#[test]
fn test_gh_pr_diff_reduction() {
    let input = (0..30)
        .map(|i| {
            format!(
                "diff --git a/file{i}.rs b/file{i}.rs\nindex abc..def 100644\n--- a/file{i}.rs\n+++ b/file{i}.rs\n@@ -1,3 +1,4 @@\n fn foo() {{}}\n+fn bar() {{}}\n"
            )
        })
        .collect::<String>();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("git-diff")
        .write_stdin(input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ratio = output.len() as f64 / input.len() as f64;
    assert!(ratio < 0.6, "expected >40% reduction, got ratio {ratio:.2}");
}
