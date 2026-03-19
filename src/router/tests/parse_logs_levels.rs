use super::*;

// ============================================================
// Enhanced Error/Warning Level Detection Tests
// ============================================================

#[test]
fn test_detect_log_level_failed() {
    assert_eq!(
        ParseHandler::detect_log_level("Test failed"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("FAILED: assertion failed"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("Build failure"),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_stack_trace() {
    assert_eq!(
        ParseHandler::detect_log_level("STACK TRACE:"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("Backtrace:"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level(
            "Exception at com.example.MyClass.myMethod(MyClass.java:42)"
        ),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_panic_crash() {
    assert_eq!(
        ParseHandler::detect_log_level("PANIC: unrecoverable error"),
        LogLevel::Fatal
    );
    assert_eq!(
        ParseHandler::detect_log_level("Application crashed"),
        LogLevel::Fatal
    );
    assert_eq!(
        ParseHandler::detect_log_level("Aborting due to critical error"),
        LogLevel::Fatal
    );
}

#[test]
fn test_detect_log_level_deprecated() {
    assert_eq!(
        ParseHandler::detect_log_level("DEPRECATED: use newFunction instead"),
        LogLevel::Warning
    );
    assert_eq!(
        ParseHandler::detect_log_level("This method is deprecated"),
        LogLevel::Warning
    );
}

#[test]
fn test_detect_log_level_caution() {
    assert_eq!(
        ParseHandler::detect_log_level("CAUTION: data may be lost"),
        LogLevel::Warning
    );
    assert_eq!(
        ParseHandler::detect_log_level("ATTENTION: read carefully"),
        LogLevel::Warning
    );
}

#[test]
fn test_detect_log_level_connection_errors() {
    assert_eq!(
        ParseHandler::detect_log_level("Connection refused"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("CONNECTION ERROR: timeout"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("ACCESS DENIED for user"),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_timeout() {
    assert_eq!(
        ParseHandler::detect_log_level("TIMEOUT ERROR"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("Request timed out"),
        LogLevel::Unknown
    );
}

#[test]
fn test_detect_log_level_segfault() {
    assert_eq!(ParseHandler::detect_log_level("SEG FAULT"), LogLevel::Error);
    assert_eq!(
        ParseHandler::detect_log_level("SEGFAULT at 0x0"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("NULL POINTER exception"),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_notice() {
    assert_eq!(
        ParseHandler::detect_log_level("NOTICE: system maintenance"),
        LogLevel::Info
    );
    assert_eq!(
        ParseHandler::detect_log_level("[NOTICE] Server starting"),
        LogLevel::Info
    );
}

#[test]
fn test_detect_log_level_slow_queries() {
    assert_eq!(
        ParseHandler::detect_log_level("SLOW QUERY detected"),
        LogLevel::Warning
    );
    assert_eq!(
        ParseHandler::detect_log_level("SLOW REQUEST: 5.2s"),
        LogLevel::Warning
    );
}

#[test]
fn test_detect_log_level_negation_patterns() {
    // These should NOT be detected as errors due to negation patterns
    assert_eq!(
        ParseHandler::detect_log_level("No errors found"),
        LogLevel::Unknown
    );
    assert_eq!(
        ParseHandler::detect_log_level("Completed with 0 errors"),
        LogLevel::Unknown
    );
    assert_eq!(
        ParseHandler::detect_log_level("Zero failures detected"),
        LogLevel::Unknown
    );
}

#[test]
fn test_detect_log_level_case_variations() {
    // Test mixed case detection
    assert_eq!(
        ParseHandler::detect_log_level("ERROR: something bad"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("Error: something bad"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("error: something bad"),
        LogLevel::Error
    );
    assert_eq!(ParseHandler::detect_log_level("FAILED"), LogLevel::Error);
    assert_eq!(ParseHandler::detect_log_level("failed"), LogLevel::Error);
    assert_eq!(ParseHandler::detect_log_level("Failed"), LogLevel::Error);
}

#[test]
fn test_parse_logs_counts_levels_correctly() {
    let input = r#"[INFO] Starting
[ERROR] Connection failed
[WARN] Deprecated API
[FATAL] System panic
[DEBUG] Trace info
Unknown line"#;
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.total_lines, 6);
    assert_eq!(result.info_count, 1);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.fatal_count, 1);
    assert_eq!(result.debug_count, 1);
    assert_eq!(result.unknown_count, 1);
}

#[test]
fn test_parse_logs_multiple_errors() {
    let input = r#"Test case 1 FAILED
Test case 2 FAILED
Exception in thread main
Connection refused"#;
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.error_count, 4);
}

#[test]
fn test_parse_logs_mixed_levels() {
    let input = r#"Starting application...
PANIC: unrecoverable error
Build failure
Deprecated method used
All systems operational"#;
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.fatal_count, 1);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.unknown_count, 2);
}

// ============================================================
// Recent Critical Lines Tests
// ============================================================

#[test]
fn test_parse_logs_tracks_recent_critical() {
    let input = r#"[INFO] Starting
[ERROR] First error
[WARN] Warning
[FATAL] Fatal error
[ERROR] Second error
[INFO] Done"#;
    let result = ParseHandler::parse_logs(input);

    // Should track 3 critical lines (ERROR and FATAL)
    assert_eq!(result.recent_critical.len(), 3);
    assert_eq!(result.recent_critical[0].message, "First error");
    assert_eq!(result.recent_critical[1].message, "Fatal error");
    assert_eq!(result.recent_critical[2].message, "Second error");
}

#[test]
fn test_parse_logs_recent_critical_only_errors_and_fatals() {
    let input = r#"[INFO] Info message
[WARN] Warning message
[DEBUG] Debug message
[ERROR] Error message
[FATAL] Fatal message"#;
    let result = ParseHandler::parse_logs(input);

    // Only ERROR and FATAL should be in recent_critical
    assert_eq!(result.recent_critical.len(), 2);
    assert_eq!(result.recent_critical[0].level, LogLevel::Error);
    assert_eq!(result.recent_critical[1].level, LogLevel::Fatal);
}

#[test]
fn test_parse_logs_recent_critical_limit() {
    // Create input with more than MAX_RECENT_CRITICAL (10) errors
    let mut input = String::new();
    for i in 1..=15 {
        input.push_str(&format!("[ERROR] Error message {}\n", i));
    }
    let result = ParseHandler::parse_logs(&input);

    // Should be limited to MAX_RECENT_CRITICAL (10)
    assert_eq!(result.recent_critical.len(), 10);
    // Should keep the most recent (last 10)
    assert_eq!(result.recent_critical[0].message, "Error message 6");
    assert_eq!(result.recent_critical[9].message, "Error message 15");
}

#[test]
fn test_parse_logs_recent_critical_order() {
    let input = r#"[ERROR] Error at line 1
[INFO] Info
[FATAL] Fatal at line 3
[ERROR] Error at line 4
[FATAL] Fatal at line 5"#;
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.recent_critical.len(), 4);
    // Should be in order of appearance
    assert_eq!(result.recent_critical[0].line_number, 1);
    assert_eq!(result.recent_critical[1].line_number, 3);
    assert_eq!(result.recent_critical[2].line_number, 4);
    assert_eq!(result.recent_critical[3].line_number, 5);
}

#[test]
fn test_parse_logs_no_critical_lines() {
    let input = r#"[INFO] Starting
[DEBUG] Debug info
[WARN] Warning"#;
    let result = ParseHandler::parse_logs(input);

    assert!(result.recent_critical.is_empty());
}

#[test]
fn test_format_logs_json_includes_recent_critical() {
    let input = "[INFO] Info\n[ERROR] Error 1\n[ERROR] Error 2\n[FATAL] Fatal";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Json);

    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Should have recent_critical array
    assert!(json["recent_critical"].is_array());
    let recent = json["recent_critical"].as_array().unwrap();
    assert_eq!(recent.len(), 3);

    // Should have counts
    assert_eq!(json["recent_critical_count"], 3);
    assert_eq!(json["total_critical"], 3);
}

#[test]
fn test_format_logs_compact_shows_recent_critical() {
    let input = "[INFO] Starting\n[ERROR] Something failed\n[FATAL] System crash\n[INFO] Done";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    // Should show recent critical section
    assert!(output.contains("recent critical"));
    assert!(output.contains("[E]"));
    assert!(output.contains("[F]"));
    assert!(output.contains("Something failed"));
    assert!(output.contains("System crash"));
}

#[test]
fn test_format_logs_compact_recent_critical_count() {
    // Create more than MAX_RECENT_CRITICAL errors
    let mut input = String::new();
    for i in 1..=15 {
        input.push_str(&format!("[ERROR] Error {}\n", i));
    }
    let result = ParseHandler::parse_logs(&input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    // Should show count as "10 of 15"
    assert!(output.contains("10 of 15"));
}
