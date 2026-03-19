//! Error filter handler for `trs err` command.
//!
//! Runs a command and filters output to show only lines containing
//! errors, warnings, panics, and fatal messages.

use super::common::{CommandContext, CommandError, CommandResult};

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

/// Execute a command and show only error/warning lines.
pub(crate) fn handle_err(command: &str, args: &[String], _ctx: &CommandContext) -> CommandResult {
    use std::process::{Command, Stdio};

    let output = match Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(CommandError::ExecutionError {
                    message: format!("Command not found: {}", command),
                    exit_code: Some(127),
                });
            }
            return Err(CommandError::IoError(format!(
                "Failed to execute '{}': {}",
                command, e
            )));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Combine stdout and stderr lines
    let all_lines: Vec<&str> = stdout.lines().chain(stderr.lines()).collect();

    let total_lines = all_lines.len();
    let mut matched_lines: Vec<String> = Vec::new();
    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    let mut i = 0;
    while i < all_lines.len() {
        let line = all_lines[i];
        if is_error_line(line) {
            if is_error_not_warning(line) {
                error_count += 1;
            } else {
                warning_count += 1;
            }
            matched_lines.push(line.to_string());

            // Include 1 line of context after the error (for stack traces)
            if i + 1 < all_lines.len() {
                let next = all_lines[i + 1];
                // Only include context if the next line is NOT itself an error line
                // (avoids duplication when errors are consecutive)
                if !is_error_line(next) {
                    matched_lines.push(format!("  {}", next));
                    i += 1;
                }
            }
        }
        i += 1;
    }

    if matched_lines.is_empty() {
        println!("clean: no errors or warnings");
    } else {
        for line in &matched_lines {
            println!("{}", line);
        }
        println!(
            "\n{} errors, {} warnings found in {} lines",
            error_count, warning_count, total_lines
        );
    }

    // Propagate exit code from the wrapped command
    if !output.status.success() {
        return Err(CommandError::ExecutionError {
            message: format!(
                "Command '{}' exited with code {}",
                command,
                output.status.code().unwrap_or(1)
            ),
            exit_code: output.status.code(),
        });
    }

    Ok(())
}
