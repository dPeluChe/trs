use assert_cmd::Command;
use predicates::prelude::*;

// gh pr checks parser tests

#[test]
fn test_gh_pr_checks_all_pass() {
    let input = "\u{2705} build (20s)\n\
\u{2705} lint (10s)\n\
\u{2705} test (45s)\n\
All checks were successful\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-pr-checks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 pass"))
        .stdout(predicate::str::contains("0 fail"));
}

#[test]
fn test_gh_pr_checks_with_failure() {
    let input = "\u{2705} build (20s)\n\
\u{274C} test (5s)\n\
\u{2705} lint (10s)\n\
Some checks were not successful\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-pr-checks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 pass"))
        .stdout(predicate::str::contains("1 fail"))
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_gh_pr_checks_tsv_format() {
    let input = "build\tpass\t20s\thttps://example.com/1\n\
lint\tpass\t10s\thttps://example.com/2\n\
test\tfail\t5s\thttps://example.com/3\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-pr-checks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("2 pass"))
        .stdout(predicate::str::contains("1 fail"));
}

#[test]
fn test_gh_pr_checks_json_format() {
    let input = "\u{2705} ci (30s)\n\
\u{274C} deploy (5s)\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-pr-checks")
        .arg("--json")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"summary\""))
        .stdout(predicate::str::contains("\"pass\""))
        .stdout(predicate::str::contains("\"fail\""));
}

#[test]
fn test_gh_pr_checks_pending() {
    let input = "\u{2705} build (20s)\n\
\u{1F7E1} deploy\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("gh-pr-checks")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));
}

#[test]
fn test_gh_pr_checks_reduction() {
    let input = (0..20)
        .map(|i| format!("\u{2705} check-job-{i} (10s)\n"))
        .collect::<String>();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("gh-pr-checks")
        .write_stdin(input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ratio = output.len() as f64 / input.len() as f64;
    assert!(ratio < 0.5, "expected >50% reduction, got ratio {ratio:.2}");
}
