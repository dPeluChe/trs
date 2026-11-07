//! CSV formatter for comma-separated output.

use super::Formatter;
use crate::OutputFormat;

/// Name only: rows come from `router::handlers::common::{csv_row, tsv_row}`.
#[allow(dead_code)]
pub struct CsvFormatter;

impl Formatter for CsvFormatter {
    fn name() -> &'static str {
        "csv"
    }

    fn format() -> OutputFormat {
        OutputFormat::Csv
    }
}
