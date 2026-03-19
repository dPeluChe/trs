//! Helper functions for process execution.

use std::io;
use std::process::ExitStatus;
use std::time::{Duration, Instant};

use super::{ProcessError, ProcessOutput};

/// Classify a spawn error into a more specific ProcessError.
pub(super) fn classify_spawn_error(command: &str, error: io::Error) -> ProcessError {
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
pub(super) fn capture_output(
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
pub(super) fn capture_partial_output(child: &mut std::process::Child) -> (String, String) {
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
pub(super) trait ChildExt {
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
