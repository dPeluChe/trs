//! Integration tests for the txt2md conversion command: output formats
//! (JSON, raw, agent), output file, empty input, stats, error handling,
//! multiline list items, and mixed content.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
}

// ============================================================
// TXT2MD JSON Output Tests
// ============================================================

#[test]
fn test_txt2md_json_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("txt2md")
        .write_stdin("TITLE\n\nParagraph text.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Should have markdown field
    assert!(json["markdown"].is_string());
    assert!(json["markdown"].as_str().unwrap().contains("# Title"));

    // Should have metadata field
    assert!(json["metadata"].is_object());
}

#[test]
fn test_txt2md_json_includes_metadata() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("txt2md")
        .write_stdin("DOCUMENT TITLE\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    // Metadata should include type and title
    assert_eq!(json["metadata"]["type"].as_str().unwrap(), "stdin");
    assert!(json["metadata"]["title"]
        .as_str()
        .unwrap()
        .contains("DOCUMENT TITLE"));
}

// ============================================================
// TXT2MD Output File Tests
// ============================================================

#[test]
fn test_txt2md_output_to_file() {
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("-o")
        .arg(&output_path)
        .write_stdin("SECTION\n\nContent here.")
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Section"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_txt2md_file_input_file_output() {
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("test_txt2md_file_output.md");

    // Clean up any existing file
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_simple.txt"))
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Hello World"));

    // Cleanup
    let _ = std::fs::remove_file(&output_path);
}

// ============================================================
// TXT2MD Raw Output Tests
// ============================================================

#[test]
fn test_txt2md_raw_output() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--raw")
        .arg("txt2md")
        .write_stdin("TITLE\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Raw output should contain markdown
    assert!(stdout.contains("# Title"));
    // Raw output should NOT include metadata
    assert!(!stdout.contains("\"metadata\""));
}

// ============================================================
// TXT2MD Empty Input Tests
// ============================================================

#[test]
fn test_txt2md_empty_stdin() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md").write_stdin("").assert().success();
}

#[test]
fn test_txt2md_empty_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_empty.txt"))
        .assert()
        .success();
}

// ============================================================
// TXT2MD Stats Tests
// ============================================================

#[test]
fn test_txt2md_stats_shows_input_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin("Heading\n\nParagraph text.")
        .assert()
        .success()
        .stderr(predicate::str::contains("Input bytes:"));
}

#[test]
fn test_txt2md_stats_shows_output_bytes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin("Heading\n\nParagraph text.")
        .assert()
        .success()
        .stderr(predicate::str::contains("Output bytes:"));
}

#[test]
fn test_txt2md_stats_shows_reducer() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--stats")
        .arg("txt2md")
        .write_stdin("Heading\n\nParagraph text.")
        .assert()
        .success()
        .stderr(predicate::str::contains("txt2md"));
}

// ============================================================
// TXT2MD Error Handling Tests
// ============================================================

#[test]
fn test_txt2md_file_not_found() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("-i")
        .arg("/nonexistent/path/file.txt")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("File not found").or(predicate::str::contains("not found")),
        );
}

// ============================================================
// TXT2MD Multiline List Item Tests
// ============================================================

#[test]
fn test_txt2md_multiline_list_item() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("ITEMS\n\n- First item with\n  continuation line\n- Second item")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("- First item with"));
    assert!(stdout.contains("continuation line"));
    assert!(stdout.contains("- Second item"));
}

// ============================================================
// TXT2MD Mixed Format Tests
// ============================================================

#[test]
fn test_txt2md_mixed_content() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("DOCUMENT TITLE\n\nIntroduction paragraph.\n\nSECTION ONE\n\n- Item 1\n- Item 2\n\n```\ncode block\n```\n\n> Quote here\n\nCONCLUSION\n\nFinal thoughts.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Check all elements are present
    assert!(stdout.contains("# Document Title"));
    assert!(stdout.contains("## Section One"));
    assert!(stdout.contains("- Item 1"));
    assert!(stdout.contains("```"));
    assert!(stdout.contains("> Quote here"));
    assert!(stdout.contains("## Conclusion"));
}

// ============================================================
// TXT2MD Agent Format Tests
// ============================================================

#[test]
fn test_txt2md_agent_format() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--agent")
        .arg("txt2md")
        .write_stdin("TITLE\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Agent format should include metadata
    assert!(stdout.contains("metadata") || stdout.contains("# Title"));
}
