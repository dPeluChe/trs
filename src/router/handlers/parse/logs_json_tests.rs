use super::super::ParseHandler;
use crate::router::handlers::types::LogLevel;

#[test]
fn pino_numeric_level_and_msg() {
    // pino: numeric level (50 = error), msg field.
    let e = ParseHandler::try_parse_json_log_line(
        r#"{"level":50,"time":1700000000000,"msg":"db connection failed","pid":42}"#,
        1,
    )
    .expect("json log");
    assert_eq!(e.level, LogLevel::Error);
    assert!(e.message.starts_with("db connection failed"));
    assert!(e.timestamp.is_some());
}

#[test]
fn string_level_message_logger_appended() {
    let e = ParseHandler::try_parse_json_log_line(
        r#"{"level":"warn","message":"slow query","logger":"db"}"#,
        2,
    )
    .expect("json log");
    assert_eq!(e.level, LogLevel::Warning);
    assert!(e.message.contains("slow query"));
    assert!(e.message.contains("(db)"));
}

#[test]
fn error_field_is_appended() {
    let e = ParseHandler::try_parse_json_log_line(
        r#"{"severity":"ERROR","msg":"request failed","error":"timeout after 30s"}"#,
        3,
    )
    .expect("json log");
    assert_eq!(e.level, LogLevel::Error);
    assert!(e.message.contains("err: timeout after 30s"));
}

#[test]
fn bunyan_fatal_60() {
    let e = ParseHandler::try_parse_json_log_line(r#"{"level":60,"msg":"oom"}"#, 4).unwrap();
    assert_eq!(e.level, LogLevel::Fatal);
}

#[test]
fn non_json_returns_none() {
    assert!(ParseHandler::try_parse_json_log_line("2024-01-01 INFO starting up", 1).is_none());
}

#[test]
fn arbitrary_json_data_passes_through() {
    // No level and no message field → not a log line; must not be eaten.
    assert!(
        ParseHandler::try_parse_json_log_line(r#"{"id":7,"name":"acme","total":42}"#, 1).is_none()
    );
}

#[test]
fn message_only_no_level_is_accepted() {
    let e = ParseHandler::try_parse_json_log_line(r#"{"msg":"hello world"}"#, 1).unwrap();
    assert_eq!(e.level, LogLevel::Unknown);
    assert_eq!(e.message, "hello world");
}

#[test]
fn compact_output_drops_json_noise() {
    // The compact render must show the extracted message, not the raw JSON
    // (no trace_id / span_id / hostname bloat reaching the agent).
    let input = r#"{"level":"error","ts":"t0","msg":"db timeout","logger":"pool","trace_id":"deadbeefdeadbeef","span_id":"abcd1234","hostname":"prod-7","pid":99}"#;
    let out = ParseHandler::parse_logs(input);
    let compact = ParseHandler::format_logs_compact(&out);
    assert!(compact.contains("db timeout"), "missing message: {compact}");
    assert!(
        !compact.contains("trace_id"),
        "leaked json noise: {compact}"
    );
    assert!(!compact.contains("span_id"), "leaked json noise: {compact}");
}

#[test]
fn verbose_json_logs_compress_hard() {
    // Representative pino-style lines: every key repeats. The compact render
    // should be well under half the raw size (field extraction + fold).
    let mut raw = String::new();
    for i in 0..40 {
        let lvl = if i % 13 == 0 { "error" } else { "info" };
        raw.push_str(&format!(
            r#"{{"level":"{lvl}","time":170000000{i},"pid":42317,"hostname":"prod-api-7d9f","reqId":"req-{i}{i}{i}","msg":"request completed","logger":"http","durationMs":{i},"trace_id":"8a607854d8f00ad{i}","span_id":"7e59bb40"}}"#
        ));
        raw.push('\n');
    }
    let out = ParseHandler::parse_logs(&raw);
    let compact = ParseHandler::format_logs_compact(&out);
    assert!(
        compact.len() * 100 / raw.len() < 40,
        "expected <40% of raw, got {}% ({} -> {})",
        compact.len() * 100 / raw.len(),
        raw.len(),
        compact.len()
    );
}

#[test]
fn level_drives_parse_logs_counts() {
    // End-to-end through parse_logs: JSON error line counts as error.
    let input = r#"{"level":"info","msg":"a"}
{"level":"error","msg":"b"}
{"level":"error","msg":"c"}"#;
    let out = ParseHandler::parse_logs(input);
    assert_eq!(out.error_count, 2);
    assert_eq!(out.info_count, 1);
}
