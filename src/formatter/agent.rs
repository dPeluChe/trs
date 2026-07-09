//! Agent formatter for AI-optimized output.

use super::Formatter;
use crate::OutputFormat;

/// Formatter for AI agent-optimized output.
///
/// The agent formatter produces output that:
/// - Is optimized for AI consumption
/// - Uses structured markdown-like format
/// - Includes metadata sections
/// - Highlights key information
/// - Uses concise key-value pairs
/// - Groups related data with headers
#[allow(dead_code)]
pub struct AgentFormatter;

impl Formatter for AgentFormatter {
    fn name() -> &'static str {
        "agent"
    }

    fn format() -> OutputFormat {
        OutputFormat::Agent
    }
}
