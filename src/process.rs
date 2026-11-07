//! Process execution module for trs (Token-Reducing Shell).
//!
//! This module provides a robust interface for executing external commands
//! and capturing their output, exit codes, and execution duration.

use std::process::Stdio;
use std::time::{Duration, Instant};

#[path = "process_helpers.rs"]
mod helpers;
use helpers::{classify_spawn_error, ChildExt, OutputReaders};
pub use helpers::{exit_code_of, spawn_failure};

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
    /// Exit code of the process; 128+N when signal N killed it.
    pub exit_code: i32,
    /// Duration of the execution.
    pub duration: Duration,
}

impl ProcessOutput {
    /// Returns true if the process exited successfully (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Returns the exit code.
    pub fn code(&self) -> i32 {
        self.exit_code
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
    Timeout { command: String, duration: Duration },
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
            ProcessError::Timeout { command, duration } => {
                write!(
                    f,
                    "Command '{}' timed out after {:.2}s",
                    command,
                    duration.as_secs_f64()
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

/// Builder for configuring and executing a process.
#[derive(Debug, Clone)]
pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    timeout: Option<Duration>,
    capture_stdout: bool,
    capture_stderr: bool,
    capture_duration: bool,
}

impl ProcessBuilder {
    /// Create a new process builder for the given command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            timeout: None,
            capture_stdout: true,
            capture_stderr: true,
            capture_duration: true,
        }
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

        // Build the command. On Windows this routes through the shell so
        // .cmd/.bat shims, .ps1 scripts, and builtins resolve (issue #53);
        // POSIX spawns directly.
        let mut cmd = crate::classifier::build_command(&self.command, &self.args);

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

        let readers = OutputReaders::start(&mut child);

        let waited = match self.timeout {
            Some(timeout) => child.wait_timeout(timeout),
            None => child.wait().map(Some),
        };

        let status = match waited {
            Ok(Some(status)) => status,
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProcessError::Timeout {
                    command: self.command.clone(),
                    duration: start.elapsed(),
                });
            }
            Err(e) => {
                return Err(ProcessError::IoError {
                    command: self.command.clone(),
                    message: e.to_string(),
                });
            }
        };

        let (stdout, stderr) = readers.finish();
        Ok(ProcessOutput {
            command: self.command.clone(),
            args: self.args.clone(),
            stdout,
            stderr,
            exit_code: exit_code_of(&status),
            duration: if self.capture_duration {
                start.elapsed()
            } else {
                Duration::ZERO
            },
        })
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
