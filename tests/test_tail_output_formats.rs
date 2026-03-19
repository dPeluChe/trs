//! Integration tests for the `tail` command - output format variations
//! (JSON, CSV, TSV, Agent, Raw, Compact) using fixture files.

use assert_cmd::Command;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// JSON Format Tests
// ============================================================

#[test]
fn test_tail_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check structure
    assert!(json["file"].is_string());
    assert!(json["lines"].is_array());
    assert!(json["total_lines"].is_number());
    assert!(json["lines_shown"].is_number());
    assert!(json["filtering_errors"].is_boolean());
}

#[test]
fn test_tail_json_lines_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("-n")
        .arg("5")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let lines = json["lines"].as_array().expect("lines should be array");
    assert!(!lines.is_empty());

    // Check line structure
    for line in lines {
        assert!(line["line_number"].is_number());
        assert!(line["line"].is_string());
        assert!(line["is_error"].is_boolean());
    }
}

#[test]
fn test_tail_json_with_errors_filter() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("tail")
        .arg("--errors")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert_eq!(json["filtering_errors"], true);
    // All lines should be errors
    let lines = json["lines"].as_array().expect("lines should be array");
    for line in lines {
        assert_eq!(line["is_error"], true);
    }
}

// ============================================================
// CSV Format Tests
// ============================================================

#[test]
fn test_tail_csv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header
    assert!(lines[0].contains("line_number"));
    assert!(lines[0].contains("line"));
    assert!(lines[0].contains("is_error"));

    // Should have data rows
    assert!(lines.len() > 1);
}

#[test]
fn test_tail_csv_with_special_chars() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle special characters without crashing
    assert!(stdout.contains("line_number"));
}

// ============================================================
// TSV Format Tests
// ============================================================

#[test]
fn test_tail_tsv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have header with tabs
    assert!(lines[0].contains('\t'));
    assert!(lines[0].contains("line_number"));
    assert!(lines[0].contains("is_error"));
}

#[test]
fn test_tail_tsv_escaped_tabs() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("tail")
        .arg(fixture_path("tail_with_special_chars.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should handle tabs in content without crashing
    assert!(stdout.contains("line_number"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_tail_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should show file path
    assert!(stdout.contains("File:"));
    // Agent format should show lines info
    assert!(stdout.contains("Lines:") || stdout.contains("total"));
}

#[test]
fn test_tail_agent_error_indicators() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format uses error indicator for error lines
    // The file contains ERROR and FATAL lines
    assert!(stdout.contains('\u{274C}'));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_tail_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should show line numbers and content
    assert!(stdout.contains(':'));
    // Should not have headers or fancy formatting
    assert!(!stdout.contains("File:"));
}

#[test]
fn test_tail_raw_no_headers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("tail")
        .arg("-n")
        .arg("3")
        .arg(fixture_path("tail_large.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should just be line_number:content
    let lines: Vec<&str> = stdout.lines().collect();
    for line in lines {
        // Each line should start with a number
        let first_char = line.chars().next().unwrap();
        assert!(first_char.is_ascii_digit());
    }
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_tail_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format shows header with file info
    assert!(stdout.contains("Last") || stdout.contains("lines"));
}

#[test]
fn test_tail_compact_error_indicators() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("tail")
        .arg(fixture_path("tail_app.log"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format uses error indicator for error lines
    assert!(stdout.contains('\u{274C}'));
}
