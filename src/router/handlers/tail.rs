use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::OutputFormat;

pub(crate) struct TailHandler;

/// A single line in tail output.
#[derive(Debug, Clone)]
pub(crate) struct TailLine {
    /// Line number (1-indexed).
    line_number: usize,
    /// The line content.
    line: String,
    /// Whether this line is an error line.
    is_error: bool,
}

/// Parsed tail output.
#[derive(Debug, Clone)]
pub(crate) struct TailOutput {
    /// The file being tailed.
    file: std::path::PathBuf,
    /// List of lines.
    lines: Vec<TailLine>,
    /// Total lines read.
    total_lines: usize,
    /// Lines shown (after filtering).
    lines_shown: usize,
    /// Whether filtering is active.
    filtering_errors: bool,
    /// Total bytes read from file (original output size).
    input_bytes: usize,
}

impl TailHandler {
    /// Read the last N lines from a file.
    pub(crate) fn read_tail_lines(&self, input: &TailInput) -> CommandResult<TailOutput> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        // Check if file exists
        if !input.file.exists() {
            return Err(CommandError::IoError(format!(
                "File not found: {}",
                input.file.display()
            )));
        }

        let file = File::open(&input.file).map_err(|e| {
            CommandError::IoError(format!(
                "Failed to open file {}: {}",
                input.file.display(),
                e
            ))
        })?;

        let reader = BufReader::new(file);
        let mut all_lines: Vec<String> = Vec::new();

        // Read all lines
        for line in reader.lines() {
            let line =
                line.map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))?;
            all_lines.push(line);
        }

        let total_lines = all_lines.len();

        // Get the last N lines (or all if file is smaller)
        let start = if all_lines.len() > input.lines {
            all_lines.len() - input.lines
        } else {
            0
        };

        let tail_lines: Vec<String> = all_lines[start..].to_vec();

        // Calculate input_bytes (total bytes of all lines read)
        let input_bytes: usize = all_lines.iter().map(|l| l.len()).sum();

        // Process lines
        let mut result_lines: Vec<TailLine> = Vec::new();
        let mut line_number = start + 1; // 1-indexed

        for line in tail_lines {
            let is_error = Self::is_error_line(&line);

            // If filtering for errors, skip non-error lines
            if input.errors && !is_error {
                line_number += 1;
                continue;
            }

            result_lines.push(TailLine {
                line_number,
                line,
                is_error,
            });

            line_number += 1;
        }

        let lines_shown = result_lines.len();

        Ok(TailOutput {
            file: input.file.clone(),
            lines: result_lines,
            total_lines,
            lines_shown,
            filtering_errors: input.errors,
            input_bytes,
        })
    }

    /// Check if a line is an error line.
    pub(crate) fn is_error_line(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        let line_trimmed = line.trim();

        // Check for common error patterns
        line_lower.contains("error")
            || line_lower.contains("exception")
            || line_lower.contains("fatal")
            || line_lower.contains("critical")
            || line_lower.contains("failed")
            || line_lower.contains("failure")
            || line_trimmed.starts_with("E/")
            || line_trimmed.starts_with("E:")
            || line_trimmed.starts_with("ERR")
            || line_trimmed.starts_with("[ERROR]")
            || line_trimmed.starts_with("[FATAL]")
            || line_trimmed.starts_with("[CRITICAL]")
            || line_trimmed.starts_with("ERROR:")
            || line_trimmed.starts_with("FATAL:")
            || line_trimmed.starts_with("CRITICAL:")
    }

    /// Stream new lines from a file (follow mode).
    pub(crate) fn stream_tail_lines(
        &self,
        input: &TailInput,
        last_line_count: usize,
        ctx: &CommandContext,
    ) -> CommandResult<()> {
        use std::io::{BufRead, BufReader};
        use std::time::Duration;

        // Open file for streaming
        let file = std::fs::File::open(&input.file).map_err(|e| {
            CommandError::IoError(format!(
                "Failed to open file {}: {}",
                input.file.display(),
                e
            ))
        })?;

        let mut reader = BufReader::new(file);
        let mut line_number = last_line_count + 1;

        // Skip to the position we've already read
        for _ in 0..last_line_count {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))?;
        }

        // Continuously poll for new lines
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // No new data, wait and retry
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Ok(_) => {
                    // New line available
                    let line = line.trim_end_matches('\n').trim_end_matches('\r');
                    let is_error = Self::is_error_line(line);

                    // If filtering for errors, skip non-error lines
                    if input.errors && !is_error {
                        line_number += 1;
                        continue;
                    }

                    // Format and print the line
                    let tail_line = TailLine {
                        line_number,
                        line: line.to_string(),
                        is_error,
                    };

                    let formatted = Self::format_streaming_line(&tail_line, ctx.format);
                    print!("{}", formatted);
                    std::io::Write::flush(&mut std::io::stdout()).map_err(|e| {
                        CommandError::IoError(format!("Failed to flush stdout: {}", e))
                    })?;

                    line_number += 1;
                }
                Err(e) => {
                    return Err(CommandError::IoError(format!("Failed to read file: {}", e)));
                }
            }
        }
    }

    /// Format a single line for streaming output.
    pub(crate) fn format_streaming_line(line: &TailLine, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => {
                serde_json::json!({
                    "line_number": line.line_number,
                    "line": line.line,
                    "is_error": line.is_error,
                })
                .to_string()
                    + "\n"
            }
            OutputFormat::Csv => {
                let line_escaped = Self::escape_csv_field(&line.line);
                format!("{},{},{}\n", line.line_number, line_escaped, line.is_error)
            }
            OutputFormat::Tsv => {
                let line_escaped = Self::escape_tsv_field(&line.line);
                format!(
                    "{}\t{}\t{}\n",
                    line.line_number, line_escaped, line.is_error
                )
            }
            OutputFormat::Agent => {
                if line.is_error {
                    format!("❌ {}:{}\n", line.line_number, line.line)
                } else {
                    format!("   {}:{}\n", line.line_number, line.line)
                }
            }
            OutputFormat::Compact => {
                if line.is_error {
                    format!("  ❌ {}:{}\n", line.line_number, line.line)
                } else {
                    format!("  {}:{}\n", line.line_number, line.line)
                }
            }
            OutputFormat::Raw => {
                format!("{}:{}\n", line.line_number, line.line)
            }
        }
    }

    /// Format tail output based on the specified format.
    pub(crate) fn format_output(output: &TailOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(output),
            OutputFormat::Csv => Self::format_csv(output),
            OutputFormat::Tsv => Self::format_tsv(output),
            OutputFormat::Agent => Self::format_agent(output),
            OutputFormat::Compact => Self::format_compact(output),
            OutputFormat::Raw => Self::format_raw(output),
        }
    }

    /// Format tail output as JSON.
    pub(crate) fn format_json(output: &TailOutput) -> String {
        let lines_json: Vec<serde_json::Value> = output
            .lines
            .iter()
            .map(|l| {
                serde_json::json!({
                    "line_number": l.line_number,
                    "line": l.line,
                    "is_error": l.is_error,
                })
            })
            .collect();

        serde_json::json!({
            "file": output.file.display().to_string(),
            "lines": lines_json,
            "total_lines": output.total_lines,
            "lines_shown": output.lines_shown,
            "filtering_errors": output.filtering_errors,
        })
        .to_string()
    }

    /// Format tail output as CSV.
    pub(crate) fn format_csv(output: &TailOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,line,is_error\n");

        for l in &output.lines {
            let line_escaped = Self::escape_csv_field(&l.line);
            result.push_str(&format!(
                "{},{},{}\n",
                l.line_number, line_escaped, l.is_error
            ));
        }

        result
    }

    /// Format tail output as TSV.
    pub(crate) fn format_tsv(output: &TailOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tline\tis_error\n");

        for l in &output.lines {
            let line_escaped = Self::escape_tsv_field(&l.line);
            result.push_str(&format!(
                "{}\t{}\t{}\n",
                l.line_number, line_escaped, l.is_error
            ));
        }

        result
    }

    /// Format tail output as agent-optimized format.
    pub(crate) fn format_agent(output: &TailOutput) -> String {
        let mut result = String::new();

        result.push_str(&format!("File: {}\n", output.file.display()));

        if output.filtering_errors {
            result.push_str(&format!(
                "Error lines: {} of {} total\n\n",
                output.lines_shown, output.total_lines
            ));
        } else {
            result.push_str(&format!(
                "Lines: {} of {} total\n\n",
                output.lines_shown, output.total_lines
            ));
        }

        for l in &output.lines {
            if l.is_error {
                result.push_str(&format!("❌ {}:{}\n", l.line_number, l.line));
            } else {
                result.push_str(&format!("   {}:{}\n", l.line_number, l.line));
            }
        }

        result
    }

    /// Format tail output as compact.
    pub(crate) fn format_compact(output: &TailOutput) -> String {
        let mut result = String::new();

        if output.lines.is_empty() {
            if output.filtering_errors {
                result.push_str("No error lines found.\n");
            } else {
                result.push_str("File is empty.\n");
            }
            return result;
        }

        // Show header
        if output.filtering_errors {
            result.push_str(&format!(
                "Error lines from {} ({} of {} total):\n\n",
                output.file.display(),
                output.lines_shown,
                output.total_lines
            ));
        } else {
            result.push_str(&format!(
                "Last {} lines from {} (total: {}):\n\n",
                output.lines_shown,
                output.file.display(),
                output.total_lines
            ));
        }

        for l in &output.lines {
            if l.is_error {
                result.push_str(&format!("  ❌ {}:{}\n", l.line_number, l.line));
            } else {
                result.push_str(&format!("  {}:{}\n", l.line_number, l.line));
            }
        }

        result
    }

    /// Format tail output as raw.
    pub(crate) fn format_raw(output: &TailOutput) -> String {
        let mut result = String::new();

        for l in &output.lines {
            result.push_str(&format!("{}:{}\n", l.line_number, l.line));
        }

        result
    }

    /// Escape a field for CSV format.
    pub(crate) fn escape_csv_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Escape a field for TSV format.
    pub(crate) fn escape_tsv_field(field: &str) -> String {
        field
            .replace('\t', "\\t")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }
}

impl CommandHandler for TailHandler {
    type Input = TailInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read initial tail lines
        let tail_output = self.read_tail_lines(input)?;

        // Format and print initial output
        let formatted = Self::format_output(&tail_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("tail")
                .with_output_mode(ctx.format)
                .with_input_bytes(tail_output.input_bytes)
                .with_items_processed(tail_output.lines_shown)
                .with_items_filtered(if tail_output.filtering_errors {
                    tail_output
                        .total_lines
                        .saturating_sub(tail_output.lines_shown)
                } else {
                    0
                })
                .with_output_bytes(formatted.len())
                .with_extra("Total lines", tail_output.total_lines.to_string())
                .with_extra("Lines shown", tail_output.lines_shown.to_string())
                .with_extra("Filtering errors", tail_output.filtering_errors.to_string());
            stats.print();
        }

        print!("{}", formatted);

        // If follow mode is enabled, stream new lines
        if input.follow {
            self.stream_tail_lines(input, tail_output.total_lines, ctx)?;
        }

        Ok(())
    }
}

/// Input data for the `tail` command.
#[derive(Debug, Clone)]
pub(crate) struct TailInput {
    pub file: std::path::PathBuf,
    pub lines: usize,
    pub errors: bool,
    pub follow: bool,
}

