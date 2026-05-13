use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::help;

#[path = "commands_parse.rs"]
mod commands_parse;
pub use commands_parse::ParseCommands;

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

    /// Rewrite a command for hook integration (called by AI tool hooks)
    Rewrite,

    /// Find missed token savings opportunities in Claude Code history
    Discover {
        /// Scan all projects (default: current project only)
        #[arg(long)]
        all: bool,

        /// Number of days to look back (default: 7)
        #[arg(long, default_value = "7")]
        since: usize,
    },

    /// Install hooks for AI coding tools (claude, gemini, cursor, codex, opencode, kilo)
    Init {
        /// Tool to configure (claude, gemini, cursor, codex, opencode, kilo)
        tool: Option<String>,

        /// Install globally (user-level) instead of project-level
        #[arg(short, long)]
        global: bool,

        /// Show current installation status
        #[arg(long)]
        show: bool,

        /// Install hooks for all detected tools
        #[arg(long)]
        all: bool,

        /// Remove competing compressor hooks (rtk, token-optimizer) before
        /// installing trs. Required when a collision is detected.
        #[arg(long)]
        replace: bool,

        /// Install trs even if a competing compressor is already configured.
        /// Risk: double-compression can corrupt command output.
        #[arg(long)]
        force: bool,

        /// Preview the install without writing anything. Lists every file
        /// that would change and the action taken (create / merge / skip).
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate trs installation health (binary, PATH, deps, config)
    Doctor {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Bundle version + platform + doctor results + recent history +
    /// recent tee logs into one paste-ready report. Useful when you
    /// hit an issue and want to file a bug without manually
    /// collecting every piece. Review the output before sharing — it
    /// includes cwd paths and failing-command output from tee logs.
    #[command(name = "debug-info")]
    DebugInfo {
        /// Write the report to PATH instead of stdout.
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Detect how trs was installed and re-run the matching install path
    /// to pick up the latest release. Supports npm and the curl|sh script;
    /// flags the cargo / Homebrew channels as manual-only for now. After
    /// a successful binary upgrade, also refreshes hook templates and any
    /// already-installed output-saver blocks — pass --binary-only to
    /// skip that step.
    Upgrade {
        /// Detect the install method and show what would run without
        /// executing anything.
        #[arg(long)]
        check: bool,

        /// Skip the interactive confirmation prompt. Useful in scripts.
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Upgrade only the binary — skip the hook-template refresh
        /// and output-saver refresh that normally follow.
        #[arg(long)]
        binary_only: bool,
    },

    /// Install a compact output-reduction rules block into agent configs
    /// (Claude, Gemini, Cursor, Codex, Windsurf). trs already compresses
    /// what agents see — this does the symmetric job for what they emit.
    OutputSaver {
        /// Target a specific agent (claude, gemini, cursor, codex, windsurf).
        /// Omit to act on every supported agent that's detected.
        #[arg(value_name = "AGENT")]
        tool: Option<String>,

        /// Write the rules block into the target config(s) (default is a
        /// read-only check that prints what would change).
        #[arg(long)]
        install: bool,

        /// Remove a previously installed output-saver block.
        #[arg(long)]
        remove: bool,

        /// Print the rules block to stdout and exit — useful for piping
        /// into a custom location.
        #[arg(long)]
        print: bool,

        /// Re-install the block only where it's already present. Skips
        /// agents that don't have it yet (no new installs). Intended
        /// for version bumps: picks up template changes without
        /// surprising users who haven't opted in.
        #[arg(long)]
        refresh: bool,
    },

    /// Benchmark a command showing compression metrics
    #[command(long_about = help::BENCHMARK_HELP)]
    Benchmark {
        /// Command to benchmark
        #[arg(required = true)]
        command: String,

        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Number of iterations
        #[arg(long, default_value = "1")]
        repeat: usize,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Generate an LLM-ready digest of a project
    #[command(long_about = help::INGEST_HELP)]
    Ingest {
        /// Project path (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// List saved digests instead of generating
        #[arg(long)]
        list: bool,

        /// Read a saved digest (by project name, or current repo if omitted)
        #[arg(long)]
        read: Option<Option<String>>,

        /// Compression level: full, minimal, aggressive
        #[arg(short, long, default_value = "minimal")]
        level: String,

        /// Token budget (e.g. 128k, 64000)
        #[arg(short, long)]
        budget: Option<String>,

        /// Only include files with uncommitted changes
        #[arg(long)]
        changed: bool,

        /// Only include files changed since a git ref (e.g. HEAD~5, main)
        #[arg(long)]
        since: Option<String>,

        /// Exclude paths containing this pattern (repeatable)
        #[arg(short, long)]
        exclude: Vec<String>,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,

        /// Format digest with local Ollama model (e.g. llama3, mistral)
        #[arg(long)]
        ollama: Option<String>,

        /// Output only the import/dependency graph (no file content)
        #[arg(long)]
        deps: bool,

        /// Only include files changed since the last ingest (uses stored HEAD)
        #[arg(long)]
        since_last: bool,

        /// Skip regeneration if HEAD unchanged since last ingest (use cached digest)
        #[arg(long)]
        fresh: bool,

        /// Force regeneration even if HEAD unchanged (disables --fresh)
        #[arg(long)]
        force: bool,

        /// Print digest contents to stdout instead of just the path
        #[arg(long)]
        print: bool,

        /// Warn on stderr when digest exceeds N tokens (accepts "40k", "200k"). 0 disables.
        #[arg(long, default_value = "40k")]
        warn_at: String,

        /// Emit a flat symbol → file index after the Structure tree
        #[arg(long)]
        symbols: bool,

        /// For URL/remote input: shallow-clone into a tempdir (not saved).
        /// Ignored when the input is a local path.
        #[arg(long)]
        tmp: bool,
    },

    /// Execute command without filtering but track usage
    Raw {
        /// Command and arguments to execute
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
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
        /// Break down totals by AI agent (claude, gemini, cursor,
        /// opencode, kilo). Detected via the TRS_AGENT env var
        /// injected by hook/plugin templates; rules-based agents
        /// (codex, antigravity, windsurf) show as "(untagged)".
        #[arg(long = "by-agent")]
        by_agent: bool,
        /// Aggregate by normalised command family (git diff, cargo test,
        /// npm run lint…) — strips paths and flags, groups variants.
        #[arg(long = "by-command")]
        by_command: bool,
        /// Number of rows to show. Applies to `--history` (default 20)
        /// and to the summary's Top Commands table (default 15).
        #[arg(long, short = 'n')]
        limit: Option<usize>,
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

        /// Query path to extract (e.g. ".users[0].name", ".[].id")
        #[arg(short, long)]
        query: Option<String>,
    },

    /// Run a command and show only errors and warnings
    #[command(
        long_about = "Run any command and filter output to show only errors and warnings.\n\nExamples:\n  trs err cargo build\n  trs err npm test\n  trs err make all"
    )]
    Err {
        /// Command to run
        #[arg(required = true)]
        command: String,
        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Audit AI-agent instruction files (CLAUDE.md, AGENTS.md, .windsurfrules,
    /// .cursor/rules/*, .agent/rules/*) for bloat, cross-file duplicates, dead
    /// references, and staleness. Surfaces what's silently inflating every
    /// agent session's context.
    AuditDocs {
        /// Project root to audit (default: current directory).
        #[arg(default_value = ".")]
        path: String,
    },

    /// External command (auto-detected via allow_external_subcommands)
    #[command(external_subcommand)]
    External(Vec<String>),
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
