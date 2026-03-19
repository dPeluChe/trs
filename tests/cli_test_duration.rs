use assert_cmd::Command;

// Test Runner Duration Extraction Tests
// ============================================================

#[test]
fn test_parse_pytest_duration_extraction() {
    // Test that pytest parser correctly extracts execution duration
    let pytest_input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
2 passed in 1.23s"#;
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
    assert_eq!(json["summary"]["passed"], 2);
    // Verify duration is extracted and is approximately 1.23 seconds
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.23).abs() < 0.01,
        "Expected duration ~1.23s, got {}",
        duration
    );
}

#[test]
fn test_parse_pytest_duration_in_milliseconds() {
    // Test pytest duration extraction with milliseconds format
    let pytest_input = r#"tests/test_main.py::test_quick PASSED
1 passed in 0.05s"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.05).abs() < 0.01,
        "Expected duration ~0.05s, got {}",
        duration
    );
}

#[test]
fn test_parse_jest_duration_extraction() {
    // Test that Jest parser correctly extracts execution duration from time summary
    let jest_input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total
Time:        1.5 s"#;
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
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.5).abs() < 0.1,
        "Expected duration ~1.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_jest_duration_in_ms() {
    // Test Jest duration extraction with milliseconds format
    let jest_input = r#"PASS src/utils.test.js
  ✓ test (1 ms)

Test Suites: 1 passed, 1 total
Tests:       1 passed, 1 total
Time:        500 ms"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 500 ms = 0.5 s
    assert!(
        (duration - 0.5).abs() < 0.1,
        "Expected duration ~0.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_vitest_duration_extraction() {
    // Test that Vitest parser correctly extracts execution duration
    let vitest_input = r#" ✓ test/example.test.ts (5 tests) 306ms

 Test Files  1 passed (1)
      Tests  5 passed (5)
   Start at  11:01:36
   Duration  2.50s"#;
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
    // Verify duration is extracted
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 2.50).abs() < 0.1,
        "Expected duration ~2.50s, got {}",
        duration
    );
}

#[test]
fn test_parse_vitest_duration_in_ms() {
    // Test Vitest duration extraction with milliseconds format
    let vitest_input = r#" ✓ test/quick.test.ts (1 test) 50ms

 Test Files  1 passed (1)
      Tests  1 passed (1)
   Duration  150ms"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 150ms = 0.15s
    assert!(
        (duration - 0.15).abs() < 0.05,
        "Expected duration ~0.15s, got {}",
        duration
    );
}

#[test]
fn test_parse_npm_test_duration_extraction() {
    // Test that npm test parser correctly extracts execution duration
    let npm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 25.5ms"#;
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
    // Verify duration is extracted (25.5ms = 0.0255s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.0255).abs() < 0.01,
        "Expected duration ~0.0255s, got {}",
        duration
    );
}

#[test]
fn test_parse_npm_test_duration_in_seconds() {
    // Test npm test duration extraction with seconds format
    let npm_input = r#"▶ test/slow.test.js
  ✔ slow test (1000.123ms)
▶ test/slow.test.js (1.5s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 2.5s"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 2.5).abs() < 0.1,
        "Expected duration ~2.5s, got {}",
        duration
    );
}

#[test]
fn test_parse_pnpm_test_duration_extraction() {
    // Test that pnpm test parser correctly extracts execution duration
    let pnpm_input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 30.25ms"#;
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
    // Verify duration is extracted (30.25ms = 0.03025s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.03025).abs() < 0.01,
        "Expected duration ~0.03025s, got {}",
        duration
    );
}

#[test]
fn test_parse_pnpm_test_duration_in_seconds() {
    // Test pnpm test duration extraction with seconds format
    let pnpm_input = r#"▶ test/integration.test.js
  ✔ integration test (500ms)
▶ test/integration.test.js (0.75s)

ℹ tests 1 passed (1)
ℹ test files 1 passed (1)
ℹ duration 1.25s"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 1.25).abs() < 0.1,
        "Expected duration ~1.25s, got {}",
        duration
    );
}

#[test]
fn test_parse_bun_test_duration_extraction() {
    // Test that Bun test parser correctly extracts execution duration
    let bun_input = r#"test/example.test.ts:
✓ test case [0.05s]

 1 pass
 0 fail
 1 expect() calls
Ran 1 tests in 150ms"#;
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
    // Verify duration is extracted (150ms = 0.15s)
    let duration = json["summary"]["duration"].as_f64().unwrap();
    assert!(
        (duration - 0.15).abs() < 0.05,
        "Expected duration ~0.15s, got {}",
        duration
    );
}

#[test]
fn test_parse_bun_test_duration_in_ms() {
    // Test Bun test duration extraction with milliseconds format
    let bun_input = r#"test/quick.test.ts:
✓ quick test [5ms]

 1 pass
 0 fail
Ran 1 tests in 50ms"#;
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
    let duration = json["summary"]["duration"].as_f64().unwrap();
    // 50ms = 0.05s
    assert!(
        (duration - 0.05).abs() < 0.02,
        "Expected duration ~0.05s, got {}",
        duration
    );
}

// ============================================================
