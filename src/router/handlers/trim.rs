use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::OutputFormat;

pub(crate) struct TrimInput {
    pub file: Option<std::path::PathBuf>,
    pub leading: bool,
    pub trailing: bool,
}

/// Handler for the `trim` command.
pub(crate) struct TrimHandler;

impl TrimHandler {
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
                std::fs::read_to_string(path)
                    .map_err(|e| CommandError::IoError(format!("Failed to read file: {}", e)))
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

    /// Trim whitespace from text based on options.
    pub(crate) fn trim_text(&self, text: &str, leading: bool, trailing: bool) -> String {
        text.lines()
            .map(|line| {
                if leading && trailing {
                    line.trim()
                } else if leading {
                    line.trim_start()
                } else if trailing {
                    line.trim_end()
                } else {
                    // Default: trim both when no specific flag is set
                    line.trim()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format the trimmed output based on the output format.
    pub(crate) fn format_output(
        &self,
        original: &str,
        trimmed: &str,
        options: &TrimInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => {
                serde_json::json!({
                    "content": trimmed,
                    "stats": {
                        "input_length": original.len(),
                        "output_length": trimmed.len(),
                        "reduction": if original.len() > 0 {
                            ((original.len() - trimmed.len()) as f64 / original.len() as f64 * 100.0) as i32
                        } else {
                            0
                        },
                        "lines_removed": original.lines().count().saturating_sub(trimmed.lines().count())
                    },
                    "options": {
                        "leading": options.leading,
                        "trailing": options.trailing
                    }
                })
                .to_string()
            }
            OutputFormat::Csv => {
                let lines: Vec<&str> = trimmed.lines().collect();
                let mut output = String::from("line\n");
                for line in lines {
                    output.push_str(&format!("\"{}\"\n", line.replace('"', "\"\"")));
                }
                output
            }
            OutputFormat::Tsv => {
                let lines: Vec<&str> = trimmed.lines().collect();
                let mut output = String::from("line\n");
                for line in lines {
                    output.push_str(&format!("{}\n", line));
                }
                output
            }
            OutputFormat::Agent => {
                let input_len = original.len();
                let output_len = trimmed.len();
                let reduction = if input_len > 0 {
                    ((input_len - output_len) as f64 / input_len as f64 * 100.0) as i32
                } else {
                    0
                };

                format!(
                    "Content:\n{}\n\nStats:\n  Input: {} bytes\n  Output: {} bytes\n  Reduction: {}%\n  Mode: {}\n",
                    trimmed,
                    input_len,
                    output_len,
                    reduction,
                    if options.leading && options.trailing {
                        "both"
                    } else if options.leading {
                        "leading"
                    } else if options.trailing {
                        "trailing"
                    } else {
                        "both"
                    }
                )
            }
            OutputFormat::Raw => trimmed.to_string(),
            OutputFormat::Compact => {
                let input_len = original.len();
                let output_len = trimmed.len();
                let reduction = if input_len > 0 {
                    ((input_len - output_len) as f64 / input_len as f64 * 100.0) as i32
                } else {
                    0
                };

                let mode = if options.leading && options.trailing {
                    "both"
                } else if options.leading {
                    "leading"
                } else if options.trailing {
                    "trailing"
                } else {
                    "both"
                };

                if reduction > 0 {
                    format!("{} ({}% reduction, mode: {})", trimmed, reduction, mode)
                } else {
                    format!("{} (mode: {})", trimmed, mode)
                }
            }
        }
    }
}

impl CommandHandler for TrimHandler {
    type Input = TrimInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input
        let original = self.read_input(&input.file)?;

        // Trim the text
        let trimmed = self.trim_text(&original, input.leading, input.trailing);

        // Format and output the result
        let formatted = self.format_output(&original, &trimmed, input, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("trim")
                .with_output_mode(ctx.format)
                .with_input_bytes(original.len())
                .with_output_bytes(formatted.len())
                .with_extra("Leading", input.leading.to_string())
                .with_extra("Trailing", input.trailing.to_string());
            stats.print();
        }

        print!("{}", formatted);

        Ok(())
    }
}
