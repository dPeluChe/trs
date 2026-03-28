//! Execution tracking module.
//!
//! Logs every trs execution to `~/.trs/history.jsonl` for token savings analytics.
//! The tracker is designed to be fire-and-forget: it must never fail or slow down
//! the main command execution.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single history entry representing one trs command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unix timestamp of execution.
    pub ts: u64,
    /// The command that was executed (e.g. "git status").
    pub cmd: String,
    /// Input size in bytes (raw command output).
    pub in_bytes: usize,
    /// Output size in bytes (after trs processing).
    pub out_bytes: usize,
    /// Percentage of bytes saved.
    pub saved_pct: u8,
    /// Execution duration in milliseconds.
    pub ms: u64,
    /// Working directory where the command was run.
    pub cwd: String,
}

/// Returns the path to the history file: `~/.trs/history.jsonl`.
fn history_path() -> Option<PathBuf> {
    dirs_path().map(|d| d.join("history.jsonl"))
}

/// Returns the path to the trs data directory: `~/.trs/`.
fn dirs_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".trs"))
}

/// Cross-platform home directory lookup.
pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Log a command execution to the history file.
///
/// This function is fire-and-forget: if anything fails (directory creation,
/// file open, serialization, write), it silently returns without affecting
/// the main command flow.
pub fn log_execution(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64) {
    // Silently bail if we can't determine paths
    let Some(dir) = dirs_path() else { return };
    let Some(path) = history_path() else { return };

    // Create directory if needed
    if !dir.exists() {
        if fs::create_dir_all(&dir).is_err() {
            return;
        }
    }

    // Calculate saved percentage
    let saved_pct = if in_bytes == 0 {
        0u8
    } else if out_bytes >= in_bytes {
        0u8
    } else {
        (((in_bytes - out_bytes) as f64 / in_bytes as f64) * 100.0) as u8
    };

    // Get current working directory
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Get current timestamp
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entry = HistoryEntry {
        ts,
        cmd: cmd.to_string(),
        in_bytes,
        out_bytes,
        saved_pct,
        ms: duration_ms,
        cwd,
    };

    // Serialize and append
    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let _ = file.write_all(line.as_bytes());
}

/// Read all history entries from `~/.trs/history.jsonl`.
///
/// Returns an empty Vec if the file doesn't exist or can't be read.
/// Malformed lines are silently skipped.
pub fn read_history() -> Vec<HistoryEntry> {
    let Some(path) = history_path() else {
        return Vec::new();
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<HistoryEntry>(line).ok())
        .collect()
}

/// Read history entries filtered to the current working directory.
///
/// Returns an empty Vec if cwd can't be determined or the file doesn't exist.
pub fn read_project_history() -> Vec<HistoryEntry> {
    let cwd = match std::env::current_dir() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => return Vec::new(),
    };

    read_history()
        .into_iter()
        .filter(|e| e.cwd == cwd)
        .collect()
}

/// Format a byte count into a human-readable string (e.g. "12.4K", "1.2M").
pub fn format_bytes_human(bytes: usize) -> String {
    if bytes < 1000 {
        format!("{}", bytes)
    } else if bytes < 1_000_000 {
        format!("{:.1}K", bytes as f64 / 1000.0)
    } else {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_human() {
        assert_eq!(format_bytes_human(0), "0");
        assert_eq!(format_bytes_human(500), "500");
        assert_eq!(format_bytes_human(1000), "1.0K");
        assert_eq!(format_bytes_human(12400), "12.4K");
        assert_eq!(format_bytes_human(1_500_000), "1.5M");
    }

    #[test]
    fn test_saved_pct_calculation() {
        // 0 input -> 0%
        let in_b = 0usize;
        let out_b = 0usize;
        let pct = if in_b == 0 {
            0u8
        } else {
            (((in_b - out_b) as f64 / in_b as f64) * 100.0) as u8
        };
        assert_eq!(pct, 0);

        // 100 input, 20 output -> 80%
        let in_b = 100usize;
        let out_b = 20usize;
        let pct = (((in_b - out_b) as f64 / in_b as f64) * 100.0) as u8;
        assert_eq!(pct, 80);
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry {
            ts: 1773771663,
            cmd: "git status".to_string(),
            in_bytes: 497,
            out_bytes: 81,
            saved_pct: 83,
            ms: 12,
            cwd: "/path/to/project".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cmd, "git status");
        assert_eq!(parsed.saved_pct, 83);
    }
}
