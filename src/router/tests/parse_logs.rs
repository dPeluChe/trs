use super::*;

// ============================================================
// Logs Parser Tests
// ============================================================

#[test]
fn test_parse_logs_empty() {
    let result = ParseHandler::parse_logs("");
    assert!(result.is_empty);
    assert_eq!(result.total_lines, 0);
}

#[test]
fn test_parse_logs_single_line() {
    let input = "This is a log line";
    let result = ParseHandler::parse_logs(input);

    assert!(!result.is_empty);
    assert_eq!(result.total_lines, 1);
    assert_eq!(result.unknown_count, 1);
}

#[test]
fn test_parse_logs_with_levels() {
    let input = r#"[INFO] Starting application
[ERROR] Something went wrong
[WARN] Warning message
[DEBUG] Debug info"#;
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.total_lines, 4);
    assert_eq!(result.info_count, 1);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.debug_count, 1);
}

#[test]
fn test_parse_logs_with_fatal() {
    let input = "[FATAL] Critical error\n[CRITICAL] Also critical";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.fatal_count, 2);
}

#[test]
fn test_parse_logs_with_repeated_lines() {
    let input = "Repeated line\nDifferent line\nRepeated line\nRepeated line";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.repeated_lines.len(), 1);
    assert_eq!(result.repeated_lines[0].line, "Repeated line");
    assert_eq!(result.repeated_lines[0].count, 3);
}

#[test]
fn test_parse_logs_timestamp_iso8601() {
    let input = "2024-01-15T10:30:00 [INFO] Message";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(
        result.entries[0].timestamp,
        Some("2024-01-15T10:30:00".to_string())
    );
    assert_eq!(result.entries[0].level, LogLevel::Info);
}

#[test]
fn test_parse_logs_timestamp_iso8601_space() {
    let input = "2024-01-15 10:30:00 [ERROR] Error message";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(
        result.entries[0].timestamp,
        Some("2024-01-15 10:30:00".to_string())
    );
    assert_eq!(result.entries[0].level, LogLevel::Error);
}

#[test]
fn test_parse_logs_timestamp_syslog() {
    let input = "Jan 15 10:30:00 server daemon[123]: Message";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(
        result.entries[0].timestamp,
        Some("Jan 15 10:30:00".to_string())
    );
}

#[test]
fn test_parse_logs_timestamp_time_only() {
    let input = "10:30:00 INFO: Message";
    let result = ParseHandler::parse_logs(input);

    assert_eq!(result.entries[0].timestamp, Some("10:30:00".to_string()));
}

#[test]
fn test_detect_log_level_brackets() {
    assert_eq!(
        ParseHandler::detect_log_level("[DEBUG] test"),
        LogLevel::Debug
    );
    assert_eq!(
        ParseHandler::detect_log_level("[INFO] test"),
        LogLevel::Info
    );
    assert_eq!(
        ParseHandler::detect_log_level("[WARN] test"),
        LogLevel::Warning
    );
    assert_eq!(
        ParseHandler::detect_log_level("[ERROR] test"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("[FATAL] test"),
        LogLevel::Fatal
    );
}

#[test]
fn test_detect_log_level_colon() {
    assert_eq!(
        ParseHandler::detect_log_level("DEBUG: test"),
        LogLevel::Debug
    );
    assert_eq!(ParseHandler::detect_log_level("INFO: test"), LogLevel::Info);
    assert_eq!(
        ParseHandler::detect_log_level("WARNING: test"),
        LogLevel::Warning
    );
    assert_eq!(
        ParseHandler::detect_log_level("ERROR: test"),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_pipes() {
    assert_eq!(
        ParseHandler::detect_log_level("|DEBUG| test"),
        LogLevel::Debug
    );
    assert_eq!(
        ParseHandler::detect_log_level("|INFO| test"),
        LogLevel::Info
    );
    assert_eq!(
        ParseHandler::detect_log_level("|ERROR| test"),
        LogLevel::Error
    );
}

#[test]
fn test_detect_log_level_case_insensitive() {
    assert_eq!(
        ParseHandler::detect_log_level("[info] test"),
        LogLevel::Info
    );
    assert_eq!(
        ParseHandler::detect_log_level("[Error] test"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("WARN: test"),
        LogLevel::Warning
    );
}

#[test]
fn test_detect_log_level_exception() {
    assert_eq!(
        ParseHandler::detect_log_level("Exception: test"),
        LogLevel::Error
    );
    assert_eq!(
        ParseHandler::detect_log_level("java.lang.Exception: test"),
        LogLevel::Error
    );
}

#[test]
fn test_extract_message_removes_timestamp() {
    let timestamp = Some("2024-01-15T10:30:00".to_string());
    let message = ParseHandler::extract_message(
        "2024-01-15T10:30:00 [INFO] Hello world",
        &timestamp,
        &LogLevel::Info,
    );
    assert!(message.contains("Hello world"));
    assert!(!message.contains("2024-01-15T10:30:00"));
}

#[test]
fn test_extract_message_removes_level() {
    let message = ParseHandler::extract_message("[INFO] Hello world", &None, &LogLevel::Info);
    assert!(message.contains("Hello world"));
    assert!(!message.contains("[INFO]"));
}

#[test]
fn test_format_logs_json() {
    let input = "[INFO] Test message\n[ERROR] Error message";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Json);

    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["counts"]["total_lines"], 2);
    assert_eq!(json["counts"]["info"], 1);
    assert_eq!(json["counts"]["error"], 1);
}

#[test]
fn test_format_logs_csv() {
    let input = "[INFO] Test message";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Csv);

    assert!(output.starts_with("line_number,level,timestamp,message\n"));
    assert!(output.contains("1,info,,Test message"));
}

#[test]
fn test_format_logs_tsv() {
    let input = "[INFO] Test message";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Tsv);

    assert!(output.starts_with("line_number\tlevel\ttimestamp\tmessage\n"));
    assert!(output.contains("1\tinfo\t\tTest message"));
}

#[test]
fn test_format_logs_compact() {
    let input = "[INFO] Info message\n[ERROR] Error message\n[WARN] Warning";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    assert!(output.contains("lines: 3"));
    assert!(output.contains("levels:"));
    assert!(output.contains("error:1"));
    assert!(output.contains("warn:1"));
    assert!(output.contains("info:1"));
}

#[test]
fn test_format_logs_compact_empty() {
    let mut result = LogsOutput::default();
    result.is_empty = true;
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    assert!(output.contains("logs: empty"));
}

#[test]
fn test_format_logs_raw() {
    let input = "[INFO] Test message\n[ERROR] Error";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Raw);

    assert!(output.contains("[INFO] Test message"));
    assert!(output.contains("[ERROR] Error"));
}

#[test]
fn test_format_logs_compact_with_repeated() {
    let input = "Same line\nSame line\nSame line";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    assert!(output.contains("repeated:"));
    assert!(output.contains("[x3]"));
}

#[test]
fn test_format_logs_compact_collapses_consecutive_entries_no_levels() {
    // Test LogCrunch-style folding: first + [...repeated N...] + last
    let input = "Same line\nSame line\nSame line\nDifferent\nDifferent\nUnique";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    // 3 identical → first + [...repeated 1 times...] + last
    assert!(output.contains("1 Same line"));
    assert!(output.contains("repeated"));
    assert!(output.contains("3 Same line"));
    // 2 identical → shown individually (below fold threshold)
    assert!(output.contains("4 Different"));
    assert!(output.contains("5 Different"));
    assert!(output.contains("6 Unique"));
}

#[test]
fn test_format_logs_compact_collapses_consecutive_entries_with_levels() {
    // Test LogCrunch-style folding with log levels
    let input = "[INFO] Starting\n[INFO] Starting\n[INFO] Starting\n[ERROR] Failed\n[ERROR] Failed\n[WARN] Warning";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    // 3 INFO → folded (first + fold marker + last not shown since same message)
    assert!(output.contains("[I] 1 Starting"));
    assert!(output.contains("similar info"));
    // ERROR/WARN always shown individually
    assert!(output.contains("[E] 4 Failed"));
    assert!(output.contains("[E] 5 Failed"));
    assert!(output.contains("[W] 6 Warning"));
}

#[test]
fn test_format_logs_compact_no_collapse_non_consecutive() {
    // Test that non-consecutive identical lines are NOT collapsed in entries section
    let input = "Line A\nLine B\nLine A";
    let result = ParseHandler::parse_logs(input);
    let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

    // The entries section should show lines separately (not collapsed since not consecutive)
    // The format is "  N content" (2-space indent)
    // Should contain all three entries individually
    assert!(output.contains("  1 Line A"));
    assert!(output.contains("  2 Line B"));
    assert!(output.contains("  3 Line A"));
    // Should not have collapsed format like "1-3 Line A [x2]" in lines section
    // (non-consecutive entries don't collapse)
}
