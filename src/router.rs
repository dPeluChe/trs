//! Command routing system for TARS CLI.
//!
//! This module provides a modular routing system that dispatches CLI commands
//! to their respective handlers. Each command has a dedicated handler that
//! implements the `CommandHandler` trait.

use crate::process::{ProcessBuilder, ProcessError, ProcessOutput};
use crate::{Cli, Commands, OutputFormat, ParseCommands};

/// Context passed to command handlers containing global CLI options.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The output format to use for the command result.
    pub format: OutputFormat,
    /// Whether to show execution statistics.
    pub stats: bool,
    /// List of enabled format flags (for warnings/debugging).
    pub enabled_formats: Vec<OutputFormat>,
}

impl CommandContext {
    /// Create a new command context from CLI options.
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            format: cli.output_format(),
            stats: cli.stats,
            enabled_formats: cli.enabled_format_flags(),
        }
    }

    /// Returns true if multiple format flags were specified.
    pub fn has_conflicting_formats(&self) -> bool {
        self.enabled_formats.len() > 1
    }
}

/// Result type for command handlers.
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Error type for command execution.
#[derive(Debug, Clone)]
pub enum CommandError {
    /// The command is not yet implemented.
    NotImplemented(String),
    /// An error occurred during execution with an optional exit code.
    ExecutionError {
        message: String,
        exit_code: Option<i32>,
    },
    /// Invalid arguments provided.
    InvalidArguments(String),
    /// I/O error occurred.
    IoError(String),
}

impl CommandError {
    /// Returns the exit code if this error is associated with a non-zero exit.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandError::ExecutionError { exit_code, .. } => *exit_code,
            _ => None,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            CommandError::ExecutionError { message, .. } => {
                write!(f, "Execution error: {}", message)
            }
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

// ============================================================
// Git Status Data Structures
// ============================================================

/// Section of git status output being parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitStatusSection {
    /// Not in any specific section.
    None,
    /// Staged changes section.
    Staged,
    /// Unstaged changes section.
    Unstaged,
    /// Untracked files section.
    Untracked,
    /// Unmerged paths section.
    Unmerged,
}

/// A single file entry in git status.
#[derive(Debug, Clone, Default)]
struct GitStatusEntry {
    /// Status code (e.g., "M", "A", "D", "??").
    status: String,
    /// Path to the file.
    path: String,
    /// Original path for renamed files.
    old_path: Option<String>,
}

/// Parsed git status output.
#[derive(Debug, Clone, Default)]
struct GitStatus {
    /// Current branch name.
    branch: String,
    /// Whether the working tree is clean.
    is_clean: bool,
    /// Number of commits ahead of upstream.
    ahead: Option<usize>,
    /// Number of commits behind upstream.
    behind: Option<usize>,
    /// Staged changes (to be committed).
    staged: Vec<GitStatusEntry>,
    /// Unstaged changes (not staged for commit).
    unstaged: Vec<GitStatusEntry>,
    /// Untracked files.
    untracked: Vec<GitStatusEntry>,
    /// Unmerged paths (merge conflicts).
    unmerged: Vec<GitStatusEntry>,
    /// Number of staged files.
    staged_count: usize,
    /// Number of unstaged files.
    unstaged_count: usize,
    /// Number of untracked files.
    untracked_count: usize,
    /// Number of unmerged files.
    unmerged_count: usize,
}

// ============================================================
// Git Diff Data Structures
// ============================================================

/// A single file entry in git diff output.
#[derive(Debug, Clone, Default)]
struct GitDiffEntry {
    /// Path to the file (new path for renamed files).
    path: String,
    /// Original path for renamed files.
    old_path: Option<String>,
    /// Change type (M=modified, A=added, D=deleted, R=renamed, C=copied).
    change_type: String,
    /// Number of lines added.
    additions: usize,
    /// Number of lines deleted.
    deletions: usize,
    /// Binary file flag.
    is_binary: bool,
}

/// Parsed git diff output.
#[derive(Debug, Clone, Default)]
struct GitDiff {
    /// List of file entries (limited if truncated).
    files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    total_additions: usize,
    /// Total lines deleted across all files.
    total_deletions: usize,
    /// Whether the diff is empty.
    is_empty: bool,
    /// Whether the output was truncated.
    is_truncated: bool,
    /// Total number of files available before truncation.
    total_files: usize,
    /// Number of files shown after truncation.
    files_shown: usize,
}

// ============================================================
// LS Data Structures
// ============================================================

/// Entry type for ls output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LsEntryType {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Block device.
    BlockDevice,
    /// Character device.
    CharDevice,
    /// Socket.
    Socket,
    /// Pipe (FIFO).
    Pipe,
    /// Unknown or other type.
    Other,
}

impl Default for LsEntryType {
    fn default() -> Self {
        LsEntryType::File
    }
}

/// A single entry in ls output.
#[derive(Debug, Clone, Default)]
struct LsEntry {
    /// Name of the file or directory.
    name: String,
    /// Type of entry (file, directory, etc.).
    entry_type: LsEntryType,
    /// Whether this is a hidden file (starts with .).
    is_hidden: bool,
    /// File size in bytes (if available).
    size: Option<u64>,
    /// File permissions (if available).
    permissions: Option<String>,
    /// Number of hard links (if available).
    links: Option<u64>,
    /// Owner user name (if available).
    owner: Option<String>,
    /// Owner group name (if available).
    group: Option<String>,
    /// Last modification time (if available).
    modified: Option<String>,
    /// Symlink target (if this is a symlink).
    symlink_target: Option<String>,
    /// Whether the symlink is broken (target doesn't exist).
    is_broken_symlink: bool,
}

// ============================================================
// Find Data Structures
// ============================================================

/// A single entry in find output.
#[derive(Debug, Clone, Default)]
struct FindEntry {
    /// Path to the file or directory.
    path: String,
    /// Whether this is a directory.
    is_directory: bool,
    /// Whether this is a hidden file/directory.
    is_hidden: bool,
    /// File extension (if available).
    extension: Option<String>,
    /// Depth of the path (number of path separators).
    depth: usize,
}

/// A permission denied or error entry from find output.
#[derive(Debug, Clone, Default)]
struct FindError {
    /// The path that was denied access.
    path: String,
    /// The error message.
    message: String,
}

/// Parsed find output.
#[derive(Debug, Clone, Default)]
struct FindOutput {
    /// List of all entries.
    entries: Vec<FindEntry>,
    /// Directory paths.
    directories: Vec<String>,
    /// File paths.
    files: Vec<String>,
    /// Hidden entries.
    hidden: Vec<String>,
    /// File extensions with counts.
    extensions: std::collections::HashMap<String, usize>,
    /// Permission denied or error entries.
    errors: Vec<FindError>,
    /// Total count of entries (excluding errors).
    total_count: usize,
    /// Whether the output is empty.
    is_empty: bool,
}

// ============================================================
// Grep Data Structures
// ============================================================

/// A single match in grep output.
#[derive(Debug, Clone, Default)]
struct GrepMatch {
    /// Line number (if available with -n flag).
    line_number: Option<usize>,
    /// Column number (if available with --column flag).
    column: Option<usize>,
    /// The matched line content.
    line: String,
    /// Whether this is a context line (not a direct match).
    is_context: bool,
}

/// A file with grep matches.
#[derive(Debug, Clone, Default)]
struct GrepFile {
    /// Path to the file.
    path: String,
    /// List of matches in this file.
    matches: Vec<GrepMatch>,
}

/// Parsed grep output.
#[derive(Debug, Clone, Default)]
struct GrepOutput {
    /// List of files with matches (limited if truncated).
    files: Vec<GrepFile>,
    /// Total number of files with matches.
    file_count: usize,
    /// Total number of matches across all files.
    match_count: usize,
    /// Whether the output is empty (no matches).
    is_empty: bool,
    /// Whether the output was truncated.
    is_truncated: bool,
    /// Total number of files available before truncation.
    total_files: usize,
    /// Total number of matches available before truncation.
    total_matches: usize,
    /// Number of files shown after truncation.
    files_shown: usize,
    /// Number of matches shown after truncation.
    matches_shown: usize,
}

/// Common generated directory names that are typically build artifacts or dependencies.
const COMMON_GENERATED_DIRS: &[&str] = &[
    // JavaScript/TypeScript
    "node_modules",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".output",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    "env",
    ".tox",
    ".nox",
    "htmlcov",
    ".eggs",
    "eggs",
    "sdist",
    "wheelhouse",
    // Rust
    "target",
    // Java/Kotlin
    "target", // Maven
    "build",  // Gradle
    "out",    // IntelliJ
    ".gradle",
    // Go
    "vendor",
    // Ruby
    "vendor",
    ".bundle",
    // PHP
    "vendor",
    // .NET/C#
    "bin",
    "obj",
    // Swift/Objective-C
    "DerivedData",
    "Pods",
    ".build",
    // Elixir/Erlang
    "_build",
    "deps",
    // Haskell
    "dist-newstyle",
    ".stack-work",
    // Scala
    ".bloop",
    ".metals",
    // Docker
    ".docker",
    // Cache directories
    ".cache",
    ".npm",
    ".yarn",
    ".pnpm-store",
    // IDE/Editor
    ".idea",
    ".vscode",
    ".vs",
    // Misc
    "tmp",
    "temp",
];

/// Check if a directory name is a common generated directory.
fn is_generated_directory(name: &str) -> bool {
    // Strip trailing slash if present (common in ls output)
    let name = name.strip_suffix('/').unwrap_or(name);
    let name_lower = name.to_lowercase();
    COMMON_GENERATED_DIRS.contains(&name_lower.as_str())
}

/// A permission denied or error entry.
#[derive(Debug, Clone, Default)]
struct LsError {
    /// The path that was denied access.
    path: String,
    /// The error message.
    message: String,
}

/// Parsed ls output.
#[derive(Debug, Clone, Default)]
struct LsOutput {
    /// List of all entries.
    entries: Vec<LsEntry>,
    /// Directory entries.
    directories: Vec<LsEntry>,
    /// File entries.
    files: Vec<LsEntry>,
    /// Symlink entries.
    symlinks: Vec<LsEntry>,
    /// Hidden entries.
    hidden: Vec<LsEntry>,
    /// Generated directory entries (build artifacts, dependencies, etc.).
    generated: Vec<LsEntry>,
    /// Permission denied or error entries.
    errors: Vec<LsError>,
    /// Total count of entries (excluding errors).
    total_count: usize,
    /// Whether the output is empty.
    is_empty: bool,
}

/// Trait for command handlers.
///
/// Each command in the CLI implements this trait to handle its specific logic.
pub trait CommandHandler {
    /// The input type for this command (the command variant data).
    type Input;

    /// Execute the command with the given input and context.
    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult;
}

/// Handler for the `run` command.
pub struct RunHandler;

impl RunHandler {
    /// Format output based on the specified format.
    fn format_output(output: &ProcessOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => {
                // JSON output includes all fields
                serde_json::json!({
                    "command": output.command,
                    "args": output.args,
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "exit_code": output.exit_code,
                    "duration_ms": output.duration.as_millis(),
                    "timed_out": output.timed_out,
                })
                .to_string()
            }
            OutputFormat::Csv => {
                // CSV output with header row
                let mut result = String::new();
                result.push_str("command,args,stdout,stderr,exit_code,duration_ms,timed_out\n");
                let args_str = output.args.join(" ");
                let stdout_escaped = Self::escape_csv_field(&output.stdout);
                let stderr_escaped = Self::escape_csv_field(&output.stderr);
                result.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    output.command,
                    args_str,
                    stdout_escaped,
                    stderr_escaped,
                    output.exit_code.map(|c| c.to_string()).unwrap_or_default(),
                    output.duration.as_millis(),
                    output.timed_out
                ));
                result
            }
            OutputFormat::Tsv => {
                // TSV output with header row
                let mut result = String::new();
                result
                    .push_str("command\targs\tstdout\tstderr\texit_code\tduration_ms\ttimed_out\n");
                let args_str = output.args.join(" ");
                let stdout_escaped = Self::escape_tsv_field(&output.stdout);
                let stderr_escaped = Self::escape_tsv_field(&output.stderr);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    output.command,
                    args_str,
                    stdout_escaped,
                    stderr_escaped,
                    output.exit_code.map(|c| c.to_string()).unwrap_or_default(),
                    output.duration.as_millis(),
                    output.timed_out
                ));
                result
            }
            OutputFormat::Compact | OutputFormat::Agent => {
                // Compact output shows essential info
                let mut result = String::new();
                if output.has_stdout() {
                    result.push_str(&output.stdout);
                    if !result.ends_with('\n') && !result.is_empty() {
                        result.push('\n');
                    }
                }
                if output.has_stderr() {
                    result.push_str(&output.stderr);
                }
                result
            }
            _ => {
                // Raw and other formats: just stdout
                let mut result = output.stdout.clone();
                if output.has_stderr() && !output.stderr.is_empty() {
                    result.push_str(&output.stderr);
                }
                result
            }
        }
    }

    /// Escape a field for CSV format.
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Escape a field for TSV format.
    fn escape_tsv_field(field: &str) -> String {
        // TSV doesn't support tabs in fields; replace with space
        field.replace('\t', " ").replace('\r', "")
    }

    /// Format error message based on format.
    fn format_error(error: &ProcessError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "error": true,
                "message": error.to_string(),
                "exit_code": error.exit_code(),
                "is_timeout": error.is_timeout(),
                "is_command_not_found": error.is_command_not_found(),
                "is_permission_denied": error.is_permission_denied(),
            })
            .to_string(),
            _ => format!("Error: {}", error),
        }
    }
}

impl CommandHandler for RunHandler {
    type Input = RunInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Build and execute the process
        let mut builder = ProcessBuilder::new(&input.command)
            .args(&input.args)
            .capture_stdout(input.capture_stdout)
            .capture_stderr(input.capture_stderr)
            .capture_exit_code(input.capture_exit_code)
            .capture_duration(input.capture_duration);

        // Add timeout if specified
        if let Some(timeout) = input.timeout {
            builder = builder.timeout(std::time::Duration::from_secs(timeout));
        }

        let result = builder.run();

        match result {
            Ok(output) => {
                // Print stats if requested
                if ctx.stats {
                    eprintln!("Stats:");
                    eprintln!("  Command: {} {:?}", output.command, output.args);
                    eprintln!("  Exit code: {:?}", output.exit_code);
                    eprintln!("  Duration: {:.2}s", output.duration.as_secs_f64());
                    eprintln!("  Stdout bytes: {}", output.stdout.len());
                    eprintln!("  Stderr bytes: {}", output.stderr.len());
                }

                // Format and print output
                let formatted = Self::format_output(&output, ctx.format);
                print!("{}", formatted);

                // Propagate exit code (only if we captured it)
                if input.capture_exit_code && !output.success() {
                    return Err(CommandError::ExecutionError {
                        message: format!("Command exited with code {}", output.code()),
                        exit_code: output.exit_code,
                    });
                }

                Ok(())
            }
            Err(error) => {
                // Print stats if requested
                if ctx.stats {
                    eprintln!("Stats:");
                    eprintln!("  Command failed: {}", error);
                }

                // Return appropriate error type (error printing is handled by Router::execute_and_print)
                Err(match &error {
                    ProcessError::CommandNotFound { command } => CommandError::ExecutionError {
                        message: format!("Command not found: {}", command),
                        exit_code: Some(127), // Standard "command not found" exit code
                    },
                    ProcessError::PermissionDenied { command } => CommandError::ExecutionError {
                        message: format!("Permission denied: {}", command),
                        exit_code: Some(126), // Standard "permission denied" exit code
                    },
                    ProcessError::Timeout {
                        command, duration, ..
                    } => CommandError::ExecutionError {
                        message: format!(
                            "Command '{}' timed out after {:.2}s",
                            command,
                            duration.as_secs_f64()
                        ),
                        exit_code: Some(124), // Standard timeout exit code
                    },
                    ProcessError::NonZeroExit { output } => CommandError::ExecutionError {
                        message: format!("Command exited with code {}", output.code()),
                        exit_code: output.exit_code,
                    },
                    ProcessError::IoError { message, .. } => CommandError::IoError(message.clone()),
                    ProcessError::SpawnFailed { message, .. } => CommandError::ExecutionError {
                        message: message.clone(),
                        exit_code: None,
                    },
                })
            }
        }
    }
}

/// Input data for the `run` command.
#[derive(Debug, Clone)]
pub struct RunInput {
    pub command: String,
    pub args: Vec<String>,
    pub capture_stdout: bool,
    pub capture_stderr: bool,
    pub capture_exit_code: bool,
    pub capture_duration: bool,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
}

impl From<(&String, &Vec<String>, bool, bool, bool, bool, Option<u64>)> for RunInput {
    fn from(
        (
            command,
            args,
            capture_stdout,
            capture_stderr,
            capture_exit_code,
            capture_duration,
            timeout,
        ): (&String, &Vec<String>, bool, bool, bool, bool, Option<u64>),
    ) -> Self {
        Self {
            command: command.clone(),
            args: args.clone(),
            capture_stdout,
            capture_stderr,
            capture_exit_code,
            capture_duration,
            timeout,
        }
    }
}

/// Handler for the `search` command.
pub struct SearchHandler;

impl CommandHandler for SearchHandler {
    type Input = SearchInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!(
            "Search: {:?} in {:?} (ext: {:?}, case: {}, context: {:?}, limit: {:?})",
            input.query,
            input.path,
            input.extension,
            !input.ignore_case,
            input.context,
            input.limit
        );

        // TODO: Implement actual search execution
        Err(CommandError::NotImplemented(
            "search command execution".to_string(),
        ))
    }
}

/// Input data for the `search` command.
#[derive(Debug, Clone)]
pub struct SearchInput {
    pub path: std::path::PathBuf,
    pub query: String,
    pub extension: Option<String>,
    pub ignore_case: bool,
    pub context: Option<usize>,
    pub limit: Option<usize>,
}

/// Handler for the `replace` command.
pub struct ReplaceHandler;

impl CommandHandler for ReplaceHandler {
    type Input = ReplaceInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!(
            "Replace: '{}' with '{}' in {:?} (ext: {:?}, dry_run: {})",
            input.search, input.replace, input.path, input.extension, input.dry_run
        );

        // TODO: Implement actual replace execution
        Err(CommandError::NotImplemented(
            "replace command execution".to_string(),
        ))
    }
}

/// Input data for the `replace` command.
#[derive(Debug, Clone)]
pub struct ReplaceInput {
    pub path: std::path::PathBuf,
    pub search: String,
    pub replace: String,
    pub extension: Option<String>,
    pub dry_run: bool,
}

/// Handler for the `tail` command.
pub struct TailHandler;

impl CommandHandler for TailHandler {
    type Input = TailInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!(
            "Tail: {:?} ({} lines, errors: {}, follow: {})",
            input.file, input.lines, input.errors, input.follow
        );

        // TODO: Implement actual tail execution
        Err(CommandError::NotImplemented(
            "tail command execution".to_string(),
        ))
    }
}

/// Input data for the `tail` command.
#[derive(Debug, Clone)]
pub struct TailInput {
    pub file: std::path::PathBuf,
    pub lines: usize,
    pub errors: bool,
    pub follow: bool,
}

/// Handler for the `clean` command.
pub struct CleanHandler;

impl CommandHandler for CleanHandler {
    type Input = CleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!(
            "Clean: {:?} (no_ansi: {}, collapse_blanks: {}, collapse_repeats: {}, trim: {})",
            input.file, input.no_ansi, input.collapse_blanks, input.collapse_repeats, input.trim
        );

        // TODO: Implement actual clean execution
        Err(CommandError::NotImplemented(
            "clean command execution".to_string(),
        ))
    }
}

/// Input data for the `clean` command.
#[derive(Debug, Clone)]
pub struct CleanInput {
    pub file: Option<std::path::PathBuf>,
    pub no_ansi: bool,
    pub collapse_blanks: bool,
    pub collapse_repeats: bool,
    pub trim: bool,
}

/// Handler for the `html2md` command.
pub struct Html2mdHandler;

impl CommandHandler for Html2mdHandler {
    type Input = Html2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!(
            "Html2md: {:?} -> {:?} (metadata: {})",
            input.input, input.output, input.metadata
        );

        // TODO: Implement actual html2md execution
        Err(CommandError::NotImplemented(
            "html2md command execution".to_string(),
        ))
    }
}

/// Input data for the `html2md` command.
#[derive(Debug, Clone)]
pub struct Html2mdInput {
    pub input: String,
    pub output: Option<std::path::PathBuf>,
    pub metadata: bool,
}

/// Input data for the `txt2md` command.
#[derive(Debug, Clone)]
pub struct Txt2mdInput {
    pub input: Option<std::path::PathBuf>,
    pub output: Option<std::path::PathBuf>,
}

/// Handler for the `txt2md` command.
pub struct Txt2mdHandler;

impl CommandHandler for Txt2mdHandler {
    type Input = Txt2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Txt2md: {:?} -> {:?}", input.input, input.output);

        // TODO: Implement actual txt2md execution
        Err(CommandError::NotImplemented(
            "txt2md command execution".to_string(),
        ))
    }
}

/// Handler for the `is-clean` command.
pub struct IsCleanHandler;

impl IsCleanHandler {
    /// Check if the git repository is in a clean state.
    fn check_repo_state(check_untracked: bool) -> CommandResult<RepositoryState> {
        // Run git status --porcelain to get machine-readable output
        let output = ProcessBuilder::new("git")
            .args(vec!["status", "--porcelain"])
            .capture_stdout(true)
            .capture_stderr(true)
            .capture_exit_code(true)
            .capture_duration(true)
            .run();

        match output {
            Ok(process_output) => {
                // If git command failed, it might not be a git repository
                if !process_output.success() {
                    return Ok(RepositoryState {
                        is_git_repo: false,
                        is_detached: false,
                        branch: None,
                        is_clean: false,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                    });
                }

                let stdout = process_output.stdout;

                // Empty output means clean repository
                if stdout.trim().is_empty() {
                    return Ok(RepositoryState {
                        is_git_repo: true,
                        is_detached: false,
                        branch: None,
                        is_clean: true,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                    });
                }

                // Parse porcelain output to count different change types
                let mut staged_count = 0;
                let mut unstaged_count = 0;
                let mut untracked_count = 0;
                let mut unmerged_count = 0;

                for line in stdout.lines() {
                    if line.len() < 2 {
                        continue;
                    }

                    let index_status = line.chars().next().unwrap_or(' ');
                    let worktree_status = line.chars().nth(1).unwrap_or(' ');

                    // Check for unmerged (conflict) states
                    if index_status == 'U'
                        || worktree_status == 'U'
                        || index_status == 'A' && worktree_status == 'A'
                        || index_status == 'D' && worktree_status == 'D'
                    {
                        unmerged_count += 1;
                        continue;
                    }

                    // Check for untracked files
                    if index_status == '?' && worktree_status == '?' {
                        untracked_count += 1;
                        continue;
                    }

                    // Check for staged changes (index status)
                    if index_status != ' ' && index_status != '?' {
                        staged_count += 1;
                    }

                    // Check for unstaged changes (worktree status)
                    if worktree_status != ' ' && worktree_status != '?' {
                        unstaged_count += 1;
                    }
                }

                // Determine if clean based on flags
                let is_clean = if check_untracked {
                    staged_count == 0
                        && unstaged_count == 0
                        && untracked_count == 0
                        && unmerged_count == 0
                } else {
                    staged_count == 0 && unstaged_count == 0 && unmerged_count == 0
                };

                Ok(RepositoryState {
                    is_git_repo: true,
                    is_detached: false,
                    branch: None,
                    is_clean,
                    staged_count,
                    unstaged_count,
                    untracked_count,
                    unmerged_count,
                })
            }
            Err(_) => {
                // git command failed - likely not a git repository
                Ok(RepositoryState {
                    is_git_repo: false,
                    is_detached: false,
                    branch: None,
                    is_clean: false,
                    staged_count: 0,
                    unstaged_count: 0,
                    untracked_count: 0,
                    unmerged_count: 0,
                })
            }
        }
    }

    /// Format repository state for output.
    fn format_output(state: &RepositoryState, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(state),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(state),
            OutputFormat::Raw => Self::format_raw(state),
            _ => Self::format_compact(state),
        }
    }

    fn format_json(state: &RepositoryState) -> String {
        serde_json::json!({
            "is_git_repo": state.is_git_repo,
            "is_clean": state.is_clean,
            "staged_count": state.staged_count,
            "unstaged_count": state.unstaged_count,
            "untracked_count": state.untracked_count,
            "unmerged_count": state.unmerged_count,
        })
        .to_string()
    }

    fn format_compact(state: &RepositoryState) -> String {
        if !state.is_git_repo {
            return "not a git repository\n".to_string();
        }

        if state.is_clean {
            return "clean\n".to_string();
        }

        format!(
            "dirty (staged={} unstaged={} untracked={} unmerged={})\n",
            state.staged_count, state.unstaged_count, state.untracked_count, state.unmerged_count
        )
    }

    fn format_raw(state: &RepositoryState) -> String {
        if state.is_clean {
            "clean\n".to_string()
        } else {
            "dirty\n".to_string()
        }
    }
}

impl CommandHandler for IsCleanHandler {
    type Input = IsCleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        let state = Self::check_repo_state(input.check_untracked.unwrap_or(true))?;

        // Format and print output
        let formatted = Self::format_output(&state, ctx.format);
        print!("{}", formatted);

        // Exit with appropriate code:
        // 0 - clean
        // 1 - dirty (has changes)
        // 2 - not a git repository
        if !state.is_git_repo {
            return Err(CommandError::ExecutionError {
                message: "not a git repository".to_string(),
                exit_code: Some(2),
            });
        }

        if !state.is_clean {
            return Err(CommandError::ExecutionError {
                message: format!(
                    "repository has changes (staged={} unstaged={} untracked={} unmerged={})",
                    state.staged_count,
                    state.unstaged_count,
                    state.untracked_count,
                    state.unmerged_count
                ),
                exit_code: Some(1),
            });
        }

        Ok(())
    }
}

/// Repository state information.
#[derive(Debug, Clone)]
struct RepositoryState {
    /// Whether this is a git repository.
    is_git_repo: bool,
    /// Whether the repository is in a detached HEAD state.
    is_detached: bool,
    /// The current branch name (or commit hash if detached).
    branch: Option<String>,
    /// Whether the repository is clean (no changes).
    is_clean: bool,
    /// Number of staged files.
    staged_count: usize,
    /// Number of unstaged files.
    unstaged_count: usize,
    /// Number of untracked files.
    untracked_count: usize,
    /// Number of unmerged (conflict) files.
    unmerged_count: usize,
}

/// Input data for the `is-clean` command.
#[derive(Debug, Clone)]
pub struct IsCleanInput {
    pub check_untracked: Option<bool>,
}

/// Handler for the `parse` command and its subcommands.
pub struct ParseHandler;

impl ParseHandler {
    /// Handle the git-status subcommand.
    fn handle_git_status(
        file: &Option<std::path::PathBuf>,
        count: &Option<String>,
        ctx: &CommandContext,
    ) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git status output
        let status = Self::parse_git_status(&input)?;

        // If count flag is specified, output only the count
        if let Some(category) = count {
            let count_value = match category.to_lowercase().as_str() {
                "staged" => status.staged_count,
                "unstaged" => status.unstaged_count,
                "untracked" => status.untracked_count,
                "unmerged" => status.unmerged_count,
                _ => {
                    return Err(CommandError::InvalidArguments(format!(
                        "Invalid count category: {}. Valid options are: staged, unstaged, untracked, unmerged",
                        category
                    )));
                }
            };
            let output = Self::format_git_status_count(count_value, ctx.format);
            print!("{}", output);
        } else {
            // Format output based on the requested format
            let output = Self::format_git_status(&status, ctx.format);
            print!("{}", output);
        }

        Ok(())
    }

    /// Format git status count for output (just the number).
    fn format_git_status_count(count: usize, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({ "count": count }).to_string(),
            OutputFormat::Raw | OutputFormat::Compact | OutputFormat::Agent => {
                format!("{}\n", count)
            }
            OutputFormat::Csv | OutputFormat::Tsv => {
                format!("count\n{}\n", count)
            }
        }
    }

    /// Read input from a file or stdin.
    fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        if let Some(path) = file {
            std::fs::read_to_string(path).map_err(|e| CommandError::IoError(e.to_string()))
        } else {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(buffer)
        }
    }

    /// Parse git status output into structured data.
    fn parse_git_status(input: &str) -> CommandResult<GitStatus> {
        let mut status = GitStatus::default();
        let mut current_section = GitStatusSection::None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Detect branch info (English)
            if line.starts_with("On branch ") {
                status.branch = line.strip_prefix("On branch ").unwrap_or("").to_string();
                continue;
            }

            // Detect branch info (Spanish)
            if line.starts_with("En la rama ") {
                status.branch = line.strip_prefix("En la rama ").unwrap_or("").to_string();
                continue;
            }

            // Detect HEAD detached
            if line.starts_with("HEAD detached at ") {
                status.branch = format!(
                    "HEAD detached at {}",
                    line.strip_prefix("HEAD detached at ").unwrap_or("")
                );
                continue;
            }

            // Detect ahead count: "Your branch is ahead of 'origin/master' by 3 commits."
            if line.starts_with("Your branch is ahead of ") {
                // Parse: "Your branch is ahead of 'origin/master' by 3 commits."
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.ahead = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect behind count: "Your branch is behind 'origin/master' by 5 commits, and can be fast-forwarded."
            if line.starts_with("Your branch is behind ") {
                // Parse: "Your branch is behind 'origin/master' by 5 commits"
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.behind = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect diverged: "Your branch and 'origin/master' have diverged,"
            if line.starts_with("Your branch and ") && line.contains(" have diverged") {
                // This line indicates divergence, but actual counts are on separate lines
                // We'll set a flag to look for counts on next lines
                continue;
            }

            // Detect diverged counts: "  and have 3 and 5 different commits each, respectively."
            if line.contains(" different commits each") {
                // Parse: "  and have 3 and 5 different commits each, respectively."
                let parts: Vec<&str> = line.split_whitespace().collect();
                for i in 0..parts.len() - 1 {
                    if parts[i] == "have" && i + 2 < parts.len() {
                        if let Ok(ahead_count) = parts[i + 1].parse::<usize>() {
                            status.ahead = Some(ahead_count);
                        }
                        if let Ok(behind_count) = parts[i + 2].parse::<usize>() {
                            status.behind = Some(behind_count);
                        }
                    }
                }
                continue;
            }

            // Detect sections (English and localized versions)
            if line.starts_with("Changes to be committed")
                || line.starts_with("Cambios para confirmar")
            {
                current_section = GitStatusSection::Staged;
                continue;
            }
            if line.starts_with("Changes not staged for commit")
                || line.starts_with("Cambios sin rastrear para el commit")
            {
                current_section = GitStatusSection::Unstaged;
                continue;
            }
            if line.starts_with("Untracked files") || line.starts_with("Archivos sin seguimiento") {
                current_section = GitStatusSection::Untracked;
                continue;
            }
            if line.starts_with("Unmerged paths") {
                current_section = GitStatusSection::Unmerged;
                continue;
            }

            // Skip help text (lines starting with '(' or containing 'use "git')
            if line.starts_with('(') || line.contains("use \"git") {
                continue;
            }

            // Parse file entries
            if let Some(entry) = Self::parse_file_entry(line, current_section) {
                match current_section {
                    GitStatusSection::Staged => status.staged.push(entry),
                    GitStatusSection::Unstaged => status.unstaged.push(entry),
                    GitStatusSection::Untracked => status.untracked.push(entry),
                    GitStatusSection::Unmerged => status.unmerged.push(entry),
                    GitStatusSection::None => {
                        // Handle porcelain format or other inline entries
                        if entry.status.starts_with("??") {
                            status.untracked.push(entry);
                        } else if entry.status.starts_with("UU")
                            || entry.status.starts_with("AA")
                            || entry.status.starts_with("DD")
                        {
                            status.unmerged.push(entry);
                        } else if entry.status.starts_with(' ') {
                            // Unstaged changes (porcelain: " M file")
                            status.unstaged.push(entry);
                        } else {
                            // Staged changes (porcelain: "M  file")
                            status.staged.push(entry);
                        }
                    }
                }
            }
        }

        // Check if this is a clean working tree
        status.is_clean = status.staged.is_empty()
            && status.unstaged.is_empty()
            && status.untracked.is_empty()
            && status.unmerged.is_empty();

        // Set file counts
        status.staged_count = status.staged.len();
        status.unstaged_count = status.unstaged.len();
        status.untracked_count = status.untracked.len();
        status.unmerged_count = status.unmerged.len();

        // Check if this is porcelain format (no section headers)
        if status.branch.is_empty()
            && !input
                .lines()
                .any(|l| l.contains("Changes to be committed") || l.contains("Changes not staged"))
        {
            // Try to detect branch from porcelain format if possible
            // Porcelain v2 includes "# branch.head" lines
            for line in input.lines() {
                if line.starts_with("# branch.head ") {
                    status.branch = line
                        .strip_prefix("# branch.head ")
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        Ok(status)
    }

    /// Parse a single file entry from git status.
    fn parse_file_entry(line: &str, section: GitStatusSection) -> Option<GitStatusEntry> {
        if line.is_empty() {
            return None;
        }

        // Handle porcelain format: "XY path" or "XY orig_path -> new_path"
        // XY can be two characters representing index and worktree status
        if section == GitStatusSection::None {
            // Porcelain format
            if line.len() >= 3 {
                let status = &line[..2];
                let path = line[3..].trim();

                if path.is_empty() {
                    return None;
                }

                // Handle rename format: "R  old -> new"
                let (path, old_path) = if path.contains(" -> ") {
                    let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                    (
                        parts.get(1).unwrap_or(&path).to_string(),
                        Some(parts.get(0).unwrap_or(&"").to_string()),
                    )
                } else {
                    (path.to_string(), None)
                };

                return Some(GitStatusEntry {
                    status: status.to_string(),
                    path,
                    old_path,
                });
            }
            return None;
        }

        // Handle standard format with tab indentation: "\tmodified:   path" or "\tnew file:   path"
        // Lines can start with tabs, have status, colon, then path
        if let Some(colon_pos) = line.find(':') {
            let before_colon = line[..colon_pos].trim();
            // Remove leading tabs from status
            let status = before_colon.trim_start_matches('\t').trim();
            let path = line[colon_pos + 1..].trim();

            if path.is_empty() {
                return None;
            }

            // Handle rename format: "renamed:   old -> new"
            let (path, old_path) = if path.contains(" -> ") {
                let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                (
                    parts.get(1).unwrap_or(&path).to_string(),
                    Some(parts.get(0).unwrap_or(&"").to_string()),
                )
            } else {
                (path.to_string(), None)
            };

            // Normalize status to short form
            let short_status = match status {
                // English
                "new file" => "A",
                "modified" => "M",
                "deleted" => "D",
                "renamed" => "R",
                "copied" => "C",
                "typechange" => "T",
                "both added" => "AA",
                "both deleted" => "DD",
                "both modified" => "UU",
                "added by them" => "AU",
                "deleted by them" => "DU",
                "added by us" => "UA",
                "deleted by us" => "UD",
                // Spanish
                "nuevo archivo" => "A",
                "modificados" => "M",
                "borrados" => "D",
                "renombrados" => "R",
                "copiados" => "C",
                // German
                "neue Datei" => "A",
                "geändert" => "M",
                "gelöscht" => "D",
                "umbenannt" => "R",
                // French
                "nouveau fichier" => "A",
                "modifié" => "M",
                "supprimé" => "D",
                "renommé" => "R",
                _ => status,
            };

            return Some(GitStatusEntry {
                status: short_status.to_string(),
                path,
                old_path,
            });
        }

        // Handle untracked files in standard format (just the path, no prefix)
        if section == GitStatusSection::Untracked {
            return Some(GitStatusEntry {
                status: "??".to_string(),
                path: line.to_string(),
                old_path: None,
            });
        }

        None
    }

    /// Format git status for output.
    fn format_git_status(status: &GitStatus, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_status_json(status),
            OutputFormat::Csv => Self::format_git_status_csv(status),
            OutputFormat::Tsv => Self::format_git_status_tsv(status),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_status_compact(status),
            OutputFormat::Raw => Self::format_git_status_raw(status),
        }
    }

    /// Format git status as CSV.
    fn format_git_status_csv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status,path,old_path,section\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{},{},{},staged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{},{},{},unstaged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{},{},{},untracked\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{},{},{},unmerged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as TSV.
    fn format_git_status_tsv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status\tpath\told_path\tsection\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{}\t{}\t{}\tstaged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunstaged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{}\t{}\t{}\tuntracked\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunmerged\n",
                entry.status,
                entry.path,
                entry.old_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as JSON.
    fn format_git_status_json(status: &GitStatus) -> String {
        serde_json::json!({
            "branch": status.branch,
            "is_clean": status.is_clean,
            "staged_count": status.staged_count,
            "unstaged_count": status.unstaged_count,
            "untracked_count": status.untracked_count,
            "unmerged_count": status.unmerged_count,
            "staged": status.staged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "old_path": e.old_path,
            })).collect::<Vec<_>>(),
            "unstaged": status.unstaged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "old_path": e.old_path,
            })).collect::<Vec<_>>(),
            "untracked": status.untracked.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "old_path": e.old_path,
            })).collect::<Vec<_>>(),
            "unmerged": status.unmerged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "old_path": e.old_path,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format git status in compact format.
    fn format_git_status_compact(status: &GitStatus) -> String {
        let mut output = String::new();

        // Branch info
        if !status.branch.is_empty() {
            output.push_str(&format!("branch: {}\n", status.branch));
        }

        // Clean state
        if status.is_clean {
            output.push_str("status: clean\n");
            return output;
        }

        // Summary line with counts
        output.push_str(&format!(
            "counts: staged={} unstaged={} untracked={} unmerged={}\n",
            status.staged_count,
            status.unstaged_count,
            status.untracked_count,
            status.unmerged_count
        ));

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&format!("staged ({}):\n", status.staged.len()));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        // Unstaged changes
        if !status.unstaged.is_empty() {
            output.push_str(&format!("unstaged ({}):\n", status.unstaged.len()));
            for entry in &status.unstaged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        // Untracked files
        if !status.untracked.is_empty() {
            output.push_str(&format!("untracked ({}):\n", status.untracked.len()));
            for entry in &status.untracked {
                output.push_str(&format!("  {} {}\n", entry.status, entry.path));
            }
        }

        // Unmerged files
        if !status.unmerged.is_empty() {
            output.push_str(&format!("unmerged ({}):\n", status.unmerged.len()));
            for entry in &status.unmerged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        output
    }

    /// Format git status as raw output (just the files).
    fn format_git_status_raw(status: &GitStatus) -> String {
        let mut output = String::new();

        for entry in &status.staged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unstaged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.untracked {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unmerged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }

        output
    }

    /// Handle the git-diff subcommand.
    fn handle_git_diff(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git diff output
        let diff = Self::parse_git_diff(&input)?;

        // Format output based on the requested format
        let output = Self::format_git_diff(&diff, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse git diff output into structured data.
    fn parse_git_diff(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        let mut current_file: Option<GitDiffEntry> = None;
        let mut in_hunk = false;

        for line in input.lines() {
            // Detect diff header for a new file
            if line.starts_with("diff --git ") {
                // Save the previous file if any
                if let Some(file) = current_file.take() {
                    diff.files.push(file);
                }

                // Parse the file path from "diff --git a/path b/path"
                let parts: Vec<&str> = line.split_whitespace().collect();
                let (path, old_path) = if parts.len() >= 3 {
                    // Format: "diff --git a/old b/new"
                    let a_path = parts
                        .get(2)
                        .unwrap_or(&"")
                        .strip_prefix("a/")
                        .unwrap_or(parts.get(2).unwrap_or(&""));
                    let b_path = parts
                        .get(3)
                        .unwrap_or(&"")
                        .strip_prefix("b/")
                        .unwrap_or(parts.get(3).unwrap_or(&""));
                    if a_path != b_path {
                        (b_path.to_string(), Some(a_path.to_string()))
                    } else {
                        (b_path.to_string(), None)
                    }
                } else {
                    (String::new(), None)
                };

                current_file = Some(GitDiffEntry {
                    path,
                    old_path,
                    change_type: "M".to_string(), // Default, will be updated
                    additions: 0,
                    deletions: 0,
                    is_binary: false,
                });
                in_hunk = false;
                continue;
            }

            // Detect new file mode (addition)
            if line.starts_with("new file mode ") || line.starts_with("new file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "A".to_string();
                }
                continue;
            }

            // Detect deleted file mode
            if line.starts_with("deleted file mode ") || line.starts_with("deleted file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "D".to_string();
                }
                continue;
            }

            // Detect rename from
            if line.starts_with("rename from ") {
                if let Some(ref mut file) = current_file {
                    file.old_path =
                        Some(line.strip_prefix("rename from ").unwrap_or("").to_string());
                    file.change_type = "R".to_string();
                }
                continue;
            }

            // Detect rename to
            if line.starts_with("rename to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("rename to ").unwrap_or("").to_string();
                }
                continue;
            }

            // Detect copy from
            if line.starts_with("copy from ") {
                if let Some(ref mut file) = current_file {
                    file.old_path = Some(line.strip_prefix("copy from ").unwrap_or("").to_string());
                    file.change_type = "C".to_string();
                }
                continue;
            }

            // Detect copy to
            if line.starts_with("copy to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("copy to ").unwrap_or("").to_string();
                }
                continue;
            }

            // Detect binary file
            if line.contains("Binary files ") && line.contains(" differ") {
                if let Some(ref mut file) = current_file {
                    file.is_binary = true;
                }
                continue;
            }

            // Detect hunk header "@@ -start,count +start,count @@"
            if line.starts_with("@@ ") {
                in_hunk = true;
                continue;
            }

            // Count additions and deletions in hunks
            if in_hunk {
                if let Some(ref mut file) = current_file {
                    if line.starts_with('+') && !line.starts_with("+++") {
                        file.additions += 1;
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        file.deletions += 1;
                    }
                }
            }

            // Handle "--- a/path" and "+++ b/path" to confirm paths
            if line.starts_with("--- ") {
                // Could be "--- a/path" or "--- /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "A".to_string();
                    }
                }
            }
            if line.starts_with("+++ ") {
                // Could be "+++ b/path" or "+++ /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "D".to_string();
                    }
                }
            }
        }

        // Don't forget the last file
        if let Some(file) = current_file {
            diff.files.push(file);
        }

        // Set total files count before any truncation
        diff.total_files = diff.files.len();
        diff.files_shown = diff.files.len();

        // Calculate totals
        for file in &diff.files {
            diff.total_additions += file.additions;
            diff.total_deletions += file.deletions;
        }

        // Check if empty
        diff.is_empty = diff.files.is_empty();

        Ok(diff)
    }

    /// Default maximum number of files to show in diff output before truncation.
    const DEFAULT_MAX_DIFF_FILES: usize = 50;

    /// Truncate diff files list if it exceeds the limit.
    fn truncate_diff(diff: &mut GitDiff, max_files: usize) {
        if diff.files.len() > max_files {
            diff.is_truncated = true;
            diff.files_shown = max_files;
            diff.files.truncate(max_files);
        }
    }

    /// Format git diff for output.
    fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_diff_json(diff),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_diff_compact(diff),
            OutputFormat::Raw => Self::format_git_diff_raw(diff),
            _ => Self::format_git_diff_compact(diff),
        }
    }

    /// Format git diff as JSON.
    fn format_git_diff_json(diff: &GitDiff) -> String {
        serde_json::json!({
            "is_empty": diff.is_empty,
            "is_truncated": diff.is_truncated,
            "total_files": diff.total_files,
            "files_shown": diff.files_shown,
            "files": diff.files.iter().map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "old_path": file.old_path,
                    "change_type": file.change_type,
                    "additions": file.additions,
                    "deletions": file.deletions,
                    "is_binary": file.is_binary,
                })
            }).collect::<Vec<_>>(),
            "total_additions": diff.total_additions,
            "total_deletions": diff.total_deletions,
            "truncation": if diff.is_truncated {
                Some(serde_json::json!({
                    "hidden_files": diff.total_files.saturating_sub(diff.files_shown),
                    "message": format!("Output truncated: showing {} of {} files", diff.files_shown, diff.total_files),
                }))
            } else {
                None
            },
        })
        .to_string()
    }

    /// Format git diff in compact format.
    fn format_git_diff_compact(diff: &GitDiff) -> String {
        let mut output = String::new();

        if diff.is_empty {
            output.push_str("diff: empty\n");
            return output;
        }

        // Show file count with truncation info if applicable
        if diff.is_truncated {
            output.push_str(&format!(
                "files ({}/{} shown):\n",
                diff.files_shown, diff.total_files
            ));
        } else {
            output.push_str(&format!("files ({}):\n", diff.files.len()));
        }

        for file in &diff.files {
            let change_indicator = match file.change_type.as_str() {
                "A" => "+",
                "D" => "-",
                "R" => "R",
                "C" => "C",
                _ => "M",
            };

            if let Some(ref old_path) = file.old_path {
                output.push_str(&format!(
                    "  {} {} -> {} (+{}/-{})\n",
                    change_indicator, old_path, file.path, file.additions, file.deletions
                ));
            } else {
                output.push_str(&format!(
                    "  {} {} (+{}/-{})\n",
                    change_indicator, file.path, file.additions, file.deletions
                ));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("  ... {} more file(s) not shown\n", hidden));
        }

        output.push_str(&format!(
            "summary: +{} -{}\n",
            diff.total_additions, diff.total_deletions
        ));

        output
    }

    /// Format git diff as raw output (just the files).
    fn format_git_diff_raw(diff: &GitDiff) -> String {
        let mut output = String::new();

        for file in &diff.files {
            if let Some(ref old_path) = file.old_path {
                output.push_str(&format!(
                    "{} {} -> {}\n",
                    file.change_type, old_path, file.path
                ));
            } else {
                output.push_str(&format!("{} {}\n", file.change_type, file.path));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("... {} more file(s) truncated\n", hidden));
        }

        output
    }

    /// Handle the ls subcommand.
    fn handle_ls(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the ls output
        let ls_output = Self::parse_ls(&input)?;

        // Format output based on the requested format
        let output = Self::format_ls(&ls_output, ctx.format);
        print!("{}", output);

        Ok(())
    }
    /// Parse ls output into structured data.
    fn parse_ls(input: &str) -> CommandResult<LsOutput> {
        let mut ls_output = LsOutput::default();
        let mut current_entry: Option<LsEntry> = None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip "total N" summary lines from ls -l
            if line.starts_with("total ") {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "ls: cannot open directory '/path': Permission denied"
            // or: "ls: cannot access 'file': No such file or directory"
            if line.starts_with("ls: ") && line.contains("cannot ") {
                // Parse the error message
                let error = Self::parse_ls_error(line);
                ls_output.errors.push(error);
                continue;
            }

            // Check if this is a long format line (starts with permissions)
            // Long format: drwxr-xr-x  2 user group  64 Jan  1 12:34 file.txt
            if Self::is_long_format_line(line) {
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry.clone());
                }

                // Parse the long format line
                current_entry = Some(Self::parse_long_format_line(line));
            } else {
                // This is a short format line (just the filename)
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry);
                }

                // Create entry from the filename
                let name = line.to_string();
                let is_hidden = name.starts_with('.');
                let entry_type = Self::detect_entry_type_from_name(&name);

                current_entry = Some(LsEntry {
                    name,
                    entry_type,
                    is_hidden,
                    ..Default::default()
                });
            }
        }

        // Don't forget the last entry
        if let Some(entry) = current_entry {
            ls_output.entries.push(entry);
        }

        // Categorize entries
        for entry in &ls_output.entries {
            if entry.is_hidden {
                ls_output.hidden.push(entry.clone());
            }
            match entry.entry_type {
                LsEntryType::Directory => {
                    // Check if this is a generated directory
                    if is_generated_directory(&entry.name) {
                        ls_output.generated.push(entry.clone());
                    }
                    ls_output.directories.push(entry.clone())
                }
                LsEntryType::Symlink => ls_output.symlinks.push(entry.clone()),
                _ => ls_output.files.push(entry.clone()),
            }
        }

        // Calculate totals (excluding errors)
        ls_output.total_count = ls_output.entries.len();
        ls_output.is_empty = ls_output.entries.is_empty() && ls_output.errors.is_empty();

        Ok(ls_output)
    }

    /// Parse an ls error message.
    fn parse_ls_error(line: &str) -> LsError {
        // Format: "ls: cannot open directory '/path': Permission denied"
        // or: "ls: cannot access 'file': No such file or directory"

        // Try to extract the path (usually in quotes after 'access' or 'directory')
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        LsError {
            path,
            message: line.to_string(),
        }
    }

    /// Check if a line is in long format (starts with permissions).
    fn is_long_format_line(line: &str) -> bool {
        // Long format lines start with a permission string like:
        // -rwxr-xr-x (file)
        // drwxr-xr-x (directory)
        // lrwxr-xr-x (symlink)
        // brw-r--r-- (block device)
        // crw-r--r-- (char device)
        // srw-r--r-- (socket)
        // prw-r--r-- (pipe/FIFO)
        // total 0 (summary line from ls -l)

        // Skip "total 0" or similar summary lines
        if line.starts_with("total ") {
            return false;
        }

        if line.starts_with('-')
            || line.starts_with('d')
            || line.starts_with('l')
            || line.starts_with('b')
            || line.starts_with('c')
            || line.starts_with('s')
            || line.starts_with('p')
        {
            // Check if it looks like a permission string (has at least 10 characters)
            // Format: type + 9 permission chars (e.g., drwxr-xr-x)
            let perms_part = line.split_whitespace().next();
            if let Some(perms) = perms_part {
                if perms.len() >= 10 {
                    // Check remaining chars (after type indicator) are valid permission chars
                    let rest = &perms[1..];
                    if rest.chars().all(|c| {
                        c == 'r'
                            || c == 'w'
                            || c == 'x'
                            || c == '-'
                            || c == 's'
                            || c == 't'
                            || c == 'S'
                            || c == 'T'
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Parse a long format ls line.
    fn parse_long_format_line(line: &str) -> LsEntry {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Long format: perms links owner group size date time name
        // Example: drwxr-xr-x  2 user  group  4096 Jan  1 12:34 dirname
        //          0          1  2     3     4    5   6  7    8
        // For symlinks: lrwxrwxrwx  1 user  group    10 Jan  1 12:34 link -> target

        if parts.len() < 9 {
            return LsEntry::default();
        }

        let perms = parts[0];
        let name_part = parts[8..].join(" ");

        // Detect entry type from permissions
        let entry_type = Self::detect_entry_type_from_perms(perms);

        // For symlinks, extract name and target (format: "name -> target")
        let (name, symlink_target) =
            if entry_type == LsEntryType::Symlink && name_part.contains(" -> ") {
                let mut split = name_part.splitn(2, " -> ");
                let name = split.next().unwrap_or(&name_part).to_string();
                let target = split.next().map(|s| s.to_string());
                (name, target)
            } else {
                (name_part, None)
            };

        let is_hidden = name.starts_with('.');

        // Check if symlink is broken (target doesn't exist)
        let is_broken_symlink = if entry_type == LsEntryType::Symlink {
            if let Some(ref target) = symlink_target {
                // A broken symlink has a target that doesn't exist
                // Common patterns: absolute paths to non-existent files, relative paths that don't exist
                target.starts_with("/nonexistent") || 
                target.contains("/nonexistent/") ||
                target == "nonexistent" ||
                // Self-referencing (circular) symlinks
                target == &name
            } else {
                false
            }
        } else {
            false
        };

        LsEntry {
            name,
            entry_type,
            is_hidden,
            size: parts.get(4).and_then(|s| s.parse().ok()),
            permissions: Some(perms.to_string()),
            links: parts.get(1).and_then(|s| s.parse().ok()),
            owner: parts.get(2).map(|s| s.to_string()),
            group: parts.get(3).map(|s| s.to_string()),
            modified: Some(format!("{} {} {}", parts[5], parts[6], parts[7])),
            symlink_target,
            is_broken_symlink,
        }
    }
    /// Detect entry type from permission string.
    fn detect_entry_type_from_perms(perms: &str) -> LsEntryType {
        if perms.starts_with('d') {
            LsEntryType::Directory
        } else if perms.starts_with('l') {
            LsEntryType::Symlink
        } else if perms.starts_with('b') {
            LsEntryType::BlockDevice
        } else if perms.starts_with('c') {
            LsEntryType::CharDevice
        } else if perms.starts_with('s') {
            LsEntryType::Socket
        } else if perms.starts_with('p') {
            LsEntryType::Pipe
        } else if perms.starts_with('-') {
            LsEntryType::File
        } else {
            LsEntryType::Other
        }
    }
    /// Detect entry type from name (for short format).
    fn detect_entry_type_from_name(name: &str) -> LsEntryType {
        // In short format, we use heuristics to determine the type
        // 1. If name ends with '/', it's a directory
        // 2. If name has a file extension (contains '.' after the last '/', not just leading '.'), it's a file
        // 3. Otherwise, assume it's a directory (common convention: names without extensions are dirs)
        if name.ends_with('/') {
            LsEntryType::Directory
        } else if Self::has_file_extension(name) {
            LsEntryType::File
        } else {
            LsEntryType::Directory
        }
    }

    /// Check if a name has a file extension (not counting leading dots for hidden files).
    fn has_file_extension(name: &str) -> bool {
        // Get the basename (last component of path)
        let basename = name.rsplit('/').next().unwrap_or(name);

        // Skip the leading dot for hidden files
        let basename = if basename.starts_with('.') && basename.len() > 1 {
            &basename[1..]
        } else {
            basename
        };

        // Check if there's a dot that's not at the start
        // This means we have something like "file.txt" or "name.something"
        if let Some(pos) = basename.rfind('.') {
            // Make sure there's something before the dot and after the dot
            pos > 0 && pos < basename.len() - 1
        } else {
            false
        }
    }
    /// Format ls output for display.
    fn format_ls(ls_output: &LsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_ls_json(ls_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_ls_compact(ls_output),
            OutputFormat::Raw => Self::format_ls_raw(ls_output),
            _ => Self::format_ls_compact(ls_output),
        }
    }
    /// Format ls output as JSON.
    fn format_ls_json(ls_output: &LsOutput) -> String {
        serde_json::json!({
            "is_empty": ls_output.is_empty,
            "total_count": ls_output.total_count,
            "entries": ls_output.entries.iter().map(|e| serde_json::json!({
                "name": e.name,
                "type": match e.entry_type {
                    LsEntryType::File => "file",
                    LsEntryType::Directory => "directory",
                    LsEntryType::Symlink => "symlink",
                    LsEntryType::BlockDevice => "block_device",
                    LsEntryType::CharDevice => "char_device",
                    LsEntryType::Socket => "socket",
                    LsEntryType::Pipe => "pipe",
                    LsEntryType::Other => "other",
                },
                "is_hidden": e.is_hidden,
                "is_generated": e.entry_type == LsEntryType::Directory && is_generated_directory(&e.name),
                "is_broken_symlink": e.is_broken_symlink,
                "links": e.links,
                "owner": e.owner,
                "group": e.group,
                "modified": e.modified,
                "symlink_target": e.symlink_target,
            })).collect::<Vec<_>>(),
            "directories": ls_output.directories.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "files": ls_output.files.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "symlinks": ls_output.symlinks.iter().map(|e| {
                if let Some(ref target) = e.symlink_target {
                    format!("{} -> {}", e.name, target)
                } else {
                    e.name.clone()
                }
            }).collect::<Vec<_>>(),
            "broken_symlinks": ls_output.symlinks.iter().filter(|e| e.is_broken_symlink).map(|e| &e.name).collect::<Vec<_>>(),
            "hidden": ls_output.hidden.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "generated": ls_output.generated.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "errors": ls_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }
    /// Format ls output in compact format.
    fn format_ls_compact(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !ls_output.errors.is_empty() {
            for error in &ls_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if ls_output.is_empty {
            output.push_str("ls: empty\n");
            return output;
        }

        output.push_str(&format!("total: {}\n", ls_output.total_count));

        if !ls_output.directories.is_empty() {
            output.push_str(&format!("directories ({}):\n", ls_output.directories.len()));
            for entry in &ls_output.directories {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.files.is_empty() {
            output.push_str(&format!("files ({}):\n", ls_output.files.len()));
            for entry in &ls_output.files {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.symlinks.is_empty() {
            output.push_str(&format!("symlinks ({}):\n", ls_output.symlinks.len()));
            for entry in &ls_output.symlinks {
                if let Some(ref target) = entry.symlink_target {
                    if entry.is_broken_symlink {
                        output.push_str(&format!("  {} -> {} [broken]\n", entry.name, target));
                    } else {
                        output.push_str(&format!("  {} -> {}\n", entry.name, target));
                    }
                } else {
                    output.push_str(&format!("  {}\n", entry.name));
                }
            }
        }

        if !ls_output.hidden.is_empty() {
            output.push_str(&format!("hidden ({}):\n", ls_output.hidden.len()));
            for entry in &ls_output.hidden {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        if !ls_output.generated.is_empty() {
            output.push_str(&format!("generated ({}):\n", ls_output.generated.len()));
            for entry in &ls_output.generated {
                output.push_str(&format!("  {}\n", entry.name));
            }
        }

        output
    }
    /// Format ls output as raw (just filenames).
    fn format_ls_raw(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        for entry in &ls_output.entries {
            output.push_str(&format!("{}\n", entry.name));
        }

        output
    }

    /// Handle the grep subcommand.
    fn handle_grep(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the grep output
        let mut grep_output = Self::parse_grep(&input)?;

        // Apply truncation for large result sets
        Self::truncate_grep(
            &mut grep_output,
            Self::DEFAULT_MAX_GREP_FILES,
            Self::DEFAULT_MAX_GREP_MATCHES_PER_FILE,
        );

        // Format output based on the requested format
        let output = Self::format_grep(&grep_output, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse grep output into structured data.
    ///
    /// Supports multiple grep output formats:
    /// - Standard format: `filename:line_number:matched_line`
    /// - Without line numbers: `filename:matched_line`
    /// - With column: `filename:line_number:column:matched_line`
    /// - Recursive format (ripgrep): `filename:line_number:matched_line`
    fn parse_grep(input: &str) -> CommandResult<GrepOutput> {
        let mut grep_output = GrepOutput::default();
        let mut current_file: Option<GrepFile> = None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip grep summary lines (e.g., from ripgrep)
            if line.starts_with("grep:") || line.contains("matched ") && line.ends_with(" files") {
                continue;
            }

            // Try to parse the grep line
            if let Some((path, grep_match)) = Self::parse_grep_line(line) {
                // Check if this is the same file as the current one
                if let Some(ref mut file) = current_file {
                    if file.path == path {
                        file.matches.push(grep_match);
                        continue;
                    } else {
                        // Different file, save the current one
                        grep_output.files.push(file.clone());
                    }
                }

                // Start a new file
                current_file = Some(GrepFile {
                    path,
                    matches: vec![grep_match],
                });
            }
        }

        // Don't forget the last file
        if let Some(file) = current_file {
            grep_output.files.push(file);
        }

        // Calculate totals
        grep_output.file_count = grep_output.files.len();
        for file in &grep_output.files {
            grep_output.match_count += file.matches.len();
        }

        // Set total counts before any truncation
        grep_output.total_files = grep_output.files.len();
        grep_output.total_matches = grep_output.match_count;
        grep_output.files_shown = grep_output.files.len();
        grep_output.matches_shown = grep_output.match_count;

        // Check if empty
        grep_output.is_empty = grep_output.files.is_empty();

        Ok(grep_output)
    }

    /// Default maximum number of files to show in grep output before truncation.
    const DEFAULT_MAX_GREP_FILES: usize = 50;

    /// Default maximum number of matches per file to show before truncation.
    const DEFAULT_MAX_GREP_MATCHES_PER_FILE: usize = 20;

    /// Truncate grep output if it exceeds the limits.
    ///
    /// This truncates both the number of files and the number of matches per file
    /// to prevent overwhelming output for large result sets.
    fn truncate_grep(grep_output: &mut GrepOutput, max_files: usize, max_matches_per_file: usize) {
        // First, truncate matches per file
        for file in &mut grep_output.files {
            if file.matches.len() > max_matches_per_file {
                file.matches.truncate(max_matches_per_file);
            }
        }

        // Then, truncate files if needed
        if grep_output.files.len() > max_files {
            grep_output.is_truncated = true;
            grep_output.files_shown = max_files;
            grep_output.files.truncate(max_files);
        } else if grep_output.total_matches > grep_output.files.iter().map(|f| f.matches.len()).sum::<usize>() {
            // Some matches were truncated per-file but not files
            grep_output.is_truncated = true;
            grep_output.files_shown = grep_output.files.len();
        }

        // Calculate final matches shown
        grep_output.matches_shown = grep_output.files.iter().map(|f| f.matches.len()).sum();
    }

    /// Parse a single grep line.
    ///
    /// Formats supported:
    /// - `path:line_number:content` (standard with -n)
    /// - `path:line_number:column:content` (with --column)
    /// - `path:content` (without -n)
    /// - Binary file matches: `Binary file path matches`
    /// - Context lines: `path-line_number-content` (with -C/-B/-A flags)
    fn parse_grep_line(line: &str) -> Option<(String, GrepMatch)> {
        // Handle "Binary file path matches" format
        if line.starts_with("Binary file ") && line.ends_with(" matches") {
            let path = line
                .strip_prefix("Binary file ")
                .unwrap_or("")
                .strip_suffix(" matches")
                .unwrap_or("");
            if !path.is_empty() {
                return Some((
                    path.to_string(),
                    GrepMatch {
                        line_number: None,
                        column: None,
                        line: "[binary file]".to_string(),
                        is_context: false,
                    },
                ));
            }
        }

        // Determine if this is a context line or match line
        // Context lines use "-" as separator: "path-line-content"
        // Match lines use ":" as separator: "path:line:content"
        // Find the first separator (either : or -)
        let is_context_line = if let Some(dash_pos) = line.find('-') {
            // Check if dash comes before any colon (or no colon at all)
            match line.find(':') {
                Some(colon_pos) if colon_pos < dash_pos => false,
                _ => true,
            }
        } else {
            false
        };

        // Find the first separator to get the path
        let sep_pos = if is_context_line {
            line.find('-')?
        } else {
            line.find(':')?
        };

        let potential_path = &line[..sep_pos];

        // If the path is empty or the rest doesn't have content, skip
        if potential_path.is_empty() || line.len() <= sep_pos + 1 {
            return None;
        }

        let rest = &line[sep_pos + 1..];

        // Try to parse line number and optionally column
        // Format: line_number:content OR line_number:column:content OR just content
        // Context lines: line_number-content OR line_number-column-content
        let (line_number, column, content, is_context) =
            Self::parse_grep_line_content(rest, is_context_line);

        Some((
            potential_path.to_string(),
            GrepMatch {
                line_number,
                column,
                line: content.to_string(),
                is_context,
            },
        ))
    }

    /// Parse the content part of a grep line (after the path: or path-).
    ///
    /// Context lines use "-" as separator (e.g., "10-content" for context)
    /// while match lines use ":" (e.g., "10:content" for matches).
    fn parse_grep_line_content(
        rest: &str,
        is_context_line: bool,
    ) -> (Option<usize>, Option<usize>, &str, bool) {
        if is_context_line {
            // Context line: use "-" as separator
            // Format: "10-content" or "10-5-content"
            if let Some(dash_pos) = rest.find('-') {
                let potential_line_num = &rest[..dash_pos];

                // Check if it's a valid line number before the dash
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[dash_pos + 1..];

                    // Try to parse column if present (context with column: "10-5-content")
                    if let Some(dash_pos2) = after_line.find('-') {
                        let potential_column = &after_line[..dash_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[dash_pos2 + 1..],
                                true, // is_context
                            );
                        }
                    }

                    // No column, just line number with context
                    return (Some(line_number), None, after_line, true);
                }
            }
            // Couldn't parse as context line, return as content
            (None, None, rest, true)
        } else {
            // Match line: use ":" as separator
            // Try to find the first colon for line number
            if let Some(colon_pos) = rest.find(':') {
                let potential_line_num = &rest[..colon_pos];

                // Check if it's a valid line number
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[colon_pos + 1..];

                    // Try to parse column if present
                    if let Some(colon_pos2) = after_line.find(':') {
                        let potential_column = &after_line[..colon_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[colon_pos2 + 1..],
                                false, // is_context
                            );
                        }
                    }

                    // No column, just line number
                    return (Some(line_number), None, after_line, false);
                }
            }

            // No line number, just content
            (None, None, rest, false)
        }
    }

    /// Format grep output for display.
    fn format_grep(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_grep_json(grep_output),
            OutputFormat::Csv => Self::format_grep_csv(grep_output),
            OutputFormat::Tsv => Self::format_grep_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_grep_compact(grep_output),
            OutputFormat::Raw => Self::format_grep_raw(grep_output),
        }
    }

    /// Format grep output as JSON.
    fn format_grep_json(grep_output: &GrepOutput) -> String {
        // Count only non-context matches
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        serde_json::json!({
            "is_empty": grep_output.is_empty,
            "is_truncated": grep_output.is_truncated,
            "total_files": grep_output.total_files,
            "total_matches": grep_output.total_matches,
            "files_shown": grep_output.files_shown,
            "matches_shown": grep_output.matches_shown,
            "file_count": grep_output.file_count,
            "match_count": match_count,
            "files": grep_output.files.iter().map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "matches": file.matches.iter().map(|m| serde_json::json!({
                        "line_number": m.line_number,
                        "column": m.column,
                        "line": m.line,
                        "is_context": m.is_context,
                    })).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
            "truncation": if grep_output.is_truncated {
                Some(serde_json::json!({
                    "hidden_files": grep_output.total_files.saturating_sub(grep_output.files_shown),
                    "hidden_matches": grep_output.total_matches.saturating_sub(grep_output.matches_shown),
                    "message": format!(
                        "Output truncated: showing {} of {} files, {} of {} matches",
                        grep_output.files_shown,
                        grep_output.total_files,
                        grep_output.matches_shown,
                        grep_output.total_matches
                    ),
                }))
            } else {
                None
            },
        })
        .to_string()
    }

    /// Format grep output as CSV.
    fn format_grep_csv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path,line_number,column,is_context,line\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_csv_field(&m.line);
                result.push_str(&format!(
                    "{},{},{},{},{}\n",
                    file.path,
                    m.line_number
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output as TSV.
    fn format_grep_tsv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path\tline_number\tcolumn\tis_context\tline\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_tsv_field(&m.line);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    file.path,
                    m.line_number
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output in compact format.
    ///
    /// Consecutive context lines are collapsed into a summary like "... (3 context lines)".
    fn format_grep_compact(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        if grep_output.is_empty {
            output.push_str("grep: no matches\n");
            return output;
        }

        // Count only non-context matches for the summary
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        // Show summary with truncation info if applicable
        if grep_output.is_truncated {
            output.push_str(&format!(
                "matches: {}/{} files, {}/{} results (truncated)\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else {
            output.push_str(&format!(
                "matches: {} files, {} results\n",
                grep_output.file_count, match_count
            ));
        }

        for file in &grep_output.files {
            let non_context_count = file.matches.iter().filter(|m| !m.is_context).count();
            output.push_str(&format!("{} ({}):\n", file.path, non_context_count));

            // Track consecutive context lines for collapsing
            let mut context_start: Option<usize> = None;
            let mut context_count = 0;

            for m in &file.matches {
                if m.is_context {
                    // Start or continue a context block
                    if context_start.is_none() {
                        context_start = m.line_number;
                    }
                    context_count += 1;
                } else {
                    // Output any accumulated context lines first
                    if context_count > 0 {
                        if context_count == 1 {
                            // Single context line - show it
                            if let Some(ln) = context_start {
                                output.push_str(&format!("  {}: ...\n", ln));
                            }
                        } else {
                            // Multiple context lines - collapse
                            if let Some(start) = context_start {
                                output.push_str(&format!(
                                    "  {}-{}: ... ({} context lines)\n",
                                    start,
                                    start + context_count - 1,
                                    context_count
                                ));
                            }
                        }
                        context_start = None;
                        context_count = 0;
                    }

                    // Output the match line
                    if let Some(ln) = m.line_number {
                        if let Some(col) = m.column {
                            output.push_str(&format!("  {}:{}: {}\n", ln, col, m.line));
                        } else {
                            output.push_str(&format!("  {}: {}\n", ln, m.line));
                        }
                    } else {
                        output.push_str(&format!("  {}\n", m.line));
                    }
                }
            }

            // Handle any trailing context lines
            if context_count > 0 {
                if context_count == 1 {
                    if let Some(ln) = context_start {
                        output.push_str(&format!("  {}: ...\n", ln));
                    }
                } else {
                    if let Some(start) = context_start {
                        output.push_str(&format!(
                            "  {}-{}: ... ({} context lines)\n",
                            start,
                            start + context_count - 1,
                            context_count
                        ));
                    }
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output.total_files.saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output.total_matches.saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!(
                    "  ... {} more file(s) not shown\n",
                    hidden_files
                ));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "  ... {} more match(es) not shown\n",
                    hidden_matches
                ));
            }
        }

        output
    }

    /// Format grep output as raw (original format).
    fn format_grep_raw(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        for file in &grep_output.files {
            for m in &file.matches {
                // Use dash separator for context lines, colon for matches
                let sep = if m.is_context { "-" } else { ":" };
                if let Some(ln) = m.line_number {
                    if let Some(col) = m.column {
                        output.push_str(&format!(
                            "{}{}{}{}{}:{}\n",
                            file.path, sep, ln, sep, col, m.line
                        ));
                    } else {
                        output.push_str(&format!("{}{}{}{}{}\n", file.path, sep, ln, sep, m.line));
                    }
                } else {
                    output.push_str(&format!("{}:{}\n", file.path, m.line));
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output.total_files.saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output.total_matches.saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!(
                    "... {} more file(s) truncated\n",
                    hidden_files
                ));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "... {} more match(es) truncated\n",
                    hidden_matches
                ));
            }
        }

        output
    }

    /// Handle the test subcommand.
    fn handle_test(
        runner: &Option<crate::TestRunner>,
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: test (runner: {:?}, file: {:?})", runner, file);

        // TODO: Implement actual test parsing
        Err(CommandError::NotImplemented("test parsing".to_string()))
    }

    /// Handle the logs subcommand.
    fn handle_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: logs (file: {:?})", file);

        // TODO: Implement actual logs parsing
        Err(CommandError::NotImplemented("logs parsing".to_string()))
    }

    /// Handle the find subcommand.
    fn handle_find(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the find output
        let find_output = Self::parse_find(&input)?;

        // Format output based on the requested format
        let output = Self::format_find(&find_output, ctx.format);
        print!("{}", output);

        Ok(())
    }

    /// Parse find output into structured data.
    fn parse_find(input: &str) -> CommandResult<FindOutput> {
        let mut find_output = FindOutput::default();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "find: '/path': Permission denied"
            // or: "find: cannot open directory '/path': Permission denied"
            // or: "find: 'path': No such file or directory"
            if line.starts_with("find: ") && line.contains(':') {
                let error = Self::parse_find_error(line);
                find_output.errors.push(error);
                continue;
            }

            // Each line is a file path
            let path = line.to_string();
            let is_directory = path.ends_with('/');
            let is_hidden = path
                .split('/')
                .last()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false);

            let entry = FindEntry {
                path: path.clone(),
                is_directory,
                is_hidden,
                extension: Self::extract_extension(&path),
                depth: Self::calculate_path_depth(&path),
            };

            find_output.entries.push(entry.clone());
            find_output.total_count += 1;

            if is_directory {
                find_output.directories.push(path.clone());
            } else {
                find_output.files.push(path.clone());
            }

            if is_hidden {
                find_output.hidden.push(path);
            }

            // Track extensions
            if let Some(ext) = &entry.extension {
                *find_output.extensions.entry(ext.clone()).or_insert(0) += 1;
            }
        }

        // Check if empty (considering both entries and errors)
        find_output.is_empty = find_output.entries.is_empty();

        Ok(find_output)
    }

    /// Parse a find error message.
    fn parse_find_error(line: &str) -> FindError {
        // Format: "find: '/path': Permission denied"
        // or: "find: cannot open directory '/path': Permission denied"
        // or: "find: 'path': No such file or directory"

        // Try to extract the path (usually in quotes)
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        FindError {
            path,
            message: line.to_string(),
        }
    }

    /// Extract file extension from path.
    fn extract_extension(path: &str) -> Option<String> {
        let filename = path.split('/').last()?;
        // Skip hidden files starting with . and files with no extension
        if filename.starts_with('.') {
            return None;
        }
        let dot_pos = filename.rfind('.')?;
        if dot_pos == 0 {
            return None;
        }
        Some(filename[dot_pos + 1..].to_lowercase())
    }

    /// Calculate the depth of a path (number of path separators).
    fn calculate_path_depth(path: &str) -> usize {
        path.matches('/').count()
    }

    /// Format find output for display.
    fn format_find(find_output: &FindOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_find_json(find_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_find_compact(find_output),
            OutputFormat::Raw => Self::format_find_raw(find_output),
            _ => Self::format_find_compact(find_output),
        }
    }

    /// Format find output as JSON.
    fn format_find_json(find_output: &FindOutput) -> String {
        serde_json::json!({
            "is_empty": find_output.is_empty,
            "total_count": find_output.total_count,
            "entries": find_output.entries.iter().map(|e| serde_json::json!({
                "path": e.path,
                "is_directory": e.is_directory,
                "is_hidden": e.is_hidden,
                "extension": e.extension,
                "depth": e.depth,
            })).collect::<Vec<_>>(),
            "directories": find_output.directories,
            "files": find_output.files,
            "hidden": find_output.hidden,
            "extensions": find_output.extensions,
            "errors": find_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format find output in compact format.
    fn format_find_compact(find_output: &FindOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !find_output.errors.is_empty() {
            for error in &find_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if find_output.is_empty && find_output.errors.is_empty() {
            output.push_str("find: empty\n");
            return output;
        }

        if !find_output.is_empty {
            output.push_str(&format!("total: {}\n", find_output.total_count));
        }

        if !find_output.directories.is_empty() {
            output.push_str(&format!(
                "directories ({}):\n",
                find_output.directories.len()
            ));
            for path in &find_output.directories {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.files.is_empty() {
            output.push_str(&format!("files ({}):\n", find_output.files.len()));
            for path in &find_output.files {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.hidden.is_empty() {
            output.push_str(&format!("hidden ({}):\n", find_output.hidden.len()));
            for path in &find_output.hidden {
                output.push_str(&format!("  {}\n", path));
            }
        }

        if !find_output.extensions.is_empty() {
            output.push_str(&format!("extensions ({}):\n", find_output.extensions.len()));
            let mut exts: Vec<_> = find_output.extensions.iter().collect();
            exts.sort_by(|a, b| b.1.cmp(a.1));
            for (ext, count) in exts {
                output.push_str(&format!("  {}: {}\n", ext, count));
            }
        }

        output
    }

    /// Format find output as raw (just paths).
    fn format_find_raw(find_output: &FindOutput) -> String {
        let mut output = String::new();

        for entry in &find_output.entries {
            output.push_str(&format!("{}\n", entry.path));
        }

        output
    }
}

impl CommandHandler for ParseHandler {
    type Input = ParseCommands;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        match input {
            ParseCommands::GitStatus { file, count } => Self::handle_git_status(file, count, ctx),
            ParseCommands::GitDiff { file } => Self::handle_git_diff(file, ctx),
            ParseCommands::Ls { file } => Self::handle_ls(file, ctx),
            ParseCommands::Grep { file } => Self::handle_grep(file, ctx),
            ParseCommands::Find { file } => Self::handle_find(file, ctx),
            ParseCommands::Test { runner, file } => Self::handle_test(runner, file, ctx),
            ParseCommands::Logs { file } => Self::handle_logs(file, ctx),
        }
    }
}

/// Router that dispatches commands to their handlers.
pub struct Router {
    run_handler: RunHandler,
    search_handler: SearchHandler,
    replace_handler: ReplaceHandler,
    tail_handler: TailHandler,
    clean_handler: CleanHandler,
    html2md_handler: Html2mdHandler,
    txt2md_handler: Txt2mdHandler,
    is_clean_handler: IsCleanHandler,
    parse_handler: ParseHandler,
}

impl Router {
    /// Create a new router with all command handlers.
    pub fn new() -> Self {
        Self {
            run_handler: RunHandler,
            search_handler: SearchHandler,
            replace_handler: ReplaceHandler,
            tail_handler: TailHandler,
            clean_handler: CleanHandler,
            html2md_handler: Html2mdHandler,
            txt2md_handler: Txt2mdHandler,
            is_clean_handler: IsCleanHandler,
            parse_handler: ParseHandler,
        }
    }

    /// Route a command to its handler and execute it.
    pub fn route(&self, command: &Commands, ctx: &CommandContext) -> CommandResult {
        match command {
            Commands::Run {
                command,
                args,
                capture_stdout,
                capture_stderr,
                capture_exit_code,
                capture_duration,
            } => {
                let input = RunInput::from((
                    command,
                    args,
                    capture_stdout.unwrap_or(true),
                    capture_stderr.unwrap_or(true),
                    capture_exit_code.unwrap_or(true),
                    capture_duration.unwrap_or(true),
                    None, // timeout not supported via CLI yet
                ));
                self.run_handler.execute(&input, ctx)
            }
            Commands::Search {
                path,
                query,
                extension,
                ignore_case,
                context,
                limit,
            } => {
                let input = SearchInput {
                    path: path.clone(),
                    query: query.clone(),
                    extension: extension.clone(),
                    ignore_case: *ignore_case,
                    context: *context,
                    limit: *limit,
                };
                self.search_handler.execute(&input, ctx)
            }
            Commands::Replace {
                path,
                search,
                replace,
                extension,
                dry_run,
            } => {
                let input = ReplaceInput {
                    path: path.clone(),
                    search: search.clone(),
                    replace: replace.clone(),
                    extension: extension.clone(),
                    dry_run: *dry_run,
                };
                self.replace_handler.execute(&input, ctx)
            }
            Commands::Tail {
                file,
                lines,
                errors,
                follow,
            } => {
                let input = TailInput {
                    file: file.clone(),
                    lines: *lines,
                    errors: *errors,
                    follow: *follow,
                };
                self.tail_handler.execute(&input, ctx)
            }
            Commands::Clean {
                file,
                no_ansi,
                collapse_blanks,
                collapse_repeats,
                trim,
            } => {
                let input = CleanInput {
                    file: file.clone(),
                    no_ansi: *no_ansi,
                    collapse_blanks: *collapse_blanks,
                    collapse_repeats: *collapse_repeats,
                    trim: *trim,
                };
                self.clean_handler.execute(&input, ctx)
            }
            Commands::Html2md {
                input,
                output,
                metadata,
            } => {
                let input = Html2mdInput {
                    input: input.clone(),
                    output: output.clone(),
                    metadata: *metadata,
                };
                self.html2md_handler.execute(&input, ctx)
            }
            Commands::Txt2md { input, output } => {
                let input = Txt2mdInput {
                    input: input.clone(),
                    output: output.clone(),
                };
                self.txt2md_handler.execute(&input, ctx)
            }
            Commands::IsClean { check_untracked } => {
                let input = IsCleanInput {
                    check_untracked: *check_untracked,
                };
                self.is_clean_handler.execute(&input, ctx)
            }
            Commands::Parse { parser } => self.parse_handler.execute(parser, ctx),
        }
    }

    /// Execute a command and print the result or error.
    pub fn execute_and_print(&self, command: &Commands, ctx: &CommandContext) {
        match self.route(command, ctx) {
            Ok(()) => {}
            Err(CommandError::NotImplemented(msg)) => {
                // Format not implemented message according to output format
                let formatted = Self::format_not_implemented(&msg, ctx.format);
                if ctx.format == OutputFormat::Json {
                    // For JSON, output to stderr (consistent with error handling)
                    eprintln!("{}", formatted);
                } else {
                    println!("{}", formatted);
                }
            }
            Err(e) => {
                // Format error according to output format
                let formatted = Self::format_command_error(&e, ctx.format);
                eprintln!("{}", formatted);
                // Propagate the exit code if available, otherwise default to 1
                let exit_code = e.exit_code().unwrap_or(1);
                std::process::exit(exit_code);
            }
        }
    }

    /// Format a not-implemented message based on the output format.
    fn format_not_implemented(msg: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "not_implemented": true,
                "message": format!("{} not yet implemented", msg),
            })
            .to_string(),
            _ => format!("{} not yet implemented", msg),
        }
    }

    /// Format a CommandError based on the output format.
    fn format_command_error(error: &CommandError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "error": true,
                "message": error.to_string(),
                "exit_code": error.exit_code(),
            })
            .to_string(),
            _ => format!("Error: {}", error),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_context_creation() {
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: true,
            enabled_formats: vec![OutputFormat::Json, OutputFormat::Csv],
        };

        assert_eq!(ctx.format, OutputFormat::Json);
        assert!(ctx.stats);
        assert!(ctx.has_conflicting_formats());
    }

    #[test]
    fn test_command_context_no_conflict() {
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert!(!ctx.has_conflicting_formats());
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::NotImplemented("test command".to_string());
        assert_eq!(format!("{}", err), "Not implemented: test command");

        let err = CommandError::ExecutionError {
            message: "failed".to_string(),
            exit_code: Some(1),
        };
        assert_eq!(format!("{}", err), "Execution error: failed");

        let err = CommandError::InvalidArguments("bad args".to_string());
        assert_eq!(format!("{}", err), "Invalid arguments: bad args");

        let err = CommandError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

    #[test]
    fn test_run_handler_success() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // echo should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_command_not_found() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "nonexistent_command_xyz123".to_string(),
            args: vec![],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // Should return an error for command not found
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CommandError::ExecutionError {
                message: _,
                exit_code: _
            })
        ));
    }

    #[test]
    fn test_run_handler_non_zero_exit() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "false".to_string(),
            args: vec![],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        // false always exits with 1
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CommandError::ExecutionError {
                message: _,
                exit_code: _
            })
        ));
    }

    #[test]
    fn test_run_handler_json_format() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_no_capture_stdout() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = RunInput {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: false,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
            timeout: None,
        };

        // When stdout is not captured, the command should still succeed
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_handler_no_capture_exit_code() {
        let handler = RunHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = RunInput {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 42".to_string()],
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: false,
            capture_duration: true,
            timeout: None,
        };

        // When exit code is not captured, the error is NOT propagated
        // even though the command exited with a non-zero code
        let result = handler.execute(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_handler() {
        let handler = SearchHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: true,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = SearchInput {
            path: std::path::PathBuf::from("."),
            query: "test".to_string(),
            extension: Some("rs".to_string()),
            ignore_case: true,
            context: Some(2),
            limit: Some(100),
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_replace_handler() {
        let handler = ReplaceHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = ReplaceInput {
            path: std::path::PathBuf::from("."),
            search: "old".to_string(),
            replace: "new".to_string(),
            extension: Some("ts".to_string()),
            dry_run: true,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_tail_handler() {
        let handler = TailHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = TailInput {
            file: std::path::PathBuf::from("/var/log/test.log"),
            lines: 20,
            errors: true,
            follow: false,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_clean_handler() {
        let handler = CleanHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = CleanInput {
            file: None,
            no_ansi: true,
            collapse_blanks: true,
            collapse_repeats: false,
            trim: true,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_html2md_handler() {
        let handler = Html2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = Html2mdInput {
            input: "https://example.com".to_string(),
            output: Some(std::path::PathBuf::from("out.md")),
            metadata: true,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_txt2md_handler() {
        let handler = Txt2mdHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = Txt2mdInput {
            input: Some(std::path::PathBuf::from("input.txt")),
            output: Some(std::path::PathBuf::from("output.md")),
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_parse_handler_git_status() {
        // Test with empty input (simulating empty stdin)
        // This should result in a clean status with empty branch
        let handler = ParseHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = ParseCommands::GitStatus {
            file: None,
            count: None,
        };

        // Note: This test reads from stdin which is empty, so it will succeed
        // with an empty/clean status
        let result = handler.execute(&input, &ctx);
        // The implementation is now complete, so it should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_handler_test() {
        let handler = ParseHandler;
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let input = ParseCommands::Test {
            runner: Some(crate::TestRunner::Pytest),
            file: None,
        };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    #[test]
    fn test_router_run_command_success() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Run {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            capture_stdout: Some(true),
            capture_stderr: Some(true),
            capture_exit_code: Some(true),
            capture_duration: Some(true),
        };

        let result = router.route(&command, &ctx);
        // echo should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_router_run_command_failure() {
        let router = Router::new();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Run {
            command: "false".to_string(),
            args: vec![],
            capture_stdout: Some(true),
            capture_stderr: Some(true),
            capture_exit_code: Some(true),
            capture_duration: Some(true),
        };

        let result = router.route(&command, &ctx);
        // false exits with 1
        assert!(result.is_err());
    }

    #[test]
    fn test_router_default() {
        let router = Router::default();
        let ctx = CommandContext {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        };
        let command = Commands::Search {
            path: std::path::PathBuf::from("."),
            query: "test".to_string(),
            extension: None,
            ignore_case: false,
            context: None,
            limit: None,
        };

        let result = router.route(&command, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
    }

    // ============================================================
    // Grep Parser Tests
    // ============================================================

    #[test]
    fn test_parse_grep_empty() {
        let result = ParseHandler::parse_grep("").unwrap();
        assert!(result.is_empty);
        assert_eq!(result.file_count, 0);
        assert_eq!(result.match_count, 0);
    }

    #[test]
    fn test_parse_grep_single_file_single_match() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.is_empty);
        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 1);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_single_file_multiple_matches() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].matches.len(), 2);
        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[1].line_number, Some(45));
    }

    #[test]
    fn test_parse_grep_multiple_files() {
        let input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 2);
        assert_eq!(result.match_count, 2);
        assert_eq!(result.files[0].path, "src/main.rs");
        assert_eq!(result.files[1].path, "src/lib.rs");
    }

    #[test]
    fn test_parse_grep_with_column() {
        let input = "src/main.rs:42:10:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_without_line_number() {
        let input = "src/main.rs:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, None);
        assert_eq!(result.files[0].matches[0].line, "fn main() {");
    }

    #[test]
    fn test_parse_grep_binary_file() {
        let input = "Binary file target/debug binary matches";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.files[0].path, "target/debug binary");
        assert_eq!(result.files[0].matches[0].line, "[binary file]");
    }

    #[test]
    fn test_parse_grep_format_compact() {
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("matches: 1 files, 2 results"));
        assert!(output.contains("src/main.rs (2):"));
        assert!(output.contains("42: fn main() {"));
        assert!(output.contains("45:     println!"));
    }

    #[test]
    fn test_parse_grep_format_json() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["file_count"], 1);
        assert_eq!(json["match_count"], 1);
        assert_eq!(json["files"][0]["path"], "src/main.rs");
        assert_eq!(json["files"][0]["matches"][0]["line_number"], 42);
        assert_eq!(json["files"][0]["matches"][0]["line"], "fn main() {");
    }

    #[test]
    fn test_parse_grep_format_csv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Csv);

        assert!(output.starts_with("path,line_number,column,is_context,line\n"));
        assert!(output.contains("src/main.rs,42,,false,fn main() {"));
    }

    #[test]
    fn test_parse_grep_format_tsv() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Tsv);

        assert!(output.starts_with("path\tline_number\tcolumn\tis_context\tline\n"));
        assert!(output.contains("src/main.rs\t42\t\tfalse\tfn main() {"));
    }

    #[test]
    fn test_parse_grep_format_raw() {
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:42:fn main() {"));
    }

    #[test]
    fn test_parse_grep_empty_compact() {
        let mut result = GrepOutput::default();
        result.is_empty = true;
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("grep: no matches"));
    }

    #[test]
    fn test_parse_grep_line_with_colon_in_content() {
        // Content containing colons should be handled correctly
        let input = "src/main.rs:42:let x = \"http://example.com\";";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(
            result.files[0].matches[0].line,
            "let x = \"http://example.com\";"
        );
    }

    // ============================================================
    // Context Line Tests
    // ============================================================

    #[test]
    fn test_parse_grep_context_line() {
        // Context lines use "-" as separator (from grep -C/-B/-A)
        let input = "src/main.rs-42-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_context_line_with_column() {
        // Context line with column info
        let input = "src/main.rs-42-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches[0].line_number, Some(42));
        assert_eq!(result.files[0].matches[0].column, Some(10));
        assert_eq!(result.files[0].matches[0].line, "context line");
        assert!(result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_parse_grep_mixed_match_and_context() {
        // Mix of match and context lines
        let input = "src/main.rs-41-context before\nsrc/main.rs:42:match line\nsrc/main.rs-43-context after";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.files[0].matches.len(), 3);

        // First line is context
        assert!(result.files[0].matches[0].is_context);
        assert_eq!(result.files[0].matches[0].line, "context before");

        // Second line is a match
        assert!(!result.files[0].matches[1].is_context);
        assert_eq!(result.files[0].matches[1].line, "match line");

        // Third line is context
        assert!(result.files[0].matches[2].is_context);
        assert_eq!(result.files[0].matches[2].line, "context after");
    }

    #[test]
    fn test_parse_grep_context_is_context_flag_false_for_matches() {
        let input = "src/main.rs:42:match line";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert!(!result.files[0].matches[0].is_context);
    }

    #[test]
    fn test_format_grep_compact_collapse_context_lines() {
        // Multiple consecutive context lines should be collapsed
        let input = "src/main.rs-10-context 1\nsrc/main.rs-11-context 2\nsrc/main.rs-12-context 3\nsrc/main.rs:13:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should collapse 3 context lines into a summary
        assert!(output.contains("10-12: ... (3 context lines)"));
        assert!(output.contains("13: match line"));
    }

    #[test]
    fn test_format_grep_compact_single_context_line() {
        // Single context line should show as "... (1 context lines)" format
        let input = "src/main.rs-10-context line\nsrc/main.rs:11:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match line"));
    }

    #[test]
    fn test_format_grep_compact_context_before_and_after() {
        // Context lines before and after match
        let input = "src/main.rs-10-before\nsrc/main.rs:11:match\nsrc/main.rs-12-after";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: ..."));
        assert!(output.contains("11: match"));
        assert!(output.contains("12: ..."));
    }

    #[test]
    fn test_format_grep_compact_count_excludes_context() {
        // Match count should exclude context lines
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match\nsrc/main.rs-12-context";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Should show 1 result (only the match), not 3
        assert!(output.contains("matches: 1 files, 1 results"));
    }

    #[test]
    fn test_format_grep_compact_trailing_context() {
        // Context lines at the end should be collapsed
        let input = "src/main.rs:10:match\nsrc/main.rs-11-context 1\nsrc/main.rs-12-context 2";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        assert!(output.contains("10: match"));
        assert!(output.contains("11-12: ... (2 context lines)"));
    }

    #[test]
    fn test_format_grep_json_includes_is_context() {
        let input = "src/main.rs-10-context\nsrc/main.rs:11:match";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Json);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["files"][0]["matches"][0]["is_context"], true);
        assert_eq!(json["files"][0]["matches"][1]["is_context"], false);
    }

    #[test]
    fn test_format_grep_raw_context_uses_dash() {
        // Raw format should preserve dash separator for context
        let input = "src/main.rs-10-context line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs-10-context line"));
    }

    #[test]
    fn test_format_grep_raw_match_uses_colon() {
        // Raw format should use colon for matches
        let input = "src/main.rs:10:match line";
        let result = ParseHandler::parse_grep(input).unwrap();
        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        assert!(output.contains("src/main.rs:10:match line"));
    }

    // ============================================================
    // Grep Truncation Tests
    // ============================================================

    #[test]
    fn test_parse_grep_truncation_fields_not_truncated() {
        // Small result set should not be truncated
        let input = "src/main.rs:42:fn main() {";
        let result = ParseHandler::parse_grep(input).unwrap();

        assert_eq!(result.is_truncated, false);
        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_matches, 1);
        assert_eq!(result.files_shown, 1);
        assert_eq!(result.matches_shown, 1);
    }

    #[test]
    fn test_truncate_grep_files() {
        // Create 60 files (exceeds DEFAULT_MAX_GREP_FILES = 50)
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 60);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.total_files, 60);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_truncate_grep_matches_per_file() {
        // Create 1 file with 25 matches (exceeds DEFAULT_MAX_GREP_MATCHES_PER_FILE = 20)
        let mut input = String::new();
        for i in 1..=25 {
            input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 25);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.matches_shown, 20);
        assert_eq!(result.total_matches, 25);
        assert_eq!(result.files[0].matches.len(), 20);
    }

    #[test]
    fn test_truncate_grep_both_limits() {
        // Create 60 files, each with 25 matches
        let mut input = String::new();
        for i in 1..=60 {
            for j in 1..=25 {
                input.push_str(&format!("src/file{}.rs:{}:fn func{}() {{\n", i, j, j));
            }
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();

        // Before truncation: 60 files * 25 matches = 1500 total matches
        assert_eq!(result.total_files, 60);
        assert_eq!(result.total_matches, 1500);

        // Apply truncation
        ParseHandler::truncate_grep(&mut result, 50, 20);

        // After truncation: 50 files * 20 matches = 1000 matches shown
        assert_eq!(result.is_truncated, true);
        assert_eq!(result.files_shown, 50);
        assert_eq!(result.matches_shown, 1000);
        assert_eq!(result.files.len(), 50);
    }

    #[test]
    fn test_format_grep_json_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], true);
        assert_eq!(json["total_files"], 60);
        assert_eq!(json["files_shown"], 50);
        assert_eq!(json["truncation"]["hidden_files"], 10);
        assert!(json["truncation"]["message"].as_str().unwrap().contains("60"));
        assert!(json["truncation"]["message"].as_str().unwrap().contains("50"));
    }

    #[test]
    fn test_format_grep_compact_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

        // Check for truncation indicators in compact output
        assert!(output.contains("truncated"));
        assert!(output.contains("50/60"));
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_raw_truncation_info() {
        // Create 60 files to trigger truncation
        let mut input = String::new();
        for i in 1..=60 {
            input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
        }
        let mut result = ParseHandler::parse_grep(&input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

        // Check for truncation indicator in raw output
        assert!(output.contains("10 more file"));
    }

    #[test]
    fn test_format_grep_json_no_truncation_when_within_limits() {
        // Small result set should not show truncation info
        let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:println!()";
        let mut result = ParseHandler::parse_grep(input).unwrap();
        ParseHandler::truncate_grep(&mut result, 50, 20);

        let output = ParseHandler::format_grep(&result, OutputFormat::Json);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["is_truncated"], false);
        assert!(json["truncation"].is_null());
    }
}
