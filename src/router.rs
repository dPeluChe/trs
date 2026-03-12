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
    /// An error occurred during execution.
    ExecutionError(String),
    /// Invalid arguments provided.
    InvalidArguments(String),
    /// I/O error occurred.
    IoError(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            CommandError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

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
        let result = ProcessBuilder::new(&input.command).args(&input.args).run();

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

                // Propagate exit code
                if !output.success() {
                    return Err(CommandError::ExecutionError(format!(
                        "Command exited with code {}",
                        output.code()
                    )));
                }

                Ok(())
            }
            Err(error) => {
                // Print stats if requested
                if ctx.stats {
                    eprintln!("Stats:");
                    eprintln!("  Command failed: {}", error);
                }

                // Format and print error
                let formatted = Self::format_error(&error, ctx.format);
                eprintln!("{}", formatted);

                // Return appropriate error type
                Err(match &error {
                    ProcessError::CommandNotFound { command } => {
                        CommandError::ExecutionError(format!("Command not found: {}", command))
                    }
                    ProcessError::PermissionDenied { command } => {
                        CommandError::ExecutionError(format!("Permission denied: {}", command))
                    }
                    ProcessError::Timeout {
                        command, duration, ..
                    } => CommandError::ExecutionError(format!(
                        "Command '{}' timed out after {:.2}s",
                        command,
                        duration.as_secs_f64()
                    )),
                    ProcessError::NonZeroExit { output } => CommandError::ExecutionError(format!(
                        "Command exited with code {}",
                        output.code()
                    )),
                    ProcessError::IoError { message, .. } => CommandError::IoError(message.clone()),
                    ProcessError::SpawnFailed { message, .. } => {
                        CommandError::ExecutionError(message.clone())
                    }
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
}

impl From<(&String, &Vec<String>)> for RunInput {
    fn from((command, args): (&String, &Vec<String>)) -> Self {
        Self {
            command: command.clone(),
            args: args.clone(),
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
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: git-status (file: {:?})", file);

        // TODO: Implement actual git-status parsing
        Err(CommandError::NotImplemented(
            "git-status parsing".to_string(),
        ))
    }

    /// Handle the git-diff subcommand.
    fn handle_git_diff(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        if ctx.stats {
            eprintln!("Stats: enabled");
        }
        eprintln!("Output format: {:?}", ctx.format);
        eprintln!("Parser: git-diff (file: {:?})", file);

        // TODO: Implement actual git-diff parsing
        Err(CommandError::NotImplemented("git-diff parsing".to_string()))
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
            Commands::Run { command, args } => {
                let input = RunInput::from((command, args));
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
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
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

        let err = CommandError::ExecutionError("failed".to_string());
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
        };

        let result = handler.execute(&input, &ctx);
        // Should return an error for command not found
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::ExecutionError(_))));
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
        };

        let result = handler.execute(&input, &ctx);
        // false always exits with 1
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::ExecutionError(_))));
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
        };

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
        let handler = ParseHandler;
        let ctx = CommandContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };
        let input = ParseCommands::GitStatus { file: None };

        let result = handler.execute(&input, &ctx);
        assert!(matches!(result, Err(CommandError::NotImplemented(_))));
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
