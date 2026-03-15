//! Reducer system for TARS CLI.
//!
//! This module provides a reducer interface for transforming and aggregating command outputs.
//! Reducers can process input data and produce output in various formats.
//!
//! # Example
//!
//! ```rust
//! use crate::reducer::{Reducer, ReducerContext, ReducerOutput};
//! use crate::OutputFormat;
//!
//! struct LogEntry {
//!     level: String,
//!     message: String,
//!     timestamp: String,
//! }
//!
//! struct LogReducer;
//!
//! impl Reducer for LogReducer {
//!     type Input = String;
//!     type Output = ReducerOutput;
//!
//!     fn reduce(&self, input: &Self::Input, context: &ReducerContext) -> ReducerResult<Self::Output> {
//!         // Parse log lines into structured entries
//!         let mut entries = Vec::new();
//!         for line in input.lines() {
//!             // Parse log format: "2024-01-15 10:30:00 [INFO] message"
//!             if let Some(entry) = parse_log_line(line) {
//!                 entries.push(entry);
//!             }
//!         }
//!         Ok(ReducerOutput::new(entries))
//!     }
//!
//!     fn name(&self) -> &'static str {
//!         "log"
//!     }
//! }
//! ```
//!
//! # Output Formats
//!
//! Reducers can output their results in different formats based on the context:
//!
//! ## JSON Format
//!
//! Output is serialized as JSON using `serde_json::json!`.
//!
//! ## Compact Format
//!
//! Output is formatted for human readability with reduced verbosity.
//!
//! ## Raw Format
//!
//! Output is passed through with minimal processing.

use crate::OutputFormat;
use serde::Serialize;
use std::collections::HashMap;

// ============================================================
// Truncation Detection
// ============================================================

/// Information about truncation detection.
///
/// Truncation occurs when output is cut off due to limits, size thresholds,
/// or other constraints. This structure tracks when and why truncation occurred.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TruncationInfo {
    /// Whether the output was truncated.
    pub is_truncated: bool,
    /// Total number of items available before truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_available: Option<usize>,
    /// Number of items included after truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_shown: Option<usize>,
    /// Number of items hidden due to truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_hidden: Option<usize>,
    /// Reason for truncation (e.g., "limit", "size_threshold", "timeout").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Threshold value that caused truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<usize>,
    /// Warning message to display about truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl TruncationInfo {
    /// Create a new truncation info indicating no truncation.
    pub fn none() -> Self {
        Self {
            is_truncated: false,
            ..Default::default()
        }
    }

    /// Create truncation info for a limit-based truncation.
    pub fn limited(total: usize, shown: usize, limit: usize) -> Self {
        let hidden = total.saturating_sub(shown);
        Self {
            is_truncated: hidden > 0,
            total_available: Some(total),
            items_shown: Some(shown),
            items_hidden: Some(hidden),
            reason: Some("limit".to_string()),
            threshold: Some(limit),
            warning: if hidden > 0 {
                Some(format!(
                    "Output truncated: showing {} of {} items (limit: {})",
                    shown, total, limit
                ))
            } else {
                None
            },
        }
    }

    /// Create truncation info for a size-based truncation.
    pub fn size_threshold(total_bytes: usize, shown_bytes: usize, threshold_bytes: usize) -> Self {
        Self {
            is_truncated: true,
            total_available: Some(total_bytes),
            items_shown: Some(shown_bytes),
            items_hidden: Some(total_bytes.saturating_sub(shown_bytes)),
            reason: Some("size_threshold".to_string()),
            threshold: Some(threshold_bytes),
            warning: Some(format!(
                "Output truncated: showing {} of {} bytes (threshold: {})",
                shown_bytes, total_bytes, threshold_bytes
            )),
        }
    }

    /// Create truncation info detected from output patterns.
    pub fn detected(pattern: &str, original_size: usize) -> Self {
        Self {
            is_truncated: true,
            total_available: None,
            items_shown: Some(original_size),
            items_hidden: None,
            reason: Some("detected".to_string()),
            threshold: None,
            warning: Some(format!(
                "Truncation detected: output appears to be incomplete (pattern: '{}')",
                pattern
            )),
        }
    }

    /// Check if output appears to be truncated based on common patterns.
    pub fn detect_from_output(output: &str) -> Self {
        // Common truncation indicators
        let truncation_patterns = [
            // Generic truncation markers
            "... truncated",
            "...truncated",
            "[truncated]",
            "<truncated>",
            "output truncated",
            // CLI tool truncation messages
            "results limited to",
            "showing first",
            "showing top",
            "more results available",
            "... and more",
            "...and more",
            "use --limit",
            "use -n to see more",
            // System truncation
            "output too large",
            "output size limit",
            "max output exceeded",
        ];

        let output_lower = output.to_lowercase();
        for pattern in truncation_patterns {
            if output_lower.contains(pattern) {
                return Self::detected(pattern, output.len());
            }
        }

        // Check for incomplete JSON/array patterns
        if Self::detect_incomplete_json(output) {
            return Self::detected("incomplete_json", output.len());
        }

        // Check for cut-off lines
        if Self::detect_cutoff_line(output) {
            return Self::detected("cutoff_line", output.len());
        }

        Self::none()
    }

    /// Detect if output contains incomplete JSON.
    fn detect_incomplete_json(output: &str) -> bool {
        let trimmed = output.trim();
        
        // Check for incomplete array
        if trimmed.starts_with('[') && !trimmed.ends_with(']') {
            return true;
        }
        
        // Check for incomplete object
        if trimmed.starts_with('{') && !trimmed.ends_with('}') {
            return true;
        }
        
        // Check for truncated JSON value
        if trimmed.ends_with("...") || trimmed.ends_with(",") {
            // Likely truncated if it looks like JSON
            if trimmed.starts_with('[') || trimmed.starts_with('{') {
                return true;
            }
        }

        false
    }

    /// Detect if the last line appears to be cut off.
    fn detect_cutoff_line(output: &str) -> bool {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return false;
        }

        let last_line = lines.last().unwrap();
        
        // Check for common cutoff patterns
        // Line ending with incomplete word or punctuation
        if last_line.ends_with("...") && last_line.len() > 3 {
            return true;
        }

        // Line that looks like it should continue
        // (ends with connecting words or operators)
        let cutoff_indicators = [" and", " or", " with", ",", " -", " +", " &"];
        for indicator in cutoff_indicators {
            if last_line.to_lowercase().ends_with(indicator) {
                return true;
            }
        }

        false
    }

    /// Returns true if truncation was detected.
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    /// Get a human-readable summary of the truncation.
    pub fn summary(&self) -> Option<String> {
        if !self.is_truncated {
            return None;
        }

        if let Some(ref warning) = self.warning {
            Some(warning.clone())
        } else if let (Some(shown), Some(hidden)) = (self.items_shown, self.items_hidden) {
            Some(format!("Truncated: {} items shown, {} hidden", shown, hidden))
        } else {
            Some("Output was truncated".to_string())
        }
    }
}

/// Configuration for truncation detection and handling.
#[derive(Debug, Clone)]
pub struct TruncationConfig {
    /// Maximum number of items to show before truncating.
    pub max_items: Option<usize>,
    /// Maximum output size in bytes before truncating.
    pub max_bytes: Option<usize>,
    /// Whether to detect truncation from output patterns.
    pub detect_patterns: bool,
    /// Whether to include truncation warnings in output.
    pub include_warnings: bool,
}

impl Default for TruncationConfig {
    fn default() -> Self {
        Self {
            max_items: None,
            max_bytes: None,
            detect_patterns: true,
            include_warnings: true,
        }
    }
}

impl TruncationConfig {
    /// Create a new truncation config with item limit.
    pub fn with_max_items(max_items: usize) -> Self {
        Self {
            max_items: Some(max_items),
            ..Default::default()
        }
    }

    /// Create a new truncation config with byte limit.
    pub fn with_max_bytes(max_bytes: usize) -> Self {
        Self {
            max_bytes: Some(max_bytes),
            ..Default::default()
        }
    }

    /// Apply truncation to a list of items.
    pub fn truncate_items<T>(&self, items: Vec<T>) -> (Vec<T>, TruncationInfo) {
        let total = items.len();
        
        if let Some(limit) = self.max_items {
            if total > limit {
                let shown: Vec<T> = items.into_iter().take(limit).collect();
                return (shown, TruncationInfo::limited(total, limit, limit));
            }
        }

        (items, TruncationInfo::none())
    }

    /// Apply truncation to output string.
    pub fn truncate_output(&self, output: String) -> (String, TruncationInfo) {
        let total_bytes = output.len();
        
        if let Some(limit) = self.max_bytes {
            if total_bytes > limit {
                let truncated = output.chars().take(limit).collect();
                return (truncated, TruncationInfo::size_threshold(total_bytes, limit, limit));
            }
        }

        // Detect truncation patterns if enabled
        if self.detect_patterns {
            let info = TruncationInfo::detect_from_output(&output);
            if info.is_truncated {
                return (output, info);
            }
        }

        (output, TruncationInfo::none())
    }
}

/// Context passed to reducers during transformation.
#[derive(Debug, Clone)]
pub struct ReducerContext {
    /// The output format to use.
    pub format: OutputFormat,
    /// Whether to show statistics.
    pub stats: bool,
    /// List of enabled format flags (for warnings/debugging).
    pub enabled_formats: Vec<OutputFormat>,
}

impl ReducerContext {
    /// Create a new reducer context from CLI options.
    pub fn from_cli(cli: &crate::Cli) -> Self {
        Self {
            format: cli.output_format(),
            stats: cli.stats,
            enabled_formats: cli.enabled_format_flags(),
        }
    }

    /// Returns true if multiple format flags were specified.
    pub fn has_conflicting_formats(&self) -> bool {
        self.enabled_formats.len() > 1
    }
}

/// Result type for reducer operations.
pub type ReducerResult<T = ()> = Result<T, ReducerError>;

/// Error type for reducer operations.
#[derive(Debug, Clone)]
pub enum ReducerError {
    /// The reducer is not yet implemented.
    NotImplemented(String),
    /// An error occurred during processing.
    ProcessingError {
        message: String,
    },
    /// Invalid input provided.
    InvalidInput(String),
    /// I/O error occurred.
    IoError(String),
}

impl ReducerError {
    /// Returns true if this is a NotImplemented error.
    pub fn is_not_implemented(&self) -> bool {
        matches!(self, ReducerError::NotImplemented(_))
    }

    /// Returns true if this is an InvalidInput error.
    pub fn is_invalid_input(&self) -> bool {
        matches!(self, ReducerError::InvalidInput(_))
    }

    /// Returns true if this is an IoError error.
    pub fn is_io_error(&self) -> bool {
        matches!(self, ReducerError::IoError(_))
    }

    /// Returns true if this is a ProcessingError error.
    pub fn is_processing_error(&self) -> bool {
        matches!(self, ReducerError::ProcessingError { .. })
    }
}

impl std::fmt::Display for ReducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReducerError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ReducerError::ProcessingError { message } => write!(f, "Processing error: {}", message),
            ReducerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ReducerError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ReducerError {}

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
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// ============================================================
// Reducer Trait
// ============================================================

/// Trait for implementing reducers that transform input data.
///
/// Reducers take input data, process it, and return output in various formats
/// based on the context.
pub trait Reducer {
    /// The input type this reducer operates on.
    type Input;
    /// The output type this reducer produces.
    type Output;

    /// Reduce the input to produce output.
    ///
    /// # Arguments
    ///
    /// * `input` - The input data to process
    /// * `context` - The context containing output format and stats flag, etc.
    ///
    /// # Returns
    ///
    /// Returns the processed output on success, or a `ReducerError` on failure.
    fn reduce(&self, input: &Self::Input, context: &ReducerContext) -> ReducerResult<Self::Output>;

    /// Get the name of this reducer.
    ///
    /// This is used for logging and debugging purposes.
    fn name(&self) -> &'static str;
}

// ============================================================
// Base Reducer Implementation
// ============================================================

/// Base reducer implementation with common functionality.
///
/// This struct provides default implementations for formatting methods
/// and can be extended by concrete reducer implementations.
pub struct BaseReducer<T: Serialize> {
    /// The name of the reducer.
    name: &'static str,
    /// Phantom data for the generic type parameter.
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + std::fmt::Debug> BaseReducer<T> {
    /// Create a new base reducer with the given name.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Format output as JSON.
    pub fn format_json(output: &T) -> String {
        serde_json::to_string_pretty(&output).unwrap_or_else(|e| {
            format!("{{\"error\": true, \"message\": \"Failed to serialize: {}\"}}", e)
        })
    }

    /// Format output in compact format.
    pub fn format_compact(output: &T) -> String {
        format!("{:?}", output)
    }

    /// Format output in raw format (minimal processing).
    pub fn format_raw(output: &T) -> String {
        format!("{:?}", output)
    }
}

impl<T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug> Reducer for BaseReducer<T> {
    type Input = String;
    type Output = T;

    fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
        // Default implementation - parse as JSON and deserialize
        match serde_json::from_str(input) {
            Ok(data) => Ok(data),
            Err(e) => Err(ReducerError::InvalidInput(format!(
                "Failed to parse input as {}: {}",
                std::any::type_name::<T>(),
                e
            ))),
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

// ============================================================
// Reducer Registry
// ============================================================

/// Type alias for a reducer function.
type ReducerFn = Box<dyn Fn(&str, &ReducerContext) -> ReducerResult<ReducerOutput>>;

/// A registry for managing and executing reducers.
///
/// The registry provides a centralized place to register reducers by name
/// and execute them with input data.
#[derive(Default)]
pub struct ReducerRegistry {
    reducers: Vec<(&'static str, ReducerFn)>,
}

impl ReducerRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            reducers: Vec::new(),
        }
    }

    /// Register a reducer with the registry.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The reducer type (must implement `Reducer`)
    ///
    /// # Arguments
    ///
    /// * `reducer` - The reducer instance to register
    pub fn register<R>(&mut self, reducer: R)
    where
        R: Reducer<Input = String, Output = ReducerOutput> + 'static,
    {
        let name = reducer.name();
        let reducer_fn = Box::new(move |input: &str, context: &ReducerContext| {
            let input_string = input.to_string();
            reducer.reduce(&input_string, context)
        });

        self.reducers.push((name, reducer_fn));
    }

    /// Execute a reducer by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the reducer to execute
    /// * `input` - The input data to process
    /// * `context` - The context containing output format and stats flag
    ///
    /// # Returns
    ///
    /// Returns the reducer output on success, or a `ReducerError` on failure.
    pub fn execute(
        &self,
        name: &str,
        input: &str,
        context: &ReducerContext,
    ) -> ReducerResult<ReducerOutput> {
        for (reducer_name, reducer_fn) in &self.reducers {
            if reducer_name == &name {
                return reducer_fn(input, context);
            }
        }
        Err(ReducerError::NotImplemented(format!(
            "Reducer '{}' not found",
            name
        )))
    }

    /// Get a list of all registered reducer names.
    pub fn reducer_names(&self) -> Vec<&'static str> {
        self.reducers.iter().map(|(name, _)| *name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reducer_context_creation() {
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: true,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert_eq!(context.format, OutputFormat::Json);
        assert!(context.stats);
        assert_eq!(context.enabled_formats.len(), 1);
    }

    #[test]
    fn test_reducer_context_has_conflicting_formats() {
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json, OutputFormat::Csv],
        };

        assert!(context.has_conflicting_formats());

        let context_no_conflict = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert!(!context_no_conflict.has_conflicting_formats());
    }

    #[test]
    fn test_reducer_error_display() {
        let err = ReducerError::NotImplemented("test".to_string());
        assert_eq!(format!("{}", err), "Not implemented: test");

        let err = ReducerError::ProcessingError {
            message: "failed".to_string(),
        };
        assert_eq!(format!("{}", err), "Processing error: failed");

        let err = ReducerError::InvalidInput("bad input".to_string());
        assert_eq!(format!("{}", err), "Invalid input: bad input");

        let err = ReducerError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

    #[test]
    fn test_reducer_error_helpers() {
        let err = ReducerError::NotImplemented("test".to_string());
        assert!(err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::InvalidInput("test".to_string());
        assert!(!err.is_not_implemented());
        assert!(err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::IoError("test".to_string());
        assert!(!err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::ProcessingError {
            message: "test".to_string(),
        };
        assert!(!err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(err.is_processing_error());
    }

    // ============================================================
    // ReducerOutput Tests
    // ============================================================

    #[test]
    fn test_reducer_output_new() {
        let data = vec!["item1", "item2", "item3"];
        let output = ReducerOutput::new(&data);

        assert!(!output.is_empty);
        assert!(output.metadata.is_none());
        assert!(output.stats.is_none());
    }

    #[test]
    fn test_reducer_output_empty() {
        let output = ReducerOutput::empty();

        assert!(output.is_empty);
        assert!(output.data.is_null());
    }

    #[test]
    fn test_reducer_output_with_metadata() {
        let metadata = ReducerMetadata {
            reducer: "test".to_string(),
            items_processed: 10,
            items_filtered: 2,
            duration_ms: 50,
            custom: None,
        };

        let output = ReducerOutput::new(vec![1, 2, 3]).with_metadata(metadata);

        assert!(output.metadata.is_some());
        let meta = output.metadata.unwrap();
        assert_eq!(meta.reducer, "test");
        assert_eq!(meta.items_processed, 10);
        assert_eq!(meta.items_filtered, 2);
    }

    #[test]
    fn test_reducer_output_with_stats() {
        let stats = ReducerStats::new(1000, 500, 100, 50);

        let output = ReducerOutput::new(vec![1, 2, 3]).with_stats(stats);

        assert!(output.stats.is_some());
        let s = output.stats.unwrap();
        assert_eq!(s.input_bytes, 1000);
        assert_eq!(s.output_bytes, 500);
        assert_eq!(s.reduction_ratio, 0.5);
        assert_eq!(s.input_tokens, 250);  // 1000 / 4
        assert_eq!(s.output_tokens, 125); // 500 / 4
        assert_eq!(s.token_reduction_ratio, 0.5);
    }

    #[test]
    fn test_reducer_output_with_summary() {
        let output = ReducerOutput::new(vec![1, 2, 3]).with_summary("3 items processed");

        assert_eq!(output.summary, Some("3 items processed".to_string()));
    }

    #[test]
    fn test_reducer_output_with_items() {
        let items = vec![
            ReducerItem::new("key1", "value1"),
            ReducerItem::new("key2", "value2"),
        ];

        let output = ReducerOutput::new(Vec::<i32>::new()).with_items(items);

        assert_eq!(output.items.len(), 2);
        assert_eq!(output.items[0].key, "key1");
        assert_eq!(output.items[1].value, "value2");
    }

    #[test]
    fn test_reducer_output_with_sections() {
        let section1 = ReducerSection::new("Section 1")
            .with_items(vec![ReducerItem::new("a", "1")]);

        let section2 = ReducerSection::new("Section 2")
            .with_count(5)
            .with_items(vec![ReducerItem::new("b", "2")]);

        let output = ReducerOutput::new(Vec::<i32>::new()).with_sections(vec![section1, section2]);

        assert_eq!(output.sections.len(), 2);
        assert_eq!(output.sections[0].name, "Section 1");
        assert_eq!(output.sections[1].count, Some(5));
    }

    #[test]
    fn test_reducer_output_format_json() {
        let output = ReducerOutput::new(vec![1, 2, 3]);
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("\"data\""));
        // JSON may have spaces in array output
        assert!(formatted.contains("1") && formatted.contains("2") && formatted.contains("3"));
    }

    #[test]
    fn test_reducer_output_format_compact() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_summary("test summary")
            .with_sections(vec![
                ReducerSection::new("Files")
                    .with_count(3)
                    .with_items(vec![
                        ReducerItem::new("file1.txt", "100"),
                        ReducerItem::new("file2.txt", "200"),
                    ]),
            ]);

        let context = ReducerContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("test summary"));
        assert!(formatted.contains("Files (3):"));
        assert!(formatted.contains("file1.txt"));
    }

    #[test]
    fn test_reducer_output_format_raw() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("item1", "value1"),
                ReducerItem::new("item2", "value2"),
            ]);

        let context = ReducerContext {
            format: OutputFormat::Raw,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("item1"));
        assert!(formatted.contains("item2"));
    }

    #[test]
    fn test_reducer_output_format_agent() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_summary("Agent summary")
            .with_sections(vec![
                ReducerSection::new("Results")
                    .with_items(vec![
                        ReducerItem::new("result1", "value1").with_label("label1"),
                    ]),
            ]);

        let context = ReducerContext {
            format: OutputFormat::Agent,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("SUMMARY: Agent summary"));
        assert!(formatted.contains("## Results"));
        assert!(formatted.contains("[label1]"));
    }

    #[test]
    fn test_reducer_output_format_csv() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1").with_label("label1"),
                ReducerItem::new("key2", "value2"),
            ]);

        let context = ReducerContext {
            format: OutputFormat::Csv,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("key,value,label"));
        assert!(formatted.contains("key1,value1,label1"));
        assert!(formatted.contains("key2,value2,"));
    }

    #[test]
    fn test_reducer_output_format_tsv() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1"),
                ReducerItem::new("key2", "value2"),
            ]);

        let context = ReducerContext {
            format: OutputFormat::Tsv,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("key\tvalue\tlabel"));
        assert!(formatted.contains("key1\tvalue1\t"));
    }

    #[test]
    fn test_reducer_output_empty_format() {
        let output = ReducerOutput::empty();
        let context = ReducerContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };

        let formatted = output.format(&context);
        assert!(formatted.contains("(empty)"));
    }

    // ============================================================
    // ReducerItem Tests
    // ============================================================

    #[test]
    fn test_reducer_item_new() {
        let item = ReducerItem::new("key", "value");

        assert_eq!(item.key, "key");
        assert_eq!(item.value, "value");
        assert!(item.label.is_none());
        assert!(item.data.is_none());
    }

    #[test]
    fn test_reducer_item_with_label() {
        let item = ReducerItem::new("key", "value").with_label("label");

        assert_eq!(item.label, Some("label".to_string()));
    }

    #[test]
    fn test_reducer_item_with_data() {
        let data = serde_json::json!({"extra": "info"});
        let item = ReducerItem::new("key", "value").with_data(data.clone());

        assert_eq!(item.data, Some(data));
    }

    // ============================================================
    // ReducerSection Tests
    // ============================================================

    #[test]
    fn test_reducer_section_new() {
        let section = ReducerSection::new("Test Section");

        assert_eq!(section.name, "Test Section");
        assert!(section.count.is_none());
        assert!(section.items.is_empty());
    }

    #[test]
    fn test_reducer_section_with_count() {
        let section = ReducerSection::new("Test").with_count(10);

        assert_eq!(section.count, Some(10));
    }

    #[test]
    fn test_reducer_section_with_items() {
        let items = vec![
            ReducerItem::new("a", "1"),
            ReducerItem::new("b", "2"),
        ];

        let section = ReducerSection::new("Test").with_items(items);

        assert_eq!(section.items.len(), 2);
    }

    #[test]
    fn test_reducer_section_add_item() {
        let mut section = ReducerSection::new("Test");
        section.add_item(ReducerItem::new("a", "1"));

        assert_eq!(section.items.len(), 1);
        assert_eq!(section.items[0].key, "a");
    }

    // ============================================================
    // ReducerMetadata Tests
    // ============================================================

    #[test]
    fn test_reducer_metadata_default() {
        let metadata = ReducerMetadata::default();

        assert!(metadata.reducer.is_empty());
        assert_eq!(metadata.items_processed, 0);
        assert_eq!(metadata.items_filtered, 0);
        assert_eq!(metadata.duration_ms, 0);
        assert!(metadata.custom.is_none());
    }

    // ============================================================
    // ReducerStats Tests
    // ============================================================

    #[test]
    fn test_reducer_stats_default() {
        let stats = ReducerStats::default();

        assert_eq!(stats.input_bytes, 0);
        assert_eq!(stats.output_bytes, 0);
        assert_eq!(stats.reduction_ratio, 0.0);
        assert_eq!(stats.input_lines, 0);
        assert_eq!(stats.output_lines, 0);
        assert_eq!(stats.input_tokens, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.token_reduction_ratio, 0.0);
    }

    #[test]
    fn test_reducer_stats_new() {
        let stats = ReducerStats::new(4000, 1000, 200, 50);

        assert_eq!(stats.input_bytes, 4000);
        assert_eq!(stats.output_bytes, 1000);
        assert_eq!(stats.reduction_ratio, 0.25);
        assert_eq!(stats.input_lines, 200);
        assert_eq!(stats.output_lines, 50);
        assert_eq!(stats.input_tokens, 1000);  // 4000 / 4
        assert_eq!(stats.output_tokens, 250);  // 1000 / 4
        assert_eq!(stats.token_reduction_ratio, 0.25);
    }

    #[test]
    fn test_reducer_stats_zero_input() {
        let stats = ReducerStats::new(0, 0, 0, 0);

        assert_eq!(stats.reduction_ratio, 0.0);
        assert_eq!(stats.token_reduction_ratio, 0.0);
    }

    // ============================================================
    // BaseReducer Tests
    // ============================================================

    #[test]
    fn test_base_reducer_creation() {
        use serde::Deserialize;
        
        #[derive(Serialize, Deserialize, Debug)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        assert_eq!(reducer.name(), "test");
    }

    #[test]
    fn test_base_reducer_valid_json_input() {
        use serde::Deserialize;
        
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = r#"{"value": 42}"#.to_string();
        let result = reducer.reduce(&input, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestData { value: 42 });
    }

    #[test]
    fn test_base_reducer_invalid_json_input() {
        use serde::Deserialize;
        
        #[derive(Serialize, Deserialize, Debug)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = "not valid json".to_string();
        let result = reducer.reduce(&input, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_invalid_input());
    }

    // ============================================================
    // ReducerRegistry Tests
    // ============================================================

    #[test]
    fn test_reducer_registry_new() {
        let registry = ReducerRegistry::new();
        assert!(registry.reducer_names().is_empty());
    }

    #[test]
    fn test_reducer_registry_default() {
        let registry = ReducerRegistry::default();
        assert!(registry.reducer_names().is_empty());
    }

    #[test]
    fn test_reducer_registry_register() {
        struct TestReducer;

        impl Reducer for TestReducer {
            type Input = String;
            type Output = ReducerOutput;

            fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
                Ok(ReducerOutput::new(input.clone()))
            }

            fn name(&self) -> &'static str {
                "test_reducer"
            }
        }

        let mut registry = ReducerRegistry::new();
        registry.register(TestReducer);

        assert_eq!(registry.reducer_names().len(), 1);
        assert!(registry.reducer_names().contains(&"test_reducer"));
    }

    #[test]
    fn test_reducer_registry_execute() {
        struct TestReducer;

        impl Reducer for TestReducer {
            type Input = String;
            type Output = ReducerOutput;

            fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
                Ok(ReducerOutput::new(input.clone())
                    .with_summary("Processed"))
            }

            fn name(&self) -> &'static str {
                "test_reducer"
            }
        }

        let mut registry = ReducerRegistry::new();
        registry.register(TestReducer);

        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let result = registry.execute("test_reducer", "test input", &context);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.summary, Some("Processed".to_string()));
    }

    #[test]
    fn test_reducer_registry_execute_not_found() {
        let registry = ReducerRegistry::new();
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let result = registry.execute("nonexistent", "input", &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_not_implemented());
    }

    // ============================================================
    // Escape CSV Tests
    // ============================================================

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("simple"), "simple");
    }

    #[test]
    fn test_escape_csv_with_comma() {
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_with_quotes() {
        assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_escape_csv_with_newline() {
        assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
    }

    // ============================================================
    // TruncationInfo Tests
    // ============================================================

    #[test]
    fn test_truncation_info_none() {
        let info = TruncationInfo::none();

        assert!(!info.is_truncated);
        assert!(info.total_available.is_none());
        assert!(info.items_shown.is_none());
        assert!(info.items_hidden.is_none());
        assert!(info.reason.is_none());
        assert!(info.threshold.is_none());
        assert!(info.warning.is_none());
    }

    #[test]
    fn test_truncation_info_limited_no_hidden() {
        let info = TruncationInfo::limited(10, 10, 20);

        assert!(!info.is_truncated);
        assert_eq!(info.total_available, Some(10));
        assert_eq!(info.items_shown, Some(10));
        assert_eq!(info.items_hidden, Some(0));
        assert_eq!(info.reason, Some("limit".to_string()));
        assert_eq!(info.threshold, Some(20));
        assert!(info.warning.is_none());
    }

    #[test]
    fn test_truncation_info_limited_with_hidden() {
        let info = TruncationInfo::limited(100, 20, 20);

        assert!(info.is_truncated);
        assert_eq!(info.total_available, Some(100));
        assert_eq!(info.items_shown, Some(20));
        assert_eq!(info.items_hidden, Some(80));
        assert_eq!(info.reason, Some("limit".to_string()));
        assert_eq!(info.threshold, Some(20));
        assert!(info.warning.is_some());
        assert!(info.warning.unwrap().contains("20 of 100"));
    }

    #[test]
    fn test_truncation_info_size_threshold() {
        let info = TruncationInfo::size_threshold(10000, 5000, 5000);

        assert!(info.is_truncated);
        assert_eq!(info.total_available, Some(10000));
        assert_eq!(info.items_shown, Some(5000));
        assert_eq!(info.items_hidden, Some(5000));
        assert_eq!(info.reason, Some("size_threshold".to_string()));
        assert_eq!(info.threshold, Some(5000));
        assert!(info.warning.is_some());
        assert!(info.warning.unwrap().contains("5000 of 10000 bytes"));
    }

    #[test]
    fn test_truncation_info_detected() {
        let info = TruncationInfo::detected("incomplete_json", 500);

        assert!(info.is_truncated);
        assert!(info.total_available.is_none());
        assert_eq!(info.items_shown, Some(500));
        assert!(info.items_hidden.is_none());
        assert_eq!(info.reason, Some("detected".to_string()));
        assert!(info.threshold.is_none());
        assert!(info.warning.is_some());
        assert!(info.warning.as_ref().unwrap().contains("incomplete_json"));
    }

    #[test]
    fn test_truncation_info_detect_from_output_no_truncation() {
        let output = "This is normal output\nWith multiple lines\nNo truncation here";
        let info = TruncationInfo::detect_from_output(output);

        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_truncated_marker() {
        let output = "Some output\n... truncated";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
        assert_eq!(info.reason, Some("detected".to_string()));
    }

    #[test]
    fn test_truncation_info_detect_from_output_truncated_brackets() {
        let output = "Results [truncated]";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_showing_first() {
        let output = "Showing first 10 results...";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_more_results() {
        let output = "10 items shown, more results available";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_incomplete_json_array() {
        let output = "[1, 2, 3,";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
        assert!(info.warning.as_ref().unwrap().contains("incomplete_json"));
    }

    #[test]
    fn test_truncation_info_detect_from_output_incomplete_json_object() {
        let output = r#"{"key": "value""#;
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_complete_json() {
        let output = r#"{"key": "value"}"#;
        let info = TruncationInfo::detect_from_output(output);

        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_complete_array() {
        let output = "[1, 2, 3]";
        let info = TruncationInfo::detect_from_output(output);

        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_cutoff_line_ellipsis() {
        // Last line ends with ... (more than 3 chars total)
        let output = "Some text here\nAnd more content...";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_detect_from_output_cutoff_and() {
        let output = "Item 1\nItem 2\n and";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    #[test]
    fn test_truncation_info_is_truncated_method() {
        let truncated = TruncationInfo::detected("test", 100);
        assert!(truncated.is_truncated());

        let not_truncated = TruncationInfo::none();
        assert!(!not_truncated.is_truncated());
    }

    #[test]
    fn test_truncation_info_summary() {
        let info = TruncationInfo::limited(100, 20, 20);
        let summary = info.summary();

        assert!(summary.is_some());
        assert!(summary.unwrap().contains("20"));
    }

    #[test]
    fn test_truncation_info_summary_none() {
        let info = TruncationInfo::none();
        let summary = info.summary();

        assert!(summary.is_none());
    }

    #[test]
    fn test_truncation_info_summary_minimal() {
        let mut info = TruncationInfo::default();
        info.is_truncated = true;
        info.items_shown = Some(10);
        info.items_hidden = None;
        info.warning = None;

        let summary = info.summary();
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "Output was truncated");
    }

    #[test]
    fn test_truncation_info_summary_with_counts() {
        let mut info = TruncationInfo::default();
        info.is_truncated = true;
        info.items_shown = Some(10);
        info.items_hidden = Some(5);
        info.warning = None;

        let summary = info.summary();
        assert!(summary.is_some());
        assert!(summary.unwrap().contains("10 items shown"));
    }

    #[test]
    fn test_truncation_info_detect_case_insensitive() {
        let output = "OUTPUT TRUNCATED due to size";
        let info = TruncationInfo::detect_from_output(output);

        assert!(info.is_truncated);
    }

    // ============================================================
    // TruncationConfig Tests
    // ============================================================

    #[test]
    fn test_truncation_config_default() {
        let config = TruncationConfig::default();

        assert!(config.max_items.is_none());
        assert!(config.max_bytes.is_none());
        assert!(config.detect_patterns);
        assert!(config.include_warnings);
    }

    #[test]
    fn test_truncation_config_with_max_items() {
        let config = TruncationConfig::with_max_items(50);

        assert_eq!(config.max_items, Some(50));
        assert!(config.max_bytes.is_none());
        assert!(config.detect_patterns);
    }

    #[test]
    fn test_truncation_config_with_max_bytes() {
        let config = TruncationConfig::with_max_bytes(1024);

        assert!(config.max_items.is_none());
        assert_eq!(config.max_bytes, Some(1024));
        assert!(config.detect_patterns);
    }

    #[test]
    fn test_truncation_config_truncate_items_no_limit() {
        let config = TruncationConfig::default();
        let items = vec![1, 2, 3, 4, 5];
        let (result, info) = config.truncate_items(items);

        assert_eq!(result.len(), 5);
        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_config_truncate_items_with_limit() {
        let config = TruncationConfig::with_max_items(3);
        let items = vec![1, 2, 3, 4, 5];
        let (result, info) = config.truncate_items(items);

        assert_eq!(result.len(), 3);
        assert!(info.is_truncated);
        assert_eq!(info.total_available, Some(5));
        assert_eq!(info.items_shown, Some(3));
        assert_eq!(info.items_hidden, Some(2));
    }

    #[test]
    fn test_truncation_config_truncate_items_within_limit() {
        let config = TruncationConfig::with_max_items(10);
        let items = vec![1, 2, 3];
        let (result, info) = config.truncate_items(items);

        assert_eq!(result.len(), 3);
        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_config_truncate_output_no_limit() {
        let config = TruncationConfig {
            detect_patterns: false,
            ..Default::default()
        };
        let output = "Hello, world!".to_string();
        let (result, info) = config.truncate_output(output);

        assert_eq!(result, "Hello, world!");
        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_config_truncate_output_with_byte_limit() {
        let config = TruncationConfig {
            max_bytes: Some(5),
            detect_patterns: false,
            ..Default::default()
        };
        let output = "Hello, world!".to_string();
        let (result, info) = config.truncate_output(output);

        assert_eq!(result.len(), 5);
        assert!(info.is_truncated);
        assert_eq!(info.reason, Some("size_threshold".to_string()));
    }

    #[test]
    fn test_truncation_config_truncate_output_detect_patterns() {
        let config = TruncationConfig {
            detect_patterns: true,
            ..Default::default()
        };
        let output = "Some data\n... truncated".to_string();
        let (result, info) = config.truncate_output(output);

        assert_eq!(result, "Some data\n... truncated");
        assert!(info.is_truncated);
        assert_eq!(info.reason, Some("detected".to_string()));
    }

    #[test]
    fn test_truncation_config_truncate_output_no_detect_patterns() {
        let config = TruncationConfig {
            detect_patterns: false,
            ..Default::default()
        };
        let output = "Some data\n... truncated".to_string();
        let (result, info) = config.truncate_output(output);

        assert_eq!(result, "Some data\n... truncated");
        assert!(!info.is_truncated);
    }

    // ============================================================
    // Additional Edge Cases
    // ============================================================

    #[test]
    fn test_truncation_info_limited_saturating_sub() {
        // Test that saturating_sub works correctly when shown > total
        let info = TruncationInfo::limited(5, 10, 20);

        assert_eq!(info.items_hidden, Some(0));
        assert!(!info.is_truncated);
    }

    #[test]
    fn test_truncation_info_size_threshold_saturating_sub() {
        let info = TruncationInfo::size_threshold(100, 200, 200);

        assert_eq!(info.items_hidden, Some(0));
    }

    #[test]
    fn test_reducer_output_serialization() {
        let output = ReducerOutput::new(vec![1, 2, 3])
            .with_summary("Test summary");

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"summary\""));
        assert!(json.contains("Test summary"));
    }

    #[test]
    fn test_reducer_item_serialization() {
        let item = ReducerItem::new("key", "value")
            .with_label("label");

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"key\""));
        assert!(json.contains("\"value\""));
        assert!(json.contains("\"label\""));
    }

    #[test]
    fn test_reducer_item_serialization_skip_none() {
        let item = ReducerItem::new("key", "value");

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"key\""));
        assert!(!json.contains("\"label\""));
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_reducer_section_serialization() {
        let section = ReducerSection::new("Test")
            .with_count(5)
            .with_items(vec![ReducerItem::new("a", "1")]);

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("Test"));
        assert!(json.contains("\"count\""));
    }

    #[test]
    fn test_reducer_metadata_serialization() {
        let mut custom = HashMap::new();
        custom.insert("version".to_string(), "1.0".to_string());

        let metadata = ReducerMetadata {
            reducer: "test".to_string(),
            items_processed: 100,
            items_filtered: 10,
            duration_ms: 50,
            custom: Some(custom),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"reducer\""));
        assert!(json.contains("\"custom\""));
    }

    #[test]
    fn test_reducer_metadata_skip_none_custom() {
        let metadata = ReducerMetadata {
            reducer: "test".to_string(),
            items_processed: 100,
            items_filtered: 10,
            duration_ms: 50,
            custom: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(!json.contains("\"custom\""));
    }

    #[test]
    fn test_reducer_stats_serialization() {
        let stats = ReducerStats::new(1000, 500, 100, 50);

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"input_bytes\""));
        assert!(json.contains("\"output_bytes\""));
        assert!(json.contains("\"reduction_ratio\""));
        assert!(json.contains("\"input_tokens\""));
    }

    #[test]
    fn test_truncation_info_serialization() {
        let info = TruncationInfo::limited(100, 50, 50);

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"is_truncated\""));
        assert!(json.contains("\"total_available\""));
    }

    #[test]
    fn test_truncation_info_serialization_skip_none() {
        let info = TruncationInfo::none();

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"is_truncated\""));
        // These should be skipped due to skip_serializing_if
        assert!(!json.contains("\"total_available\""));
        assert!(!json.contains("\"warning\""));
    }

    #[test]
    fn test_reducer_error_from_processing() {
        let err = ReducerError::ProcessingError {
            message: "Something went wrong".to_string(),
        };

        assert!(err.is_processing_error());
        assert!(!err.is_not_implemented());
    }

    #[test]
    fn test_reducer_output_with_exit_code() {
        let output = ReducerOutput {
            data: serde_json::Value::Null,
            metadata: None,
            stats: None,
            is_empty: false,
            summary: None,
            items: vec![],
            sections: vec![],
            exit_code: Some(0),
        };

        assert_eq!(output.exit_code, Some(0));
    }

    #[test]
    fn test_format_csv_fallback_to_json() {
        // When no items, format_csv falls back to JSON
        let output = ReducerOutput::new(vec![1, 2, 3]);
        let csv = output.format_csv();

        // Should be JSON format since items is empty
        assert!(csv.starts_with('{'));
    }

    #[test]
    fn test_format_tsv_fallback_to_json() {
        // When no items, format_tsv falls back to JSON
        let output = ReducerOutput::new(vec![1, 2, 3]);
        let tsv = output.format_tsv();

        // Should be JSON format since items is empty
        assert!(tsv.starts_with('{'));
    }

    #[test]
    fn test_format_raw_with_sections() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_sections(vec![
                ReducerSection::new("Files")
                    .with_items(vec![
                        ReducerItem::new("file1.txt", "100"),
                        ReducerItem::new("file2.txt", "200"),
                    ]),
            ]);

        let raw = output.format_raw();
        assert!(raw.contains("file1.txt"));
        assert!(raw.contains("file2.txt"));
    }

    #[test]
    fn test_reducer_output_format_agent_with_metadata() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_metadata(ReducerMetadata {
                reducer: "test".to_string(),
                items_processed: 10,
                items_filtered: 2,
                duration_ms: 5,
                custom: None,
            });

        let agent = output.format_agent();
        assert!(agent.contains("## Metadata"));
        assert!(agent.contains("reducer: test"));
    }

    #[test]
    fn test_format_compact_items_without_label() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1"),
            ]);

        let compact = output.format_compact();
        assert!(compact.contains("key1: value1"));
    }

    #[test]
    fn test_format_compact_section_without_count() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_sections(vec![
                ReducerSection::new("Files")
                    .with_items(vec![ReducerItem::new("file.txt", "100")]),
            ]);

        let compact = output.format_compact();
        assert!(compact.contains("Files:"));
        assert!(!compact.contains("Files ("));
    }

    #[test]
    fn test_format_agent_items_without_sections() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1").with_label("label1"),
                ReducerItem::new("key2", "value2"),
            ]);

        let agent = output.format_agent();
        assert!(agent.contains("## Items"));
        assert!(agent.contains("key1 [label1]: value1"));
        assert!(agent.contains("key2: value2"));
    }
}
