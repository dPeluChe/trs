use super::*;

// ============================================================
// PNPM Test Parser Tests
// ============================================================

#[test]
fn test_parse_pnpm_test_empty() {
    let result = ParseHandler::parse_pnpm_test("").unwrap();
    assert!(result.is_empty);
    assert!(result.test_suites.is_empty());
    assert_eq!(result.summary.tests_total, 0);
}

#[test]
fn test_parse_pnpm_test_single_suite_passed() {
    let input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let result = ParseHandler::parse_pnpm_test(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert_eq!(result.test_suites[0].file, "test/utils.test.js");
    assert!(result.test_suites[0].passed);
    assert_eq!(result.test_suites[0].tests.len(), 2);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.suites_passed, 1);
}

#[test]
fn test_parse_pnpm_test_single_suite_failed() {
    let input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError [ERR_ASSERTION]: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
    let result = ParseHandler::parse_pnpm_test(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert!(!result.test_suites[0].passed);
    assert_eq!(result.test_suites[0].tests.len(), 2);
    assert_eq!(result.summary.tests_passed, 1);
    assert_eq!(result.summary.tests_failed, 1);
}

#[test]
fn test_parse_pnpm_test_multiple_suites() {
    let input = r#"▶ test/utils.test.js
  ✔ test 1 (5.123ms)
▶ test/utils.test.js (7.234ms)

▶ test/math.test.js
  ✖ test 2
▶ test/math.test.js (3.456ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 passed 1 failed (2)
ℹ duration 15ms"#;
    let result = ParseHandler::parse_pnpm_test(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.test_suites.len(), 2);
    assert!(result.test_suites[0].passed);
    assert!(!result.test_suites[1].passed);
}

#[test]
fn test_parse_pnpm_test_with_skipped() {
    let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let result = ParseHandler::parse_pnpm_test(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites[0].tests.len(), 3);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_skipped, 1);
}

#[test]
fn test_parse_pnpm_test_with_todo() {
    let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # TODO
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 todo (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
    let result = ParseHandler::parse_pnpm_test(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites[0].tests.len(), 3);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_todo, 1);
}

#[test]
fn test_parse_pnpm_test_line() {
    let result =
        ParseHandler::parse_pnpm_test_line("✔ should work correctly (5.123ms)", &[]).unwrap();
    assert_eq!(result.status, PnpmTestStatus::Passed);
    assert_eq!(result.test_name, "should work correctly");
    assert!(result.duration.is_some());

    let result = ParseHandler::parse_pnpm_test_line("✖ should fail", &[]).unwrap();
    assert_eq!(result.status, PnpmTestStatus::Failed);
    assert_eq!(result.test_name, "should fail");

    let result = ParseHandler::parse_pnpm_test_line("ℹ skipped test # SKIP", &[]).unwrap();
    assert_eq!(result.status, PnpmTestStatus::Skipped);
    assert_eq!(result.test_name, "skipped test");

    let result = ParseHandler::parse_pnpm_test_line("ℹ todo test # TODO", &[]).unwrap();
    assert_eq!(result.status, PnpmTestStatus::Todo);
    assert_eq!(result.test_name, "todo test");
}

#[test]
fn test_parse_pnpm_duration() {
    assert_eq!(ParseHandler::parse_pnpm_duration("5.123ms"), Some(0.005123));
    assert_eq!(ParseHandler::parse_pnpm_duration("1.234s"), Some(1.234));
    assert_eq!(ParseHandler::parse_pnpm_duration("1000ms"), Some(1.0));
    assert_eq!(ParseHandler::parse_pnpm_duration("invalid"), None);
}

#[test]
fn test_split_pnpm_test_name_and_duration() {
    let (name, duration) = ParseHandler::split_pnpm_test_name_and_duration("test name (5.123ms)");
    assert_eq!(name, "test name");
    assert_eq!(duration, Some(0.005123));

    let (name, duration) = ParseHandler::split_pnpm_test_name_and_duration("test name (1.234s)");
    assert_eq!(name, "test name");
    assert_eq!(duration, Some(1.234));

    let (name, duration) =
        ParseHandler::split_pnpm_test_name_and_duration("test name without duration");
    assert_eq!(name, "test name without duration");
    assert!(duration.is_none());
}

#[test]
fn test_format_pnpm_test_json() {
    let mut output = PnpmTestOutput::default();
    output.test_suites.push(PnpmTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![PnpmTest {
            name: "test name".to_string(),
            test_name: "test name".to_string(),
            ancestors: vec![],
            status: PnpmTestStatus::Passed,
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

    let json = ParseHandler::format_pnpm_test_json(&output);
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"tests_passed\":1"));
    assert!(json.contains("\"test.js\""));
}

#[test]
fn test_format_pnpm_test_compact() {
    let mut output = PnpmTestOutput::default();
    output.test_suites.push(PnpmTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![PnpmTest {
            name: "test name".to_string(),
            test_name: "test name".to_string(),
            ancestors: vec![],
            status: PnpmTestStatus::Passed,
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

    let compact = ParseHandler::format_pnpm_test_compact(&output);
    assert!(compact.contains("PASS:"));
    assert!(compact.contains("1 suites"));
    assert!(compact.contains("1 tests"));
}

#[test]
fn test_format_pnpm_test_raw() {
    let mut output = PnpmTestOutput::default();
    output.test_suites.push(PnpmTestSuite {
        file: "test.js".to_string(),
        passed: false,
        duration: None,
        tests: vec![
            PnpmTest {
                name: "passing test".to_string(),
                test_name: "passing test".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: None,
                error_message: None,
            },
            PnpmTest {
                name: "failing test".to_string(),
                test_name: "failing test".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Failed,
                duration: None,
                error_message: Some("Error message".to_string()),
            },
        ],
    });
    output.is_empty = false;

    let raw = ParseHandler::format_pnpm_test_raw(&output);
    assert!(raw.contains("FAIL test.js"));
    assert!(raw.contains("PASS passing test"));
    assert!(raw.contains("FAIL failing test"));
}

#[test]
fn test_format_pnpm_test_agent() {
    let mut output = PnpmTestOutput::default();
    output.test_suites.push(PnpmTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![PnpmTest {
            name: "test name".to_string(),
            test_name: "test name".to_string(),
            ancestors: vec![],
            status: PnpmTestStatus::Passed,
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

    let agent = ParseHandler::format_pnpm_test_agent(&output);
    assert!(agent.contains("# Test Results"));
    assert!(agent.contains("Status: SUCCESS"));
    assert!(agent.contains("## Summary"));
}

#[test]
fn test_parse_pnpm_test_with_ancestors() {
    // Test that nested tests track ancestor names
    let result = ParseHandler::parse_pnpm_test_line(
        "✔ nested test (5.123ms)",
        &["describe block".to_string()],
    )
    .unwrap();
    assert_eq!(result.test_name, "nested test");
    assert_eq!(result.ancestors, vec!["describe block"]);
    assert_eq!(result.name, "describe block > nested test");
}
