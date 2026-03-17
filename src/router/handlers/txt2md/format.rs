//! Output formatting for txt2md conversion.
//!
//! Handles metadata extraction, spacing normalization, and format-specific output.

use super::Txt2mdHandler;
use crate::OutputFormat;

impl Txt2mdHandler {
    /// Extract metadata from the text.
    pub(crate) fn extract_metadata(
        &self,
        text: &str,
        input_path: &Option<std::path::PathBuf>,
    ) -> serde_json::Value {
        let lines: Vec<&str> = text.lines().collect();
        let word_count: usize = text.split_whitespace().count();
        let line_count = lines.len();

        let mut metadata = serde_json::json!({
            "lines": line_count,
            "words": word_count,
            "characters": text.len(),
        });

        // Try to extract a title from the first non-empty line
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // Use first line as title (truncate if too long)
                let title = if trimmed.len() > 80 {
                    format!("{}...", &trimmed[..77])
                } else {
                    trimmed.to_string()
                };
                metadata["title"] = serde_json::json!(title);
                break;
            }
        }

        // Add source info if input is a file
        if let Some(ref path) = input_path {
            metadata["source"] = serde_json::json!(path.display().to_string());
            metadata["type"] = serde_json::json!("file");
        } else {
            metadata["type"] = serde_json::json!("stdin");
        }

        metadata
    }

    /// Normalize spacing in the markdown output.
    ///
    /// This function:
    /// - Trims trailing whitespace from each line
    /// - Collapses consecutive blank lines into a single blank line
    /// - Removes leading and trailing blank lines
    pub(crate) fn normalize_spacing(&self, text: &str) -> String {
        // Trim trailing whitespace from each line
        let mut result: String = text
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple blank lines into single blank lines
        let lines: Vec<&str> = result.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in &lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(*line);
            prev_blank = is_blank;
        }

        result = collapsed_lines.join("\n");

        // Remove leading/trailing blank lines
        result.trim().to_string()
    }

    /// Format output based on the output format.
    pub(crate) fn format_output(
        &self,
        markdown: &str,
        metadata: &serde_json::Value,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "markdown": markdown,
                "metadata": metadata,
            })
            .to_string(),
            OutputFormat::Compact | OutputFormat::Agent => {
                format!(
                    "---\n{}\n---\n\n{}",
                    serde_json::to_string_pretty(metadata).unwrap(),
                    markdown
                )
            }
            OutputFormat::Raw | OutputFormat::Csv | OutputFormat::Tsv => markdown.to_string(),
        }
    }
}
