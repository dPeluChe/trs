//! TSV formatter for tab-separated output.

use super::Formatter;
use crate::OutputFormat;

/// Name only: rows come from `router::handlers::common::{csv_row, tsv_row}`.
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
