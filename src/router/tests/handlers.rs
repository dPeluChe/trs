use super::*;

// ============================================================
// Run Handler Tests
// ============================================================

#[test]
fn test_run_handler_success() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = RunInput {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        capture_stdout: true,
        capture_stderr: true,
        capture_exit_code: true,
        capture_duration: true,
        timeout: None,
    };

    let result = handler.execute(&input, &ctx);
    // echo should succeed
    assert!(result.is_ok());
}

#[test]
fn test_run_handler_command_not_found() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = RunInput {
        command: "nonexistent_command_xyz123".to_string(),
        args: vec![],
        capture_stdout: true,
        capture_stderr: true,
        capture_exit_code: true,
        capture_duration: true,
        timeout: None,
    };

    let result = handler.execute(&input, &ctx);
    // Should return an error for command not found
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CommandError::ExecutionError {
            message: _,
            exit_code: _
        })
    ));
}

#[test]
fn test_run_handler_non_zero_exit() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = RunInput {
        command: "false".to_string(),
        args: vec![],
        capture_stdout: true,
        capture_stderr: true,
        capture_exit_code: true,
        capture_duration: true,
        timeout: None,
    };

    let result = handler.execute(&input, &ctx);
    // false always exits with 1
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(CommandError::ExecutionError {
            message: _,
            exit_code: _
        })
    ));
}

#[test]
fn test_run_handler_json_format() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![OutputFormat::Json],
    };
    let input = RunInput {
        command: "echo".to_string(),
        args: vec!["test".to_string()],
        capture_stdout: true,
        capture_stderr: true,
        capture_exit_code: true,
        capture_duration: true,
        timeout: None,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_run_handler_no_capture_stdout() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = RunInput {
        command: "echo".to_string(),
        args: vec!["test".to_string()],
        capture_stdout: false,
        capture_stderr: true,
        capture_exit_code: true,
        capture_duration: true,
        timeout: None,
    };

    // When stdout is not captured, the command should still succeed
    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_run_handler_no_capture_exit_code() {
    let handler = RunHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![OutputFormat::Json],
    };
    let input = RunInput {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), "exit 42".to_string()],
        capture_stdout: true,
        capture_stderr: true,
        capture_exit_code: false,
        capture_duration: true,
        timeout: None,
    };

    // When exit code is not captured, the error is NOT propagated
    // even though the command exited with a non-zero code
    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

// ============================================================
// Search Handler Tests
// ============================================================

#[test]
fn test_search_handler() {
    let handler = SearchHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = SearchInput {
        path: std::path::PathBuf::from("src"),
        query: "SearchHandler".to_string(),
        extension: Some("rs".to_string()),
        ignore_case: false,
        context: None,
        limit: Some(10),
    };

    // The search handler should now execute successfully
    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

// ============================================================
// Replace Handler Tests
// ============================================================

#[test]
fn test_replace_handler() {
    let handler = ReplaceHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = ReplaceInput {
        path: std::path::PathBuf::from("."),
        search: "new_unique_string_xyz".to_string(),
        replace: "new".to_string(),
        extension: Some("rs".to_string()),
        dry_run: true,
        count: false,
    };

    // The replace handler should execute successfully (dry run, no actual changes)
    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_replace_handler_json_format() {
    let handler = ReplaceHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };
    let input = ReplaceInput {
        path: std::path::PathBuf::from("."),
        search: "nonexistent_pattern_abc123".to_string(),
        replace: "new".to_string(),
        extension: Some("rs".to_string()),
        dry_run: true,
        count: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_replace_truncate_line() {
    let short_line = "short line";
    assert_eq!(
        ReplaceHandler::truncate_line(short_line, 80),
        short_line.to_string()
    );

    let long_line = "a".repeat(100);
    let truncated = ReplaceHandler::truncate_line(&long_line, 80);
    assert!(truncated.len() <= 83); // 80 + "..."
    assert!(truncated.ends_with("..."));
}

#[test]
fn test_replace_escape_csv_field() {
    assert_eq!(ReplaceHandler::escape_csv_field("simple"), "simple");
    assert_eq!(
        ReplaceHandler::escape_csv_field("with,comma"),
        "\"with,comma\""
    );
    assert_eq!(
        ReplaceHandler::escape_csv_field("with\"quote"),
        "\"with\"\"quote\""
    );
    assert_eq!(
        ReplaceHandler::escape_csv_field("with\nnewline"),
        "\"with\nnewline\""
    );
}

#[test]
fn test_replace_escape_tsv_field() {
    assert_eq!(ReplaceHandler::escape_tsv_field("simple"), "simple");
    assert_eq!(ReplaceHandler::escape_tsv_field("with\ttab"), "with\\ttab");
    assert_eq!(
        ReplaceHandler::escape_tsv_field("with\nnewline"),
        "with\\nnewline"
    );
}

// ============================================================
// Tail Handler Tests
// ============================================================

#[test]
fn test_tail_handler_file_not_found() {
    let handler = TailHandler;
    let ctx = CommandContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };
    let input = TailInput {
        file: std::path::PathBuf::from("/nonexistent/file.log"),
        lines: 20,
        errors: true,
        follow: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(matches!(result, Err(CommandError::IoError(_))));
}

#[test]
fn test_tail_is_error_line() {
    assert!(TailHandler::is_error_line("ERROR: something went wrong"));
    assert!(TailHandler::is_error_line("error in processing"));
    assert!(TailHandler::is_error_line("FATAL: critical failure"));
    assert!(TailHandler::is_error_line("Exception: null pointer"));
    assert!(TailHandler::is_error_line("CRITICAL: system failure"));
    assert!(TailHandler::is_error_line("Failed to connect"));
    assert!(TailHandler::is_error_line("[ERROR] connection timeout"));
    assert!(TailHandler::is_error_line("ERR connection refused"));
    assert!(TailHandler::is_error_line(
        "E/AndroidRuntime: FATAL EXCEPTION"
    ));

    assert!(!TailHandler::is_error_line("INFO: process started"));
    assert!(!TailHandler::is_error_line("success: operation completed"));
    assert!(!TailHandler::is_error_line("warning: deprecated API"));
    assert!(!TailHandler::is_error_line("debug: processing request"));
}

// ============================================================
// Clean Handler Tests
// ============================================================

#[test]
fn test_clean_handler() {
    let handler = CleanHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };
    // Use a temp file instead of stdin to avoid blocking
    let tmp = std::env::temp_dir().join("trs_test_clean.tmp");
    std::fs::write(&tmp, "hello\x1b[31m world\x1b[0m\n\n\ntest\n").unwrap();
    let input = CleanInput {
        file: Some(tmp.clone()),
        no_ansi: true,
        collapse_blanks: true,
        collapse_repeats: false,
        trim: true,
    };

    let result = handler.execute(&input, &ctx);
    let _ = std::fs::remove_file(&tmp);
    assert!(result.is_ok());
}
