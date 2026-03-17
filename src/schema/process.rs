//! Process and error schema types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SchemaVersion;

// ============================================================
// Process Output Schema
// ============================================================

/// Schema for process/command output.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "process_output" },
///   "command": "echo",
///   "args": ["hello", "world"],
///   "stdout": "hello world\n",
///   "stderr": "",
///   "exit_code": 0,
///   "duration_ms": 5,
///   "timed_out": false,
///   "success": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessOutputSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// The command that was executed.
    pub command: String,
    /// Arguments passed to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Standard output.
    #[serde(default)]
    pub stdout: String,
    /// Standard error.
    #[serde(default)]
    pub stderr: String,
    /// Exit code (if captured).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Duration in milliseconds.
    #[serde(default)]
    pub duration_ms: u64,
    /// Whether the command timed out.
    #[serde(default)]
    pub timed_out: bool,
    /// Whether the command succeeded (exit code 0).
    #[serde(default)]
    pub success: bool,
}

impl ProcessOutputSchema {
    /// Create a new process output schema.
    pub fn new(command: &str) -> Self {
        Self {
            schema: SchemaVersion::new("process_output"),
            command: command.to_string(),
            args: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            duration_ms: 0,
            timed_out: false,
            success: true,
        }
    }
}

// ============================================================
// Error Schema
// ============================================================

/// Schema for error responses.
///
/// # Example JSON
///
/// ```json
/// {
///   "schema": { "version": "1.0.0", "type": "error" },
///   "error": true,
///   "message": "Command not found: foo",
///   "error_type": "command_not_found",
///   "exit_code": 127
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorSchema {
    /// Schema version information.
    pub schema: SchemaVersion,
    /// Always true for error responses.
    pub error: bool,
    /// Human-readable error message.
    pub message: String,
    /// Error type classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    /// Exit code (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Additional context.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, String>,
}

impl ErrorSchema {
    /// Create a new error schema.
    pub fn new(message: &str) -> Self {
        Self {
            schema: SchemaVersion::new("error"),
            error: true,
            message: message.to_string(),
            error_type: None,
            exit_code: None,
            context: HashMap::new(),
        }
    }

    /// Create an error with a specific type.
    pub fn with_type(message: &str, error_type: &str) -> Self {
        Self {
            schema: SchemaVersion::new("error"),
            error: true,
            message: message.to_string(),
            error_type: Some(error_type.to_string()),
            exit_code: None,
            context: HashMap::new(),
        }
    }
}
