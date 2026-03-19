//! Validation tests for Agent, Compact, and Raw output formats.
//!
//! Covers:
//! - Agent format validation
//! - Compact format validation
//! - Raw format validation

use assert_cmd::Command;
use std::io::Write;

// ============================================================
// Agent Format Validation Tests
// ============================================================

#[test]
fn test_agent_git_status_has_structure() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have key: value structure
    assert!(
        stdout.contains("main") || stdout.contains("clean") || stdout.contains("branch:"),
        "Agent format should have key: value structure"
    );
}

#[test]
fn test_agent_search_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output
    assert!(
        stdout.contains("matches:") || stdout.contains("results:"),
        "Agent format should have matches/results structure"
    );
}

#[test]
fn test_agent_tail_has_structure() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1").unwrap();
    writeln!(file, "ERROR: test error").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output
    assert!(
        stdout.contains("File:") || stdout.contains("file:"),
        "Agent format should have file structure"
    );
}

#[test]
fn test_agent_replace_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_AGENT_12345")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should have structured output or indicate no matches
    assert!(
        stdout.contains("total:")
            || stdout.contains("dry_run:")
            || stdout.contains("No matches")
            || stdout.contains("replaced:")
            || stdout.contains("replacements:"),
        "Agent format should have structured output or indicate no matches"
    );
}

// ============================================================
// Compact Format Validation Tests
// ============================================================

#[test]
fn test_compact_git_status_has_structure() {
    let input = "On branch main\nnothing to commit, working tree clean";

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("git-status")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have key: value structure
    assert!(
        stdout.contains("main") || stdout.contains("clean"),
        "Compact format should contain branch name or clean state"
    );
    assert!(
        stdout.contains("clean") || stdout.contains("status:"),
        "Compact format should indicate clean state"
    );
}

#[test]
fn test_compact_search_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have structured output
    assert!(
        stdout.contains("matches:"),
        "Compact format should have 'matches:' key"
    );
}

#[test]
fn test_compact_replace_has_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("replace")
        .arg("src")
        .arg("NONEXISTENT_PATTERN_COMPACT_12345")
        .arg("replacement")
        .arg("--dry-run")
        .arg("--extension")
        .arg("rs")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should have structured output or indicate no matches
    assert!(
        stdout.contains("total:")
            || stdout.contains("replaced:")
            || stdout.contains("No matches")
            || stdout.contains("re:")
            || stdout.contains("dry_run:"),
        "Compact format should have structured output"
    );
}

// ============================================================
// Raw Format Validation Tests
// ============================================================

#[test]
fn test_raw_search_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("search")
        .arg("src")
        .arg("fn")
        .arg("--extension")
        .arg("rs")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show file:line:content pattern
    assert!(
        stdout.contains(".rs:"),
        "Raw search format should show file:line pattern"
    );
}

#[test]
fn test_raw_tail_format() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let mut file = std::fs::File::create(path).unwrap();
    writeln!(file, "line 1 content").unwrap();
    writeln!(file, "line 2 content").unwrap();

    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show line_number:content pattern
    assert!(
        stdout.contains(":line"),
        "Raw tail format should show line_number:content pattern"
    );
}
