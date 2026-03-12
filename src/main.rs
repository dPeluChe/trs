use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod help;

/// TARS CLI - Transform noisy terminal output into compact, structured signal
///
/// A CLI toolkit for developers, automation pipelines, and AI agents.
#[derive(Parser)]
#[command(name = "trs", bin_name = "trs")]
#[command(version, about, long_about = Some(help::LONG_ABOUT))]
#[command(propagate_version = true)]
#[command(next_display_order = None)]
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
    pub command: Commands,
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
const fn format_precedence(format: OutputFormat) -> u8 {
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

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a command and process its output
    #[command(long_about = help::RUN_HELP)]
    #[command(allow_external_subcommands = true)]
    Run {
        /// The command to execute
        #[arg(required = true)]
        command: String,

        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Parse structured input from stdin or file
    #[command(long_about = help::PARSE_HELP)]
    Parse {
        #[command(subcommand)]
        parser: ParseCommands,
    },

    /// Search for patterns in files (ripgrep-powered)
    #[command(long_about = help::SEARCH_HELP)]
    Search {
        /// Path to search in
        path: PathBuf,

        /// Search pattern (regex supported)
        query: String,

        /// File extension filter (e.g., "rs", "ts")
        #[arg(short = 'e', long)]
        extension: Option<String>,

        /// Case-insensitive search
        #[arg(short, long)]
        ignore_case: bool,

        /// Number of context lines to show around matches
        #[arg(short = 'C', long)]
        context: Option<usize>,

        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Search and replace patterns in files
    #[command(long_about = help::REPLACE_HELP)]
    Replace {
        /// Path to search in
        path: PathBuf,

        /// Search pattern
        search: String,

        /// Replacement string
        replace: String,

        /// File extension filter
        #[arg(short = 'e', long)]
        extension: Option<String>,

        /// Preview changes without modifying files
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Tail a file with compact log output
    #[command(long_about = help::TAIL_HELP)]
    Tail {
        /// File to tail
        file: PathBuf,

        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "10")]
        lines: usize,

        /// Filter for error lines only
        #[arg(short, long)]
        errors: bool,

        /// Follow the file for new lines (streaming mode)
        #[arg(short = 'f', long)]
        follow: bool,
    },

    /// Clean and format text output
    #[command(long_about = help::CLEAN_HELP)]
    Clean {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Remove ANSI escape codes
        #[arg(long)]
        no_ansi: bool,

        /// Collapse repeated blank lines
        #[arg(long)]
        collapse_blanks: bool,

        /// Collapse repeated lines
        #[arg(long)]
        collapse_repeats: bool,

        /// Trim whitespace from lines
        #[arg(long)]
        trim: bool,
    },

    /// Convert HTML to Markdown
    #[command(long_about = help::HTML2MD_HELP)]
    Html2md {
        /// Input HTML file or URL
        input: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include metadata in JSON format
        #[arg(long)]
        metadata: bool,
    },

    /// Convert plain text to Markdown
    #[command(long_about = help::TXT2MD_HELP)]
    Txt2md {
        /// Input text file (stdin if not specified)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ParseCommands {
    /// Parse git status output
    ///
    /// Transforms git status output into structured format showing
    /// branch info, staged/unstaged files, and untracked files.
    ///
    /// Example: git status | trs parse git-status
    GitStatus {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git diff output
    ///
    /// Transforms git diff output into structured format showing
    /// changed files and summary statistics.
    ///
    /// Example: git diff | trs parse git-diff
    GitDiff {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse ls output
    ///
    /// Transforms ls output into structured format separating
    /// directories, files, and hidden items.
    ///
    /// Example: ls -la | trs parse ls
    Ls {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse grep output
    ///
    /// Transforms grep results into structured format grouping
    /// matches by file with line numbers.
    ///
    /// Example: grep -rn "pattern" . | trs parse grep
    Grep {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse test runner output
    ///
    /// Transforms test runner output into structured format showing
    /// passed/failed/skipped counts and execution time.
    ///
    /// Supported runners: pytest, jest, vitest, npm, pnpm, bun
    ///
    /// Example: pytest | trs parse test --runner pytest
    Test {
        /// Test runner type (pytest, jest, vitest, npm, pnpm, bun)
        #[arg(short = 't', long, value_enum)]
        runner: Option<TestRunner>,

        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse log/tail output
    ///
    /// Transforms log streams into structured format detecting
    /// repeated lines and error/warning levels.
    ///
    /// Example: tail -f /var/log/app.log | trs parse logs
    Logs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

/// Supported test runners
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TestRunner {
    /// Python pytest
    Pytest,
    /// JavaScript Jest
    Jest,
    /// JavaScript Vitest
    Vitest,
    /// npm test
    Npm,
    /// pnpm test
    Pnpm,
    /// bun test
    Bun,
}

fn main() {
    let cli = Cli::parse();

    // For now, just acknowledge the parsed command
    // Actual command execution will be implemented later
    let format = cli.output_format();

    if cli.stats {
        eprintln!("Stats: enabled");
    }
    eprintln!("Output format: {:?}", format);

    match &cli.command {
        Commands::Run { command, args } => {
            eprintln!("Command: {} {:?}", command, args);
            // TODO: Implement command execution
            println!("Command execution not yet implemented");
        }
        Commands::Parse { parser } => {
            eprintln!("Parser: {:?}", parser);
            // TODO: Implement parsers
            println!("Parsing not yet implemented");
        }
        Commands::Search {
            path,
            query,
            extension,
            ignore_case,
            context,
            limit,
        } => {
            eprintln!(
                "Search: {:?} in {:?} (ext: {:?}, case: {}, context: {:?}, limit: {:?})",
                query, path, extension, !ignore_case, context, limit
            );
            // TODO: Implement search
            println!("Search not yet implemented");
        }
        Commands::Replace {
            path,
            search,
            replace,
            extension,
            dry_run,
        } => {
            eprintln!(
                "Replace: '{}' with '{}' in {:?} (ext: {:?}, dry_run: {})",
                search, replace, path, extension, dry_run
            );
            // TODO: Implement replace
            println!("Replace not yet implemented");
        }
        Commands::Tail {
            file,
            lines,
            errors,
            follow,
        } => {
            eprintln!(
                "Tail: {:?} ({} lines, errors: {}, follow: {})",
                file, lines, errors, follow
            );
            // TODO: Implement tail
            println!("Tail not yet implemented");
        }
        Commands::Clean {
            file,
            no_ansi,
            collapse_blanks,
            collapse_repeats,
            trim,
        } => {
            eprintln!(
                "Clean: {:?} (no_ansi: {}, collapse_blanks: {}, collapse_repeats: {}, trim: {})",
                file, no_ansi, collapse_blanks, collapse_repeats, trim
            );
            // TODO: Implement clean
            println!("Clean not yet implemented");
        }
        Commands::Html2md {
            input,
            output,
            metadata,
        } => {
            eprintln!(
                "Html2md: {:?} -> {:?} (metadata: {})",
                input, output, metadata
            );
            // TODO: Implement html2md
            println!("HTML to Markdown conversion not yet implemented");
        }
        Commands::Txt2md { input, output } => {
            eprintln!("Txt2md: {:?} -> {:?}", input, output);
            // TODO: Implement txt2md
            println!("Text to Markdown conversion not yet implemented");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        let cli = Cli::try_parse_from(["trs", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Compact);
    }

    #[test]
    fn test_output_format_json_precedence() {
        let cli = Cli::try_parse_from(["trs", "--json", "--compact", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_output_format_csv() {
        let cli = Cli::try_parse_from(["trs", "--csv", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_output_format_tsv() {
        let cli = Cli::try_parse_from(["trs", "--tsv", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_output_format_agent() {
        let cli = Cli::try_parse_from(["trs", "--agent", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    #[test]
    fn test_output_format_raw() {
        let cli = Cli::try_parse_from(["trs", "--raw", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Raw);
    }

    #[test]
    fn test_output_format_compact() {
        let cli = Cli::try_parse_from(["trs", "--compact", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Compact);
    }

    #[test]
    fn test_output_format_precedence_json_over_csv() {
        let cli = Cli::try_parse_from(["trs", "--json", "--csv", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_output_format_precedence_csv_over_tsv() {
        let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_output_format_precedence_tsv_over_agent() {
        let cli = Cli::try_parse_from(["trs", "--tsv", "--agent", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_output_format_precedence_agent_over_compact() {
        let cli = Cli::try_parse_from(["trs", "--agent", "--compact", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    #[test]
    fn test_stats_flag() {
        let cli = Cli::try_parse_from(["trs", "--stats", "search", ".", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.stats);
    }

    #[test]
    fn test_search_command_parsing() {
        let cli = Cli::try_parse_from([
            "trs",
            "search",
            "/path/to/dir",
            "pattern",
            "--extension",
            "rs",
            "--ignore-case",
            "--context",
            "3",
            "--limit",
            "100",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Search {
                path,
                query,
                extension,
                ignore_case,
                context,
                limit,
            } => {
                assert_eq!(path, PathBuf::from("/path/to/dir"));
                assert_eq!(query, "pattern");
                assert_eq!(extension, Some("rs".to_string()));
                assert!(ignore_case);
                assert_eq!(context, Some(3));
                assert_eq!(limit, Some(100));
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_replace_command_parsing() {
        let cli = Cli::try_parse_from([
            "trs",
            "replace",
            "/path/to/dir",
            "old",
            "new",
            "--extension",
            "ts",
            "--dry-run",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Replace {
                path,
                search,
                replace,
                extension,
                dry_run,
            } => {
                assert_eq!(path, PathBuf::from("/path/to/dir"));
                assert_eq!(search, "old");
                assert_eq!(replace, "new");
                assert_eq!(extension, Some("ts".to_string()));
                assert!(dry_run);
            }
            _ => panic!("Expected Replace command"),
        }
    }

    #[test]
    fn test_tail_command_parsing() {
        let cli = Cli::try_parse_from([
            "trs",
            "tail",
            "/var/log/app.log",
            "--lines",
            "50",
            "--errors",
            "--follow",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Tail {
                file,
                lines,
                errors,
                follow,
            } => {
                assert_eq!(file, PathBuf::from("/var/log/app.log"));
                assert_eq!(lines, 50);
                assert!(errors);
                assert!(follow);
            }
            _ => panic!("Expected Tail command"),
        }
    }

    #[test]
    fn test_clean_command_parsing() {
        let cli = Cli::try_parse_from([
            "trs",
            "clean",
            "--file",
            "input.txt",
            "--no-ansi",
            "--collapse-blanks",
            "--collapse-repeats",
            "--trim",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Clean {
                file,
                no_ansi,
                collapse_blanks,
                collapse_repeats,
                trim,
            } => {
                assert_eq!(file, Some(PathBuf::from("input.txt")));
                assert!(no_ansi);
                assert!(collapse_blanks);
                assert!(collapse_repeats);
                assert!(trim);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_parse_git_status() {
        let cli = Cli::try_parse_from(["trs", "parse", "git-status"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Parse { parser } => match parser {
                ParseCommands::GitStatus { file } => {
                    assert!(file.is_none());
                }
                _ => panic!("Expected GitStatus parser"),
            },
            _ => panic!("Expected Parse command"),
        }
    }

    #[test]
    fn test_parse_test_runner() {
        let cli = Cli::try_parse_from(["trs", "parse", "test", "--runner", "pytest"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Parse { parser } => match parser {
                ParseCommands::Test { runner, file } => {
                    assert_eq!(runner, Some(TestRunner::Pytest));
                    assert!(file.is_none());
                }
                _ => panic!("Expected Test parser"),
            },
            _ => panic!("Expected Parse command"),
        }
    }

    #[test]
    fn test_html2md_command() {
        let cli = Cli::try_parse_from([
            "trs",
            "html2md",
            "https://example.com",
            "--output",
            "out.md",
            "--metadata",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Html2md {
                input,
                output,
                metadata,
            } => {
                assert_eq!(input, "https://example.com");
                assert_eq!(output, Some(PathBuf::from("out.md")));
                assert!(metadata);
            }
            _ => panic!("Expected Html2md command"),
        }
    }

    #[test]
    fn test_txt2md_command() {
        let cli = Cli::try_parse_from([
            "trs",
            "txt2md",
            "--input",
            "input.txt",
            "--output",
            "out.md",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        match cli.command {
            Commands::Txt2md { input, output } => {
                assert_eq!(input, Some(PathBuf::from("input.txt")));
                assert_eq!(output, Some(PathBuf::from("out.md")));
            }
            _ => panic!("Expected Txt2md command"),
        }
    }

    // ============================================================
    // Output Format Precedence Tests
    // ============================================================

    #[test]
    fn test_precedence_order() {
        let precedence = Cli::output_format_precedence();
        assert_eq!(precedence.len(), 6, "Should have 6 output formats");
        assert_eq!(
            precedence[0],
            OutputFormat::Json,
            "JSON should have highest precedence"
        );
        assert_eq!(
            precedence[1],
            OutputFormat::Csv,
            "CSV should have second highest precedence"
        );
        assert_eq!(
            precedence[2],
            OutputFormat::Tsv,
            "TSV should have third highest precedence"
        );
        assert_eq!(
            precedence[3],
            OutputFormat::Agent,
            "Agent should have fourth highest precedence"
        );
        assert_eq!(
            precedence[4],
            OutputFormat::Compact,
            "Compact should have fifth highest precedence"
        );
        assert_eq!(
            precedence[5],
            OutputFormat::Raw,
            "Raw should have lowest precedence"
        );
    }

    #[test]
    fn test_format_precedence_values() {
        assert_eq!(format_precedence(OutputFormat::Json), 6);
        assert_eq!(format_precedence(OutputFormat::Csv), 5);
        assert_eq!(format_precedence(OutputFormat::Tsv), 4);
        assert_eq!(format_precedence(OutputFormat::Agent), 3);
        assert_eq!(format_precedence(OutputFormat::Compact), 2);
        assert_eq!(format_precedence(OutputFormat::Raw), 1);
    }

    #[test]
    fn test_current_format_precedence() {
        let cli = Cli::try_parse_from(["trs", "--json", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 6);

        let cli = Cli::try_parse_from(["trs", "--csv", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 5);

        let cli = Cli::try_parse_from(["trs", "--tsv", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 4);

        let cli = Cli::try_parse_from(["trs", "--agent", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 3);

        let cli = Cli::try_parse_from(["trs", "--compact", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 2);

        let cli = Cli::try_parse_from(["trs", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 1);

        // Default (no flags) should be Compact with precedence 2
        let cli = Cli::try_parse_from(["trs", "search", ".", "test"]).unwrap();
        assert_eq!(cli.current_format_precedence(), 2);
    }

    #[test]
    fn test_enabled_format_flags_single() {
        let cli = Cli::try_parse_from(["trs", "--json", "search", ".", "test"]).unwrap();
        let enabled = cli.enabled_format_flags();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0], OutputFormat::Json);
    }

    #[test]
    fn test_enabled_format_flags_multiple() {
        let cli = Cli::try_parse_from(["trs", "--json", "--csv", "--raw", "search", ".", "test"])
            .unwrap();
        let enabled = cli.enabled_format_flags();
        assert_eq!(enabled.len(), 3);
        assert!(enabled.contains(&OutputFormat::Json));
        assert!(enabled.contains(&OutputFormat::Csv));
        assert!(enabled.contains(&OutputFormat::Raw));
    }

    #[test]
    fn test_enabled_format_flags_none() {
        let cli = Cli::try_parse_from(["trs", "search", ".", "test"]).unwrap();
        let enabled = cli.enabled_format_flags();
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_has_conflicting_format_flags_true() {
        let cli = Cli::try_parse_from(["trs", "--json", "--csv", "search", ".", "test"]).unwrap();
        assert!(cli.has_conflicting_format_flags());
    }

    #[test]
    fn test_has_conflicting_format_flags_false_single() {
        let cli = Cli::try_parse_from(["trs", "--json", "search", ".", "test"]).unwrap();
        assert!(!cli.has_conflicting_format_flags());
    }

    #[test]
    fn test_has_conflicting_format_flags_false_none() {
        let cli = Cli::try_parse_from(["trs", "search", ".", "test"]).unwrap();
        assert!(!cli.has_conflicting_format_flags());
    }

    // ============================================================
    // All precedence combinations tests
    // ============================================================

    #[test]
    fn test_precedence_json_over_all() {
        // JSON should win over all other formats
        let cli = Cli::try_parse_from([
            "trs",
            "--json",
            "--csv",
            "--tsv",
            "--agent",
            "--compact",
            "--raw",
            "search",
            ".",
            "test",
        ])
        .unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_csv_over_all_except_json() {
        // CSV should win over all except JSON
        let cli = Cli::try_parse_from([
            "trs",
            "--csv",
            "--tsv",
            "--agent",
            "--compact",
            "--raw",
            "search",
            ".",
            "test",
        ])
        .unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_tsv_over_all_except_json_csv() {
        // TSV should win over all except JSON and CSV
        let cli = Cli::try_parse_from([
            "trs",
            "--tsv",
            "--agent",
            "--compact",
            "--raw",
            "search",
            ".",
            "test",
        ])
        .unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_precedence_agent_over_compact_raw() {
        // Agent should win over Compact and Raw
        let cli = Cli::try_parse_from([
            "trs",
            "--agent",
            "--compact",
            "--raw",
            "search",
            ".",
            "test",
        ])
        .unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    #[test]
    fn test_precedence_compact_over_raw() {
        // Compact should win over Raw
        let cli =
            Cli::try_parse_from(["trs", "--compact", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Compact);
    }

    #[test]
    fn test_precedence_json_over_csv() {
        let cli = Cli::try_parse_from(["trs", "--json", "--csv", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_json_over_tsv() {
        let cli = Cli::try_parse_from(["trs", "--json", "--tsv", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_json_over_agent() {
        let cli = Cli::try_parse_from(["trs", "--json", "--agent", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_json_over_compact() {
        let cli =
            Cli::try_parse_from(["trs", "--json", "--compact", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_json_over_raw() {
        let cli = Cli::try_parse_from(["trs", "--json", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_csv_over_tsv() {
        let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_csv_over_agent() {
        let cli = Cli::try_parse_from(["trs", "--csv", "--agent", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_csv_over_compact() {
        let cli =
            Cli::try_parse_from(["trs", "--csv", "--compact", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_csv_over_raw() {
        let cli = Cli::try_parse_from(["trs", "--csv", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_tsv_over_agent() {
        let cli = Cli::try_parse_from(["trs", "--tsv", "--agent", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_precedence_tsv_over_compact() {
        let cli =
            Cli::try_parse_from(["trs", "--tsv", "--compact", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_precedence_tsv_over_raw() {
        let cli = Cli::try_parse_from(["trs", "--tsv", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_precedence_agent_over_compact() {
        let cli =
            Cli::try_parse_from(["trs", "--agent", "--compact", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    #[test]
    fn test_precedence_agent_over_raw() {
        let cli = Cli::try_parse_from(["trs", "--agent", "--raw", "search", ".", "test"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    // ============================================================
    // Tests with different commands (ensure global flags work)
    // ============================================================

    #[test]
    fn test_precedence_with_run_command() {
        let cli = Cli::try_parse_from(["trs", "--json", "--csv", "run", "echo", "hello"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn test_precedence_with_parse_command() {
        let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "parse", "git-status"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Csv);
    }

    #[test]
    fn test_precedence_with_replace_command() {
        let cli =
            Cli::try_parse_from(["trs", "--tsv", "--agent", "replace", ".", "old", "new"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Tsv);
    }

    #[test]
    fn test_precedence_with_tail_command() {
        let cli = Cli::try_parse_from(["trs", "--agent", "--compact", "tail", "/var/log/test.log"])
            .unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Agent);
    }

    #[test]
    fn test_precedence_with_clean_command() {
        let cli = Cli::try_parse_from(["trs", "--compact", "--raw", "clean"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Compact);
    }

    #[test]
    fn test_precedence_with_html2md_command() {
        let cli = Cli::try_parse_from(["trs", "--raw", "html2md", "https://example.com"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Raw);
    }

    #[test]
    fn test_precedence_with_txt2md_command() {
        let cli = Cli::try_parse_from(["trs", "--json", "txt2md"]).unwrap();
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }
}
