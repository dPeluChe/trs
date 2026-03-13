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
            if let Some(first) = self.items.first() {
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

impl<T: Serialize> BaseReducer<T> {
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

impl<T: Serialize> Reducer for BaseReducer<T> {
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
        R: Reducer<Output = ReducerOutput> + 'static,
    {
        let name = reducer.name();
        let reducer_fn = Box::new(move |input: &str, context: &ReducerContext| {
            reducer.reduce(input, context)
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
        let stats = ReducerStats {
            input_bytes: 1000,
            output_bytes: 500,
            reduction_ratio: 0.5,
            input_lines: 100,
            output_lines: 50,
        };

        let output = ReducerOutput::new(vec![1, 2, 3]).with_stats(stats);

        assert!(output.stats.is_some());
        let s = output.stats.unwrap();
        assert_eq!(s.input_bytes, 1000);
        assert_eq!(s.output_bytes, 500);
        assert_eq!(s.reduction_ratio, 0.5);
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

        let output = ReducerOutput::new(vec![]).with_items(items);

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

        let output = ReducerOutput::new(vec![]).with_sections(vec![section1, section2]);

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
        assert!(formatted.contains("[1,2,3]"));
    }

    #[test]
    fn test_reducer_output_format_compact() {
        let output = ReducerOutput::new(vec![])
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
        let output = ReducerOutput::new(vec![])
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
        let output = ReducerOutput::new(vec![])
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
        let output = ReducerOutput::new(vec![])
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
        let output = ReducerOutput::new(vec![])
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
    }

    // ============================================================
    // BaseReducer Tests
    // ============================================================

    #[test]
    fn test_base_reducer_creation() {
        #[derive(Serialize)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        assert_eq!(reducer.name(), "test");
    }

    #[test]
    fn test_base_reducer_valid_json_input() {
        #[derive(Serialize, Debug, PartialEq)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = r#"{"value": 42}"#;
        let result = reducer.reduce(&input, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestData { value: 42 });
    }

    #[test]
    fn test_base_reducer_invalid_json_input() {
        #[derive(Serialize)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = "not valid json";
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
}
