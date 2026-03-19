//! Handler for the `txt2md` command - converts plain text to Markdown.

use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
pub(crate) mod detect_headings;
pub(crate) mod detect_lists;
pub(crate) mod format;
pub(crate) mod parser;

pub(crate) struct Txt2mdInput {
    pub input: Option<std::path::PathBuf>,
    pub output: Option<std::path::PathBuf>,
}

/// Handler for the `txt2md` command.
pub(crate) struct Txt2mdHandler;

impl Txt2mdHandler {
    /// Read text content from a file or stdin.
    pub(crate) fn read_input(&self, input: &Option<std::path::PathBuf>) -> CommandResult<String> {
        if let Some(ref path) = input {
            if !path.exists() {
                return Err(CommandError::IoError(format!(
                    "File not found: {}",
                    path.display()
                )));
            }
            std::fs::read_to_string(path).map_err(|e| {
                CommandError::IoError(format!("Failed to read file '{}': {}", path.display(), e))
            })
        } else {
            use std::io::{self, Read};
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| CommandError::IoError(format!("Failed to read stdin: {}", e)))?;
            Ok(buffer)
        }
    }
}

impl CommandHandler for Txt2mdHandler {
    type Input = Txt2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input text
        let text = self.read_input(&input.input)?;

        // Convert to Markdown
        let markdown = self.convert_to_markdown(&text);

        // Normalize spacing (collapse blank lines, trim trailing whitespace)
        let normalized = self.normalize_spacing(&markdown);

        // Extract metadata
        let metadata = self.extract_metadata(&text, &input.input);

        // Format output
        let formatted = self.format_output(&normalized, &metadata, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("txt2md")
                .with_output_mode(ctx.format)
                .with_input_bytes(text.len())
                .with_output_bytes(formatted.len())
                .with_extra(
                    "Source",
                    input
                        .input
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "stdin".to_string()),
                );
            stats.print();
        }

        // Write to output file or stdout
        if let Some(ref output_path) = input.output {
            std::fs::write(output_path, &formatted).map_err(|e| {
                CommandError::IoError(format!(
                    "Failed to write output file '{}': {}",
                    output_path.display(),
                    e
                ))
            })?;
        } else {
            print!("{}\n", formatted);
        }

        Ok(())
    }
}
