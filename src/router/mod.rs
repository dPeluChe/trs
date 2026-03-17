//! Command routing system for TARS CLI.
//!
//! This module provides the Router that dispatches CLI commands to their
//! respective handlers. Handler implementations are in the `handlers` submodule.

pub(crate) mod handlers;

// Re-export public types needed by main.rs
pub use handlers::common::{CommandContext, CommandError, CommandResult};

// Re-export all handler types for internal use and tests
use handlers::common::*;
use handlers::types::*;
use handlers::run::*;
use handlers::search::*;
use handlers::replace::*;
use handlers::tail::*;
use handlers::clean::*;
use handlers::trim::*;
use handlers::html2md::*;
use handlers::txt2md::*;
use handlers::isclean::*;
use handlers::parse::*;

use crate::{Commands, OutputFormat};
#[allow(unused_imports)]
use crate::ParseCommands;

pub struct Router {
    run_handler: RunHandler,
    search_handler: SearchHandler,
    replace_handler: ReplaceHandler,
    tail_handler: TailHandler,
    clean_handler: CleanHandler,
    trim_handler: TrimHandler,
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
            trim_handler: TrimHandler,
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
                count,
            } => {
                let input = ReplaceInput {
                    path: path.clone(),
                    search: search.clone(),
                    replace: replace.clone(),
                    extension: extension.clone(),
                    dry_run: *dry_run,
                    count: *count,
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
            Commands::Trim {
                file,
                leading,
                trailing,
            } => {
                let input = TrimInput {
                    file: file.clone(),
                    leading: *leading,
                    trailing: *trailing,
                };
                self.trim_handler.execute(&input, ctx)
            }
            Commands::IsClean { check_untracked } => {
                let input = IsCleanInput {
                    check_untracked: *check_untracked,
                };
                self.is_clean_handler.execute(&input, ctx)
            }
            Commands::Parse { parser } => self.parse_handler.execute(parser, ctx),
            Commands::External(_) => {
                // External commands are handled in main.rs before reaching the router
                Ok(())
            }
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

    /// Process stdin input when no command is specified.
    ///
    /// This reads from stdin and applies basic text processing:
    /// - Strips ANSI codes
    /// - Trims whitespace
    /// - Collapses blank lines
    /// - Sanitizes control characters
    pub fn process_stdin(&self, input: &str, ctx: &CommandContext) -> CommandResult<String> {
        let mut result = input.to_string();

        // Strip ANSI escape codes FIRST (before sanitizing control chars)
        // because ANSI codes start with \x1b which is a control character
        result = strip_ansi_codes(&result);

        // Sanitize control characters (remove nulls, replace other control chars)
        result = sanitize_control_chars(&result);

        // Trim trailing whitespace from each line
        result = result
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple blank lines into single blank lines
        let lines: Vec<&str> = result.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(line);
            prev_blank = is_blank;
        }

        result = collapsed_lines.join("\n");

        // Remove leading/trailing blank lines
        result = result.trim().to_string();

        // Format output based on the requested format
        let formatted = match ctx.format {
            OutputFormat::Raw => result.clone(),
            OutputFormat::Compact => result.clone(),
            OutputFormat::Json => serde_json::json!({
                "content": result,
                "stats": {
                    "input_length": input.len(),
                    "output_length": result.len(),
                }
            })
            .to_string(),
            OutputFormat::Agent => {
                format!("Content:\n{}\n", result)
            }
            OutputFormat::Csv => {
                // Output as CSV with one row per line
                result
                    .lines()
                    .map(|line| format!("\"{}\"", line.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            OutputFormat::Tsv => {
                // Output as TSV with one row per line
                result.lines().collect::<Vec<_>>().join("\n")
            }
        };

        Ok(formatted)
    }

    /// Format a not-implemented message based on the output format.
    fn format_not_implemented(msg: &str, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "not_implemented": true,
                "message": format!("{} not yet implemented", msg),
            })
            .to_string(),
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("{} not yet implemented", msg),
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
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("Error: {}", error),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests;
