use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

#[test]
fn test_txt2md_unordered_list() {
    // Test unordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n- First item\n- Second item\n- Third item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item"))
        .stdout(predicate::str::contains("- Second item"))
        .stdout(predicate::str::contains("- Third item"));
}

#[test]
fn test_txt2md_ordered_list() {
    // Test ordered list detection - numbers are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("STEPS\n\n1. First step\n2. Second step\n3. Third step")
        .assert()
        .success()
        .stdout(predicate::str::contains("1. First step"))
        .stdout(predicate::str::contains("2. Second step"))
        .stdout(predicate::str::contains("3. Third step"));
}

#[test]
fn test_txt2md_raw_output() {
    // Test raw output format
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--raw")
        .arg("txt2md")
        .write_stdin("TITLE\n\nContent here.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Title"))
        // Raw output should NOT include metadata
        .stdout(predicate::str::contains("metadata").not());
}

#[test]
fn test_txt2md_code_block() {
    // Test that code blocks are preserved
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("CODE\n\n```\nfunction test() {\n  return 1;\n}\n```")
        .assert()
        .success()
        .stdout(predicate::str::contains("```"))
        .stdout(predicate::str::contains("function test()"));
}

#[test]
fn test_txt2md_blockquote() {
    // Test blockquote detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("QUOTE\n\n> This is a quote")
        .assert()
        .success()
        .stdout(predicate::str::contains("> This is a quote"));
}

#[test]
fn test_txt2md_section_heading() {
    // Test section/chapter/part heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("INTRODUCTION\n\nSection 1: Getting Started\n\nSome content here.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Introduction"))
        .stdout(predicate::str::contains("Section 1: Getting Started"));
}

#[test]
fn test_txt2md_chapter_heading() {
    // Test chapter heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Chapter 1: The Beginning\n\nThis is the first chapter.")
        .assert()
        .success()
        .stdout(predicate::str::contains("Chapter 1: The Beginning"));
}

#[test]
fn test_txt2md_title_case_heading() {
    // Test title case heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "Getting Started With The Application\n\nThis is a paragraph about getting started.",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# Getting Started With The Application",
        ));
}

#[test]
fn test_txt2md_colon_label() {
    // Test colon-ended label heading detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("IMPORTANT NOTES:\n\nThese are important notes.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Important Notes"));
}

#[test]
fn test_txt2md_numbered_section_heading() {
    // Test numbered section heading with multiple words (should be heading, not list)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("1. Introduction to the System Architecture Overview\n\nSome content here.")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "1. Introduction to the System Architecture Overview",
        ));
}

#[test]
fn test_txt2md_single_word_section_heading() {
    // Test single-word section headings like "Introduction", "Methods", "Results"
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Document Title\n\nIntroduction\n\nThis is the intro.\n\nMethods\n\nHere are methods.\n\nResults\n\nHere are results.\n\nConclusion\n\nHere is conclusion.")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Document Title"))
        .stdout(predicate::str::contains("## Introduction"))
        .stdout(predicate::str::contains("## Methods"))
        .stdout(predicate::str::contains("## Results"))
        .stdout(predicate::str::contains("## Conclusion"));
}

#[test]
fn test_txt2md_common_section_words() {
    // Test common section words are detected as headings
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "Document\n\nAbstract\n\nThis is the abstract.\n\nSummary\n\nThis is the summary.",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("## Abstract"))
        .stdout(predicate::str::contains("## Summary"));
}

#[test]
fn test_txt2md_extended_section_words() {
    // Test extended section words like History, Future, Design
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("Project\n\nHistory\n\nThis is the history.\n\nFuture\n\nThis is the future.")
        .assert()
        .success()
        .stdout(predicate::str::contains("## History"))
        .stdout(predicate::str::contains("## Future"));
}

#[test]
fn test_txt2md_nested_unordered_list() {
    // Test nested unordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "ITEMS\n\n- Main item one\n  - Sub item one\n  - Sub item two\n- Main item two",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("- Main item one"))
        .stdout(predicate::str::contains("  - Sub item one"))
        .stdout(predicate::str::contains("  - Sub item two"))
        .stdout(predicate::str::contains("- Main item two"));
}

#[test]
fn test_txt2md_nested_ordered_list() {
    // Test nested ordered list detection
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "STEPS\n\n1. First step\n   1. Sub step one\n   2. Sub step two\n2. Second step",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("1. First step"))
        .stdout(predicate::str::contains("  1. Sub step one"))
        .stdout(predicate::str::contains("  2. Sub step two"))
        .stdout(predicate::str::contains("2. Second step"));
}

#[test]
fn test_txt2md_mixed_nested_list() {
    // Test mixed nested lists (ordered inside unordered and vice versa)
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin(
            "ITEMS\n\n- Main item\n  1. First sub step\n  2. Second sub step\n- Another item",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("- Main item"))
        .stdout(predicate::str::contains("  1. First sub step"))
        .stdout(predicate::str::contains("  2. Second sub step"))
        .stdout(predicate::str::contains("- Another item"));
}

#[test]
fn test_txt2md_multiline_list_item() {
    // Test multi-line list item with continuation
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n- First item with\n  continuation line\n- Second item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item with"))
        .stdout(predicate::str::contains("continuation line"))
        .stdout(predicate::str::contains("- Second item"));
}

#[test]
fn test_txt2md_asterisk_list() {
    // Test asterisk-based unordered list
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("txt2md")
        .write_stdin("ITEMS\n\n* First item\n* Second item\n* Third item")
        .assert()
        .success()
        .stdout(predicate::str::contains("- First item"))
        .stdout(predicate::str::contains("- Second item"))
        .stdout(predicate::str::contains("- Third item"));
}

#[test]
fn test_txt2md_ordered_list_preserves_numbers() {
    // Test that ordered list numbers are preserved (not normalized to 1.)
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
    assert!(stdout.contains("5. Fifth step"), "Should preserve number 5");
    assert!(
        stdout.contains("10. Tenth step"),
        "Should preserve number 10"
    );
    assert!(
        stdout.contains("25. Twenty-fifth step"),
        "Should preserve number 25"
    );
}

#[test]
fn test_parse_grep_json_output() {
    // Test that grep parser now works and produces valid JSON output
    let grep_input = "src/main.rs:42:fn main() {";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("grep")
        .write_stdin(grep_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["matches"], 1);
}

#[test]
fn test_parse_test_json_output() {
    // Test that pytest parser now works and produces valid JSON output
    let pytest_input = r#"tests/test_main.py::test_add PASSED
1 passed in 0.01s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 1);
    assert_eq!(json["summary"]["total"], 1);
}

#[test]
fn test_parse_test_jest_json_output() {
    // Test that Jest parser works and produces valid JSON output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["tests"]["total"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
    assert_eq!(json["summary"]["suites"]["total"], 1);
}

#[test]
fn test_parse_test_jest_compact_output() {
    // Test that Jest parser works with compact output
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)

FAIL src/api.test.js
  ✕ should fetch data (10 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 1 failed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("src/api.test.js"));
}

#[test]
fn test_parse_test_vitest_json_output() {
    // Test that Vitest parser works and produces valid JSON output
    let vitest_input = r#" ✓ test/example-1.test.ts (5 tests | 1 skipped) 306ms
 ✓ test/example-2.test.ts (5 tests) 307ms

 Test Files  2 passed (4)
      Tests  10 passed | 3 skipped (65)
   Start at  11:01:36
   Duration  2.00s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 10);
    assert_eq!(json["summary"]["tests"]["skipped"], 3);
    assert_eq!(json["summary"]["tests"]["total"], 65);
    assert_eq!(json["summary"]["suites"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["total"], 4);
}

#[test]
fn test_parse_test_vitest_compact_output() {
    // Test that Vitest parser works with compact output
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms

 ✗ test/api.test.ts (2 tests | 1 failed) 307ms

 Test Files  1 passed, 1 failed (2)
      Tests  3 passed, 1 failed, 2 skipped (6)
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("failed suites"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_test_vitest_with_tree_output() {
    // Test that Vitest parser works with tree format output
    let vitest_input = r#"✓ __tests__/file1.test.ts (2) 725ms
   ✓ first test file (2) 725ms
     ✓ 2 + 2 should equal 4
     ✓ 4 - 2 should equal 2

 Test Files  1 passed (1)
      Tests  2 passed (2)
   Start at  12:34:32
   Duration  1.26s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests"]["passed"], 2);
    assert_eq!(json["summary"]["suites"]["passed"], 1);
}

#[test]
fn test_parse_test_vitest_failed_output() {
    // Test that Vitest parser handles failed tests
    let vitest_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 306ms

 Test Files  1 failed (1)
      Tests  1 passed, 1 failed (2)
   Duration  0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests"]["passed"], 1);
    assert_eq!(json["summary"]["tests"]["failed"], 1);
    assert_eq!(json["summary"]["suites"]["failed"], 1);
}

// ============================================================
