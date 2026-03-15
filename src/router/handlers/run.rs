use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::process::{ProcessBuilder, ProcessError, ProcessOutput};
use crate::OutputFormat;

pub(crate) struct RunHandler;

impl RunHandler {
    /// Format output based on the specified format.
    pub(crate) fn format_output(output: &ProcessOutput, format: OutputFormat) -> String {
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
            OutputFormat::Raw => {
                // Raw output: unprocessed stdout and stderr
                let mut result = output.stdout.clone();
                if output.has_stderr() && !output.stderr.is_empty() {
                    result.push_str(&output.stderr);
                }
                result
            }
        }
    }

    /// Escape a field for CSV format.
    pub(crate) fn escape_csv_field(field: &str) -> String {
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
    pub(crate) fn escape_tsv_field(field: &str) -> String {
        // TSV doesn't support tabs in fields; replace with space
        field.replace('\t', " ").replace('\r', "")
    }

    /// Format error message based on format.
    #[allow(dead_code)]
    pub(crate) fn format_error(error: &ProcessError, format: OutputFormat) -> String {
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
            OutputFormat::Raw
            | OutputFormat::Compact
            | OutputFormat::Agent
            | OutputFormat::Csv
            | OutputFormat::Tsv => format!("Error: {}", error),
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
                // Format output first to calculate reduced size
                let formatted = Self::format_output(&output, ctx.format);

                // Print stats if requested
                if ctx.stats {
                    let input_bytes = output.stdout.len() + output.stderr.len();
                    let output_bytes = formatted.len();
                    let stats = CommandStats::new()
                        .with_command(format!("{} {:?}", output.command, output.args))
                        .with_exit_code(output.code())
                        .with_duration_ms(output.duration.as_millis() as u64)
                        .with_input_bytes(input_bytes)
                        .with_output_bytes(output_bytes)
                        .with_output_mode(ctx.format)
                        .with_extra("Stdout bytes", output.stdout.len().to_string())
                        .with_extra("Stderr bytes", output.stderr.len().to_string());
                    stats.print();
                }

                // Print output
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
                    let stats = CommandStats::new()
                        .with_output_mode(ctx.format)
                        .with_extra("Error", error.to_string());
                    stats.print();
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
pub(crate) struct RunInput {
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
