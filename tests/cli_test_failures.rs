use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Failing Test Identifiers Extraction Tests
// ============================================================

#[test]
fn test_parse_pytest_failed_tests_identifiers() {
    // Test that pytest parser correctly extracts failing test identifiers
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
tests/test_utils.py::test_helper FAILED
3 passed, 2 failed in 1.23s"#;
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
    // Verify failed_tests array is present and contains correct identifiers
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    assert!(failed_tests.contains(&serde_json::json!("tests/test_main.py::test_subtract")));
    assert!(failed_tests.contains(&serde_json::json!("tests/test_utils.py::test_helper")));
}

#[test]
fn test_parse_pytest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 0.50s"#;
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
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_jest_failed_tests_identifiers() {
    // Test that Jest parser correctly extracts failing test identifiers
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)

FAIL src/api.test.js
  ✕ should fetch data (10 ms)
  ✕ should post data (8 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 2 failed, 3 total"#;
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
    // Verify failed_tests array is present and contains correct identifiers
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should be in format: file::test_name
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("src/api.test.js")));
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("should fetch data")));
    assert!(failed_tests
        .iter()
        .any(|t| t.as_str().unwrap().contains("should post data")));
}

#[test]
fn test_parse_jest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
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
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_vitest_failed_tests_identifiers() {
    // Test that Vitest parser correctly extracts failing test identifiers
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms
   ✓ should add numbers
   ✓ should subtract numbers

 ✗ test/api.test.ts (2 tests | 2 failed) 307ms
   ✓ should get items
   ✕ should fetch data
   ✕ should post data

 Test Files  1 passed, 1 failed (2)
      Tests  3 passed, 2 failed (5)
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
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.ts")));
}

#[test]
fn test_parse_vitest_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let vitest_input = r#" ✓ test/utils.test.ts (2 tests) 306ms

 Test Files  1 passed (1)
      Tests  2 passed (2)
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
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_npm_test_failed_tests_identifiers() {
    // Test that npm test parser correctly extracts failing test identifiers
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)

▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✖ should post data
    Error: connection refused
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 2 failed (3)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.js")));
}

#[test]
fn test_parse_npm_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_pnpm_test_failed_tests_identifiers() {
    // Test that pnpm test parser correctly extracts failing test identifiers
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)

▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✖ should post data
    Error: connection refused
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 2 failed (3)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.js")));
}

#[test]
fn test_parse_pnpm_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_bun_test_failed_tests_identifiers() {
    // Test that Bun test parser correctly extracts failing test identifiers
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]

test/api.test.ts:
✓ should get items [0.18ms]
✗ should fetch data
✗ should post data

 2 pass
 2 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Verify failed_tests array is present
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert_eq!(failed_tests.len(), 2);
    // Failed tests should contain the file path
    assert!(failed_tests
        .iter()
        .all(|t| t.as_str().unwrap().contains("test/api.test.ts")));
}

#[test]
fn test_parse_bun_test_failed_tests_empty_when_all_pass() {
    // Test that failed_tests is empty when all tests pass
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
✓ should subtract numbers [0.18ms]

 2 pass
 0 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failed_tests = json["failed_tests"].as_array().unwrap();
    assert!(failed_tests.is_empty());
}

#[test]
fn test_parse_logs_json_output() {
    let log_input =
        "[INFO] Starting application\n[ERROR] Something went wrong\n[WARN] Warning message";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["counts"]["total_lines"], 3);
    assert_eq!(json["counts"]["info"], 1);
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["warning"], 1);
}

#[test]
fn test_parse_logs_detects_repeated_lines() {
    // Test that repeated lines are detected and counted
    let log_input = "Same line\nDifferent line\nSame line\nSame line\nAnother line\nAnother line";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have 6 total lines
    assert_eq!(json["counts"]["total_lines"], 6);

    // Should detect 2 unique repeated lines
    let repeated = json["repeated_lines"].as_array().unwrap();
    assert_eq!(repeated.len(), 2);

    // Find "Same line" in repeated lines
    let same_line = repeated.iter().find(|r| r["line"] == "Same line").unwrap();
    assert_eq!(same_line["count"], 3);
    assert_eq!(same_line["first_line"], 1);
    assert_eq!(same_line["last_line"], 4);

    // Find "Another line" in repeated lines
    let another_line = repeated
        .iter()
        .find(|r| r["line"] == "Another line")
        .unwrap();
    assert_eq!(another_line["count"], 2);
    assert_eq!(another_line["first_line"], 5);
    assert_eq!(another_line["last_line"], 6);
}

#[test]
fn test_parse_logs_compact_shows_repeated() {
    // Test that compact output shows repeated lines summary
    let log_input = "Repeated message\nOther line\nRepeated message\nRepeated message";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show repeated count
    assert!(stdout.contains("repeated:"));
    // Should show the count [x3]
    assert!(stdout.contains("[x3]"));
    // Should show the line content
    assert!(stdout.contains("Repeated message"));
}

// ============================================================
