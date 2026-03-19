use assert_cmd::Command;
use predicates::prelude::*;

// Error/Warning Level Detection Tests
// ============================================================

#[test]
fn test_parse_logs_detects_error_levels() {
    // Test that ERROR level is properly detected
    let log_input = "[ERROR] Database connection failed\n[INFO] Retrying...";
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
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["info"], 1);
}

#[test]
fn test_parse_logs_detects_warning_levels() {
    // Test that WARNING level is properly detected
    let log_input = "[WARN] Cache miss\n[WARNING] Slow response time";
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
    assert_eq!(json["counts"]["warning"], 2);
}

#[test]
fn test_parse_logs_detects_failed_keyword() {
    // Test that "FAILED" keyword is detected as error
    let log_input = "Test case 1 PASSED\nTest case 2 FAILED\nTest case 3 PASSED";
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
    assert_eq!(json["counts"]["error"], 1);
    assert_eq!(json["counts"]["unknown"], 2);
}

#[test]
fn test_parse_logs_detects_exception() {
    // Test that "Exception" keyword is detected as error
    let log_input =
        "Starting application...\nException: NullPointerException\nError at com.example.Main.main";
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
    assert_eq!(json["counts"]["error"], 2);
}

#[test]
fn test_parse_logs_detects_fatal_levels() {
    // Test that FATAL level is properly detected
    let log_input = "[FATAL] System out of memory\n[CRITICAL] Disk full";
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
    assert_eq!(json["counts"]["fatal"], 2);
}

#[test]
fn test_parse_logs_detects_panic_crash() {
    // Test that PANIC and CRASH are detected as fatal
    let log_input = "PANIC: unrecoverable error\nApplication crashed";
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
    assert_eq!(json["counts"]["fatal"], 2);
}

#[test]
fn test_parse_logs_detects_deprecated() {
    // Test that "deprecated" is detected as warning
    let log_input = "Warning: This method is deprecated\nPlease use newMethod instead";
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
    assert_eq!(json["counts"]["warning"], 1);
}

#[test]
fn test_parse_logs_detects_connection_errors() {
    // Test that connection errors are detected
    let log_input = "Connection refused\nConnection error: timeout\nAccess denied";
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
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_detects_stack_trace() {
    // Test that stack trace and backtrace are detected as errors
    let log_input = "STACK TRACE:\nBACKTRACE:\nException occurred";
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
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_compact_shows_level_indicators() {
    // Test that compact output shows level indicators [E], [W], [I], etc.
    let log_input = "[ERROR] Something failed\n[WARN] Be careful\n[INFO] All good";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should show level indicators
    assert!(stdout.contains("[E]"));
    assert!(stdout.contains("[W]"));
    assert!(stdout.contains("[I]"));
}

#[test]
fn test_parse_logs_negation_not_detected_as_error() {
    // Test that "no errors" is NOT detected as error
    let log_input = "All tests passed\nNo errors found\nCompleted with 0 errors";
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

    // All lines should be unknown, not error
    assert_eq!(json["counts"]["error"], 0);
    assert_eq!(json["counts"]["unknown"], 3);
}

#[test]
fn test_parse_logs_various_formats() {
    // Test various log level formats: brackets, colon, pipes
    let log_input = "[ERROR] Bracket format\nERROR: Colon format\n|ERROR| Pipe format";
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
    assert_eq!(json["counts"]["error"], 3);
}

#[test]
fn test_parse_logs_slow_query_warning() {
    // Test that slow query/request are detected as warnings
    let log_input = "SLOW QUERY detected: 5.2s\nSLOW REQUEST: 3.1s";
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
    assert_eq!(json["counts"]["warning"], 2);
}

#[test]
fn test_parse_logs_notice_level() {
    // Test that NOTICE is detected as info
    let log_input = "[NOTICE] System maintenance scheduled\nNOTICE: Server restart at midnight";
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
    assert_eq!(json["counts"]["info"], 2);
}

#[test]
fn test_parse_logs_detects_recent_critical() {
    // Test that recent critical lines are tracked
    let log_input = "[INFO] Starting\n[ERROR] First error\n[WARN] Warning\n[FATAL] Fatal error\n[ERROR] Second error";
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

    // Should have recent_critical array with 3 entries (2 errors + 1 fatal)
    assert!(json["recent_critical"].is_array());
    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 3);

    // Check counts - error count should be 2, fatal should be 1
    assert_eq!(json["counts"]["error"], 2);
    assert_eq!(json["counts"]["fatal"], 1);
}

#[test]
fn test_parse_logs_recent_critical_only_errors_and_fatals() {
    // Test that only ERROR and FATAL are in recent_critical
    let log_input = "[INFO] Info\n[WARN] Warning\n[DEBUG] Debug\n[ERROR] Error\n[FATAL] Fatal";
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

    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0]["level"], "error");
    assert_eq!(recent[1]["level"], "fatal");
}

#[test]
fn test_parse_logs_compact_shows_recent_critical_section() {
    // Test that compact output shows recent critical section
    let log_input = "[INFO] Starting\n[ERROR] Something failed\n[FATAL] System crash\n[INFO] Done";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("recent critical"))
        .stdout(predicate::str::contains("[E]"))
        .stdout(predicate::str::contains("[F]"))
        .stdout(predicate::str::contains("Something failed"))
        .stdout(predicate::str::contains("System crash"));
}

#[test]
fn test_parse_logs_recent_critical_limited() {
    // Create input with more than 10 errors to test limiting
    let mut log_input = String::new();
    for i in 1..=15 {
        log_input.push_str(&format!("[ERROR] Error message {}\n", i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    let output = cmd
        .arg("--json")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input.as_str())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should be limited to 10
    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 10);
    // Total critical is the sum of error and fatal in counts
    assert_eq!(json["counts"]["error"], 15);
}

#[test]
fn test_parse_logs_compact_shows_truncated_count() {
    // Create input with more than 10 errors
    let mut log_input = String::new();
    for i in 1..=15 {
        log_input.push_str(&format!("[ERROR] Error {}\n", i));
    }
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 of 15"));
}

#[test]
fn test_parse_logs_no_recent_critical_when_none() {
    // Test that no recent critical section appears when there are no errors
    let log_input = "[INFO] Starting\n[DEBUG] Debug\n[WARN] Warning";
    let mut cmd = Command::cargo_bin("trs").unwrap();
    cmd.arg("--compact")
        .arg("parse")
        .arg("logs")
        .write_stdin(log_input)
        .assert()
        .success()
        .stdout(predicate::function(|x: &str| {
            !x.contains("recent critical")
        }));
}

// ============================================================
