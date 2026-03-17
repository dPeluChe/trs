//! Process execution module for TARS CLI.
//!
//! This module provides a robust interface for executing external commands
//! and capturing their output, exit codes, and execution duration.

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Result of a process execution.
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// The command that was executed.
    pub command: String,
    /// Arguments passed to the command.
    pub args: Vec<String>,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
    /// Exit code of the process (if available).
    pub exit_code: Option<i32>,
    /// Duration of the execution.
    pub duration: Duration,
    /// Whether the process was killed due to timeout.
    pub timed_out: bool,
}

impl ProcessOutput {
    /// Returns true if the process exited successfully (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Returns the exit code, or 1 if not available.
    pub fn code(&self) -> i32 {
        self.exit_code.unwrap_or(1)
    }

    /// Returns true if stdout is not empty.
    pub fn has_stdout(&self) -> bool {
        !self.stdout.is_empty()
    }

    /// Returns true if stderr is not empty.
    pub fn has_stderr(&self) -> bool {
        !self.stderr.is_empty()
    }
}

/// Error type for process execution failures.
#[derive(Debug, Clone)]
pub enum ProcessError {
    /// The command was not found.
    CommandNotFound { command: String },
    /// Permission denied when executing the command.
    PermissionDenied { command: String },
    /// The process timed out.
    Timeout {
        command: String,
        args: Vec<String>,
        duration: Duration,
        partial_stdout: String,
        partial_stderr: String,
    },
    /// The process exited with a non-zero exit code.
    NonZeroExit { output: ProcessOutput },
    /// I/O error during execution.
    IoError { command: String, message: String },
    /// Failed to spawn the process.
    SpawnFailed { command: String, message: String },
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::CommandNotFound { command } => {
                write!(f, "Command not found: {}", command)
            }
            ProcessError::PermissionDenied { command } => {
                write!(f, "Permission denied: {}", command)
            }
            ProcessError::Timeout {
                command, duration, ..
            } => {
                write!(
                    f,
                    "Command '{}' timed out after {:.2}s",
                    command,
                    duration.as_secs_f64()
                )
            }
            ProcessError::NonZeroExit { output } => {
                write!(
                    f,
                    "Command '{}' exited with code {}",
                    output.command,
                    output.code()
                )
            }
            ProcessError::IoError { command, message } => {
                write!(f, "I/O error executing '{}': {}", command, message)
            }
            ProcessError::SpawnFailed { command, message } => {
                write!(f, "Failed to spawn '{}': {}", command, message)
            }
        }
    }
}

impl std::error::Error for ProcessError {}

impl ProcessError {
    /// Returns the exit code if this is a NonZeroExit error.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            ProcessError::NonZeroExit { output } => Some(output.code()),
            _ => None,
        }
    }

    /// Returns true if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, ProcessError::Timeout { .. })
    }

    /// Returns true if this is a command not found error.
    pub fn is_command_not_found(&self) -> bool {
        matches!(self, ProcessError::CommandNotFound { .. })
    }

    /// Returns true if this is a permission denied error.
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, ProcessError::PermissionDenied { .. })
    }
}

/// Builder for configuring and executing a process.
#[derive(Debug, Clone)]
pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
    env: HashMap<String, String>,
    env_clear: bool,
    timeout: Option<Duration>,
    capture_stdout: bool,
    capture_stderr: bool,
    capture_exit_code: bool,
    capture_duration: bool,
}

impl ProcessBuilder {
    /// Create a new process builder for the given command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            current_dir: None,
            env: HashMap::new(),
            env_clear: false,
            timeout: None,
            capture_stdout: true,
            capture_stderr: true,
            capture_exit_code: true,
            capture_duration: true,
        }
    }

    /// Add an argument to the command.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments to the command.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    /// Set the working directory for the process.
    pub fn current_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(dir.into());
        self
    }

    /// Set an environment variable for the process.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables for the process.
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in vars {
            self.env.insert(key.into(), value.into());
        }
        self
    }

    /// Clear all environment variables before setting new ones.
    pub fn env_clear(mut self, clear: bool) -> Self {
        self.env_clear = clear;
        self
    }

    /// Set a timeout for the process execution.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set whether to capture stdout.
    pub fn capture_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    /// Set whether to capture stderr.
    pub fn capture_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }

    /// Set whether to capture exit code.
    pub fn capture_exit_code(mut self, capture: bool) -> Self {
        self.capture_exit_code = capture;
        self
    }

    /// Set whether to capture execution duration.
    pub fn capture_duration(mut self, capture: bool) -> Self {
        self.capture_duration = capture;
        self
    }

    /// Execute the process and return the output.
    ///
    /// This method captures stdout and stderr, and returns the exit code
    /// and execution duration.
    pub fn run(&self) -> Result<ProcessOutput, ProcessError> {
        let start = Instant::now();

        // Build the command
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        // Set working directory
        if let Some(ref dir) = self.current_dir {
            cmd.current_dir(dir);
        }

        // Handle environment
        if self.env_clear {
            cmd.env_clear();
        }
        cmd.envs(&self.env);

        // Configure stdio
        if self.capture_stdout {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::inherit());
        }

        if self.capture_stderr {
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stderr(Stdio::inherit());
        }

        // Spawn the process
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Err(classify_spawn_error(&self.command, e));
            }
        };

        // Handle timeout if specified
        let result = if let Some(timeout) = self.timeout {
            match child.wait_timeout(timeout) {
                Ok(Some(status)) => {
                    // Process completed within timeout
                    let output = capture_output(
                        &mut child,
                        status,
                        self.capture_stdout,
                        self.capture_stderr,
                        self.capture_exit_code,
                    );
                    Ok(output)
                }
                Ok(None) => {
                    // Process timed out
                    let _ = child.kill();
                    let _ = child.wait();

                    // Try to capture any partial output
                    let (stdout, stderr) = capture_partial_output(&mut child);

                    Err(ProcessError::Timeout {
                        command: self.command.clone(),
                        args: self.args.clone(),
                        duration: start.elapsed(),
                        partial_stdout: stdout,
                        partial_stderr: stderr,
                    })
                }
                Err(e) => Err(ProcessError::IoError {
                    command: self.command.clone(),
                    message: e.to_string(),
                }),
            }
        } else {
            // No timeout, wait indefinitely
            match child.wait() {
                Ok(status) => {
                    let output = capture_output(
                        &mut child,
                        status,
                        self.capture_stdout,
                        self.capture_stderr,
                        self.capture_exit_code,
                    );
                    Ok(output)
                }
                Err(e) => Err(ProcessError::IoError {
                    command: self.command.clone(),
                    message: e.to_string(),
                }),
            }
        };

        // Add command, args, and duration to successful output
        let duration = start.elapsed();
        let command = self.command.clone();
        let args = self.args.clone();
        let capture_duration = self.capture_duration;
        result.map(|mut output| {
            output.command = command;
            output.args = args;
            output.duration = if capture_duration {
                duration
            } else {
                Duration::ZERO
            };
            output
        })
    }

    /// Execute the process and return the output, treating non-zero exit codes as errors.
    pub fn run_checked(&self) -> Result<ProcessOutput, ProcessError> {
        let output = self.run()?;
        if output.success() {
            Ok(output)
        } else {
            Err(ProcessError::NonZeroExit { output })
        }
    }
}

/// Classify a spawn error into a more specific ProcessError.
fn classify_spawn_error(command: &str, error: io::Error) -> ProcessError {
    match error.kind() {
        io::ErrorKind::NotFound => ProcessError::CommandNotFound {
            command: command.to_string(),
        },
        io::ErrorKind::PermissionDenied => ProcessError::PermissionDenied {
            command: command.to_string(),
        },
        _ => ProcessError::SpawnFailed {
            command: command.to_string(),
            message: error.to_string(),
        },
    }
}

/// Capture output from a completed child process.
fn capture_output(
    child: &mut std::process::Child,
    status: ExitStatus,
    capture_stdout: bool,
    capture_stderr: bool,
    capture_exit_code: bool,
) -> ProcessOutput {
    let stdout = if capture_stdout {
        child
            .stdout
            .take()
            .map(|mut h| {
                let mut buf = String::new();
                let _ = std::io::Read::read_to_string(&mut h, &mut buf);
                buf
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    let stderr = if capture_stderr {
        child
            .stderr
            .take()
            .map(|mut h| {
                let mut buf = String::new();
                let _ = std::io::Read::read_to_string(&mut h, &mut buf);
                buf
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    ProcessOutput {
        command: String::new(),
        args: Vec::new(),
        stdout,
        stderr,
        exit_code: if capture_exit_code {
            status.code()
        } else {
            None
        },
        duration: Duration::ZERO,
        timed_out: false,
    }
}

/// Capture partial output from a timed-out process.
fn capture_partial_output(child: &mut std::process::Child) -> (String, String) {
    let stdout = child
        .stdout
        .take()
        .map(|mut h| {
            let mut buf = String::new();
            // Use non-blocking read if possible
            let _ = std::io::Read::read_to_string(&mut h, &mut buf);
            buf
        })
        .unwrap_or_default();

    let stderr = child
        .stderr
        .take()
        .map(|mut h| {
            let mut buf = String::new();
            let _ = std::io::Read::read_to_string(&mut h, &mut buf);
            buf
        })
        .unwrap_or_default();

    (stdout, stderr)
}

/// Extension trait for Child to support timeout.
trait ChildExt {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<ExitStatus>>;
}

impl ChildExt for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<ExitStatus>> {
        // Simple polling implementation
        // For production, consider using the `wait-timeout` crate
        let start = Instant::now();
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    // Small sleep to avoid busy-waiting
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}

/// Execute a command with arguments and return the output.
///
/// This is a convenience function for simple command execution.
pub fn run(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
    ProcessBuilder::new(command)
        .args(args.iter().copied())
        .run()
}

/// Execute a command with arguments and return the output, treating non-zero exit as error.
pub fn run_checked(command: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
    ProcessBuilder::new(command)
        .args(args.iter().copied())
        .run_checked()
}

/// Execute a command with a timeout.
pub fn run_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<ProcessOutput, ProcessError> {
    ProcessBuilder::new(command)
        .args(args.iter().copied())
        .timeout(timeout)
        .run()
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
