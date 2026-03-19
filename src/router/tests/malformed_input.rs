use super::*;

// ============================================================
// Malformed Input Handling Tests
// ============================================================

#[test]
fn test_sanitize_control_chars_removes_nulls() {
    let input = "hello\x00world";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "helloworld");
}

#[test]
fn test_sanitize_control_chars_replaces_control_chars() {
    let input = "hello\x01\x02\x03world";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "hello world");
}

#[test]
fn test_sanitize_control_chars_preserves_newlines() {
    let input = "hello\nworld\r\ntest";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "hello\nworld\r\ntest");
}

#[test]
fn test_sanitize_control_chars_preserves_tabs() {
    let input = "hello\tworld";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "hello\tworld");
}

#[test]
fn test_sanitize_control_chars_normalizes_spaces() {
    let input = "hello\x01\x02world";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "hello world"); // Multiple control chars become single space
}

#[test]
fn test_sanitize_control_chars_preserves_unicode() {
    let input = "hello 世界 🌍";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "hello 世界 🌍");
}

#[test]
fn test_sanitize_control_chars_mixed() {
    let input = "line1\x00\nline2\x01\x02end";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "line1\nline2 end");
}

#[test]
fn test_sanitize_control_chars_empty() {
    let input = "";
    let result = sanitize_control_chars(input);
    assert_eq!(result, "");
}

#[test]
fn test_sanitize_control_chars_only_control() {
    let input = "\x00\x01\x02\x03";
    let result = sanitize_control_chars(input);
    assert_eq!(result, " ");
}

#[test]
fn test_strip_ansi_codes_basic() {
    let input = "\x1b[31mRed text\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "Red text");
}

#[test]
fn test_strip_ansi_codes_multiple() {
    let input = "\x1b[1;31mBnew Red\x1b[0m \x1b[32mGreen\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "Bnew Red Green");
}

#[test]
fn test_process_stdin_with_null_bytes() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = "hello\x00world\nline2";
    let result = router.process_stdin(input, &ctx).unwrap();
    assert_eq!(result, "helloworld\nline2");
}

#[test]
fn test_process_stdin_with_control_chars() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = "hello\x01world";
    let result = router.process_stdin(input, &ctx).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_process_stdin_with_ansi_and_control() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = "\x1b[31mhello\x1b[0m\x00world";
    let result = router.process_stdin(input, &ctx).unwrap();
    assert_eq!(result, "helloworld");
}

#[test]
fn test_process_stdin_json_format() {
    let router = Router::new();
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };
    let input = "hello\x00world";
    let result = router.process_stdin(input, &ctx).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(json["content"], "helloworld");
}

#[test]
fn test_command_context_creation() {
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: true,
        enabled_formats: vec![OutputFormat::Json, OutputFormat::Csv],
    };

    assert_eq!(ctx.format, OutputFormat::Json);
    assert!(ctx.stats);
    assert!(ctx.has_conflicting_formats());
}

#[test]
fn test_command_context_no_conflict() {
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![OutputFormat::Json],
    };

    assert!(!ctx.has_conflicting_formats());
}
