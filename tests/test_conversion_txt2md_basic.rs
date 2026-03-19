//! Integration tests for the txt2md conversion command: basic conversion,
//! heading detection, list handling, code blocks, and section headings.

use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get fixture path
fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixture_data").join(name)
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
