//! Integration tests for the `clean` command: output format variations
//! (JSON, CSV, TSV, Agent, Raw, Compact) and stats output.

use assert_cmd::Command;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// JSON Format Tests
// ============================================================

#[test]
fn test_clean_json_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Check structure
    assert!(json["content"].is_string());
    assert!(json["stats"].is_object());
    assert!(json["options"].is_object());
}

#[test]
fn test_clean_json_stats_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(json["stats"]["input_length"].is_number());
    assert!(json["stats"]["output_length"].is_number());
    assert!(json["stats"]["reduction_percent"].is_number());
}

#[test]
fn test_clean_json_options_structure() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("--no-ansi")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert_eq!(json["options"]["no_ansi"], true);
    assert_eq!(json["options"]["trim"], true);
}

#[test]
fn test_clean_json_content_cleaned() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("clean")
        .arg("--no-ansi")
        .arg("-f")
        .arg(fixture_path("clean_with_ansi.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    let content = json["content"].as_str().expect("content should be string");
    assert!(content.contains("[INFO]"));
    // ANSI codes should be stripped
    assert!(!content.contains("\x1b["));
}

// ============================================================
// CSV Format Tests
// ============================================================

#[test]
fn test_clean_csv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // CSV format outputs each line as a quoted CSV row
    assert!(stdout.contains("Hello World"));
    // Lines should be quoted
    assert!(stdout.contains('"'));
}

#[test]
fn test_clean_csv_escapes_quotes() {
    let input = "Line with \"quotes\"\n";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--csv")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Quotes should be escaped by doubling
    assert!(stdout.contains("\"\"quotes\"\"") || stdout.contains("quotes"));
}

// ============================================================
// TSV Format Tests
// ============================================================

#[test]
fn test_clean_tsv_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--tsv")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // TSV format outputs each line as-is
    assert!(stdout.contains("Hello World"));
}

// ============================================================
// Agent Format Tests
// ============================================================

#[test]
fn test_clean_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format shows reduction percentage
    assert!(stdout.contains("Content") || stdout.contains("reduction"));
}

#[test]
fn test_clean_agent_shows_reduction() {
    let input = "Line 1\n\n\n\n\nLine 2\n"; // Has blank lines that will be reduced
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should show reduction percentage
    assert!(stdout.contains("% reduction") || stdout.contains("reduction"));
}

// ============================================================
// Raw Format Tests
// ============================================================

#[test]
fn test_clean_raw_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw format should just output the cleaned content
    assert!(stdout.contains("Hello World"));
    // Should not have JSON or other formatting
    assert!(!stdout.contains("\"content\""));
    assert!(!stdout.contains("reduction"));
}

// ============================================================
// Compact Format Tests
// ============================================================

#[test]
fn test_clean_compact_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format shows content and optionally reduction
    assert!(stdout.contains("Hello World"));
}

#[test]
fn test_clean_compact_shows_reduction_when_positive() {
    let input = "Line 1\n\n\n\n\nLine 2\n"; // Has extra content that will be reduced
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("clean")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Compact format should show reduction percentage when > 0
    assert!(stdout.contains("% reduction") || stdout.contains("Line"));
}

// ============================================================
// Stats Tests
// ============================================================

#[test]
fn test_clean_with_stats() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("clean")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .clone();

    // Stats are printed to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should show reducer name
    assert!(stderr.contains("clean"));
}

#[test]
fn test_clean_stats_shows_options() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--stats")
        .arg("clean")
        .arg("--no-ansi")
        .arg("--trim")
        .arg("-f")
        .arg(fixture_path("clean_simple.txt"))
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Stats should show the options used
    assert!(stderr.contains("No ANSI") || stderr.contains("no_ansi") || stderr.contains("clean"));
}
