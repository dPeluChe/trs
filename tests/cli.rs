use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_hello_default() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, world!"));
}

#[test]
fn test_hello_custom_name() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("hello")
        .arg("--name")
        .arg("TARS")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, TARS!"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tars-cli"));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TARS CLI"));
}

#[test]
fn test_hello_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("hello")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Name to greet"));
}
