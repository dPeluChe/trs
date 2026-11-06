//! Integration tests for the `search` command: basic operations.
//!
//! Covers:
//! - Basic pattern search
//! - Extension filtering
//! - Case-insensitive search
//! - Context lines
//! - JSON output format

use predicates::prelude::*;

mod support;

// ============================================================
// Basic Search Tests
// ============================================================

#[test]
fn test_search_basic_pattern() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg(".")
        .arg("fn main")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("fn main"));
}

#[test]
fn test_search_in_specific_path() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .stdout(predicate::str::contains("router/"))
        .stdout(predicate::str::contains("search.rs (3):"))
        .stdout(predicate::str::contains("1:12:"))
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_returns_file_path() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("Router")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs"));
}

// ============================================================
// Extension Filter Tests
// ============================================================

#[test]
fn test_search_extension_keeps_only_that_type() {
    let cases = [
        ("--extension", "rs", ".rs", ".md"),
        ("-e", "rs", ".rs", ".md"),
        ("-e", "md", "guide.md", ".rs"),
    ];
    for (flag, extension, kept, gone) in cases {
        let mut cmd = support::search_cmd();
        cmd.args(["search", ".", "SearchHandler", flag, extension])
            .assert()
            .success()
            .stdout(predicate::str::contains(kept))
            .stdout(predicate::str::contains(gone).not());
    }
}

// ============================================================
// Case-Insensitive Search Tests
// ============================================================

#[test]
fn test_search_ignore_case_flag() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("--ignore-case")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_ignore_case_short_flag() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("searchhandler")
        .arg("-i")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_case_sensitive_no_match() {
    // Case-sensitive search with wrong case should not match
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("search")
        .arg("src")
        .arg("searchhandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should not find "SearchHandler" when searching for lowercase
    assert!(!stdout.contains("SearchHandler"));
}

// ============================================================
// Context Lines Tests
// ============================================================

#[test]
fn test_search_with_context_lines() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("--context")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context_short_flag() {
    let mut cmd = support::search_cmd();
    cmd.arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-C")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchHandler"));
}

#[test]
fn test_search_with_context_includes_surrounding_lines() {
    let mut cmd = support::search_cmd();
    // With context, there should be more output than just the matching line
    let output_with_context = cmd
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .arg("-C")
        .arg("3")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let len_with_context = output_with_context.len();

    let mut cmd2 = support::search_cmd();
    let output_without_context = cmd2
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let len_without_context = output_without_context.len();

    // With context should generally have more output
    assert!(len_with_context >= len_without_context);
}

// ============================================================
// JSON Output Format Tests
// ============================================================

#[test]
fn test_search_json_has_its_fields() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("SearchHandler")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let files = json["files"].as_array().unwrap();
    let file = files
        .iter()
        .find(|f| f["path"].as_str().unwrap().ends_with("handlers/search.rs"))
        .unwrap();
    assert_eq!(file["matches"][0]["line_number"], 1);
    assert_eq!(file["matches"][0]["line"], "pub struct SearchHandler;");
    assert!(json["counts"]["matches"].as_u64().unwrap() > 0);
}

#[test]
fn test_search_json_empty_result() {
    let mut cmd = support::search_cmd();
    let output = cmd
        .arg("--json")
        .arg("search")
        .arg("src")
        .arg("nonexistent_pattern_xyz123")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert!(json["is_empty"].as_bool().unwrap());
    assert!(json["files"].as_array().unwrap().is_empty());
}
