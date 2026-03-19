use super::*;

// ============================================================
// Bun Test Parser Tests
// ============================================================

#[test]
fn test_parse_bun_test_empty() {
    let result = ParseHandler::parse_bun_test("").unwrap();
    assert!(result.is_empty);
    assert!(result.test_suites.is_empty());
}

#[test]
fn test_parse_bun_test_single_suite_passed() {
    let input = r#"test/package-json-lint.test.ts:
✓ test/package.json [0.88ms]
✓ test/js/third_party/grpc-js/package.json [0.18ms]

 4 pass
 0 fail
 4 expect() calls
Ran 4 tests in 1.44ms"#;
    let result = ParseHandler::parse_bun_test(input).unwrap();
    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert_eq!(result.test_suites[0].file, "test/package-json-lint.test.ts");
    assert!(result.test_suites[0].passed);
    assert_eq!(result.summary.tests_passed, 4);
    assert_eq!(result.summary.tests_failed, 0);
    assert_eq!(result.summary.expect_calls, Some(4));
    assert!(result.summary.duration.is_some());
}

#[test]
fn test_parse_bun_test_single_suite_failed() {
    let input = r#"test/api.test.ts:
✓ should pass [0.88ms]
✗ should fail

 1 pass
 1 fail
 2 expect() calls
Ran 2 tests in 1.44ms"#;
    let result = ParseHandler::parse_bun_test(input).unwrap();
    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.test_suites.len(), 1);
    assert!(!result.test_suites[0].passed);
    assert_eq!(result.summary.tests_passed, 1);
    assert_eq!(result.summary.tests_failed, 1);
}

#[test]
fn test_parse_bun_test_multiple_suites() {
    let input = r#"test/a.test.ts:
✓ test a [0.88ms]

test/b.test.ts:
✓ test b [0.18ms]

 2 pass
 0 fail
Ran 2 tests in 1.44ms"#;
    let result = ParseHandler::parse_bun_test(input).unwrap();
    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.test_suites.len(), 2);
    assert_eq!(result.summary.tests_passed, 2);
}

#[test]
fn test_parse_bun_test_non_tty_format() {
    let input = r#"test/package-json-lint.test.ts:
(pass) test/package.json [0.48ms]
(pass) test/js/third_party/grpc-js/package.json [0.10ms]
(fail) test/failing.test.ts
(skip) test/skipped.test.ts

 2 pass
 1 fail
 1 skipped
Ran 4 tests across 1 files. [0.66ms]"#;
    let result = ParseHandler::parse_bun_test(input).unwrap();
    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.summary.tests_passed, 2);
    assert_eq!(result.summary.tests_failed, 1);
    assert_eq!(result.summary.tests_skipped, 1);
    assert_eq!(result.summary.suites_total, 1);
}

#[test]
fn test_parse_bun_test_line() {
    // Test with checkmark
    let result =
        ParseHandler::parse_bun_test_line("✓ should work correctly [5.123ms]", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Passed);
    assert_eq!(result.test_name, "should work correctly");
    assert_eq!(result.duration, Some(0.005123));

    // Test with x mark (failure)
    let result = ParseHandler::parse_bun_test_line("✗ should fail", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Failed);
    assert_eq!(result.test_name, "should fail");

    // Test with × mark (failure alternative)
    let result = ParseHandler::parse_bun_test_line("× should also fail", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Failed);

    // Test non-TTY pass format
    let result = ParseHandler::parse_bun_test_line("(pass) should work [5.123ms]", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Passed);

    // Test non-TTY fail format
    let result = ParseHandler::parse_bun_test_line("(fail) should fail", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Failed);

    // Test non-TTY skip format
    let result = ParseHandler::parse_bun_test_line("(skip) skipped test", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Skipped);

    // Test non-TTY todo format
    let result = ParseHandler::parse_bun_test_line("(todo) todo test", &[]).unwrap();
    assert_eq!(result.status, BunTestStatus::Todo);
}

#[test]
fn test_split_bun_test_name_and_duration() {
    let (name, duration) = ParseHandler::split_bun_test_name_and_duration("test name [5.123ms]");
    assert_eq!(name, "test name");
    assert_eq!(duration, Some(0.005123));

    let (name, duration) = ParseHandler::split_bun_test_name_and_duration("test name [1.234s]");
    assert_eq!(name, "test name");
    assert_eq!(duration, Some(1.234));

    let (name, duration) =
        ParseHandler::split_bun_test_name_and_duration("test name without duration");
    assert_eq!(name, "test name without duration");
    assert_eq!(duration, None);
}

#[test]
fn test_parse_bun_duration() {
    assert_eq!(ParseHandler::parse_bun_duration("5.123ms"), Some(0.005123));
    assert_eq!(ParseHandler::parse_bun_duration("1.234s"), Some(1.234));
    assert_eq!(ParseHandler::parse_bun_duration("invalid"), None);
}

#[test]
fn test_parse_bun_summary_line() {
    let mut summary = BunTestSummary::default();

    ParseHandler::parse_bun_summary_line("4 pass", &mut summary);
    assert_eq!(summary.tests_passed, 4);

    ParseHandler::parse_bun_summary_line("2 fail", &mut summary);
    assert_eq!(summary.tests_failed, 2);

    ParseHandler::parse_bun_summary_line("10 expect() calls", &mut summary);
    assert_eq!(summary.expect_calls, Some(10));

    ParseHandler::parse_bun_summary_line("3 skipped", &mut summary);
    assert_eq!(summary.tests_skipped, 3);
}

#[test]
fn test_parse_bun_ran_line() {
    let mut summary = BunTestSummary::default();

    ParseHandler::parse_bun_ran_line("Ran 4 tests in 1.44ms", &mut summary);
    assert_eq!(summary.tests_total, 4);
    assert!(summary.duration.is_some());
    let duration = summary.duration.unwrap();
    assert!((duration - 0.00144).abs() < 1e-9);

    let mut summary2 = BunTestSummary::default();
    ParseHandler::parse_bun_ran_line("Ran 4 tests across 1 files. [0.66ms]", &mut summary2);
    assert_eq!(summary2.tests_total, 4);
    assert_eq!(summary2.suites_total, 1);
    assert!(summary2.duration.is_some());
    let duration2 = summary2.duration.unwrap();
    assert!((duration2 - 0.00066).abs() < 1e-9);
}

#[test]
fn test_format_bun_test_json() {
    let mut output = BunTestOutput::default();
    output.test_suites.push(BunTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![BunTest {
            name: "should pass".to_string(),
            test_name: "should pass".to_string(),
            ancestors: vec![],
            status: BunTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.summary.expect_calls = Some(1);
    output.success = true;
    output.is_empty = false;

    let json = ParseHandler::format_bun_test_json(&output);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["summary"]["tests_passed"], 1);
    assert_eq!(parsed["summary"]["expect_calls"], 1);
}

#[test]
fn test_format_bun_test_compact() {
    let mut output = BunTestOutput::default();
    output.test_suites.push(BunTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![BunTest {
            name: "should pass".to_string(),
            test_name: "should pass".to_string(),
            ancestors: vec![],
            status: BunTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.summary.duration = Some(0.01);
    output.success = true;
    output.is_empty = false;

    let compact = ParseHandler::format_bun_test_compact(&output);
    assert!(compact.contains("PASS"));
    assert!(compact.contains("1 suites"));
    assert!(compact.contains("1 tests"));
}

#[test]
fn test_format_bun_test_raw() {
    let mut output = BunTestOutput::default();
    output.test_suites.push(BunTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: None,
        tests: vec![
            BunTest {
                name: "passing test".to_string(),
                test_name: "passing test".to_string(),
                ancestors: vec![],
                status: BunTestStatus::Passed,
                duration: None,
                error_message: None,
            },
            BunTest {
                name: "failing test".to_string(),
                test_name: "failing test".to_string(),
                ancestors: vec![],
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            },
        ],
    });

    let raw = ParseHandler::format_bun_test_raw(&output);
    assert!(raw.contains("PASS test.js"));
    assert!(raw.contains("PASS passing test"));
    assert!(raw.contains("FAIL failing test"));
}

#[test]
fn test_format_bun_test_agent() {
    let mut output = BunTestOutput::default();
    output.test_suites.push(BunTestSuite {
        file: "test.js".to_string(),
        passed: true,
        duration: Some(0.01),
        tests: vec![BunTest {
            name: "should pass".to_string(),
            test_name: "should pass".to_string(),
            ancestors: vec![],
            status: BunTestStatus::Passed,
            duration: Some(0.005),
            error_message: None,
        }],
    });
    output.summary.tests_passed = 1;
    output.summary.tests_total = 1;
    output.summary.suites_passed = 1;
    output.summary.suites_total = 1;
    output.summary.expect_calls = Some(1);
    output.success = true;
    output.is_empty = false;

    let agent = ParseHandler::format_bun_test_agent(&output);
    assert!(agent.contains("# Test Results"));
    assert!(agent.contains("Status: SUCCESS"));
    assert!(agent.contains("## Summary"));
    assert!(agent.contains("Expect() calls: 1"));
}

#[test]
fn test_parse_bun_test_with_ancestors() {
    // Test that nested tests track ancestor names
    let result = ParseHandler::parse_bun_test_line(
        "✓ nested test [5.123ms]",
        &["describe block".to_string()],
    )
    .unwrap();
    assert_eq!(result.test_name, "nested test");
    assert_eq!(result.ancestors, vec!["describe block"]);
    assert_eq!(result.name, "describe block > nested test");
}
