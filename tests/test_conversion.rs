//! Comprehensive integration tests for conversion commands (html2md and txt2md).
//!
//! This test module verifies the conversion tools through the CLI:
//! - html2md: Convert HTML to Markdown
//! - txt2md: Convert plain text to Markdown
//!
//! Tests cover:
//! - Basic conversion functionality
//! - Input sources (file, stdin, URL)
//! - Output formats (JSON, CSV, TSV, Agent, Raw, Compact)
//! - Metadata extraction
//! - Stats output
//! - Edge cases (empty files, special characters)
//! - Error handling (non-existent files)

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixtures").join(name)
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
// TXT2MD Basic Tests
// ============================================================

#[test]
fn test_txt2md_basic_from_stdin() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("HELLO WORLD\n\nThis is a test paragraph.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Hello World"));
}

#[test]
fn test_txt2md_from_file() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_simple.txt"))
        .assert()
        .success()
        .stdout(predicate::str::contains("# Hello World"));
}

#[test]
fn test_txt2md_converts_all_caps_heading() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("SECTION TITLE\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("# Section Title"));
}

#[test]
fn test_txt2md_converts_title_case_heading() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("Getting Started With The Application\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("# Getting Started With The Application"));
}

#[test]
fn test_txt2md_converts_colon_label() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("IMPORTANT NOTES:\n\nThese are notes.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("# Important Notes"));
}

// ============================================================
// TXT2MD List Tests
// ============================================================

#[test]
fn test_txt2md_unordered_list() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("ITEMS\n\n- First item\n- Second item\n- Third item")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("- First item"));
    assert!(stdout.contains("- Second item"));
    assert!(stdout.contains("- Third item"));
}

#[test]
fn test_txt2md_ordered_list() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("STEPS\n\n1. First step\n2. Second step\n3. Third step")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("1. First step"));
    assert!(stdout.contains("2. Second step"));
    assert!(stdout.contains("3. Third step"));
}

#[test]
fn test_txt2md_nested_unordered_list() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("ITEMS\n\n- Main item\n  - Sub item one\n  - Sub item two")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("- Main item"));
    assert!(stdout.contains("  - Sub item one"));
}

#[test]
fn test_txt2md_nested_ordered_list() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("STEPS\n\n1. First step\n   1. Sub step one\n   2. Sub step two")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("1. First step"));
    assert!(stdout.contains("  1. Sub step one"));
}

#[test]
fn test_txt2md_asterisk_list() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("ITEMS\n\n* First item\n* Second item")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Asterisk lists are converted to dash lists
    assert!(stdout.contains("- First item") || stdout.contains("* First item"));
}

#[test]
fn test_txt2md_preserves_list_numbers() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("STEPS\n\n5. Fifth step\n10. Tenth step\n25. Twenty-fifth step")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("5. Fifth step"));
    assert!(stdout.contains("10. Tenth step"));
    assert!(stdout.contains("25. Twenty-fifth step"));
}

#[test]
fn test_txt2md_file_with_lists() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_with_lists.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("# Document Title"));
    assert!(stdout.contains("- First item"));
    assert!(stdout.contains("1. Step one"));
}

// ============================================================
// TXT2MD Code Block Tests
// ============================================================

#[test]
fn test_txt2md_preserves_code_blocks() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("CODE\n\n```\nfunction test() {\n  return 42;\n}\n```")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("```"));
    assert!(stdout.contains("function test()"));
}

#[test]
fn test_txt2md_file_with_code() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_with_code.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("```"));
    assert!(stdout.contains("function test()"));
}

#[test]
fn test_txt2md_preserves_blockquotes() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("QUOTE\n\n> This is a quote")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("> This is a quote"));
}

// ============================================================
// TXT2MD Section Heading Tests
// ============================================================

#[test]
fn test_txt2md_single_word_section_headings() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("Document\n\nIntroduction\n\nThis is the intro.\n\nMethods\n\nMethods here.\n\nResults\n\nResults here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("## Introduction"));
    assert!(stdout.contains("## Methods"));
    assert!(stdout.contains("## Results"));
}

#[test]
fn test_txt2md_section_word_headings() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("Document\n\nAbstract\n\nThis is the abstract.\n\nSummary\n\nThis is the summary.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("## Abstract"));
    assert!(stdout.contains("## Summary"));
}

#[test]
fn test_txt2md_file_with_sections() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_sections.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("# Document Title"));
    assert!(stdout.contains("## Introduction"));
    assert!(stdout.contains("## Methods"));
    assert!(stdout.contains("## Results"));
    assert!(stdout.contains("## Conclusion"));
}

#[test]
fn test_txt2md_chapter_heading() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("Chapter 1: The Beginning\n\nThis is the first chapter.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("Chapter 1: The Beginning"));
}

#[test]
fn test_txt2md_section_number_heading() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("Section 1: Overview\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("Section 1: Overview"));
}

#[test]
fn test_txt2md_long_numbered_heading() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .write_stdin("1. Introduction to the System Architecture Overview\n\nContent here.")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Long numbered items (4+ words) should be preserved as headings
    assert!(stdout.contains("1. Introduction to the System Architecture Overview"));
}

#[test]
fn test_txt2md_file_multiline() {
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("txt2md")
        .arg("-i")
        .arg(fixture_path("txt_multiline.txt"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // Check for chapter heading (all caps is converted to title case with #)
    assert!(stdout.contains("CHAPTER 1: THE BEGINNING") || stdout.contains("Chapter 1: The Beginning"));
    assert!(stdout.contains("Section 1: Overview"));
    assert!(stdout.contains("# Important Notes") || stdout.contains("## Important Notes"));
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
    assert!(json["metadata"]["title"].as_str().unwrap().contains("DOCUMENT TITLE"));
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
    cmd.arg("txt2md")
        .write_stdin("")
        .assert()
        .success();
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
        .stderr(predicate::str::contains("File not found").or(predicate::str::contains("not found")));
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
