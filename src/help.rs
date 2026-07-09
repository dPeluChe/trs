//! Help system module for trs (Token-Reducing Shell).
//!
//! This module contains comprehensive documentation and help text for all CLI commands,
//! flags, and usage examples.

/// Long about text for the main CLI.
pub const LONG_ABOUT: &str = "\
trs (Token-Reducing Shell) - Transform noisy terminal output into compact, structured signal

Reduces token consumption by 68-99% for developers, AI agents, and automation.
Just prefix any command with trs:

    trs git status               # 80% reduction
    trs git log -10              # 90% reduction
    trs ls -la                   # 82% reduction
    trs npm test                 # 90% reduction
    trs env                      # 68% reduction

FORMAT FLAGS (work before or after the command):
    --json     Structured JSON output
    --csv      CSV tabular output
    --tsv      TSV tabular output
    --agent    AI-optimized format
    --compact  Human-readable (default)
    --raw      Unprocessed passthrough
    --stats    Show reduction metrics

EXAMPLES:
    trs git status --json        # JSON output
    trs err cargo build          # Show only errors
    trs search src \"TODO\"        # Ripgrep search
    trs curl -I https://api.com  # HTTP headers compact
    trs stats                    # View token savings";

/// Help text for output format precedence.
#[allow(dead_code)]
pub const FORMAT_PRECEDENCE: &str = "\
OUTPUT FORMAT PRECEDENCE:
    When multiple format flags are specified, the following precedence applies:

    1. JSON (--json)     - Highest priority, most structured
    2. CSV (--csv)       - Structured tabular format
    3. TSV (--tsv)       - Tab-separated format
    4. Agent (--agent)   - AI-optimized format
    5. Compact (--compact) - Human-readable summary
    6. Raw (--raw)       - Unprocessed output

    Default: Compact (when no format flags are specified)

Examples:
    trs --json --csv search . \"test\"    # Uses JSON (higher precedence)
    trs --agent --compact search . \"x\"  # Uses Agent format
    trs search . \"pattern\"              # Uses Compact (default)";
pub use crate::help_text::*;
pub use crate::help_text_more::*;

/// Returns the help text for a specific command.
#[allow(dead_code)]
pub fn get_command_help(command: &str) -> Option<&'static str> {
    match command {
        "search" => Some(SEARCH_HELP),
        "replace" => Some(REPLACE_HELP),
        "tail" => Some(TAIL_HELP),
        "clean" => Some(CLEAN_HELP),
        "parse" => Some(PARSE_HELP),
        "html2md" => Some(HTML2MD_HELP),
        "txt2md" => Some(TXT2MD_HELP),
        "trim" => Some(TRIM_HELP),
        "run" => Some(RUN_HELP),
        "read" => Some(READ_HELP),
        "json" => Some(JSON_HELP),
        "err" => Some(ERR_HELP),
        "stats" => Some(STATS_HELP),
        "doctor" => Some(DOCTOR_HELP),
        "benchmark" => Some(BENCHMARK_HELP),
        "diff" => Some(DIFF_HELP),
        "ingest" => Some(INGEST_HELP),
        _ => None,
    }
}

/// Returns the format precedence help text.
#[allow(dead_code)]
pub fn get_format_precedence_help() -> &'static str {
    FORMAT_PRECEDENCE
}

#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
