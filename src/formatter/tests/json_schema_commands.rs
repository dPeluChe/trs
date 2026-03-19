use super::*;

// ============================================================
// JsonFormatter Schema Formatting Tests — Commands & Process
// ============================================================

#[test]
fn test_json_format_test_output_empty() {
    use crate::schema::{TestOutputSchema, TestRunnerType};
    let test = TestOutputSchema::new(TestRunnerType::Pytest);
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "test_output");
}

#[test]
fn test_json_format_test_output_passing() {
    use crate::schema::{TestOutputSchema, TestRunnerType, TestSummary};
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = true;
    test.summary = TestSummary {
        total: 10,
        passed: 10,
        failed: 0,
        skipped: 0,
        xfailed: 0,
        xpassed: 0,
        errors: 0,
        todo: 0,
        suites_passed: 1,
        suites_failed: 0,
        suites_total: 1,
        duration_ms: Some(500),
    };
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["success"], true);
    assert_eq!(json["summary"]["passed"], 10);
    assert_eq!(json["summary"]["failed"], 0);
    assert_eq!(json["summary"]["total"], 10);
    assert_eq!(json["summary"]["duration_ms"], 500);
}

#[test]
fn test_json_format_test_output_failing() {
    use crate::schema::{
        TestOutputSchema, TestResult, TestRunnerType, TestStatus, TestSuite, TestSummary,
    };
    let mut test = TestOutputSchema::new(TestRunnerType::Pytest);
    test.is_empty = false;
    test.success = false;
    test.summary = TestSummary {
        total: 10,
        passed: 8,
        failed: 2,
        skipped: 0,
        xfailed: 0,
        xpassed: 0,
        errors: 0,
        todo: 0,
        suites_passed: 0,
        suites_failed: 1,
        suites_total: 1,
        duration_ms: Some(500),
    };
    let mut suite = TestSuite::new("tests/test_main.py");
    suite.passed = false;
    suite
        .tests
        .push(TestResult::new("test_one", TestStatus::Failed));
    suite
        .tests
        .push(TestResult::new("test_two", TestStatus::Passed));
    test.test_suites.push(suite);
    let output = JsonFormatter::format_test_output(&test);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["summary"]["passed"], 8);
    assert_eq!(json["summary"]["failed"], 2);
    assert!(json["test_suites"].is_array());
    assert_eq!(json["test_suites"][0]["file"], "tests/test_main.py");
    assert_eq!(json["test_suites"][0]["passed"], false);
}

#[test]
fn test_json_format_logs_empty() {
    use crate::schema::LogsOutputSchema;
    let logs = LogsOutputSchema::new();
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], true);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "logs_output");
}

#[test]
fn test_json_format_logs_with_entries() {
    use crate::schema::{LogCounts, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts {
        total_lines: 10,
        debug: 2,
        info: 5,
        warning: 2,
        error: 1,
        fatal: 0,
        unknown: 0,
    };
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["counts"]["total_lines"], 10);
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["warning"], 2);
    assert_eq!(json["counts"]["info"], 5);
    assert_eq!(json["counts"]["debug"], 2);
}

#[test]
fn test_json_format_logs_with_critical() {
    use crate::schema::{LogCounts, LogEntry, LogLevel, LogsOutputSchema};
    let mut logs = LogsOutputSchema::new();
    logs.is_empty = false;
    logs.counts = LogCounts {
        total_lines: 3,
        debug: 0,
        info: 1,
        warning: 0,
        error: 2,
        fatal: 0,
        unknown: 0,
    };
    let mut entry = LogEntry::new("[ERROR] Something failed", 2);
    entry.level = LogLevel::Error;
    entry.message = "Something failed".to_string();
    logs.recent_critical.push(entry);
    let output = JsonFormatter::format_logs(&logs);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["recent_critical"].is_array());
    assert_eq!(json["recent_critical"][0]["message"], "Something failed");
    assert_eq!(json["recent_critical"][0]["level"], "error");
}

#[test]
fn test_json_format_repository_state_not_git() {
    use crate::schema::RepositoryStateSchema;
    let mut state = RepositoryStateSchema::new();
    state.is_git_repo = false;
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_git_repo"], false);
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "repository_state");
}

#[test]
fn test_json_format_repository_state_clean() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("main".to_string());
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert_eq!(json["is_detached"], false);
}

#[test]
fn test_json_format_repository_state_dirty() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("feature".to_string());
    state.is_clean = false;
    state.is_detached = false;
    state.counts = GitStatusCounts {
        staged: 1,
        unstaged: 2,
        untracked: 3,
        unmerged: 0,
    };
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["counts"]["staged"], 1);
    assert_eq!(json["counts"]["unstaged"], 2);
    assert_eq!(json["counts"]["untracked"], 3);
}

#[test]
fn test_json_format_repository_state_detached() {
    use crate::schema::{GitStatusCounts, RepositoryStateSchema};
    let mut state = RepositoryStateSchema::new();
    state.branch = Some("abc123".to_string());
    state.is_detached = true;
    state.is_clean = true;
    state.counts = GitStatusCounts::default();
    let output = JsonFormatter::format_repository_state(&state);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "abc123");
    assert_eq!(json["is_detached"], true);
}

#[test]
fn test_json_format_process_success() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("echo");
    proc.stdout = "hello\n".to_string();
    proc.success = true;
    let output = JsonFormatter::format_process(&proc);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["stdout"], "hello\n");
    assert_eq!(json["command"], "echo");
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "process_output");
}

#[test]
fn test_json_format_process_failure() {
    use crate::schema::ProcessOutputSchema;
    let mut proc = ProcessOutputSchema::new("false");
    proc.exit_code = Some(1);
    proc.success = false;
    proc.stderr = "error message\n".to_string();
    let output = JsonFormatter::format_process(&proc);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["exit_code"], 1);
    assert_eq!(json["stderr"], "error message\n");
}

#[test]
fn test_json_format_error_schema() {
    use crate::schema::ErrorSchema;
    let error = ErrorSchema::new("Something went wrong");
    let output = JsonFormatter::format_error_schema(&error);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["message"], "Something went wrong");
    assert!(json["schema"]["version"].is_string());
    assert_eq!(json["schema"]["type"], "error");
}

#[test]
fn test_json_format_error_schema_with_code() {
    use crate::schema::ErrorSchema;
    let mut error = ErrorSchema::new("Command failed");
    error.exit_code = Some(1);
    let output = JsonFormatter::format_error_schema(&error);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["message"], "Command failed");
    assert_eq!(json["exit_code"], 1);
}
