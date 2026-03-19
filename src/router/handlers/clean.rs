use super::common::{
    sanitize_control_chars, strip_ansi_codes, CommandContext, CommandError, CommandResult,
    CommandStats,
};
use super::types::CommandHandler;
use crate::OutputFormat;

pub(crate) struct CleanHandler;

impl CleanHandler {
    /// Read input from file or stdin.
    pub(crate) fn read_input(&self, file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        match file {
            Some(path) => {
                if !path.exists() {
                    return Err(CommandError::IoError(format!(
                        "File not found: {}",
                        path.display()
                    )));
                }
                std::fs::read_to_string(path).map_err(|e| {
                    CommandError::IoError(format!("Failed to read file {}: {}", path.display(), e))
                })
            }
            None => {
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| CommandError::IoError(format!("Failed to read stdin: {}", e)))?;
                Ok(String::from_utf8_lossy(&buffer).to_string())
            }
        }
    }

    /// Apply cleaning operations to the input.
    pub(crate) fn clean_text(&self, text: &str, options: &CleanInput) -> String {
        let mut result = text.to_string();

        // Strip ANSI escape codes FIRST (before sanitizing control chars)
        // because ANSI codes start with \x1b which is a control character
        if options.no_ansi {
            result = strip_ansi_codes(&result);
        }

        // Sanitize control characters (remove nulls, replace other control chars)
        result = sanitize_control_chars(&result);

        // Trim whitespace from lines
        if options.trim {
            result = result
                .lines()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n");
        } else {
            // Always trim trailing whitespace from each line
            result = result
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Collapse repeated lines or blank lines
        if options.collapse_repeats {
            result = self.collapse_repeated_lines(&result);
        } else if options.collapse_blanks {
            result = self.collapse_blank_lines(&result);
        }

        // Remove leading/trailing blank lines
        result.trim().to_string()
    }

    /// Collapse consecutive blank lines into a single blank line.
    pub(crate) fn collapse_blank_lines(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(line);
            prev_blank = is_blank;
        }

        collapsed_lines.join("\n")
    }

    /// Collapse consecutive repeated lines into a single occurrence.
    pub(crate) fn collapse_repeated_lines(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_line: Option<&str> = None;

        for line in lines {
            // Skip if this line is the same as the previous line
            if let Some(prev) = prev_line {
                if line == prev {
                    continue;
                }
            }
            collapsed_lines.push(line);
            prev_line = Some(line);
        }

        collapsed_lines.join("\n")
    }

    /// Format the cleaned output based on the output format.
    pub(crate) fn format_output(
        &self,
        original: &str,
        cleaned: &str,
        options: &CleanInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "content": cleaned,
                "stats": {
                    "input_length": original.len(),
                    "output_length": cleaned.len(),
                    "reduction_percent": if original.is_empty() {
                        0.0
                    } else {
                        ((original.len() - cleaned.len()) as f64 / original.len() as f64) * 100.0
                    },
                },
                "options": {
                    "no_ansi": options.no_ansi,
                    "collapse_blanks": options.collapse_blanks,
                    "collapse_repeats": options.collapse_repeats,
                    "trim": options.trim,
                }
            })
            .to_string(),
            OutputFormat::Csv => {
                // Output as CSV with one row per line
                cleaned
                    .lines()
                    .map(|line| format!("\"{}\"", line.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            OutputFormat::Tsv => {
                // Output as TSV with one row per line
                cleaned.lines().collect::<Vec<_>>().join("\n")
            }
            OutputFormat::Agent => {
                let reduction = if original.is_empty() {
                    0
                } else {
                    ((original.len() - cleaned.len()) as f64 / original.len() as f64 * 100.0) as i32
                };
                format!("Content ({}% reduction):\n{}\n", reduction, cleaned)
            }
            OutputFormat::Compact => {
                let reduction = if original.is_empty() {
                    0
                } else {
                    ((original.len() - cleaned.len()) as f64 / original.len() as f64 * 100.0) as i32
                };
                if reduction > 0 {
                    format!("{} ({}% reduction)\n", cleaned, reduction)
                } else {
                    format!("{}\n", cleaned)
                }
            }
            OutputFormat::Raw => cleaned.to_string(),
        }
    }
}

impl CommandHandler for CleanHandler {
    type Input = CleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let original = self.read_input(&input.file)?;

        // Apply cleaning operations
        let cleaned = self.clean_text(&original, input);

        // Format and output the result
        let formatted = self.format_output(&original, &cleaned, input, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("clean")
                .with_output_mode(ctx.format)
                .with_input_bytes(original.len())
                .with_output_bytes(formatted.len())
                .with_extra("No ANSI", input.no_ansi.to_string())
                .with_extra("Collapse blanks", input.collapse_blanks.to_string())
                .with_extra("Collapse repeats", input.collapse_repeats.to_string())
                .with_extra("Trim", input.trim.to_string());
            stats.print();
        }

        print!("{}", formatted);

        Ok(())
    }
}

/// Input data for the `clean` command.
#[derive(Debug, Clone)]
pub(crate) struct CleanInput {
    pub file: Option<std::path::PathBuf>,
    pub no_ansi: bool,
    pub collapse_blanks: bool,
    pub collapse_repeats: bool,
    pub trim: bool,
}
