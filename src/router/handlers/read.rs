//! File reader with filter levels for `trs read` command.
//!
//! Reads a file and optionally strips noise based on filter level:
//! - None: raw file content (like cat, but with line numbers)
//! - Minimal: strip comments, normalize blank lines
//! - Aggressive: signatures-only (imports + function/class/struct definitions)
//!
//! Data files (JSON, YAML, TOML, XML, CSV) are always passed through unmodified
//! to prevent corruption.

use std::path::PathBuf;

use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
#[allow(unused_imports)]
pub(crate) use super::read_filters::{
    apply_line_range, count_braces, detect_language, filter_aggressive, filter_minimal, Language,
};
use crate::OutputFormat;

/// Input for the read command.
pub(crate) struct ReadInput {
    pub file: PathBuf,
    pub level: FilterLevel,
    pub max_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub line_numbers: bool,
}

/// Filter level for file reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FilterLevel {
    /// No filtering — raw content
    #[default]
    None,
    /// Strip comments, normalize blank lines
    Minimal,
    /// Signatures only — imports + definitions, skip bodies
    Aggressive,
}

pub(crate) struct ReadHandler;

impl ReadHandler {
    pub(crate) fn execute(&self, input: &ReadInput, ctx: &CommandContext) -> CommandResult {
        let content = std::fs::read_to_string(&input.file)
            .map_err(|e| CommandError::IoError(format!("{}: {}", input.file.display(), e)))?;
        let input_bytes = content.len();

        let lang = detect_language(&input.file);

        // Apply filter level (data files always passthrough)
        let filtered = if lang == Language::Data || input.level == FilterLevel::None {
            content.clone()
        } else {
            match input.level {
                FilterLevel::None => content.clone(),
                FilterLevel::Minimal => filter_minimal(&content, lang),
                FilterLevel::Aggressive => filter_aggressive(&content, lang),
            }
        };

        // Apply line range limits
        let lines: Vec<&str> = filtered.lines().collect();
        let selected = apply_line_range(&lines, input.max_lines, input.tail_lines);

        // Format output
        let output = match ctx.format {
            OutputFormat::Json => {
                let lines_json: Vec<serde_json::Value> = selected
                    .iter()
                    .enumerate()
                    .map(|(i, line)| serde_json::json!({"line": i + 1, "content": line}))
                    .collect();
                serde_json::json!({
                    "file": input.file.display().to_string(),
                    "language": format!("{:?}", lang),
                    "filter": format!("{:?}", input.level),
                    "total_lines": lines.len(),
                    "shown_lines": selected.len(),
                    "lines": lines_json,
                })
                .to_string()
            }
            _ => {
                let mut out = String::new();
                for (i, line) in selected.iter().enumerate() {
                    if input.line_numbers {
                        out.push_str(&format!("{:>4} {}\n", i + 1, line));
                    } else {
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                out
            }
        };

        print!("{}", output);

        if ctx.stats {
            CommandStats::new()
                .with_reducer("read")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(lines.len())
                .with_items_filtered(lines.len() - selected.len())
                .print();
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
