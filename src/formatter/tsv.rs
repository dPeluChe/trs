//! TSV formatter for tab-separated output.

use super::Formatter;
use crate::OutputFormat;

/// Formatter for TSV (Tab-Separated Values) output.
///
/// The TSV formatter produces tabular output that:
/// - Has a header row
/// - Uses tabs as delimiters
/// - Properly escapes special characters
/// - Is compatible with data processing tools
#[allow(dead_code)]
pub struct TsvFormatter;

impl Formatter for TsvFormatter {
    fn name() -> &'static str {
        "tsv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Tsv
    }
}
