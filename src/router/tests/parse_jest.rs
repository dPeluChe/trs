use super::*;

// ============================================================
// Jest Parser Tests
// ============================================================

#[test]
fn test_parse_jest_empty() {
    let result = ParseHandler::parse_jest("").unwrap();
    assert!(result.is_empty);
    assert!(result.test_suites.is_empty());
    assert_eq!(result.summary.tests_total, 0);
}

#[test]
fn test_parse_jest_single_suite_passed() {
    let input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
    let result = ParseHandler::parse_jest(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert_eq!(result.test_suites[0].file, "src/utils.test.js");
    assert!(result.test_suites[0].passed);
    assert_eq!(result.test_suites[0].tests.len(), 2);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_total, 2);
}

#[test]
fn test_parse_jest_single_suite_failed() {
    let input = r#"FAIL src/math.test.js
  ✕ should multiply numbers (3 ms)
  ✓ should divide numbers (1 ms)

Test Suites: 1 failed, 1 total
Tests:       1 passed, 1 failed, 2 total"#;
    let result = ParseHandler::parse_jest(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert_eq!(result.test_suites[0].file, "src/math.test.js");
    assert!(!result.test_suites[0].passed);
    assert_eq!(result.test_suites[0].tests.len(), 2);
    assert_eq!(result.summary.tests_passed, 1);
    assert_eq!(result.summary.tests_failed, 1);
    assert_eq!(result.summary.tests_total, 2);
}

#[test]
fn test_parse_jest_multiple_suites() {
    let input = r#"PASS src/utils.test.js
  ✓ test 1 (5 ms)

FAIL src/api.test.js
  ✕ test 2 (10 ms)
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       2 passed, 1 failed, 3 total"#;
    let result = ParseHandler::parse_jest(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.test_suites.len(), 2);
    assert_eq!(result.summary.suites_passed, 1);
    assert_eq!(result.summary.suites_failed, 1);
    assert_eq!(result.summary.suites_total, 2);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_failed, 1);
    assert_eq!(result.summary.tests_total, 3);
}

#[test]
fn test_parse_jest_test_with_skipped() {
    let input = r#"PASS src/test.js
  ✓ test 1 (5 ms)
  ○ skipped test 2
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 1 skipped, 3 total"#;
    let result = ParseHandler::parse_jest(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites[0].tests.len(), 3);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_skipped, 1);
}

#[test]
fn test_parse_jest_test_line() {
    let result =
        ParseHandler::parse_jest_test_line("  ✓ should work correctly (5 ms)").unwrap();
    assert_eq!(result.status, JestTestStatus::Passed);
    assert_eq!(result.test_name, "should work correctly");
    assert!(result.duration.is_some());

    let result = ParseHandler::parse_jest_test_line("  ✕ should fail").unwrap();
    assert_eq!(result.status, JestTestStatus::Failed);
    assert_eq!(result.test_name, "should fail");

    let result = ParseHandler::parse_jest_test_line("  ○ skipped test").unwrap();
    assert_eq!(result.status, JestTestStatus::Skipped);
}

#[test]
fn test_parse_jest_duration() {
    assert_eq!(ParseHandler::parse_jest_duration("5 ms"), Some(0.005));
    assert_eq!(ParseHandler::parse_jest_duration("1.23 s"), Some(1.23));
    assert_eq!(ParseHandler::parse_jest_duration("1000ms"), Some(1.0));
    assert_eq!(ParseHandler::parse_jest_duration("invalid"), None);
}

#[test]
fn test_parse_jest_summary() {
    let summary = ParseHandler::parse_jest_summary("Test Suites: 2 passed, 1 failed, 3 total");
    assert_eq!(summary.suites_passed, 2);
    assert_eq!(summary.suites_failed, 1);
    assert_eq!(summary.suites_total, 3);
}

#[test]
fn test_parse_jest_tests_summary() {
    let mut summary = JestSummary::default();
    ParseHandler::parse_jest_tests_summary(
        "Tests:       5 passed, 2 failed, 1 skipped, 8 total",
        &mut summary,
    );
    assert_eq!(summary.tests_passed, 5);
    assert_eq!(summary.tests_failed, 2);
    assert_eq!(summary.tests_skipped, 1);
    assert_eq!(summary.tests_total, 8);
}

#[test]
fn test_parse_jest_time_summary() {
    let mut summary = JestSummary::default();
    ParseHandler::parse_jest_time_summary("Time:        1.234 s", &mut summary);
    assert_eq!(summary.duration, Some(1.234));
}

#[test]
fn test_format_jest_json() {
    let mut output = JestOutput::default();
    output.test_suites.push(JestTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.1),
        tests: vec![JestTest {
            name: "test example".to_string(),
            test_name: "test example".to_string(),
            ancestors: vec![],
            status: JestTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.success = true;
    output.is_empty = false;

    let json = ParseHandler::format_jest_json(&output);
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"passed\":1"));
    assert!(json.contains("\"test.js\""));
}

#[test]
fn test_format_jest_compact() {
    let mut output = JestOutput::default();
    output.test_suites.push(JestTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.1),
        tests: vec![JestTest {
            name: "test example".to_string(),
            test_name: "test example".to_string(),
            ancestors: vec![],
            status: JestTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.success = true;
    output.is_empty = false;

    let compact = ParseHandler::format_jest_compact(&output);
    assert!(compact.contains("PASS:"));
    assert!(compact.contains("1 suites"));
    assert!(compact.contains("1 tests"));
}

#[test]
fn test_format_jest_raw() {
    let mut output = JestOutput::default();
    output.test_suites.push(JestTestSuite {
        file: "test.js".to_string(),
        passed: false,
        duration: None,
        tests: vec![
            JestTest {
                name: "passing test".to_string(),
                test_name: "passing test".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: None,
                error_message: None,
            },
            JestTest {
                name: "failing test".to_string(),
                test_name: "failing test".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Failed,
                duration: None,
                error_message: None,
            },
        ],
    });
    output.is_empty = false;

    let raw = ParseHandler::format_jest_raw(&output);
    assert!(raw.contains("FAIL test.js"));
    assert!(raw.contains("PASS passing test"));
    assert!(raw.contains("FAIL failing test"));
}

#[test]
fn test_format_jest_agent() {
    let mut output = JestOutput::default();
    output.test_suites.push(JestTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.1),
        tests: vec![JestTest {
            name: "test example".to_string(),
            test_name: "test example".to_string(),
            ancestors: vec![],
            status: JestTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.success = true;
    output.is_empty = false;

    let agent = ParseHandler::format_jest_agent(&output);
    assert!(agent.contains("# Test Results"));
    assert!(agent.contains("Status: SUCCESS"));
    assert!(agent.contains("## Summary"));
}

#[test]
fn test_parse_jest_with_ancestors() {
    // Test with regular > separator
    let result =
        ParseHandler::parse_jest_test_line("✓ describe block > test name (5 ms)").unwrap();
    assert_eq!(result.test_name, "test name");
    assert_eq!(result.ancestors, vec!["describe block"]);

    // Test with fancy › separator (Unicode)
    let result =
        ParseHandler::parse_jest_test_line("✓ describe block › test name (5 ms)").unwrap();
    assert_eq!(result.test_name, "test name");
    assert_eq!(result.ancestors, vec!["describe block"]);
}
