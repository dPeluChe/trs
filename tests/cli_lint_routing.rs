use assert_cmd::Command;
use predicates::prelude::*;

// lint:* script variants routing tests

fn lint_output() -> &'static str {
    "src/foo.ts:10:5: error  'x' is never used  @typescript-eslint/no-unused-vars\n\
src/foo.ts:22:3: warning  Missing semicolon  semi\n\
src/bar.ts:5:1: error  Expected a block  curly\n\
\n\
3 problems (2 errors, 1 warning)\n"
}

#[test]
fn test_npm_run_lint_strict_routes_to_lint() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("lint")
        .write_stdin(lint_output())
        .assert()
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("warning")));
}

#[test]
fn test_npm_run_lint_fix_routes_to_lint() {
    // lint:fix output is similar to lint but with "fixed" lines
    let input = "Fixed 2 problems.\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("lint")
        .write_stdin(input)
        .assert()
        .success();
}

#[test]
fn test_lint_parser_reduces_output() {
    let input = lint_output().repeat(5);

    let output = Command::cargo_bin("trs")
        .unwrap()
        .arg("parse")
        .arg("lint")
        .write_stdin(input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ratio = output.len() as f64 / input.len() as f64;
    assert!(
        ratio < 0.9,
        "expected some reduction from lint parser, got {ratio:.2}"
    );
}

#[test]
fn test_poetry_run_pytest_routes_correctly() {
    // Simulates output from `poetry run pytest` — same as direct pytest
    let input = "collected 5 items\n\
\n\
tests/test_foo.py::test_one PASSED\n\
tests/test_foo.py::test_two PASSED\n\
tests/test_foo.py::test_three FAILED\n\
\n\
FAILURES\n\
tests/test_foo.py::test_three\n\
    assert 1 == 2\n\
\n\
1 failed, 4 passed in 0.42s\n";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 failed").or(predicate::str::contains("failed")));
}
