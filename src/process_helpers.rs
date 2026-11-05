//! Helper functions for process execution.

use std::io::{self, Read};
use std::process::ExitStatus;
use std::thread::JoinHandle;
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

/// Reads a child's pipes on their own threads. Waiting first would deadlock
/// once a child fills the OS pipe buffer (64 KB) and blocks on write.
pub(super) struct OutputReaders {
    stdout: Option<JoinHandle<String>>,
    stderr: Option<JoinHandle<String>>,
}

impl OutputReaders {
    pub(super) fn start(child: &mut std::process::Child) -> Self {
        Self {
            stdout: child.stdout.take().map(drain),
            stderr: child.stderr.take().map(drain),
        }
    }

    pub(super) fn finish(self) -> (String, String) {
        (collect(self.stdout), collect(self.stderr))
    }
}

fn drain(mut pipe: impl Read + Send + 'static) -> JoinHandle<String> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).into_owned()
    })
}

fn collect(reader: Option<JoinHandle<String>>) -> String {
    reader.and_then(|r| r.join().ok()).unwrap_or_default()
}

/// Build the output of a completed child process.
pub(super) fn completed_output(status: ExitStatus, readers: OutputReaders) -> ProcessOutput {
    let (stdout, stderr) = readers.finish();
    ProcessOutput {
        command: String::new(),
        args: Vec::new(),
        stdout,
        stderr,
        exit_code: status.code(),
        duration: Duration::ZERO,
        timed_out: false,
    }
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
