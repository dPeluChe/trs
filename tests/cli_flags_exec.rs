use assert_cmd::Command;
use predicates::prelude::*;

// ============================================================
// Global Flags Tests
// ============================================================

#[test]
fn test_global_flags_help() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--raw"))
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--csv"))
        .stdout(predicate::str::contains("--tsv"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--stats"));
}

// ============================================================
// Command Execution Tests
// ============================================================

#[test]
fn test_search_basic() {
    // Search should find results when searching for a pattern that exists
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_with_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg(".")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--ignore-case")
        .assert()
        .success();
}

#[test]
fn test_search_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\""))
        .stdout(predicate::str::contains("\"grep_output\""))
        .stdout(predicate::str::contains("\"files\""))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_csv_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--csv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path,line_number,column,is_context,line",
        ))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_tsv_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--tsv")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path\tline_number\tcolumn\tis_context\tline",
        ))
        .stdout(predicate::str::contains("router/"));
}

#[test]
fn test_search_raw_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("search.rs:"))
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--context")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_search_with_limit() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("matches:"));
}

#[test]
fn test_search_no_matches() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("search")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_12345")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("no matches"));
}

// ============================================================
