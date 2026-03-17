//! Reducer output structures and formatting.

use crate::OutputFormat;
use serde::Serialize;
use std::collections::HashMap;

use super::ReducerContext;

// ============================================================
// Reducer Output Structure
// ============================================================

/// Metadata about the reducer output.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReducerMetadata {
    /// Name of the reducer that produced this output.
    pub reducer: String,
    /// Number of items processed.
    pub items_processed: usize,
    /// Number of items filtered out.
    pub items_filtered: usize,
    /// Processing duration in milliseconds.
    pub duration_ms: u64,
    /// Additional custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

/// Estimate the number of tokens from byte count.
/// Uses the common approximation of ~4 characters per token.
fn estimate_tokens(bytes: usize) -> usize {
    // Most tokenizers average around 4 characters per token for English text
    // This is a rough estimate suitable for statistics display
    bytes / 4
}

/// Statistics about input/output transformation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReducerStats {
    /// Input size in bytes.
    pub input_bytes: usize,
    /// Output size in bytes.
    pub output_bytes: usize,
    /// Reduction ratio (output_bytes / input_bytes).
    pub reduction_ratio: f64,
    /// Number of lines in input.
    pub input_lines: usize,
    /// Number of lines in output.
    pub output_lines: usize,
    /// Estimated input token count.
    pub input_tokens: usize,
    /// Estimated output token count.
    pub output_tokens: usize,
    /// Token reduction ratio (output_tokens / input_tokens).
    pub token_reduction_ratio: f64,
}

impl ReducerStats {
    /// Create new stats with automatic token estimation.
    pub fn new(input_bytes: usize, output_bytes: usize, input_lines: usize, output_lines: usize) -> Self {
        let input_tokens = estimate_tokens(input_bytes);
        let output_tokens = estimate_tokens(output_bytes);
        let reduction_ratio = if input_bytes > 0 {
            output_bytes as f64 / input_bytes as f64
        } else {
            0.0
        };
        let token_reduction_ratio = if input_tokens > 0 {
            output_tokens as f64 / input_tokens as f64
        } else {
            0.0
        };
        Self {
            input_bytes,
            output_bytes,
            reduction_ratio,
            input_lines,
            output_lines,
            input_tokens,
            output_tokens,
            token_reduction_ratio,
        }
    }
}

/// The main output structure produced by reducers.
///
/// This structure wraps the processed data and provides methods
/// for formatting it in different output formats.
#[derive(Debug, Clone, Serialize)]
pub struct ReducerOutput {
    /// The processed data as JSON-compatible value.
    pub data: serde_json::Value,
    /// Metadata about the reduction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ReducerMetadata>,
    /// Statistics about the transformation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<ReducerStats>,
    /// Whether the output is empty.
    pub is_empty: bool,
    /// Summary message for compact output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Items for list-based outputs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ReducerItem>,
    /// Sections for categorized outputs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<ReducerSection>,
    /// Exit code from the original command (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl ReducerOutput {
    /// Create a new reducer output with the given data.
    pub fn new<T: Serialize>(data: T) -> Self {
        let json_value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
        Self {
            data: json_value,
            metadata: None,
            stats: None,
            is_empty: false,
            summary: None,
            items: Vec::new(),
            sections: Vec::new(),
            exit_code: None,
        }
    }

    /// Create an empty reducer output.
    pub fn empty() -> Self {
        Self {
            data: serde_json::Value::Null,
            metadata: None,
            stats: None,
            is_empty: true,
            summary: None,
            items: Vec::new(),
            sections: Vec::new(),
            exit_code: None,
        }
    }

    /// Add metadata to the output.
    pub fn with_metadata(mut self, metadata: ReducerMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add stats to the output.
    pub fn with_stats(mut self, stats: ReducerStats) -> Self {
        self.stats = Some(stats);
        self
    }

    /// Add a summary message.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Add items to the output.
    pub fn with_items(mut self, items: Vec<ReducerItem>) -> Self {
        self.items = items;
        self
    }

    /// Add sections to the output.
    pub fn with_sections(mut self, sections: Vec<ReducerSection>) -> Self {
        self.sections = sections;
        self
    }

    /// Format the output based on the context format.
    pub fn format(&self, context: &ReducerContext) -> String {
        match context.format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Csv => self.format_csv(),
            OutputFormat::Tsv => self.format_tsv(),
            OutputFormat::Agent => self.format_agent(),
            OutputFormat::Compact => self.format_compact(),
            OutputFormat::Raw => self.format_raw(),
        }
    }

    /// Format output as JSON.
    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or_else(|e| {
            serde_json::json!({
                "error": true,
                "message": format!("Failed to serialize: {}", e)
            })
            .to_string()
        })
    }

    /// Format output as CSV.
    pub fn format_csv(&self) -> String {
        // If we have items, format them as CSV
        if !self.items.is_empty() {
            let mut csv = String::new();
            // Header
            if let Some(_first) = self.items.first() {
                csv.push_str(&format!("key,value,label\n"));
            }
            // Rows
            for item in &self.items {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    escape_csv(&item.key),
                    escape_csv(&item.value),
                    escape_csv(item.label.as_deref().unwrap_or(""))
                ));
            }
            return csv;
        }

        // Fallback to JSON for complex data
        self.format_json()
    }

    /// Format output as TSV.
    pub fn format_tsv(&self) -> String {
        // If we have items, format them as TSV
        if !self.items.is_empty() {
            let mut tsv = String::new();
            // Header
            tsv.push_str("key\tvalue\tlabel\n");
            // Rows
            for item in &self.items {
                tsv.push_str(&format!(
                    "{}\t{}\t{}\n",
                    item.key,
                    item.value,
                    item.label.as_deref().unwrap_or("")
                ));
            }
            return tsv;
        }

        // Fallback to JSON for complex data
        self.format_json()
    }

    /// Format output for AI agent consumption.
    pub fn format_agent(&self) -> String {
        let mut output = String::new();

        // Add summary if available
        if let Some(ref summary) = self.summary {
            output.push_str(&format!("SUMMARY: {}\n\n", summary));
        }

        // Add sections
        for section in &self.sections {
            output.push_str(&format!("## {}\n", section.name));
            if let Some(ref count) = section.count {
                output.push_str(&format!("count: {}\n", count));
            }
            for item in &section.items {
                if let Some(ref label) = item.label {
                    output.push_str(&format!("- {} [{}]\n", item.key, label));
                } else {
                    output.push_str(&format!("- {}\n", item.key));
                }
            }
            output.push('\n');
        }

        // Add flat items if no sections
        if self.sections.is_empty() && !self.items.is_empty() {
            output.push_str("## Items\n");
            for item in &self.items {
                if let Some(ref label) = item.label {
                    output.push_str(&format!("- {} [{}]: {}\n", item.key, label, item.value));
                } else {
                    output.push_str(&format!("- {}: {}\n", item.key, item.value));
                }
            }
        }

        // Add metadata if available
        if let Some(ref metadata) = self.metadata {
            output.push_str(&format!("\n## Metadata\n"));
            output.push_str(&format!("reducer: {}\n", metadata.reducer));
            output.push_str(&format!("items_processed: {}\n", metadata.items_processed));
        }

        output
    }

    /// Format output in compact human-readable format.
    pub fn format_compact(&self) -> String {
        let mut output = String::new();

        // Add summary if available
        if let Some(ref summary) = self.summary {
            output.push_str(&format!("{}\n", summary));
        }

        // Handle empty output
        if self.is_empty {
            output.push_str("(empty)\n");
            return output;
        }

        // Add sections
        for section in &self.sections {
            if let Some(count) = section.count {
                output.push_str(&format!("{} ({}):\n", section.name, count));
            } else {
                output.push_str(&format!("{}:\n", section.name));
            }
            for item in &section.items {
                if let Some(ref label) = item.label {
                    output.push_str(&format!("  {} [{}]\n", item.key, label));
                } else {
                    output.push_str(&format!("  {}\n", item.key));
                }
            }
        }

        // Add flat items if no sections
        if self.sections.is_empty() && !self.items.is_empty() {
            for item in &self.items {
                if let Some(ref label) = item.label {
                    output.push_str(&format!("{} [{}]: {}\n", item.key, label, item.value));
                } else {
                    output.push_str(&format!("{}: {}\n", item.key, item.value));
                }
            }
        }

        output
    }

    /// Format output as raw (minimal processing).
    pub fn format_raw(&self) -> String {
        let mut output = String::new();

        // Just output the items, one per line
        for item in &self.items {
            output.push_str(&format!("{}\n", item.key));
        }

        // If no items, try to extract from sections
        if self.items.is_empty() {
            for section in &self.sections {
                for item in &section.items {
                    output.push_str(&format!("{}\n", item.key));
                }
            }
        }

        output
    }
}

/// A single item in reducer output.
#[derive(Debug, Clone, Serialize)]
pub struct ReducerItem {
    /// Key/identifier for this item.
    pub key: String,
    /// Value associated with this item.
    pub value: String,
    /// Optional label/category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Optional additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ReducerItem {
    /// Create a new reducer item.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            label: None,
            data: None,
        }
    }

    /// Add a label to the item.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Add additional data to the item.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// A section of related items in reducer output.
#[derive(Debug, Clone, Serialize)]
pub struct ReducerSection {
    /// Name of the section.
    pub name: String,
    /// Optional count of items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    /// Items in this section.
    pub items: Vec<ReducerItem>,
}

impl ReducerSection {
    /// Create a new section with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: None,
            items: Vec::new(),
        }
    }

    /// Add a count to the section.
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Add items to the section.
    pub fn with_items(mut self, items: Vec<ReducerItem>) -> Self {
        self.items = items;
        self
    }

    /// Add a single item to the section.
    pub fn add_item(&mut self, item: ReducerItem) {
        self.items.push(item);
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Escape a value for CSV output.
pub(crate) fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
