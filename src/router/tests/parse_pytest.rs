use super::*;

// ============================================================
// Parse Handler Tests (git-status, test runners)
// ============================================================

#[test]
fn test_parse_handler_git_status() {
    let handler = ParseHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![OutputFormat::Json],
    };
    // Use temp file instead of stdin to avoid blocking
    let tmp = std::env::temp_dir().join("trs_test_git_status.tmp");
    std::fs::write(&tmp, "On branch main\nnothing to commit, working tree clean\n").unwrap();
    let input = ParseCommands::GitStatus {
        file: Some(tmp.clone()),
        count: None,
    };
    let result = handler.execute(&input, &ctx);
    let _ = std::fs::remove_file(&tmp);
    assert!(result.is_ok());
}

#[test]
fn test_parse_handler_test() {
    let handler = ParseHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    // Use temp file instead of stdin to avoid blocking
    let tmp = std::env::temp_dir().join("trs_test_pytest.tmp");
    std::fs::write(&tmp, "test_example.py::test_one PASSED\n").unwrap();
    let input = ParseCommands::Test {
        runner: Some(crate::TestRunner::Pytest),
        file: Some(tmp.clone()),
    };
    let result = handler.execute(&input, &ctx);
    let _ = std::fs::remove_file(&tmp);
    assert!(result.is_ok());
}

// ============================================================
// Pytest Parser Tests
// ============================================================

#[test]
fn test_parse_pytest_empty() {
    let result = ParseHandler::parse_pytest("").unwrap();
    assert!(result.is_empty);
    assert!(result.tests.is_empty());
    assert_eq!(result.summary.total, 0);
}

#[test]
fn test_parse_pytest_single_passed() {
    let input = r#"tests/test_main.py::test_add PASSED
1 passed in 0.01s"#;
    let result = ParseHandler::parse_pytest(input).unwrap();

    assert!(!result.is_empty);
    assert!(result.success);
    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.summary.passed, 1);
    assert_eq!(result.summary.failed, 0);
    assert_eq!(result.summary.total, 1);
}

#[test]
fn test_parse_pytest_single_failed() {
    let input = r#"tests/test_main.py::test_fail FAILED
____ test_fail ____
def test_fail():
    assert False
=== FAILURES ===
1 failed in 0.01s"#;
    let result = ParseHandler::parse_pytest(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.tests.len(), 1);
    assert_eq!(result.summary.failed, 1);
    assert_eq!(result.summary.passed, 0);
}

#[test]
fn test_parse_pytest_mixed_results() {
    let input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED
tests/test_main.py::test_multiply SKIPPED
tests/test_main.py::test_fail FAILED
2 passed, 1 failed, 1 skipped in 0.05s"#;
    let result = ParseHandler::parse_pytest(input).unwrap();

    assert!(!result.is_empty);
    assert!(!result.success);
    assert_eq!(result.tests.len(), 4);
    assert_eq!(result.summary.passed, 2);
    assert_eq!(result.summary.failed, 1);
    assert_eq!(result.summary.skipped, 1);
    assert_eq!(result.summary.total, 4);
}

#[test]
fn test_parse_pytest_with_xfail() {
    let input = r#"tests/test_main.py::test_add PASSED
tests/test_main.py::test_expected_fail XFAIL
2 passed, 1 xfailed in 0.01s"#;
    let result = ParseHandler::parse_pytest(input).unwrap();

    assert!(result.success);
    assert_eq!(result.summary.xfailed, 1);
}

#[test]
fn test_parse_pytest_summary_line() {
    let summary = ParseHandler::parse_pytest_summary("2 passed in 0.01s");
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 0);
    assert!(summary.duration.is_some());

    let summary = ParseHandler::parse_pytest_summary("2 passed, 1 failed in 0.05s");
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);

    let summary = ParseHandler::parse_pytest_summary("3 passed, 1 failed, 2 skipped in 1.23s");
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.skipped, 2);
    assert_eq!(summary.duration, Some(1.23));
}

#[test]
fn test_is_pytest_summary_line() {
    assert!(ParseHandler::is_pytest_summary_line("2 passed in 0.01s"));
    assert!(ParseHandler::is_pytest_summary_line(
        "2 passed, 1 failed in 0.05s"
    ));
    assert!(ParseHandler::is_pytest_summary_line(
        "=== 2 passed in 0.01s ==="
    ));
    assert!(ParseHandler::is_pytest_summary_line(
        "1 failed, 2 passed in 0.05s"
    ));
    assert!(!ParseHandler::is_pytest_summary_line(
        "test_file.py::test_name PASSED"
    ));
    assert!(!ParseHandler::is_pytest_summary_line("PASSED"));
}

#[test]
fn test_parse_pytest_test_line() {
    let result =
        ParseHandler::parse_pytest_test_line("tests/test_main.py::test_add PASSED").unwrap();
    assert_eq!(result.name, "tests/test_main.py::test_add");
    assert_eq!(result.status, TestStatus::Passed);
    assert_eq!(result.file, Some("tests/test_main.py".to_string()));

    let result =
        ParseHandler::parse_pytest_test_line("tests/test_main.py::test_fail FAILED").unwrap();
    assert_eq!(result.status, TestStatus::Failed);

    let result =
        ParseHandler::parse_pytest_test_line("tests/test_main.py::test_skip SKIPPED").unwrap();
    assert_eq!(result.status, TestStatus::Skipped);
}

#[test]
fn test_format_pytest_json() {
    let mut output = PytestOutput::default();
    output.tests.push(TestResult {
        name: "test_example".to_string(),
        status: TestStatus::Passed,
        duration: None,
        file: None,
        line: None,
        error_message: None,
    });
    output.summary.passed = 1;
    output.summary.total = 1;
    output.success = true;
    output.is_empty = false;

    let json = ParseHandler::format_pytest_json(&output);
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"passed\":1"));
    assert!(json.contains("\"total\":1"));
}

#[test]
fn test_format_pytest_compact() {
    let mut output = PytestOutput::default();
    output.tests.push(TestResult {
        name: "test_example".to_string(),
        status: TestStatus::Passed,
        duration: None,
        file: None,
        line: None,
        error_message: None,
    });
    output.summary.passed = 1;
    output.summary.total = 1;
    output.success = true;
    output.is_empty = false;

    let compact = ParseHandler::format_pytest_compact(&output);
    assert!(compact.contains("PASS:"));
    // Compact success summary shows "X tests" not "X passed"
    assert!(compact.contains("1 tests"));
}

#[test]
fn test_format_pytest_raw() {
    let mut output = PytestOutput::default();
    output.tests.push(TestResult {
        name: "test_example".to_string(),
        status: TestStatus::Passed,
        duration: None,
        file: None,
        line: None,
        error_message: None,
    });
    output.tests.push(TestResult {
        name: "test_fail".to_string(),
        status: TestStatus::Failed,
        duration: None,
        file: None,
        line: None,
        error_message: None,
    });

    let raw = ParseHandler::format_pytest_raw(&output);
    assert!(raw.contains("PASS test_example"));
    assert!(raw.contains("FAIL test_fail"));
}

#[test]
fn test_format_pytest_agent() {
    let mut output = PytestOutput::default();
    output.tests.push(TestResult {
        name: "test_example".to_string(),
        status: TestStatus::Passed,
        duration: None,
        file: None,
        line: None,
        error_message: None,
    });
    output.summary.passed = 1;
    output.summary.total = 1;
    output.success = true;
    output.is_empty = false;

    let agent = ParseHandler::format_pytest_agent(&output);
    assert!(agent.contains("# Test Results"));
    assert!(agent.contains("Status: SUCCESS"));
    assert!(agent.contains("## Summary"));
}

#[test]
fn test_parse_pytest_with_header_info() {
    let input = r#"============================= test session starts ==============================
platform darwin -- Python 3.12.0, pytest-8.0.0, pluggy-1.4.0
rootdir: /Users/user/project
collected 2 items

tests/test_main.py::test_add PASSED
tests/test_main.py::test_subtract PASSED

2 passed in 0.01s"#;
    let result = ParseHandler::parse_pytest(input).unwrap();

    assert!(result.success);
    assert_eq!(result.python_version, Some("3.12.0".to_string()));
    assert_eq!(result.pytest_version, Some("8.0.0".to_string()));
    assert_eq!(result.rootdir, Some("/Users/user/project".to_string()));
}

// ============================================================
// Router Integration Tests
// ============================================================

#[test]
fn test_router_run_command_success() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let command = Commands::Run {
        command: "echo".to_string(),
        args: vec!["test".to_string()],
        capture_stdout: Some(true),
        capture_stderr: Some(true),
        capture_exit_code: Some(true),
        capture_duration: Some(true),
    };

    let result = router.route(&command, &ctx);
    // echo should succeed
    assert!(result.is_ok());
}

#[test]
fn test_router_run_command_failure() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let command = Commands::Run {
        command: "false".to_string(),
        args: vec![],
        capture_stdout: Some(true),
        capture_stderr: Some(true),
        capture_exit_code: Some(true),
        capture_duration: Some(true),
    };

    let result = router.route(&command, &ctx);
    // false exits with 1
    assert!(result.is_err());
}

#[test]
fn test_router_default() {
    let router = Router::default();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let command = Commands::Search {
        path: std::path::PathBuf::from("."),
        query: "test".to_string(),
        extension: None,
        ignore_case: false,
        context: None,
        limit: None,
    };

    let result = router.route(&command, &ctx);
    // Search is now implemented, so it should succeed
    assert!(result.is_ok());
}
