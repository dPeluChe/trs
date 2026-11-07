//! Help system module for trs (Token-Reducing Shell).
//!
//! This module contains comprehensive documentation and help text for all CLI commands,
//! flags, and usage examples.

/// Long about text for the main CLI.
pub const LONG_ABOUT: &str = "\
trs (Token-Reducing Shell) - Transform noisy terminal output into compact, structured signal

Cuts the tokens AI agents and automation spend on terminal output (about 70%
on real agent commands; measured per command in docs/development/benchmarks).
When it leaves something out it says so, and where the rest is.
Just prefix any command with trs:

    trs git status
    trs git log -10
    trs ls -la
    trs npm test
    trs env

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

pub use crate::help_text::*;
pub use crate::help_text_more::*;
