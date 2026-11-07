use super::common::{csv_row, tsv_row, CommandContext, CommandError, CommandResult, CommandStats};
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
                    "timed_out": false,
                })
                .to_string()
            }
            OutputFormat::Csv => Self::table(output, csv_row),
            OutputFormat::Tsv => Self::table(output, tsv_row),
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
                if output.has_stderr() {
                    result.push_str(&output.stderr);
                }
                result
            }
        }
    }

    fn table(output: &ProcessOutput, row: fn(&[&str]) -> String) -> String {
        let header = row(&[
            "command",
            "args",
            "stdout",
            "stderr",
            "exit_code",
            "duration_ms",
            "timed_out",
        ]);
        header
            + &row(&[
                &output.command,
                &output.args.join(" "),
                &output.stdout,
                &output.stderr,
                &output.exit_code.to_string(),
                &output.duration.as_millis().to_string(),
                "false",
            ])
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

                if !output.success() {
                    // The document carries the code; a second one breaks `2>&1` parsers.
                    if ctx.format == OutputFormat::Json {
                        return Err(CommandError::Reported {
                            exit_code: output.code(),
                        });
                    }
                    // Agents run with `2>/dev/null`, so the error alone can vanish.
                    if matches!(ctx.format, OutputFormat::Compact | OutputFormat::Agent) {
                        let line = crate::exec_verdict::exit_line(output.code(), None);
                        crate::exec_verdict::notice(ctx.format, &line);
                    }
                    return Err(CommandError::ExecutionError {
                        message: format!("Command exited with code {}", output.code()),
                        exit_code: Some(output.exit_code),
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
                    ProcessError::Timeout { .. } => CommandError::ExecutionError {
                        message: error.to_string(),
                        exit_code: Some(124),
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
    pub capture_duration: bool,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
}
