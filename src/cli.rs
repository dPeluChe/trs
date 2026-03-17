use clap::{Parser, ValueEnum};

use crate::commands::Commands;
use crate::help;

/// TARS CLI - Transform noisy terminal output into compact, structured signal
///
/// A CLI toolkit for developers, automation pipelines, and AI agents.
#[derive(Parser)]
#[command(name = "trs", bin_name = "trs")]
#[command(version, about, long_about = Some(help::LONG_ABOUT))]
#[command(propagate_version = true)]
#[command(next_display_order = None)]
#[command(allow_external_subcommands = true)]
pub struct Cli {
    /// Output raw, unprocessed input
    #[arg(long, global = true)]
    pub raw: bool,

    /// Output in compact format (default for most commands)
    #[arg(long, global = true)]
    pub compact: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Output in CSV format
    #[arg(long, global = true)]
    pub csv: bool,

    /// Output in TSV format
    #[arg(long, global = true)]
    pub tsv: bool,

    /// Output in agent-optimized format (structured for AI consumption)
    #[arg(long, global = true)]
    pub agent: bool,

    /// Show execution statistics (input/output size, token reduction)
    #[arg(long, global = true)]
    pub stats: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Output format options with defined precedence rules.
///
/// # Precedence Order (highest to lowest)
///
/// When multiple output format flags are specified, the following precedence order applies:
///
/// 1. **JSON** (`--json`) - Highest priority, most structured format
/// 2. **CSV** (`--csv`) - Structured tabular format
/// 3. **TSV** (`--tsv`) - Tab-separated structured format
/// 4. **Agent** (`--agent`) - AI-optimized structured format
/// 5. **Compact** (`--compact`) - Reduced human-readable format
/// 6. **Raw** (`--raw`) - Unprocessed output
/// 7. **Default** - Falls back to Compact when no flags are specified
///
/// # Examples
///
/// ```ignore
/// // JSON wins over all other formats
/// trs --json --csv --agent search . "pattern"
///
/// // CSV wins over TSV, Agent, Compact, and Raw
/// trs --csv --tsv --compact search . "pattern"
///
/// // TSV wins over Agent, Compact, and Raw
/// trs --tsv --agent --raw search . "pattern"
///
/// // Agent wins over Compact and Raw
/// trs --agent --compact --raw search . "pattern"
///
/// // Compact wins over Raw
/// trs --compact --raw search . "pattern"
///
/// // Raw when only --raw is specified
/// trs --raw search . "pattern"
///
/// // Default (Compact) when no format flags are specified
/// trs search . "pattern"
/// ```
///
/// # Rationale
///
/// The precedence order is designed with the following principles:
///
/// - **Structured formats take priority**: JSON, CSV, and TSV provide machine-readable
///   structured output, which is more specific than human-readable formats.
/// - **JSON is most expressive**: JSON supports nested structures and complex data,
///   making it the highest priority structured format.
/// - **Agent format for AI**: The agent format is optimized for AI consumption and
///   takes precedence over general compact output.
/// - **Raw is the fallback for debugging**: When explicitly requested, raw output
///   provides unprocessed data for debugging purposes.
/// - **Compact is the default**: When no format is specified, compact output provides
///   a balance of information density and readability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Raw, unprocessed output (lowest precedence among explicit flags)
    Raw,
    /// Compact, summarized output (default when no format flags specified)
    #[default]
    Compact,
    /// JSON structured output (highest precedence)
    Json,
    /// CSV formatted output (second highest precedence)
    Csv,
    /// TSV formatted output (third highest precedence)
    Tsv,
    /// Agent-optimized format for AI consumption
    Agent,
}

/// Precedence rank for output formats (higher number = higher precedence)
pub(crate) const fn format_precedence(format: OutputFormat) -> u8 {
    match format {
        OutputFormat::Json => 6,
        OutputFormat::Csv => 5,
        OutputFormat::Tsv => 4,
        OutputFormat::Agent => 3,
        OutputFormat::Compact => 2,
        OutputFormat::Raw => 1,
    }
}

impl Cli {
    /// Returns the precedence order for output formats as a slice.
    ///
    /// The order is from highest to lowest precedence.
    pub fn output_format_precedence() -> &'static [OutputFormat] {
        &[
            OutputFormat::Json,
            OutputFormat::Csv,
            OutputFormat::Tsv,
            OutputFormat::Agent,
            OutputFormat::Compact,
            OutputFormat::Raw,
        ]
    }

    /// Determine the output format based on flag precedence.
    ///
    /// Uses the defined precedence order: json > csv > tsv > agent > compact > raw
    /// If no format flags are specified, returns the default format (Compact).
    pub fn output_format(&self) -> OutputFormat {
        // Check formats in precedence order (highest to lowest)
        if self.json {
            OutputFormat::Json
        } else if self.csv {
            OutputFormat::Csv
        } else if self.tsv {
            OutputFormat::Tsv
        } else if self.agent {
            OutputFormat::Agent
        } else if self.compact {
            OutputFormat::Compact
        } else if self.raw {
            OutputFormat::Raw
        } else {
            OutputFormat::default()
        }
    }

    /// Returns a list of all enabled output format flags.
    ///
    /// Useful for debugging or warning users about conflicting flags.
    pub fn enabled_format_flags(&self) -> Vec<OutputFormat> {
        let mut enabled = Vec::new();
        if self.json {
            enabled.push(OutputFormat::Json);
        }
        if self.csv {
            enabled.push(OutputFormat::Csv);
        }
        if self.tsv {
            enabled.push(OutputFormat::Tsv);
        }
        if self.agent {
            enabled.push(OutputFormat::Agent);
        }
        if self.compact {
            enabled.push(OutputFormat::Compact);
        }
        if self.raw {
            enabled.push(OutputFormat::Raw);
        }
        enabled
    }

    /// Returns true if multiple format flags are enabled (potential conflict).
    pub fn has_conflicting_format_flags(&self) -> bool {
        self.enabled_format_flags().len() > 1
    }

    /// Returns the precedence rank of the currently selected format.
    pub fn current_format_precedence(&self) -> u8 {
        format_precedence(self.output_format())
    }
}
