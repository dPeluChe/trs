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

pub(crate) mod output;
mod registry;
mod truncation;

#[cfg(test)]
mod tests;

use crate::OutputFormat;

// Re-export all public types (used by tests and future consumers)
#[allow(unused_imports)]
pub(crate) use output::escape_csv;
#[allow(unused_imports)]
pub use output::{ReducerItem, ReducerMetadata, ReducerOutput, ReducerSection, ReducerStats};
#[allow(unused_imports)]
pub use registry::{BaseReducer, ReducerRegistry};
#[allow(unused_imports)]
pub use truncation::{TruncationConfig, TruncationInfo};

// ============================================================
// Reducer Context
// ============================================================

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

// ============================================================
// Reducer Result and Error
// ============================================================

/// Result type for reducer operations.
pub type ReducerResult<T = ()> = Result<T, ReducerError>;

/// Error type for reducer operations.
#[derive(Debug, Clone)]
pub enum ReducerError {
    /// The reducer is not yet implemented.
    NotImplemented(String),
    /// An error occurred during processing.
    ProcessingError { message: String },
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
