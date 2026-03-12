use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// TARS CLI - Transform noisy terminal output into compact, structured signal
///
/// A CLI toolkit for developers, automation pipelines, and AI agents.
#[derive(Parser)]
#[command(name = "trs", bin_name = "trs")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
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

impl Cli {
    /// Determine the output format based on flag precedence.
    /// Precedence: json > csv > tsv > agent > compact > raw > default
    pub fn output_format(&self) -> OutputFormat {
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
}

/// Output format options with precedence rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Raw, unprocessed output
    Raw,
    /// Compact, summarized output
    #[default]
    Compact,
    /// JSON structured output
    Json,
    /// CSV formatted output
    Csv,
    /// TSV formatted output
    Tsv,
    /// Agent-optimized format for AI consumption
    Agent,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a command and process its output
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
    Parse {
        #[command(subcommand)]
        parser: ParseCommands,
    },

    /// Search for patterns in files
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
    GitStatus {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git diff output
    GitDiff {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse ls output
    Ls {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse grep output
    Grep {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse test runner output
    Test {
        /// Test runner type
        #[arg(short = 't', long, value_enum)]
        runner: Option<TestRunner>,

        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse log/tail output
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
}
