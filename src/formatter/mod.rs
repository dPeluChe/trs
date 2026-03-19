//! Formatter system for TARS CLI.
//!
//! This module provides a centralized formatter interface for producing output
//! in different formats (Compact, JSON, CSV, TSV, Agent, Raw).
//!
//! # Architecture
//!
//! The formatter system is built around a trait-based design:
//!
//! - `Formatter` - Core trait for formatting data to string output
//! - `CompactFormatter` - Formats data in a human-readable compact format
//! - `JsonFormatter` - Formats data as JSON
//! - `CsvFormatter` - Formats data as CSV
//! - `TsvFormatter` - Formats data as TSV
//! - `AgentFormatter` - Formats data for AI consumption
//! - `RawFormatter` - Formats data with minimal processing
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::formatter::{Formatter, CompactFormatter};
//! use crate::OutputFormat;
//!
//! let status = GitStatus { /* ... */ };
//! let output = CompactFormatter::format_git_status(&status);
//! println!("{}", output);
//! ```

mod compact;
mod compact_schema_git;
mod compact_schema_output;
mod json;
mod json_schema;
mod csv;
mod csv_schema;
mod tsv;
mod tsv_schema;
mod agent;
mod agent_schema;
mod raw;
pub mod helpers;

#[cfg(test)]
mod tests;

pub use compact::CompactFormatter;
pub use json::JsonFormatter;
pub use csv::CsvFormatter;
pub use tsv::TsvFormatter;
pub use agent::AgentFormatter;
pub use raw::RawFormatter;
#[allow(unused_imports)]
pub use helpers::*;

use crate::OutputFormat;

// ============================================================
// Core Formatter Trait
// ============================================================

/// Core trait for formatting data to string output.
///
/// This trait defines the interface that all formatters must implement.
/// Each formatter handles a specific output format (Compact, JSON, etc.).
#[allow(dead_code)]
pub trait Formatter {
    /// Returns the name of this formatter.
    fn name() -> &'static str;

    /// Returns the output format this formatter handles.
    fn format() -> OutputFormat;
}

// ============================================================
// Format Selection
// ============================================================

/// Select the appropriate formatter for the given output format.
///
/// This is a convenience function for dispatching to the right formatter
/// based on the output format.
#[allow(dead_code)]
pub fn select_formatter(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Json => JsonFormatter::name(),
        OutputFormat::Csv => CsvFormatter::name(),
        OutputFormat::Tsv => TsvFormatter::name(),
        OutputFormat::Agent => AgentFormatter::name(),
        OutputFormat::Compact => CompactFormatter::name(),
        OutputFormat::Raw => RawFormatter::name(),
    }
}
