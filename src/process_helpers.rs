//! Helper functions for process execution.

use std::io::{self, Read};
use std::process::ExitStatus;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::ProcessError;

/// A signal leaves no exit code; 128+N is what a shell reports for it.
pub fn exit_code_of(status: &ExitStatus) -> i32 {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(status) {
        return 128 + signal;
    }
    status.code().unwrap_or(1)
}

/// Report a command that never started: 127 and 126, as in a shell.
pub fn exit_on_spawn_failure(cmd: &str, error: &io::Error) -> ! {
    if error.kind() == io::ErrorKind::NotFound {
        eprintln!("Command not found: {}", cmd);
        std::process::exit(127);
    }
    eprintln!("Failed to execute '{}': {}", cmd, error);
    std::process::exit(126);
}

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
        String::from_utf8(buf)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
    })
}

fn collect(reader: Option<JoinHandle<String>>) -> String {
    reader.and_then(|r| r.join().ok()).unwrap_or_default()
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
