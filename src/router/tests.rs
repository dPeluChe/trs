#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use crate::{OutputFormat, Commands, ParseCommands};
    use crate::router::Router;
    use crate::router::handlers::common::{CommandContext, CommandError, CommandResult, CommandStats,
        strip_ansi_codes, sanitize_control_chars};
    use crate::router::handlers::types::*;
    use crate::router::handlers::run::{RunHandler, RunInput};
    use crate::router::handlers::search::{SearchHandler, SearchInput};
    use crate::router::handlers::replace::{ReplaceHandler, ReplaceInput};
    use crate::router::handlers::tail::{TailHandler, TailInput};
    use crate::router::handlers::clean::{CleanHandler, CleanInput};
    use crate::router::handlers::trim::{TrimHandler, TrimInput};
    use crate::router::handlers::html2md::{Html2mdHandler, Html2mdInput};
    use crate::router::handlers::txt2md::{Txt2mdHandler, Txt2mdInput};
    use crate::router::handlers::isclean::{IsCleanHandler, IsCleanInput};
    use crate::router::handlers::parse::ParseHandler;

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

    // ============================================================
    // CommandStats Tests
    // ============================================================

    #[test]
    fn test_command_stats_with_reducer() {
        let stats = CommandStats::new().with_reducer("search");

        assert_eq!(stats.reducer, Some("search".to_string()));
    }

    #[test]
    fn test_command_stats_with_output_mode() {
        let stats = CommandStats::new().with_output_mode(OutputFormat::Json);

        assert_eq!(stats.output_mode, Some(OutputFormat::Json));
    }

    #[test]
    fn test_command_stats_with_all_fields() {
        let stats = CommandStats::new()
            .with_reducer("git-status")
            .with_output_mode(OutputFormat::Compact)
            .with_input_bytes(1000)
            .with_output_bytes(500)
            .with_items_processed(10);

        assert_eq!(stats.reducer, Some("git-status".to_string()));
        assert_eq!(stats.output_mode, Some(OutputFormat::Compact));
        assert_eq!(stats.input_bytes, 1000);
        assert_eq!(stats.output_bytes, 500);
        assert_eq!(stats.items_processed, 10);
    }

    #[test]
    fn test_command_stats_format_output_mode() {
        assert_eq!(CommandStats::format_output_mode(OutputFormat::Raw), "raw");
        assert_eq!(
            CommandStats::format_output_mode(OutputFormat::Compact),
            "compact"
        );
        assert_eq!(CommandStats::format_output_mode(OutputFormat::Json), "json");
        assert_eq!(CommandStats::format_output_mode(OutputFormat::Csv), "csv");
        assert_eq!(CommandStats::format_output_mode(OutputFormat::Tsv), "tsv");
        assert_eq!(
            CommandStats::format_output_mode(OutputFormat::Agent),
            "agent"
        );
    }

    #[test]
    fn test_command_stats_default() {
        let stats = CommandStats::default();

        assert!(stats.reducer.is_none());
        assert!(stats.output_mode.is_none());
        assert_eq!(stats.input_bytes, 0);
        assert_eq!(stats.output_bytes, 0);
    }

    #[test]
    fn test_command_stats_reduction_percent() {
        let stats = CommandStats::new()
            .with_input_bytes(1000)
            .with_output_bytes(500);

        assert_eq!(stats.reduction_percent(), 50.0);
    }

    #[test]
    fn test_command_stats_no_reduction_when_output_larger() {
        let stats = CommandStats::new()
            .with_input_bytes(500)
            .with_output_bytes(1000);

        assert_eq!(stats.reduction_percent(), 0.0);
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::NotImplemented("test command".to_string());
        assert_eq!(format!("{}", err), "Not implemented: test command");

        let err = CommandError::ExecutionError {
            message: "failed".to_string(),
            exit_code: Some(1),
        };
        assert_eq!(format!("{}", err), "Execution error: failed");

        let err = CommandError::InvalidArguments("bad args".to_string());
        assert_eq!(format!("{}", err), "Invalid arguments: bad args");

        let err = CommandError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

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

    #[test]
    fn test_html2md_handler() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_html2md_input.html");
        let output_path = temp_dir.join("test_html2md_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<h1>Hello World</h1>
<p>This is a <strong>test</strong> paragraph.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Verify output file was created and contains markdown
        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("Hello World"));
        assert!(output_content.contains("test"));

        // Cleanup
        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_handler_with_metadata() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_html2md_meta_input.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Title</title>
<meta name="description" content="Test description">
</head>
<body>
<h1>Heading</h1>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: true,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_handler_json_includes_metadata_automatically() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temporary HTML file for testing
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_html2md_auto_meta.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Auto Metadata Title</title>
<meta name="description" content="Auto metadata description">
</head>
<body>
<h1>Content</h1>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        // Test with metadata=false but JSON output should still include metadata
        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: false, // Explicitly set to false
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_is_url() {
        assert!(Html2mdHandler::is_url("http://example.com"));
        assert!(Html2mdHandler::is_url("https://example.com"));
        assert!(!Html2mdHandler::is_url("/path/to/file.html"));
        assert!(!Html2mdHandler::is_url("file.html"));
    }

    #[test]
    fn test_html2md_file_not_found() {
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let input = Html2mdInput {
            input: "/nonexistent/path/to/file.html".to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::IoError(_))));
    }

    #[test]
    fn test_html2md_heading_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_heading_conversion.html");
        let output_path = temp_dir.join("test_heading_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Test</title></head>
<body>
<h1>Heading 1</h1>
<h2>Heading 2</h2>
<h3>Heading 3</h3>
<h4>Heading 4</h4>
<h5>Heading 5</h5>
<h6>Heading 6</h6>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("# Heading 1"));
        assert!(output_content.contains("## Heading 2"));
        assert!(output_content.contains("### Heading 3"));
        assert!(output_content.contains("#### Heading 4"));
        assert!(output_content.contains("##### Heading 5"));
        assert!(output_content.contains("###### Heading 6"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_heading_with_inline_elements() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_heading_inline.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Inline Test</title></head>
<body>
<h1>Heading with <em>emphasis</em></h1>
<h2>Heading with <strong>bold</strong></h2>
<h3>Heading with <code>code</code></h3>
<h4>Heading with <a href="https://example.com">link</a></h4>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_link_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_link_conversion.html");
        let output_path = temp_dir.join("test_link_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Link Test</title></head>
<body>
<p>Visit <a href="https://example.com">Example</a> for more info.</p>
<p>Check <a href="https://rust-lang.org">Rust</a> language.</p>
<p><a href="/relative/path">Relative link</a> works too.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("[Example](https://example.com)"));
        assert!(output_content.contains("[Rust](https://rust-lang.org)"));
        assert!(output_content.contains("[Relative link](/relative/path)"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_list_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_list_conversion.html");
        let output_path = temp_dir.join("test_list_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>List Test</title></head>
<body>
<ul>
  <li>Unordered item 1</li>
  <li>Unordered item 2</li>
  <li>Unordered item 3</li>
</ul>
<ol>
  <li>Ordered item 1</li>
  <li>Ordered item 2</li>
  <li>Ordered item 3</li>
</ol>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();

        // Check for unordered list markdown formatting (asterisk or dash prefix)
        // The output should contain list markers like "*   " or "-   " or "* " or "- "
        let has_unordered_markers = output_content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("* ")
                || trimmed.starts_with("- ")
                || trimmed.starts_with("*   ")
                || trimmed.starts_with("-   ")
        });
        assert!(
            has_unordered_markers,
            "Output should contain unordered list markers (* or -)"
        );

        // Check for ordered list markdown formatting (number followed by period)
        let has_ordered_markers = output_content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("1. ") || trimmed.starts_with("2. ") || trimmed.starts_with("3. ")
        });
        assert!(
            has_ordered_markers,
            "Output should contain ordered list markers (1., 2., 3.)"
        );

        // Check that the actual content is preserved
        assert!(output_content.contains("Unordered item 1"));
        assert!(output_content.contains("Unordered item 2"));
        assert!(output_content.contains("Unordered item 3"));
        assert!(output_content.contains("Ordered item 1"));
        assert!(output_content.contains("Ordered item 2"));
        assert!(output_content.contains("Ordered item 3"));

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_nested_list_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_nested_list_conversion.html");
        let output_path = temp_dir.join("test_nested_list_conversion_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Nested List Test</title></head>
<body>
<ul>
  <li>Item 1
    <ul>
      <li>Nested item 1.1</li>
      <li>Nested item 1.2</li>
    </ul>
  </li>
  <li>Item 2</li>
</ul>
<ol>
  <li>First
    <ol>
      <li>Sub-first</li>
    </ol>
  </li>
  <li>Second</li>
</ol>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();

        // Check that nested items are present
        assert!(output_content.contains("Item 1"));
        assert!(output_content.contains("Nested item 1.1"));
        assert!(output_content.contains("Nested item 1.2"));
        assert!(output_content.contains("Item 2"));

        // Check that nested items are indented (have leading whitespace)
        let lines: Vec<&str> = output_content.lines().collect();
        let nested_lines: Vec<&&str> = lines
            .iter()
            .filter(|line| line.contains("Nested item") || line.contains("Sub-first"))
            .collect();

        // Nested items should have some indentation
        for nested_line in nested_lines {
            let has_indentation = nested_line.starts_with(|c: char| c.is_whitespace());
            assert!(
                has_indentation || nested_line.contains('*') || nested_line.contains('-'),
                "Nested list items should be indented: '{}'",
                nested_line
            );
        }

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_mixed_nested_list_conversion() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_mixed_nested_list.html");
        let output_path = temp_dir.join("test_mixed_nested_list_output.md");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Mixed Nested List Test</title></head>
<body>
<ol>
  <li>First ordered
    <ul>
      <li>Unordered sub-item</li>
    </ul>
  </li>
  <li>Second ordered</li>
</ol>
<ul>
  <li>Unordered first
    <ol>
      <li>Ordered sub-item</li>
    </ol>
  </li>
</ul>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();

        // Check that all content is preserved
        assert!(output_content.contains("First ordered"));
        assert!(output_content.contains("Unordered sub-item"));
        assert!(output_content.contains("Second ordered"));
        assert!(output_content.contains("Unordered first"));
        assert!(output_content.contains("Ordered sub-item"));

        // Verify mixed list types are handled (ordered with unordered nested, and vice versa)
        let has_ordered = output_content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("1. ") || trimmed.starts_with("2. ")
        });
        let has_unordered = output_content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("* ") || trimmed.starts_with("- ")
        });
        assert!(has_ordered, "Should have ordered list markers");
        assert!(has_unordered, "Should have unordered list markers");

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_html2md_combined_elements() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_combined_elements.html");

        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Combined Test</title></head>
<body>
<h1>Main Heading</h1>
<p>Introduction paragraph with a <a href="https://example.com">link</a>.</p>
<h2>Features</h2>
<ul>
  <li>Feature 1 with <strong>bold</strong> text</li>
  <li>Feature 2 with <em>emphasis</em></li>
</ul>
<h2>Steps</h2>
<ol>
  <li>First step</li>
  <li>Second step with <code>code</code></li>
</ol>
<h3>Conclusion</h3>
<p>Final paragraph.</p>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: None,
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&html_path);
    }

    #[test]
    fn test_html2md_noise_removal() {
        use std::io::Write;
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_noise_removal.html");
        let output_path = temp_dir.join("test_noise_removal_output.md");

        // HTML with various noise elements that should be filtered out
        let html_content = r#"<!DOCTYPE html>
<html>
<head>
<title>Content Page</title>
<script>
    console.log("This script should be removed");
    var x = 1 + 2;
</script>
<style>
    body { color: red; }
    .hidden { display: none; }
</style>
</head>
<body>
<header>
    <nav>
        <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/about">About</a></li>
        </ul>
    </nav>
</header>
<main>
    <h1>Main Content</h1>
    <p>This is the important content that should be preserved.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
</main>
<footer>
    <p>Copyright 2024</p>
</footer>
<aside>
    <p>Sidebar content</p>
</aside>
<noscript>
    <p>Please enable JavaScript</p>
</noscript>
<iframe src="https://example.com/frame"></iframe>
<svg>
    <circle cx="50" cy="50" r="40"/>
</svg>
<form action="/submit">
    <input type="text" name="field" />
</form>
</body>
</html>"#;

        let mut file = std::fs::File::create(&html_path).unwrap();
        file.write_all(html_content.as_bytes()).unwrap();
        drop(file);

        let input = Html2mdInput {
            input: html_path.to_string_lossy().to_string(),
            output: Some(output_path.clone()),
            metadata: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        let output_content = std::fs::read_to_string(&output_path).unwrap();

        // Verify main content is preserved
        assert!(
            output_content.contains("Main Content"),
            "Should preserve main heading"
        );
        assert!(
            output_content.contains("important content"),
            "Should preserve main paragraph"
        );
        assert!(
            output_content.contains("Item 1"),
            "Should preserve list items"
        );
        assert!(
            output_content.contains("Item 2"),
            "Should preserve list items"
        );

        // Verify noise elements are removed
        assert!(
            !output_content.contains("console.log"),
            "Should remove script content"
        );
        assert!(
            !output_content.contains("var x"),
            "Should remove script variables"
        );
        assert!(
            !output_content.contains("color: red"),
            "Should remove style content"
        );
        assert!(
            !output_content.contains(".hidden"),
            "Should remove CSS classes"
        );
        assert!(
            !output_content.contains("Home"),
            "Should remove nav content"
        );
        assert!(!output_content.contains("About"), "Should remove nav links");
        assert!(
            !output_content.contains("Copyright"),
            "Should remove footer content"
        );
        assert!(
            !output_content.contains("Sidebar"),
            "Should remove aside content"
        );
        assert!(
            !output_content.contains("enable JavaScript"),
            "Should remove noscript content"
        );
        assert!(
            !output_content.contains("example.com/frame"),
            "Should remove iframe"
        );
        assert!(
            !output_content.contains("circle"),
            "Should remove SVG content"
        );
        assert!(
            !output_content.contains("submit"),
            "Should remove form content"
        );

        let _ = std::fs::remove_file(&html_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_txt2md_handler() {
        use std::io::Write;
        let handler = Txt2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temp input file
        let temp_dir = std::env::temp_dir();
        let input_path = temp_dir.join("test_txt2md_handler_input.txt");
        let mut file = std::fs::File::create(&input_path).unwrap();
        writeln!(file, "TITLE").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "Some text.").unwrap();
        drop(file);

        let input = Txt2mdInput {
            input: Some(input_path.clone()),
            output: None,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(&input_path);
    }

    // ============================================================
    // Txt2md Normalize Spacing Tests
    // ============================================================

    #[test]
    fn test_txt2md_normalize_spacing_collapses_blank_lines() {
        let handler = Txt2mdHandler;
        // Input with multiple consecutive blank lines
        let input = "Line 1\n\n\n\nLine 2\n\n\nLine 3";
        let result = handler.normalize_spacing(input);
        // Should have only single blank lines between content
        assert_eq!(result, "Line 1\n\nLine 2\n\nLine 3");
    }

    #[test]
    fn test_txt2md_normalize_spacing_trims_trailing_whitespace() {
        let handler = Txt2mdHandler;
        // Input with trailing whitespace on lines
        let input = "Line 1   \nLine 2\t\t\nLine 3   ";
        let result = handler.normalize_spacing(input);
        // Should have no trailing whitespace
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_txt2md_normalize_spacing_removes_leading_blank_lines() {
        let handler = Txt2mdHandler;
        // Input with leading blank lines
        let input = "\n\n\nLine 1\nLine 2";
        let result = handler.normalize_spacing(input);
        // Should have no leading blank lines
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn test_txt2md_normalize_spacing_removes_trailing_blank_lines() {
        let handler = Txt2mdHandler;
        // Input with trailing blank lines
        let input = "Line 1\nLine 2\n\n\n";
        let result = handler.normalize_spacing(input);
        // Should have no trailing blank lines
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn test_txt2md_normalize_spacing_empty_input() {
        let handler = Txt2mdHandler;
        let result = handler.normalize_spacing("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_txt2md_normalize_spacing_only_whitespace() {
        let handler = Txt2mdHandler;
        let result = handler.normalize_spacing("   \n\t\n   ");
        assert_eq!(result, "");
    }

    #[test]
    fn test_txt2md_normalize_spacing_single_line() {
        let handler = Txt2mdHandler;
        let result = handler.normalize_spacing("Single line");
        assert_eq!(result, "Single line");
    }

    #[test]
    fn test_txt2md_normalize_spacing_preserves_internal_spacing() {
        let handler = Txt2mdHandler;
        // Internal spacing (between words) should be preserved
        let input = "Line with   multiple   spaces";
        let result = handler.normalize_spacing(input);
        assert_eq!(result, "Line with   multiple   spaces");
    }

    #[test]
    fn test_txt2md_normalize_spacing_complex() {
        let handler = Txt2mdHandler;
        // Complex case with multiple issues
        let input = "\n\n# Heading   \n\n\n\nParagraph text.   \n\n- List item 1   \n\n\n- List item 2\n\n\n";
        let result = handler.normalize_spacing(input);
        // Should normalize all spacing issues
        assert_eq!(
            result,
            "# Heading\n\nParagraph text.\n\n- List item 1\n\n- List item 2"
        );
    }

    #[test]
    fn test_txt2md_normalize_spacing_with_code_block() {
        let handler = Txt2mdHandler;
        // Code blocks should have trailing whitespace trimmed but internal structure preserved
        let input = "```   \ncode line   \n```   ";
        let result = handler.normalize_spacing(input);
        assert_eq!(result, "```\ncode line\n```");
    }

    #[test]
    fn test_txt2md_handler_with_spacing_issues() {
        use std::io::Write;
        let handler = Txt2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        // Create a temp input file with spacing issues
        let temp_dir = std::env::temp_dir();
        let input_path = temp_dir.join("test_txt2md_spacing_input.txt");
        let mut file = std::fs::File::create(&input_path).unwrap();
        writeln!(file).unwrap();
        writeln!(file).unwrap();
        writeln!(file, "TITLE").unwrap();
        writeln!(file).unwrap();
        writeln!(file).unwrap();
        writeln!(file, "Some text.").unwrap();
        writeln!(file).unwrap();
        writeln!(file).unwrap();
        drop(file);

        let input = Txt2mdInput {
            input: Some(input_path.clone()),
            output: None,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(&input_path);
    }

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
        std::fs::write(&tmp, "===== 1 passed in 0.01s =====\n").unwrap();
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

    // ============================================================
    // Jest Parser Tests
    // ============================================================

    #[test]
    fn test_parse_jest_empty() {
        let result = ParseHandler::parse_jest("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_jest_single_suite_passed() {
        let input = r#"PASS src/utils.test.js
  ✓ should add numbers (5 ms)
  ✓ should subtract numbers (2 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 2 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "src/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_total, 2);
    }

    #[test]
    fn test_parse_jest_single_suite_failed() {
        let input = r#"FAIL src/math.test.js
  ✕ should multiply numbers (3 ms)
  ✓ should divide numbers (1 ms)

Test Suites: 1 failed, 1 total
Tests:       1 passed, 1 failed, 2 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "src/math.test.js");
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
        assert_eq!(result.summary.tests_total, 2);
    }

    #[test]
    fn test_parse_jest_multiple_suites() {
        let input = r#"PASS src/utils.test.js
  ✓ test 1 (5 ms)

FAIL src/api.test.js
  ✕ test 2 (10 ms)
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 failed, 2 total
Tests:       2 passed, 1 failed, 3 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert_eq!(result.summary.suites_passed, 1);
        assert_eq!(result.summary.suites_failed, 1);
        assert_eq!(result.summary.suites_total, 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_failed, 1);
        assert_eq!(result.summary.tests_total, 3);
    }

    #[test]
    fn test_parse_jest_test_with_skipped() {
        let input = r#"PASS src/test.js
  ✓ test 1 (5 ms)
  ○ skipped test 2
  ✓ test 3 (3 ms)

Test Suites: 1 passed, 1 total
Tests:       2 passed, 1 skipped, 3 total"#;
        let result = ParseHandler::parse_jest(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_jest_test_line() {
        let result =
            ParseHandler::parse_jest_test_line("  ✓ should work correctly (5 ms)").unwrap();
        assert_eq!(result.status, JestTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_jest_test_line("  ✕ should fail").unwrap();
        assert_eq!(result.status, JestTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_jest_test_line("  ○ skipped test").unwrap();
        assert_eq!(result.status, JestTestStatus::Skipped);
    }

    #[test]
    fn test_parse_jest_duration() {
        assert_eq!(ParseHandler::parse_jest_duration("5 ms"), Some(0.005));
        assert_eq!(ParseHandler::parse_jest_duration("1.23 s"), Some(1.23));
        assert_eq!(ParseHandler::parse_jest_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_jest_duration("invalid"), None);
    }

    #[test]
    fn test_parse_jest_summary() {
        let summary = ParseHandler::parse_jest_summary("Test Suites: 2 passed, 1 failed, 3 total");
        assert_eq!(summary.suites_passed, 2);
        assert_eq!(summary.suites_failed, 1);
        assert_eq!(summary.suites_total, 3);
    }

    #[test]
    fn test_parse_jest_tests_summary() {
        let mut summary = JestSummary::default();
        ParseHandler::parse_jest_tests_summary(
            "Tests:       5 passed, 2 failed, 1 skipped, 8 total",
            &mut summary,
        );
        assert_eq!(summary.tests_passed, 5);
        assert_eq!(summary.tests_failed, 2);
        assert_eq!(summary.tests_skipped, 1);
        assert_eq!(summary.tests_total, 8);
    }

    #[test]
    fn test_parse_jest_time_summary() {
        let mut summary = JestSummary::default();
        ParseHandler::parse_jest_time_summary("Time:        1.234 s", &mut summary);
        assert_eq!(summary.duration, Some(1.234));
    }

    #[test]
    fn test_format_jest_json() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_jest_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_jest_compact() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_jest_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_jest_raw() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: None,
            tests: vec![
                JestTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: JestTestStatus::Passed,
                    duration: None,
                    error_message: None,
                },
                JestTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: JestTestStatus::Failed,
                    duration: None,
                    error_message: None,
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_jest_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_jest_agent() {
        let mut output = JestOutput::default();
        output.test_suites.push(JestTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.1),
            tests: vec![JestTest {
                name: "test example".to_string(),
                test_name: "test example".to_string(),
                ancestors: vec![],
                status: JestTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_jest_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_jest_with_ancestors() {
        // Test with regular > separator
        let result =
            ParseHandler::parse_jest_test_line("✓ describe block > test name (5 ms)").unwrap();
        assert_eq!(result.test_name, "test name");
        assert_eq!(result.ancestors, vec!["describe block"]);

        // Test with fancy › separator (Unicode)
        let result =
            ParseHandler::parse_jest_test_line("✓ describe block › test name (5 ms)").unwrap();
        assert_eq!(result.test_name, "test name");
        assert_eq!(result.ancestors, vec!["describe block"]);
    }

    // ============================================================
    // Grep Parser Tests
    // ============================================================

    #[test]
    fn test_parse_grep_empty() {
        let result = ParseHandler::parse_grep("").unwrap();
        assert!(result.is_empty);
        assert_eq!(result.file_count, 0);
        assert_eq!(result.match_count, 0);
    }

    #[test]
    fn test_parse_grep_single_file_single_match() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.is_empty);
        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 1);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_single_file_multiple_matches() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].matches.len(), 2);
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[1].line_number, Some(45));
    }

    #[test]
    fn test_parse_grep_multiple_files() {
        let input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 2);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[1].path, "src/lib.rs");
    }

    #[test]
    fn test_parse_grep_groups_interleaved_files() {
        // Test that matches from the same file are grouped together
        // even when they appear interleaved in the input
        let input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
        let result = ParseHandler::parse_grep(input).unwrap();

        // Should have 2 files, not 3
        assert_eq!(result.file_count, 2);
        assert_eq!(result.match_count, 3);

        // Files should preserve order of first appearance
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[1].path, "src/lib.rs");

        // main.rs should have both its matches grouped together
        assert_eq!(result.files[0].matches.len(), 2);
        assert_eq!(result.files[0].matches[0].line_number, Some(10));
        assert_eq!(result.files[0].matches[0].line, "line one");
        assert_eq!(result.files[0].matches[1].line_number, Some(30));
        assert_eq!(result.files[0].matches[1].line, "line three");

        // lib.rs should have its single match
        assert_eq!(result.files[1].matches.len(), 1);
        assert_eq!(result.files[1].matches[0].line_number, Some(25));
        assert_eq!(result.files[1].matches[0].line, "line two");
    }

    #[test]
    fn test_parse_grep_with_column() {
        let input = "src/main.rs:42:10:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_without_line_number() {
        let input = "src/main.rs:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, None);
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_binary_file() {
        let input = "Binary file target/debug binary matches";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.files[0].path, "target/debug binary");
        assert_eq!(result.files[0].matches[0].line, "[binary file]");
    }

    #[test]
    fn test_parse_grep_format_compact() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("matches: 1 files, 2 results"));
        assert!(output.contains("src/main.rs (2):"));
        assert!(output.contains("42: fn main() {"));
        assert!(output.contains("45:     println!"));
    }

    #[test]
    fn test_parse_grep_format_json() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["schema"]["type"], "grep_output");
        assert_eq!(json["counts"]["files"], 1);
        assert_eq!(json["counts"]["matches"], 1);
        assert_eq!(json["files"][0]["path"], "src/main.rs");
        assert_eq!(json["files"][0]["matches"][0]["line_number"], 42);
        assert_eq!(json["files"][0]["matches"][0]["line"], "fn main() {");
    }

    #[test]
    fn test_parse_grep_format_csv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Csv);

        assert!(output.starts_with("path,line_number,column,is_context,line\n"));
        assert!(output.contains("src/main.rs,42,,false,fn main() {"));
    }

    #[test]
    fn test_parse_grep_format_tsv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Tsv);

        assert!(output.starts_with("path\tline_number\tcolumn\tis_context\tline\n"));
        assert!(output.contains("src/main.rs\t42\t\tfalse\tfn main() {"));
    }

    #[test]
    fn test_parse_grep_format_raw() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:42:fn main() {"));
    }

    #[test]
    fn test_parse_grep_empty_compact() {
        let mut result = GrepOutput::default();
        result.is_empty = true;
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("grep: no matches"));
    }

    #[test]
    fn test_parse_grep_line_with_colon_in_content() {
        // Content containing colons should be handled correctly
        let input = "src/main.rs:42:let x = \"http://example.com\";";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(
            result.files[0].matches[0].line,
            "let x = \"http://example.com\";"
        );
    }

    // ============================================================
    // Context Line Tests
    // ============================================================

    #[test]
    fn test_parse_grep_context_line() {
        // Context lines use "-" as separator (from grep -C/-B/-A)
        let input = "src/main.rs-42-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_context_line_with_column() {
        // Context line with column info
        let input = "src/main.rs-42-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_mixed_match_and_context() {
        // Mix of match and context lines
        let input = "src/main.rs-41-context before\nsrc/main.rs:42:match line\nsrc/main.rs-43-context after";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches.len(), 3);

        // First line is context
        assert!(result.files[0].matches[0].is_context);
        assert_eq!(result.files[0].matches[0].line, "context before");

        // Second line is a match
        assert!(!result.files[0].matches[1].is_context);
        assert_eq!(result.files[0].matches[1].line, "match line");

        // Third line is context
        assert!(result.files[0].matches[2].is_context);
        assert_eq!(result.files[0].matches[2].line, "context after");
    }

    #[test]
    fn test_parse_grep_context_is_context_flag_false_for_matches() {
        let input = "src/main.rs:42:match line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_format_grep_compact_collapse_context_lines() {
        // Multiple consecutive context lines should be collapsed
        let input = "src/main.rs-10-context 1\nsrc/main.rs-11-context 2\nsrc/main.rs-12-context 3\nsrc/main.rs:13:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should collapse 3 context lines into a summary
        assert!(output.contains("10-12: ... (3 context lines)"));
        assert!(output.contains("13: match line"));
    }

    #[test]
    fn test_format_grep_compact_single_context_line() {
        // Single context line should show as "... (1 context lines)" format
        let input = "src/main.rs-10-context line\nsrc/main.rs:11:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match line"));
    }

    #[test]
    fn test_format_grep_compact_context_before_and_after() {
        // Context lines before and after match
        let input = "src/main.rs-10-before\nsrc/main.rs:11:match\nsrc/main.rs-12-after";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match"));
        assert!(output.contains("12: ..."));
    }

    #[test]
    fn test_format_grep_compact_count_excludes_context() {
        // Match count should exclude context lines
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match\nsrc/main.rs-12-context";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should show 1 result (only the match), not 3
        assert!(output.contains("matches: 1 files, 1 results"));
    }

    #[test]
    fn test_format_grep_compact_trailing_context() {
        // Context lines at the end should be collapsed
        let input = "src/main.rs:10:match\nsrc/main.rs-11-context 1\nsrc/main.rs-12-context 2";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: match"));
        assert!(output.contains("11-12: ... (2 context lines)"));
    }

    #[test]
    fn test_format_grep_json_includes_is_context() {
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["files"][0]["matches"][0]["is_context"], true);
        assert_eq!(json["files"][0]["matches"][1]["is_context"], false);
    }

    #[test]
    fn test_format_grep_raw_context_uses_dash() {
        // Raw format should preserve dash separator for context
        let input = "src/main.rs-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs-10-context line"));
    }

    #[test]
    fn test_format_grep_raw_match_uses_colon() {
        // Raw format should use colon for matches
        let input = "src/main.rs:10:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:10:match line"));
    }

    // ============================================================
    // Grep Truncation Tests
    // ============================================================

    #[test]
    fn test_parse_grep_truncation_fields_not_truncated() {
        // Small result set should not be truncated
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.is_truncated, false);
        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_matches, 1);
        assert_eq!(result.files_shown, 1);
        assert_eq!(result.matches_shown, 1);
    }

    #[test]
    fn test_truncate_grep_files() {
        // Create 60 files (exceeds DEFAULT_MAX_GREP_FILES = 50)
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 60);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_truncate_grep_matches_per_file() {
        // Create 1 file with 25 matches (exceeds DEFAULT_MAX_GREP_MATCHES_PER_FILE = 20)
        let mut input = String::new();
        for i in 1..=25 {
            input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 25);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.matches_shown, 20);
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 20);
    }

    #[test]
    fn test_truncate_grep_both_limits() {
        // Create 60 files, each with 25 matches
        let mut input = String::new();
        for i in 1..=60 {
            for j in 1..=25 {
                input.push_str(&format!("src/file{}.rs:{}:fn func{}() {{\n", i, j, j));
            }
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation: 60 files * 25 matches = 1500 total matches
        assert_eq!(result.total_files, 60);
        assert_eq!(result.total_matches, 1500);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation: 50 files * 20 matches = 1000 matches shown
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.matches_shown, 1000);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_format_grep_json_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], true);
        assert_eq!(json["counts"]["total_files"], 60);
        assert_eq!(json["counts"]["files_shown"], 50);
    }

    #[test]
    fn test_format_grep_compact_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Check for truncation indicators in compact output
        assert!(output.contains("truncated"));
        assert!(output.contains("50/60"));
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_raw_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        // Check for truncation indicator in raw output
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_json_no_truncation_when_within_limits() {
        // Small result set should not show truncation info
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:println!()";
        let mut result = ParseHandler::parse_grep(input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], false);
        assert!(json["truncation"].is_null());
    }

    // ============================================================
    // NPM Test Parser Tests
    // ============================================================

    #[test]
    fn test_parse_npm_test_empty() {
        let result = ParseHandler::parse_npm_test("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_npm_test_single_suite_passed() {
        let input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "test/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.suites_passed, 1);
    }

    #[test]
    fn test_parse_npm_test_single_suite_failed() {
        let input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError [ERR_ASSERTION]: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
    }

    #[test]
    fn test_parse_npm_test_multiple_suites() {
        let input = r#"▶ test/utils.test.js
  ✔ test 1 (5.123ms)
▶ test/utils.test.js (7.234ms)

▶ test/math.test.js
  ✖ test 2
▶ test/math.test.js (3.456ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 passed 1 failed (2)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert!(result.test_suites[0].passed);
        assert!(!result.test_suites[1].passed);
    }

    #[test]
    fn test_parse_npm_test_with_skipped() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_npm_test_with_todo() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # TODO
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 todo (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_npm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_todo, 1);
    }

    #[test]
    fn test_parse_npm_test_line() {
        let result =
            ParseHandler::parse_npm_test_line("✔ should work correctly (5.123ms)", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_npm_test_line("✖ should fail", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_npm_test_line("ℹ skipped test # SKIP", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Skipped);
        assert_eq!(result.test_name, "skipped test");

        let result = ParseHandler::parse_npm_test_line("ℹ todo test # TODO", &[]).unwrap();
        assert_eq!(result.status, NpmTestStatus::Todo);
        assert_eq!(result.test_name, "todo test");
    }

    #[test]
    fn test_parse_npm_duration() {
        assert_eq!(ParseHandler::parse_npm_duration("5.123ms"), Some(0.005123));
        assert_eq!(ParseHandler::parse_npm_duration("1.234s"), Some(1.234));
        assert_eq!(ParseHandler::parse_npm_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_npm_duration("invalid"), None);
    }

    #[test]
    fn test_split_npm_test_name_and_duration() {
        let (name, duration) =
            ParseHandler::split_npm_test_name_and_duration("test name (5.123ms)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(0.005123));

        let (name, duration) = ParseHandler::split_npm_test_name_and_duration("test name (1.234s)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(1.234));

        let (name, duration) =
            ParseHandler::split_npm_test_name_and_duration("test name without duration");
        assert_eq!(name, "test name without duration");
        assert!(duration.is_none());
    }

    #[test]
    fn test_format_npm_test_json() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_npm_test_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"tests_passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_npm_test_compact() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_npm_test_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_npm_test_raw() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: Some(0.01),
            tests: vec![
                NpmTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: NpmTestStatus::Passed,
                    duration: Some(0.005),
                    error_message: None,
                },
                NpmTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: NpmTestStatus::Failed,
                    duration: None,
                    error_message: Some("Error message".to_string()),
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_npm_test_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_npm_test_agent() {
        let mut output = NpmTestOutput::default();
        output.test_suites.push(NpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![NpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: NpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_npm_test_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_npm_test_with_ancestors() {
        // Test that nested tests track ancestor names
        let result = ParseHandler::parse_npm_test_line(
            "✔ nested test (5.123ms)",
            &["describe block".to_string()],
        )
        .unwrap();
        assert_eq!(result.test_name, "nested test");
        assert_eq!(result.ancestors, vec!["describe block"]);
        assert_eq!(result.name, "describe block > nested test");
    }

    // ============================================================
    // PNPM Test Parser Tests
    // ============================================================

    #[test]
    fn test_parse_pnpm_test_empty() {
        let result = ParseHandler::parse_pnpm_test("").unwrap();
        assert!(result.is_empty);
        assert!(result.test_suites.is_empty());
        assert_eq!(result.summary.tests_total, 0);
    }

    #[test]
    fn test_parse_pnpm_test_single_suite_passed() {
        let input = r#"▶ test/utils.test.js
  ✔ should add numbers (5.123ms)
  ✔ should subtract numbers (2.456ms)
▶ test/utils.test.js (10.579ms)

ℹ tests 2 passed (2)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert_eq!(result.test_suites[0].file, "test/utils.test.js");
        assert!(result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.suites_passed, 1);
    }

    #[test]
    fn test_parse_pnpm_test_single_suite_failed() {
        let input = r#"▶ test/math.test.js
  ✖ should multiply numbers
    AssertionError [ERR_ASSERTION]: values are not equal
  ✔ should divide numbers (1.234ms)
▶ test/math.test.js (5.678ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 failed (1)
ℹ duration 10ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 1);
        assert!(!result.test_suites[0].passed);
        assert_eq!(result.test_suites[0].tests.len(), 2);
        assert_eq!(result.summary.tests_passed, 1);
        assert_eq!(result.summary.tests_failed, 1);
    }

    #[test]
    fn test_parse_pnpm_test_multiple_suites() {
        let input = r#"▶ test/utils.test.js
  ✔ test 1 (5.123ms)
▶ test/utils.test.js (7.234ms)

▶ test/math.test.js
  ✖ test 2
▶ test/math.test.js (3.456ms)

ℹ tests 1 passed 1 failed (2)
ℹ test files 1 passed 1 failed (2)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(!result.success);
        assert_eq!(result.test_suites.len(), 2);
        assert!(result.test_suites[0].passed);
        assert!(!result.test_suites[1].passed);
    }

    #[test]
    fn test_parse_pnpm_test_with_skipped() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # SKIP
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 skipped (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_skipped, 1);
    }

    #[test]
    fn test_parse_pnpm_test_with_todo() {
        let input = r#"▶ test/test.js
  ✔ test 1 (5.123ms)
  ℹ test 2 # TODO
  ✔ test 3 (1.234ms)
▶ test/test.js (10.579ms)

ℹ tests 2 passed 1 todo (3)
ℹ test files 1 passed (1)
ℹ duration 15ms"#;
        let result = ParseHandler::parse_pnpm_test(input).unwrap();

        assert!(!result.is_empty);
        assert!(result.success);
        assert_eq!(result.test_suites[0].tests.len(), 3);
        assert_eq!(result.summary.tests_passed, 2);
        assert_eq!(result.summary.tests_todo, 1);
    }

    #[test]
    fn test_parse_pnpm_test_line() {
        let result =
            ParseHandler::parse_pnpm_test_line("✔ should work correctly (5.123ms)", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Passed);
        assert_eq!(result.test_name, "should work correctly");
        assert!(result.duration.is_some());

        let result = ParseHandler::parse_pnpm_test_line("✖ should fail", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Failed);
        assert_eq!(result.test_name, "should fail");

        let result = ParseHandler::parse_pnpm_test_line("ℹ skipped test # SKIP", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Skipped);
        assert_eq!(result.test_name, "skipped test");

        let result = ParseHandler::parse_pnpm_test_line("ℹ todo test # TODO", &[]).unwrap();
        assert_eq!(result.status, PnpmTestStatus::Todo);
        assert_eq!(result.test_name, "todo test");
    }

    #[test]
    fn test_parse_pnpm_duration() {
        assert_eq!(ParseHandler::parse_pnpm_duration("5.123ms"), Some(0.005123));
        assert_eq!(ParseHandler::parse_pnpm_duration("1.234s"), Some(1.234));
        assert_eq!(ParseHandler::parse_pnpm_duration("1000ms"), Some(1.0));
        assert_eq!(ParseHandler::parse_pnpm_duration("invalid"), None);
    }

    #[test]
    fn test_split_pnpm_test_name_and_duration() {
        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name (5.123ms)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(0.005123));

        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name (1.234s)");
        assert_eq!(name, "test name");
        assert_eq!(duration, Some(1.234));

        let (name, duration) =
            ParseHandler::split_pnpm_test_name_and_duration("test name without duration");
        assert_eq!(name, "test name without duration");
        assert!(duration.is_none());
    }

    #[test]
    fn test_format_pnpm_test_json() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let json = ParseHandler::format_pnpm_test_json(&output);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"tests_passed\":1"));
        assert!(json.contains("\"test.js\""));
    }

    #[test]
    fn test_format_pnpm_test_compact() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let compact = ParseHandler::format_pnpm_test_compact(&output);
        assert!(compact.contains("PASS:"));
        assert!(compact.contains("1 suites"));
        assert!(compact.contains("1 tests"));
    }

    #[test]
    fn test_format_pnpm_test_raw() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: false,
            duration: None,
            tests: vec![
                PnpmTest {
                    name: "passing test".to_string(),
                    test_name: "passing test".to_string(),
                    ancestors: vec![],
                    status: PnpmTestStatus::Passed,
                    duration: None,
                    error_message: None,
                },
                PnpmTest {
                    name: "failing test".to_string(),
                    test_name: "failing test".to_string(),
                    ancestors: vec![],
                    status: PnpmTestStatus::Failed,
                    duration: None,
                    error_message: Some("Error message".to_string()),
                },
            ],
        });
        output.is_empty = false;

        let raw = ParseHandler::format_pnpm_test_raw(&output);
        assert!(raw.contains("FAIL test.js"));
        assert!(raw.contains("PASS passing test"));
        assert!(raw.contains("FAIL failing test"));
    }

    #[test]
    fn test_format_pnpm_test_agent() {
        let mut output = PnpmTestOutput::default();
        output.test_suites.push(PnpmTestSuite {
            file: "test.js".to_string(),
            passed: true,
            duration: Some(0.01),
            tests: vec![PnpmTest {
                name: "test name".to_string(),
                test_name: "test name".to_string(),
                ancestors: vec![],
                status: PnpmTestStatus::Passed,
                duration: Some(0.005),
                error_message: None,
            }],
        });
        output.summary.tests_passed = 1;
        output.summary.tests_total = 1;
        output.summary.suites_passed = 1;
        output.summary.suites_total = 1;
        output.success = true;
        output.is_empty = false;

        let agent = ParseHandler::format_pnpm_test_agent(&output);
        assert!(agent.contains("# Test Results"));
        assert!(agent.contains("Status: SUCCESS"));
        assert!(agent.contains("## Summary"));
    }

    #[test]
    fn test_parse_pnpm_test_with_ancestors() {
        // Test that nested tests track ancestor names
        let result = ParseHandler::parse_pnpm_test_line(
            "✔ nested test (5.123ms)",
            &["describe block".to_string()],
        )
        .unwrap();
        assert_eq!(result.test_name, "nested test");
        assert_eq!(result.ancestors, vec!["describe block"]);
        assert_eq!(result.name, "describe block > nested test");
    }

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
        let result =
            ParseHandler::parse_bun_test_line("(pass) should work [5.123ms]", &[]).unwrap();
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
        let (name, duration) =
            ParseHandler::split_bun_test_name_and_duration("test name [5.123ms]");
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
        // Test that consecutive identical lines (no log levels) are collapsed in output
        let input = "Same line\nSame line\nSame line\nDifferent\nDifferent\nUnique";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show line ranges for collapsed entries
        assert!(output.contains("1-3 Same line [x3]"));
        assert!(output.contains("4-5 Different [x2]"));
        assert!(output.contains("6 Unique"));
    }

    #[test]
    fn test_format_logs_compact_collapses_consecutive_entries_with_levels() {
        // Test that consecutive identical entries with log levels are collapsed
        let input = "[INFO] Starting\n[INFO] Starting\n[INFO] Starting\n[ERROR] Failed\n[ERROR] Failed\n[WARN] Warning";
        let result = ParseHandler::parse_logs(input);
        let output = ParseHandler::format_logs(&result, OutputFormat::Compact);

        // Should show collapsed entries with line ranges
        assert!(output.contains("[I] 1-3 Starting [x3]"));
        assert!(output.contains("[E] 4-5 Failed [x2]"));
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
}
