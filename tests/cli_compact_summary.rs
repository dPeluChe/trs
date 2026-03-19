use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

// Compact Success Summary Tests
// ============================================================

#[test]
fn test_parse_pytest_compact_success_summary() {
    // Test that pytest shows compact summary when all tests pass
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 2 tests"))
        .stdout(predicate::str::contains("[0.50s]"))
        // Should NOT show detailed breakdown
        .stdout(predicate::str::contains("passed,").not());
}

#[test]
fn test_parse_pytest_compact_failure_summary() {
    // Test that pytest shows detailed failure info when tests fail
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
1 passed, 1 failed in 1.23s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed (1):"));
}

#[test]
fn test_parse_jest_compact_success_summary() {
    // Test that Jest shows compact summary when all tests pass
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)
Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total
Time:        1.5 s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"))
        .stdout(predicate::str::contains("[1.50s]"))
        // Should NOT show detailed breakdown with passed/failed counts
        .stdout(predicate::str::contains("passed, 0 failed").not());
}

#[test]
fn test_parse_jest_compact_failure_summary() {
    // Test that Jest shows detailed failure info when tests fail
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
FAIL src/api.test.js
  ✕ should fetch data (10 ms)
Test Suites: 1 passed, 1 failed, 2 total
Tests:       1 passed, 1 failed, 2 total"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("jest")
        .write_stdin(jest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed suites (1):"));
}

#[test]
fn test_parse_vitest_compact_success_summary() {
    // Test that Vitest shows compact summary when all tests pass
    let vitest_input = r#" ✓ src/utils.test.js (2 tests) 150ms
 Test Files  1 passed (1)
      Tests  2 passed (2)
   Start at  12:00:00
   Duration  1.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 test files, 2 tests"))
        .stdout(predicate::str::contains("[1.50s]"));
}

#[test]
fn test_parse_vitest_compact_failure_summary() {
    // Test that Vitest shows detailed failure info when tests fail
    let vitest_input = r#" ✓ src/utils.test.js (1 test) 100ms
   ✓ should add numbers
 ✗ src/api.test.js (1 test | 1 failed) 150ms
   ✕ should fetch data
 Test Files  1 passed, 1 failed (2)
      Tests  1 passed, 1 failed (2)"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("vitest")
        .write_stdin(vitest_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed suites (1):"));
}

#[test]
fn test_parse_npm_test_compact_success_summary() {
    // Test that npm test shows compact summary when all tests pass
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)
ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_npm_test_compact_failure_summary() {
    // Test that npm test shows detailed failure info when tests fail
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
▶ test/api.test.js
  ✖ should fetch data
ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("npm")
        .write_stdin(npm_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 1 failed"));
}

#[test]
fn test_parse_pnpm_test_compact_success_summary() {
    // Test that pnpm test shows compact summary when all tests pass
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)
ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_pnpm_test_compact_failure_summary() {
    // Test that pnpm test shows detailed failure info when tests fail
    let pnpm_input = r#"▶ test/api.test.js
  ✖ should fetch data
    Error: network timeout
  ✔ should create item (2.345ms)
▶ test/api.test.js (8.123ms)
ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 12ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pnpm")
        .write_stdin(pnpm_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 1 failed"));
}

#[test]
fn test_parse_bun_test_compact_success_summary() {
    // Test that Bun test shows compact summary when all tests pass
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
✓ should subtract numbers [0.45ms]
 2 pass
 0 fail
Ran 2 tests in 1.50ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success()
        // Should show minimal compact summary
        .stdout(predicate::str::contains("PASS: 1 suites, 2 tests"));
}

#[test]
fn test_parse_bun_test_compact_failure_summary() {
    // Test that Bun test shows detailed failure info when tests fail
    let bun_input = r#"test/utils.test.ts:
✓ should add numbers [0.88ms]
test/api.test.ts:
✗ should fetch data
✗ should post data
 1 pass
 2 fail
Ran 3 tests in 1.44ms"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("bun")
        .write_stdin(bun_input)
        .assert()
        .success()
        // Should show detailed failure info
        .stdout(predicate::str::contains("[FAIL]"))
        .stdout(predicate::str::contains("1 passed, 2 failed"));
}

#[test]
fn test_parse_pytest_compact_success_with_skipped() {
    // Test that skipped tests are shown in compact success summary
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_slow SKIPPED
1 passed, 1 skipped in 0.50s"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show compact summary with skipped count
        .stdout(predicate::str::contains("PASS: 1 tests, 1 skipped"));
}

#[test]
fn test_parse_pytest_failure_with_error_message() {
    // Test that failure-focused summary shows error messages
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
1 passed, 1 failed in 1.23s
=== FAILURES ===
____ test_subtract ____

    def test_subtract():
>       assert 1 == 2
E       assert 1 == 2"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show failure-focused summary
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("1 passed, 1 failed"))
        .stdout(predicate::str::contains("failed (1):"))
        // Should show error message (first line)
        .stdout(predicate::str::contains("def test_subtract():"));
}

#[test]
fn test_parse_pytest_multiple_failures_with_error_messages() {
    // Test that failure-focused summary shows error messages for multiple failures
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract FAILED
tests/test_main.py::test_multiply FAILED
2 passed, 2 failed in 1.23s
=== FAILURES ===
____ test_subtract ____

    def test_subtract():
>       assert 1 == 2
E       assert 1 == 2
____ test_multiply ____

    def test_multiply():
>       assert 2 * 3 == 5
E       assert 6 == 5"#;
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("test")
        .arg("--runner")
        .arg("pytest")
        .write_stdin(pytest_input)
        .assert()
        .success()
        // Should show failure-focused summary
        .stdout(predicate::str::contains("FAIL:"))
        .stdout(predicate::str::contains("2 passed, 2 failed"))
        .stdout(predicate::str::contains("failed (2):"))
        // Should show both test names
        .stdout(predicate::str::contains("test_subtract"))
        .stdout(predicate::str::contains("test_multiply"));
}

// ============================================================
