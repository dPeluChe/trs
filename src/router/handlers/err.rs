//! Error filter handler for `trs err` command.
//!
//! Runs a command and filters output to show only lines containing
//! errors, warnings, panics, and fatal messages.

use super::common::{escape_csv_field, CommandContext, CommandError, CommandResult};
use super::run::RunHandler;
use crate::OutputFormat;

/// Check if a line looks like an error or warning.
fn is_error_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Check for prefix patterns
    if trimmed.starts_with("E ")
        || trimmed.starts_with("W ")
        || trimmed.starts_with("[E]")
        || trimmed.starts_with("[W]")
        || trimmed.starts_with("[ERROR]")
        || trimmed.starts_with("[WARN]")
    {
        return true;
    }

    // Check for keyword patterns (case-sensitive variants)
    let keywords = [
        "error", "Error", "ERROR", "warn", "Warn", "WARNING", "WARN", "failed", "FAILED", "panic",
        "PANIC", "fatal", "FATAL",
    ];

    for kw in &keywords {
        if line.contains(kw) {
            return true;
        }
    }

    false
}

/// Classify a matching line as error or warning for summary counting.
fn is_error_not_warning(line: &str) -> bool {
    let trimmed = line.trim();

    // Prefix-based classification
    if trimmed.starts_with("E ") || trimmed.starts_with("[E]") || trimmed.starts_with("[ERROR]") {
        return true;
    }
    if trimmed.starts_with("W ") || trimmed.starts_with("[W]") || trimmed.starts_with("[WARN]") {
        return false;
    }

    // Keyword-based classification
    let error_keywords = [
        "error", "Error", "ERROR", "failed", "FAILED", "panic", "PANIC", "fatal", "FATAL",
    ];
    let warning_keywords = ["warn", "Warn", "WARNING", "WARN"];

    for kw in &error_keywords {
        if line.contains(kw) {
            return true;
        }
    }
    for kw in &warning_keywords {
        if line.contains(kw) {
            return false;
        }
    }

    true // default to error
}

/// A shown line in text: context sits under the line it belongs to.
fn indented(kind: &str, line: &str) -> String {
    match kind {
        "context" => format!("  {line}"),
        _ => line.to_string(),
    }
}

/// The shown lines as a `kind,line` table.
fn table(shown: &[(&str, String)], separator: char, escape: fn(&str) -> String) -> String {
    let mut table = format!("kind{separator}line\n");
    for (kind, line) in shown {
        table.push_str(&format!("{kind}{separator}{}\n", escape(line)));
    }
    table
}

fn count(shown: &[(&str, String)], kind: &str) -> usize {
    shown.iter().filter(|(k, _)| *k == kind).count()
}

fn format_output(
    shown: &[(&str, String)],
    total_lines: usize,
    failed: bool,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Csv => table(shown, ',', escape_csv_field),
        OutputFormat::Tsv => table(shown, '\t', RunHandler::escape_tsv_field),
        OutputFormat::Json | OutputFormat::Compact | OutputFormat::Agent | OutputFormat::Raw => {
            let mut text = String::new();
            if shown.is_empty() && !failed {
                text.push_str("clean: no errors or warnings\n");
            }
            for (kind, line) in shown {
                text.push_str(&indented(kind, line));
                text.push('\n');
            }
            let (errors, warnings) = (count(shown, "error"), count(shown, "warning"));
            if errors + warnings > 0 {
                text.push_str(&format!(
                    "\n{errors} errors, {warnings} warnings found in {total_lines} lines\n"
                ));
            }
            text
        }
    }
}

/// Execute a command and show only error/warning lines.
pub(crate) fn handle_err(command: &str, args: &[String], ctx: &CommandContext) -> CommandResult {
    let start = std::time::Instant::now();
    let output = match crate::classifier::build_command(command, args).output() {
        Ok(o) => o,
        Err(e) => {
            let (code, message) = crate::process::spawn_failure(command, &e);
            return Err(CommandError::ExecutionError {
                message,
                exit_code: Some(code),
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Combine stdout and stderr lines
    let all_lines: Vec<&str> = stdout.lines().chain(stderr.lines()).collect();

    let total_lines = all_lines.len();
    let mut matched_lines: Vec<(&str, String)> = Vec::new();

    let mut i = 0;
    while i < all_lines.len() {
        let line = all_lines[i];
        if is_error_line(line) {
            let kind = if is_error_not_warning(line) {
                "error"
            } else {
                "warning"
            };
            matched_lines.push((kind, line.to_string()));

            // Include 1 line of context after the error (for stack traces)
            if i + 1 < all_lines.len() {
                let next = all_lines[i + 1];
                // Only include context if the next line is NOT itself an error line
                // (avoids duplication when errors are consecutive)
                if !is_error_line(next) {
                    matched_lines.push(("context", next.to_string()));
                    i += 1;
                }
            }
        }
        i += 1;
    }

    let code = crate::process::exit_code_of(&output.status);
    let failed = !output.status.success();
    let shown = if matched_lines.is_empty() && failed {
        // Failed with nothing the keywords caught: never hide that behind "clean".
        let text = all_lines.join("\n");
        let max = crate::config::config().limits.passthrough_max_chars;
        let text = crate::passthrough_cut::cut(&text, max).unwrap_or(text);
        text.lines().map(|l| ("output", l.to_string())).collect()
    } else {
        matched_lines
    };

    if ctx.format == OutputFormat::Json {
        let report = serde_json::json!({
            "success": !failed,
            "errors": count(&shown, "error"),
            "warnings": count(&shown, "warning"),
            "total_lines": total_lines,
            "lines": shown.iter().map(|(kind, line)| indented(kind, line)).collect::<Vec<_>>(),
        })
        .to_string();
        crate::exec_verdict::JsonRun {
            fcmd: crate::classifier::full_cmd(command, args),
            stdout: &stdout,
            stderr: &stderr,
            unread_stderr: "",
            code,
            in_bytes: output.stdout.len() + output.stderr.len(),
            start,
            status: &output.status,
        }
        .finish(&report, false);
    }

    print!("{}", format_output(&shown, total_lines, failed, ctx.format));

    // Propagate exit code from the wrapped command
    if failed {
        return Err(CommandError::ExecutionError {
            message: format!("Command '{}' exited with code {}", command, code),
            exit_code: Some(code),
        });
    }

    Ok(())
}
