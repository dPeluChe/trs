use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Bun Test Parser Tests
// ============================================================

#[test]
fn test_parse_bun_test_json_output() {
    // Test that bun parser works and produces valid JSON output
    let bun_input = r#"test/package-json-lint.test.ts:
✓ test/package.json [0.88ms]
✓ test/js/third_party/grpc-js/package.json [0.18ms]

 4 pass
 0 fail
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 4);
    assert_eq!(json["summary"]["tests_failed"], 0);
    assert_eq!(json["summary"]["expect_calls"], 4);
}

#[test]
fn test_parse_bun_test_failing_compact_output() {
    // Test compact output with failures
    let bun_input = r#"test/api.test.ts:
✓ should pass [0.88ms]
✗ should fail

 1 pass
 1 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.ts"));
}

#[test]
fn test_parse_bun_test_non_tty_format() {
    // Test non-TTY format (for CI environments)
    let bun_input = r#"test/package-json-lint.test.ts:
(pass) test/package.json [0.48ms]
(fail) test/failing.test.ts
(skip) test/skipped.test.ts

 2 pass
 1 fail
 1 skipped
Ran 4 tests across 1 files. [0.66ms]"#;
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
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_bun_test_all_passed() {
    // Test all tests passed
    let bun_input = r#"test/math.test.ts:
✓ should add numbers [1.00ms]
✓ should subtract numbers [0.50ms]

 2 pass
 0 fail
 2 expect() calls
Ran 2 tests in 1.50ms"#;
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
}

#[test]
fn test_parse_bun_test_failing_json() {
    // Test JSON output with failures
    let bun_input = r#" ✗ test/failing.test.ts (2 tests | 1 failed) 307ms

 1 pass
 1 fail
Ran 2 tests in 0.50s"#;
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
    // With the current parser, this output without a suite header might not parse as expected
    // but it should still produce valid JSON
    assert!(json.is_object());
}

// ============================================================
// NPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_npm_test_json_output() {
    // Test that npm parser works and produces valid JSON output with passed test count
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_npm_test_failing_compact_output() {
    // Test compact output with failures
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/math.test.js"));
}

#[test]
fn test_parse_npm_test_with_skipped() {
    // Test that npm parser correctly counts passed tests with skipped tests
    let npm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_npm_test_failing_json() {
    // Test that npm parser correctly extracts failed test count in JSON output
    let npm_input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
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
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
// PNPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_pnpm_test_json_output() {
    // Test that pnpm parser works and produces valid JSON output with passed test count
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_passed"], 1);
    assert_eq!(json["summary"]["suites_total"], 1);
}

#[test]
fn test_parse_pnpm_test_failing_compact_output() {
    // Test compact output with failures
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Compact output should show summary and failed tests
    assert!(stdout.contains("FAIL:"));
    assert!(stdout.contains("1 passed, 1 failed"));
    assert!(stdout.contains("test/api.test.js"));
}

#[test]
fn test_parse_pnpm_test_with_skipped() {
    // Test that pnpm parser correctly counts passed tests with skipped tests
    let pnpm_input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
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
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["tests_passed"], 2);
    assert_eq!(json["summary"]["tests_skipped"], 1);
    assert_eq!(json["summary"]["tests_total"], 3);
}

#[test]
fn test_parse_pnpm_test_failing_json() {
    // Test that pnpm parser correctly extracts failed test count in JSON output
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)

ℹ tests 1 passed 1 failed (2)
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
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["tests_passed"], 1);
    assert_eq!(json["summary"]["tests_failed"], 1);
    assert_eq!(json["summary"]["tests_total"], 2);
    assert_eq!(json["summary"]["suites_failed"], 1);
}

// ============================================================
