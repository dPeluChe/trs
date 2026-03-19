//! Integration tests for the html2md conversion command.
//!
//! Tests cover: basic conversion, heading/formatting/list/code/link conversion,
//! metadata extraction, JSON output, output file, stats, and error handling.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// HTML2MD Basic Tests
// ============================================================

#[test]
fn test_html2md_basic_from_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"))
        .stdout(predicate::str::contains("test paragraph"));
}

#[test]
fn test_html2md_converts_headings() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // H1 should become # heading
    assert!(stdout.contains("# Hello World") || stdout.contains("#  Hello World"));
    // H2 should become ## heading
    assert!(stdout.contains("## Section Heading") || stdout.contains("##  Section Heading"));
}

#[test]
fn test_html2md_converts_formatting() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Strong should become **bold**
    assert!(stdout.contains("**bold**"));
    // Em should become *italic*
    assert!(stdout.contains("*italic*"));
}

#[test]
fn test_html2md_converts_lists() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_lists.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Unordered list items (may be * or - with various spacing)
    assert!(stdout.contains("First item"));
    assert!(stdout.contains("Second item"));
    // Ordered list items
    assert!(stdout.contains("Step one") || stdout.contains("1."));
}

#[test]
fn test_html2md_converts_code_blocks() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_code.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should contain code block markers
    assert!(stdout.contains("```"));
    // Should contain function
    assert!(stdout.contains("function test()"));
}

#[test]
fn test_html2md_converts_blockquotes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_code.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Blockquote should be converted
    assert!(stdout.contains(">"));
}

#[test]
fn test_html2md_converts_links() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_links.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Links should be converted to markdown format [text](url)
    assert!(stdout.contains("[") && stdout.contains("]("));
    assert!(stdout.contains("https://example.com"));
}

#[test]
fn test_html2md_empty_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(fixture_path("html_empty.html"))
        .assert()
        .success();
}

// ============================================================
// HTML2MD Metadata Tests
// ============================================================

#[test]
fn test_html2md_with_metadata_flag() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_metadata.html"))
        .arg("--metadata")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Should include metadata section
    assert!(stdout.contains("source") || stdout.contains("title"));
}

#[test]
fn test_html2md_extracts_title() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_metadata.html"))
        .arg("--metadata")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Title should be extracted
    assert!(stdout.contains("Test Page with Metadata"));
}

#[test]
fn test_html2md_extracts_description() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("html2md")
        .arg(fixture_path("html_with_metadata.html"))
        .arg("--metadata")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Description should be extracted
    assert!(stdout.contains("test page for metadata extraction"));
}

// ============================================================
// HTML2MD JSON Output Tests
// ============================================================

#[test]
fn test_html2md_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have markdown field
    assert!(json["markdown"].is_string());
    assert!(json["markdown"].as_str().unwrap().contains("Hello World"));
}

#[test]
fn test_html2md_json_includes_metadata() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("html2md")
        .arg(fixture_path("html_with_metadata.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // JSON output should always include metadata
    assert!(json["metadata"].is_object());
    assert_eq!(
        json["metadata"]["title"].as_str().unwrap(),
        "Test Page with Metadata"
    );
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "file");
}

// ============================================================
// HTML2MD Output File Tests
// ============================================================

#[test]
fn test_html2md_output_to_file() {
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_html2md_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello World"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

// ============================================================
// HTML2MD Stats Tests
// ============================================================

#[test]
fn test_html2md_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_html2md_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_html2md_stats_shows_reducer() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .stderr(predicate::str::contains("html2md"));
}

// ============================================================
// HTML2MD Error Handling Tests
// ============================================================

#[test]
fn test_html2md_file_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("html2md")
        .arg("/nonexistent/path/file.html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found").or(predicate::str::contains("not found")));
}

// ============================================================
// HTML2MD Agent Format Tests
// ============================================================

#[test]
fn test_html2md_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("html2md")
        .arg(fixture_path("html_simple.html"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should include markdown content
    assert!(stdout.contains("Hello World"));
}
