//! Validation tests for vitest, npm, pnpm, bun, and logs fixture loader functions.

mod fixtures;
use fixtures::*;

// Vitest Fixture Tests

#[test]
fn test_load_vitest_empty() {
    let content = vitest_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_vitest_single_passed() {
    let content = vitest_single_passed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("1 passed"));
}

#[test]
fn test_load_vitest_single_failed() {
    let content = vitest_single_failed();
    assert!(content.contains("\u{276F}"));
    assert!(content.contains("AssertionError"));
    assert!(content.contains("1 failed"));
}

#[test]
fn test_load_vitest_mixed() {
    let content = vitest_mixed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("\u{276F}"));
    assert!(content.contains("1 passed, 1 failed"));
}

#[test]
fn test_load_vitest_all_passed() {
    let content = vitest_all_passed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("3 passed (3)"));
    assert!(content.contains("9 passed (9)"));
}

#[test]
fn test_load_vitest_all_failed() {
    let content = vitest_all_failed();
    assert!(content.contains("\u{276F}"));
    assert!(content.contains("2 failed"));
}

#[test]
fn test_load_vitest_with_skipped() {
    let content = vitest_with_skipped();
    assert!(content.contains("1 skipped"));
}

#[test]
fn test_load_vitest_mixed_skipped() {
    let content = vitest_mixed_skipped();
    assert!(content.contains("10 passed"));
    assert!(content.contains("3 skipped"));
}

#[test]
fn test_load_vitest_with_todo() {
    let content = vitest_with_todo();
    assert!(content.contains("1 todo"));
}

#[test]
fn test_load_vitest_large() {
    let content = vitest_large();
    assert!(content.contains("test01.test.ts"));
    assert!(content.contains("test10.test.ts"));
    assert!(content.contains("10 passed"));
    assert!(content.contains("20 passed"));
}

// NPM Test Fixture Tests

#[test]
fn test_load_npm_test_empty() {
    let content = npm_test_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_npm_test_single_passed() {
    let content = npm_test_single_passed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("1 passed"));
}

#[test]
fn test_load_npm_test_single_failed() {
    let content = npm_test_single_failed();
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("AssertionError"));
    assert!(content.contains("1 failed"));
}

#[test]
fn test_load_npm_test_mixed() {
    let content = npm_test_mixed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("# SKIP"));
}

#[test]
fn test_load_npm_test_all_passed() {
    let content = npm_test_all_passed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("5 passed"));
}

#[test]
fn test_load_npm_test_all_failed() {
    let content = npm_test_all_failed();
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("2 failed"));
}

#[test]
fn test_load_npm_test_with_skipped() {
    let content = npm_test_with_skipped();
    assert!(content.contains("# SKIP"));
    assert!(content.contains("2 passed"));
}

#[test]
fn test_load_npm_test_with_todo() {
    let content = npm_test_with_todo();
    assert!(content.contains("# TODO"));
    assert!(content.contains("# SKIP"));
}

#[test]
fn test_load_npm_test_multiple_suites() {
    let content = npm_test_multiple_suites();
    assert!(content.contains("Button.test.js"));
    assert!(content.contains("Input.test.js"));
    assert!(content.contains("Modal.test.js"));
    assert!(content.contains("7 passed"));
}

#[test]
fn test_load_npm_test_large() {
    let content = npm_test_large();
    assert!(content.contains("test01.test.js"));
    assert!(content.contains("test10.test.js"));
    assert!(content.contains("20 passed"));
}

#[test]
fn test_load_npm_test_with_header() {
    let content = npm_test_with_header();
    assert!(content.contains("> project@1.0.0 test"));
    assert!(content.contains("> node --test"));
}

// ============================================================
// PNPM Test Fixture Tests
// ============================================================

#[test]
fn test_load_pnpm_test_empty() {
    let content = pnpm_test_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_pnpm_test_single_passed() {
    let content = pnpm_test_single_passed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("1 passed"));
}

#[test]
fn test_load_pnpm_test_single_failed() {
    let content = pnpm_test_single_failed();
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("AssertionError"));
    assert!(content.contains("1 failed"));
}

#[test]
fn test_load_pnpm_test_mixed() {
    let content = pnpm_test_mixed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("# SKIP"));
}

#[test]
fn test_load_pnpm_test_all_passed() {
    let content = pnpm_test_all_passed();
    assert!(content.contains("\u{2714}"));
    assert!(content.contains("5 passed"));
}

#[test]
fn test_load_pnpm_test_all_failed() {
    let content = pnpm_test_all_failed();
    assert!(content.contains("\u{2716}"));
    assert!(content.contains("2 failed"));
}

#[test]
fn test_load_pnpm_test_with_skipped() {
    let content = pnpm_test_with_skipped();
    assert!(content.contains("# SKIP"));
    assert!(content.contains("2 passed"));
}

#[test]
fn test_load_pnpm_test_with_todo() {
    let content = pnpm_test_with_todo();
    assert!(content.contains("# TODO"));
    assert!(content.contains("# SKIP"));
}

#[test]
fn test_load_pnpm_test_large() {
    let content = pnpm_test_large();
    assert!(content.contains("test01.test.js"));
    assert!(content.contains("test10.test.js"));
    assert!(content.contains("20 passed"));
}

#[test]
fn test_load_pnpm_test_with_header() {
    let content = pnpm_test_with_header();
    assert!(content.contains("pnpm: 9.0.0"));
}

// ============================================================
// Bun Test Fixture Tests
// ============================================================

#[test]
fn test_load_bun_test_empty() {
    let content = bun_test_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_bun_test_single_passed() {
    let content = bun_test_single_passed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("1 pass"));
}

#[test]
fn test_load_bun_test_single_failed() {
    let content = bun_test_single_failed();
    assert!(content.contains("\u{2717}"));
    assert!(content.contains("AssertionError"));
    assert!(content.contains("1 fail"));
}

#[test]
fn test_load_bun_test_mixed() {
    let content = bun_test_mixed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("\u{2717}"));
    assert!(content.contains("3 pass"));
    assert!(content.contains("1 fail"));
}

#[test]
fn test_load_bun_test_all_passed() {
    let content = bun_test_all_passed();
    assert!(content.contains("\u{2713}"));
    assert!(content.contains("5 pass"));
    assert!(content.contains("0 fail"));
}

#[test]
fn test_load_bun_test_all_failed() {
    let content = bun_test_all_failed();
    assert!(content.contains("\u{2717}"));
    assert!(content.contains("0 pass"));
    assert!(content.contains("2 fail"));
}

#[test]
fn test_load_bun_test_with_skipped() {
    let content = bun_test_with_skipped();
    assert!(content.contains("(skip)"));
    assert!(content.contains("2 skip"));
}

#[test]
fn test_load_bun_test_with_todo() {
    let content = bun_test_with_todo();
    assert!(content.contains("(todo)"));
    assert!(content.contains("(skip)"));
    assert!(content.contains("1 todo"));
}

#[test]
fn test_load_bun_test_multiple_suites() {
    let content = bun_test_multiple_suites();
    assert!(content.contains("Button.test.js"));
    assert!(content.contains("Input.test.js"));
    assert!(content.contains("Modal.test.js"));
    assert!(content.contains("7 pass"));
}

#[test]
fn test_load_bun_test_large() {
    let content = bun_test_large();
    assert!(content.contains("test01.test.js"));
    assert!(content.contains("test10.test.js"));
    assert!(content.contains("20 pass"));
}

#[test]
fn test_load_bun_test_non_tty() {
    let content = bun_test_non_tty();
    assert!(content.contains("(pass)"));
    assert!(content.contains("2 pass"));
}

// ============================================================
// Logs Fixture Tests
// ============================================================

#[test]
fn test_load_logs_empty() {
    let content = logs_empty();
    assert!(content.is_empty());
}

#[test]
fn test_load_logs_simple() {
    let content = logs_simple();
    assert!(content.contains("[INFO]"));
    assert!(content.contains("[DEBUG]"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("[FATAL]"));
}

#[test]
fn test_load_logs_iso8601_timestamp() {
    let content = logs_iso8601_timestamp();
    assert!(content.contains("2024-01-15T10:30:00"));
    assert!(content.contains("[INFO]"));
    assert!(content.contains("[ERROR]"));
}

#[test]
fn test_load_logs_syslog_format() {
    let content = logs_syslog_format();
    assert!(content.contains("Jan 15 10:30:00"));
    assert!(content.contains("server01"));
    assert!(content.contains("INFO"));
    assert!(content.contains("ERROR"));
    assert!(content.contains("CRITICAL"));
}

#[test]
fn test_load_logs_all_levels() {
    let content = logs_all_levels();
    assert!(content.contains("[TRACE]"));
    assert!(content.contains("[DEBUG]"));
    assert!(content.contains("[VERBOSE]"));
    assert!(content.contains("[INFO]"));
    assert!(content.contains("[NOTICE]"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("[WARNING]"));
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("[CRITICAL]"));
    assert!(content.contains("[FATAL]"));
    assert!(content.contains("[PANIC]"));
}

#[test]
fn test_load_logs_errors_only() {
    let content = logs_errors_only();
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("ERROR:"));
    assert!(content.contains("Exception"));
    assert!(content.contains("FAILED"));
    assert!(content.contains("STACK TRACE"));
    assert!(content.contains("Connection refused"));
    assert!(content.contains("ACCESS DENIED"));
}

#[test]
fn test_load_logs_with_exceptions() {
    let content = logs_with_exceptions();
    assert!(content.contains("java.lang.NullPointerException"));
    assert!(content.contains("Traceback"));
    assert!(content.contains("ConnectionError"));
    assert!(content.contains("PANIC"));
    assert!(content.contains("Backtrace"));
}

#[test]
fn test_load_logs_mixed_format() {
    let content = logs_mixed_format();
    assert!(content.contains("2024-01-15 10:30:00"));
    assert!(content.contains("2024/01/15"));
    assert!(content.contains("|INFO|"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("WARNING:"));
}

#[test]
fn test_load_logs_repeated_lines() {
    let content = logs_repeated_lines();
    assert!(content.contains("[DEBUG] Processing request"));
    assert!(content.contains("[WARN] Cache miss"));
    assert!(content.contains("[ERROR] Connection failed"));
}

#[test]
fn test_load_logs_large() {
    let content = logs_large();
    assert!(content.contains("[INFO] Application starting"));
    assert!(content.contains("[DEBUG] Initializing module"));
    assert!(content.contains("[ERROR] Database connection pool exhausted"));
    assert!(content.contains("[FATAL] Out of memory error"));
    assert!(content.contains("Shutdown complete"));
}

#[test]
fn test_load_logs_json_format() {
    let content = logs_json_format();
    assert!(content.contains("\"level\": \"INFO\""));
    assert!(content.contains("\"level\": \"ERROR\""));
    assert!(content.contains("\"level\": \"FATAL\""));
    assert!(content.contains("\"message\""));
    assert!(content.contains("\"timestamp\""));
}

#[test]
fn test_load_logs_with_source() {
    let content = logs_with_source();
    assert!(content.contains("[auth]"));
    assert!(content.contains("[database]"));
    assert!(content.contains("[api]"));
    assert!(content.contains("[cache]"));
    assert!(content.contains("[payment]"));
}

#[test]
fn test_load_logs_application() {
    let content = logs_application();
    assert!(content.contains("Starting MyApp"));
    assert!(content.contains("Database connection established"));
    assert!(content.contains("Server ready"));
    assert!(content.contains("Graceful shutdown"));
}

#[test]
fn test_load_logs_access() {
    let content = logs_access();
    assert!(content.contains("192.168.1.100"));
    assert!(content.contains("GET /api/health"));
    assert!(content.contains("HTTP/1.1"));
    assert!(content.contains("200"));
    assert!(content.contains("404"));
    assert!(content.contains("500"));
}

#[test]
fn test_load_logs_colon_format() {
    let content = logs_colon_format();
    assert!(content.contains("DEBUG:"));
    assert!(content.contains("INFO:"));
    assert!(content.contains("WARNING:"));
    assert!(content.contains("ERROR:"));
    assert!(content.contains("CRITICAL:"));
    assert!(content.contains("FATAL:"));
}

#[test]
fn test_load_logs_pipe_format() {
    let content = logs_pipe_format();
    assert!(content.contains("|DEBUG|"));
    assert!(content.contains("|INFO|"));
    assert!(content.contains("|WARN|"));
    assert!(content.contains("|ERROR|"));
    assert!(content.contains("|CRIT|"));
    assert!(content.contains("|FATAL|"));
}

#[test]
fn test_load_logs_multiline_error() {
    let content = logs_multiline_error();
    assert!(content.contains("java.lang.IllegalArgumentException"));
    assert!(content.contains("Caused by:"));
    assert!(content.contains("Traceback"));
    assert!(content.contains("json.decoder.JSONDecodeError"));
}
