use assert_cmd::Command;
use predicates::prelude::*;

// gh run view parser tests

#[test]
fn test_gh_run_view_success() {
    let input =
        "\u{2705} main: chore: release v0.5.12 workflow run Release / Build aarch64-apple-darwin\n\
Triggered via push about 2 hours ago\n\
\n\
JOBS\n\
\u{2705} Build aarch64-apple-darwin (ID 12345678901)\n\
\u{2705} Build x86_64-apple-darwin (ID 12345678902)\n\
\n\
For more information about a job, try: gh run view --job=<job-id>\n\
View this run on GitHub: https://github.com/dPeluChe/trs/actions/runs/12345678901\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-run-view")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("success"))
        .stdout(predicate::str::contains("2 ok"))
        .stdout(predicate::str::contains("https://github.com"));
}

#[test]
fn test_gh_run_view_failure() {
    let input = "\u{274C} main: fix: broken build workflow run Release / Build x86_64\n\
Triggered via push about 1 hour ago\n\
\n\
JOBS\n\
\u{2705} Build aarch64-apple-darwin (ID 11111)\n\
\u{274C} Build x86_64-unknown-linux-gnu (ID 11112)\n\
\n\
ANNOTATIONS\n\
Build / Build x86_64: error: linker `cc` not found\n\
\n\
View this run on GitHub: https://github.com/dPeluChe/trs/actions/runs/11111\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-run-view")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("failure"))
        .stdout(predicate::str::contains("1 ok, 1 fail"))
        .stdout(predicate::str::contains("linker"));
}

#[test]
fn test_gh_run_view_json_format() {
    let input = "\u{2705} main: docs: add llms.txt workflow run Release\n\
Triggered via push\n\
\n\
JOBS\n\
\u{2705} Build darwin-arm64 (ID 99999)\n\
\n\
View this run on GitHub: https://github.com/dPeluChe/trs/actions/runs/99999\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("gh-run-view")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output)).expect("invalid JSON");
    assert_eq!(json["conclusion"], "success");
    assert_eq!(json["jobs"]["ok"], 1);
    assert_eq!(json["jobs"]["fail"], 0);
}

#[test]
fn test_gh_run_view_reduces_output() {
    let input = "\u{2705} main: chore: bump version workflow run Release\n\
Triggered via push about 3 hours ago\n\
\n\
JOBS\n\
\u{2705} Build aarch64-apple-darwin (ID 1)\n\
\u{2705} Build x86_64-apple-darwin (ID 2)\n\
\u{2705} Build x86_64-unknown-linux-gnu (ID 3)\n\
\u{2705} Build aarch64-unknown-linux-gnu (ID 4)\n\
\u{2705} Build x86_64-pc-windows-msvc (ID 5)\n\
\n\
For more information about a job, try: gh run view --job=<job-id>\n\
View this run on GitHub: https://github.com/dPeluChe/trs/actions/runs/123\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let out = cmd
        .arg("parse")
        .arg("gh-run-view")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Output should be much smaller than input
    assert!(out.len() < input.len() / 2, "expected >50% reduction");
}
