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
            CommandError::ExecutionError { message, .. } => write!(f, "Execution error: {}", message),
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
    /// Staged changes (to be committed).
    staged: Vec<GitStatusEntry>,
    /// Unstaged changes (not staged for commit).
    unstaged: Vec<GitStatusEntry>,
    /// Untracked files.
    untracked: Vec<GitStatusEntry>,
    /// Unmerged paths (merge conflicts).
    unmerged: Vec<GitStatusEntry>,
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
    /// List of changed files.
    files: Vec<GitDiffEntry>,
    /// Total lines added across all files.
    total_additions: usize,
    /// Total lines deleted across all files.
    total_deletions: usize,
    /// Whether the diff is empty.
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
        (command, args, capture_stdout, capture_stderr, capture_exit_code, capture_duration, timeout): (
            &String,
            &Vec<String>,
            bool,
            bool,
            bool,
            bool,
            Option<u64>,
        ),
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

/// Input data for the `txt2md` command.
#[derive(Debug, Clone)]
pub struct Txt2mdInput {
    pub input: Option<std::path::PathBuf>,
    pub output: Option<std::path::PathBuf>,
}

/// Handler for the `parse` command and its subcommands.
pub struct ParseHandler;

impl ParseHandler {
    /// Handle the git-status subcommand.
    fn handle_git_status(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }

        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git status output
        let status = Self::parse_git_status(&input)?;

        // Format output based on the requested format
        let output = Self::format_git_status(&status, ctx.format);
        print!("{}", output);

        Ok(())
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
                status.branch = format!("HEAD detached at {}", line.strip_prefix("HEAD detached at ").unwrap_or(""));
                continue;
            }

            // Detect sections (English and localized versions)
            if line.starts_with("Changes to be committed") || line.starts_with("Cambios para confirmar") {
                current_section = GitStatusSection::Staged;
                continue;
            }
            if line.starts_with("Changes not staged for commit") || line.starts_with("Cambios sin rastrear para el commit") {
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
                        } else if entry.status.starts_with("UU") || entry.status.starts_with("AA") || entry.status.starts_with("DD") {
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

        // Check if this is porcelain format (no section headers)
        if status.branch.is_empty() && !input.lines().any(|l| l.contains("Changes to be committed") || l.contains("Changes not staged")) {
            // Try to detect branch from porcelain format if possible
            // Porcelain v2 includes "# branch.head" lines
            for line in input.lines() {
                if line.starts_with("# branch.head ") {
                    status.branch = line.strip_prefix("# branch.head ").unwrap_or("").to_string();
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
                    (parts.get(1).unwrap_or(&path).to_string(), Some(parts.get(0).unwrap_or(&"").to_string()))
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
                (parts.get(1).unwrap_or(&path).to_string(), Some(parts.get(0).unwrap_or(&"").to_string()))
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
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_status_compact(status),
            OutputFormat::Raw => Self::format_git_status_raw(status),
            _ => Self::format_git_status_compact(status),
        }
    }

    /// Format git status as JSON.
    fn format_git_status_json(status: &GitStatus) -> String {
        serde_json::json!({
            "branch": status.branch,
            "is_clean": status.is_clean,
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

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&format!("staged ({}):\n", status.staged.len()));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!("  {} {} -> {}\n", entry.status, old_path, entry.path));
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
                    output.push_str(&format!("  {} {} -> {}\n", entry.status, old_path, entry.path));
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
                    output.push_str(&format!("  {} {} -> {}\n", entry.status, old_path, entry.path));
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
                    let a_path = parts.get(2).unwrap_or(&"").strip_prefix("a/").unwrap_or(parts.get(2).unwrap_or(&""));
                    let b_path = parts.get(3).unwrap_or(&"").strip_prefix("b/").unwrap_or(parts.get(3).unwrap_or(&""));
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
                    file.old_path = Some(line.strip_prefix("rename from ").unwrap_or("").to_string());
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

        // Calculate totals
        for file in &diff.files {
            diff.total_additions += file.additions;
            diff.total_deletions += file.deletions;
        }

        // Check if empty
        diff.is_empty = diff.files.is_empty();

        Ok(diff)
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
            "files": diff.files.iter().map(|f| serde_json::json!({
                "path": f.path,
                "old_path": f.old_path,
                "change_type": f.change_type,
                "additions": f.additions,
                "deletions": f.deletions,
                "is_binary": f.is_binary,
            })).collect::<Vec<_>>(),
            "total_additions": diff.total_additions,
            "total_deletions": diff.total_deletions,
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

        output.push_str(&format!("files ({}):\n", diff.files.len()));
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

        output
    }

    /// Handle the ls subcommand.
    fn handle_ls(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: ls (file: {:?})", file);

        // TODO: Implement actual ls parsing
        Err(CommandError::NotImplemented("ls parsing".to_string()))
    }

    /// Handle the grep subcommand.
    fn handle_grep(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: grep (file: {:?})", file);

        // TODO: Implement actual grep parsing
        Err(CommandError::NotImplemented("grep parsing".to_string()))
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
}

impl CommandHandler for ParseHandler {
    type Input = ParseCommands;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        match input {
            ParseCommands::GitStatus { file } => Self::handle_git_status(file, ctx),
            ParseCommands::GitDiff { file } => Self::handle_git_diff(file, ctx),
            ParseCommands::Ls { file } => Self::handle_ls(file, ctx),
            ParseCommands::Grep { file } => Self::handle_grep(file, ctx),
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
            Commands::Parse { parser } => self.parse_handler.execute(parser, ctx),
        }
    }

    /// Execute a command and print the result or error.
    pub fn execute_and_print(&self, command: &Commands, ctx: &CommandContext) {
        match self.route(command, ctx) {
            Ok(()) => {}
            Err(CommandError::NotImplemented(msg)) => {
                println!("{} not yet implemented", msg);
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

    /// Format a CommandError based on the output format.
    fn format_command_error(error: &CommandError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => {
                serde_json::json!({
                    "error": true,
                    "message": error.to_string(),
                    "exit_code": error.exit_code(),
                })
                .to_string()
            }
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
        assert!(matches!(result, Err(CommandError::ExecutionError { message: _, exit_code: _ })));
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
        assert!(matches!(result, Err(CommandError::ExecutionError { message: _, exit_code: _ })));
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
        let input = ParseCommands::GitStatus { file: None };

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
}
