use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::help;

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

        /// Capture stdout (default: true, set --no-capture-stdout to inherit)
        #[arg(long, default_missing_value = "true", default_value = "true", num_args = 0..=1)]
        capture_stdout: Option<bool>,

        /// Capture stderr (default: true, set --no-capture-stderr to inherit)
        #[arg(long, default_missing_value = "true", default_value = "true", num_args = 0..=1)]
        capture_stderr: Option<bool>,

        /// Capture exit code (default: true, set --no-capture-exit-code to disable)
        #[arg(long, default_missing_value = "true", default_value = "true", num_args = 0..=1)]
        capture_exit_code: Option<bool>,

        /// Capture execution duration (default: true, set --no-capture-duration to disable)
        #[arg(long, default_missing_value = "true", default_value = "true", num_args = 0..=1)]
        capture_duration: Option<bool>,
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
        #[arg(short, long, alias = "preview")]
        dry_run: bool,

        /// Output only the total replacement count
        #[arg(long)]
        count: bool,
    },

    /// Tail a file with compact log output
    #[command(long_about = help::TAIL_HELP)]
    Tail {
        /// File to tail
        file: PathBuf,

        /// Number of lines to show (supports -N shorthand, e.g., -5 for last 5 lines)
        #[arg(short = 'n', long, default_value = "10", value_name = "N")]
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

    /// Trim whitespace from text lines
    #[command(long_about = help::TRIM_HELP)]
    Trim {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Trim leading whitespace only
        #[arg(long)]
        leading: bool,

        /// Trim trailing whitespace only
        #[arg(long)]
        trailing: bool,
    },

    /// Check if git repository is in a clean state
    ///
    /// Detects whether the git repository has any uncommitted changes.
    /// A clean repository has:
    /// - No staged changes
    /// - No unstaged changes
    /// - No untracked files
    /// - No unmerged paths (conflicts)
    ///
    /// Exit codes:
    ///   0 - Repository is clean
    ///   1 - Repository has changes (dirty)
    ///   2 - Not a git repository or other error
    ///
    /// Examples:
    ///   trs is-clean                    # Check if repo is clean
    ///   trs is-clean --json             # Output in JSON format
    ///   trs is-clean && git push        # Only push if clean
    #[command(aliases = ["clean?", "repo-clean"])]
    IsClean {
        /// Also check for untracked files (default: true)
        /// Use --no-check-untracked to ignore untracked files
        #[arg(long, default_missing_value = "true", default_value = "true", num_args = 0..=1)]
        check_untracked: Option<bool>,
    },

    /// Show token savings statistics
    Stats {
        /// Show recent command history
        #[arg(long, short = 'H')]
        history: bool,
        /// Filter to current project only
        #[arg(long, short)]
        project: bool,
        /// Output format (text or json)
        #[arg(long)]
        json: bool,
    },

    /// Read a file with optional filtering (strip comments, signatures-only)
    #[command(long_about = help::READ_HELP)]
    Read {
        /// File to read
        file: PathBuf,

        /// Filter level: minimal (strip comments) or aggressive (signatures only)
        #[arg(short = 'l', long, value_enum, default_value = "none")]
        level: ReadLevel,

        /// Maximum number of lines to show (from start)
        #[arg(short = 'n', long)]
        lines: Option<usize>,

        /// Show last N lines (from end)
        #[arg(short = 't', long)]
        tail: Option<usize>,

        /// Show line numbers
        #[arg(short = 'N', long)]
        line_numbers: bool,
    },

    /// Show JSON structure without values (keys + types + array lengths)
    #[command(long_about = help::JSON_HELP)]
    Json {
        /// Input JSON file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Maximum depth to display
        #[arg(short, long)]
        depth: Option<usize>,
    },

    /// Run a command and show only errors and warnings
    #[command(long_about = "Run any command and filter output to show only errors and warnings.\n\nExamples:\n  trs err cargo build\n  trs err npm test\n  trs err make all")]
    Err {
        /// Command to run
        #[arg(required = true)]
        command: String,
        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// External command (auto-detected via allow_external_subcommands)
    #[command(external_subcommand)]
    External(Vec<String>),
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

        /// Output only the count for the specified category (staged, unstaged, untracked, unmerged)
        /// Default: unstaged
        #[arg(long)]
        count: Option<String>,
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

    /// Parse find output
    ///
    /// Transforms find results into structured format categorizing
    /// files, directories, and other entries by type.
    ///
    /// Example: find . -name "*.rs" | trs parse find
    Find {
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

    /// Parse git log output
    ///
    /// Example: git log | trs parse git-log
    GitLog {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git branch output
    ///
    /// Example: git branch -a | trs parse git-branch
    GitBranch {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse tree command output
    ///
    /// Example: tree | trs parse tree
    Tree {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse docker ps output
    ///
    /// Example: docker ps | trs parse docker-ps
    DockerPs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse docker logs output
    ///
    /// Example: docker logs container | trs parse docker-logs
    DockerLogs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse dependency list output (npm ls, pip list, cargo tree)
    ///
    /// Example: npm ls | trs parse deps
    Deps {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse package install output (npm install, pip install, cargo build)
    ///
    /// Example: npm install | trs parse install
    Install {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse build output (cargo build, tsc, gcc, make)
    ///
    /// Example: cargo build | trs parse build
    Build {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse environment variables
    ///
    /// Example: env | trs parse env
    Env {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse wc (word count) output
    ///
    /// Example: wc file.txt | trs parse wc
    Wc {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse wget/curl download output
    ///
    /// Example: curl -v https://example.com 2>&1 | trs parse download
    Download {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh pr list output
    ///
    /// Example: gh pr list | trs parse gh-pr
    GhPr {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh issue list output
    ///
    /// Example: gh issue list | trs parse gh-issue
    GhIssue {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh run list output
    ///
    /// Example: gh run list | trs parse gh-run
    GhRun {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

/// Filter level for `trs read`
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReadLevel {
    /// No filtering — raw content
    None,
    /// Strip comments, normalize blank lines
    Minimal,
    /// Signatures only — imports + definitions
    Aggressive,
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
