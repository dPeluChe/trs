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
    /// Which AI agent triggered this execution, when detectable.
    /// Populated from the `TRS_AGENT` env var that `trs rewrite` and
    /// the OpenCode/Kilo plugin templates inject into the rewritten
    /// command. Stays `None` for direct-shell runs and for rules-
    /// based agents that type `trs <cmd>` voluntarily (Codex,
    /// Antigravity, Windsurf) since we have no programmatic signal
    /// to capture there. Optional field — old history lines without
    /// it still deserialize.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// `Some(true)` when this entry records a bypass observation —
    /// the caller prefixed `TRS_SKIP=1` so trs stepped aside and the
    /// shell ran the command raw. We log the *attempt* (not the
    /// output, which we never see) so `stats --by-agent` can surface
    /// which agents reach for the escape hatch and whether prompt
    /// changes are reducing it. Bypass entries carry zero
    /// in_bytes/out_bytes/ms — they don't affect savings totals,
    /// only counts. Defaults to `None`; old history lines without
    /// the field still deserialize.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass: Option<bool>,
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

/// Append a single entry to `~/.trs/history.jsonl`. Fire-and-forget:
/// every failure path silently returns so logging never affects the
/// caller. Inline append is faster than spawning a thread for one
/// small write.
fn append_history_entry(entry: &HistoryEntry) {
    let Some(dir) = dirs_path() else { return };
    if !dir.exists() && fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("history.jsonl");

    let Ok(mut line) = serde_json::to_string(entry) else {
        return;
    };
    line.push('\n');

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = file.write_all(line.as_bytes());
}

/// Log a command execution to the history file.
pub fn log_execution(cmd: &str, in_bytes: usize, out_bytes: usize, duration_ms: u64) {
    let saved_pct = if in_bytes == 0 || out_bytes >= in_bytes {
        0u8
    } else {
        (((in_bytes - out_bytes) as f64 / in_bytes as f64) * 100.0) as u8
    };

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Agent attribution: when trs rewrite or a plugin template
    // injected TRS_AGENT=<name>, the env var is live in this process.
    // An empty value is treated as absent so we don't record "".
    let agent = std::env::var("TRS_AGENT").ok().filter(|v| !v.is_empty());

    append_history_entry(&HistoryEntry {
        ts,
        cmd: cmd.to_string(),
        in_bytes,
        out_bytes,
        saved_pct,
        ms: duration_ms,
        cwd,
        agent,
        bypass: None,
    });
}

/// Log a bypass observation: the caller prefixed `TRS_SKIP=1` so the
/// hook returned without rewriting. We record the *attempt* (no
/// output to measure) so `stats --by-agent` can surface which agents
/// reach for the escape hatch and whether prompt-level changes
/// reduce it. Bypass entries carry zero in_bytes/out_bytes/ms — they
/// don't perturb savings totals, only counts.
pub fn log_bypass(cmd: &str, agent: Option<&str>) {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    append_history_entry(&HistoryEntry {
        ts,
        cmd: cmd.to_string(),
        in_bytes: 0,
        out_bytes: 0,
        saved_pct: 0,
        ms: 0,
        cwd,
        agent: agent.map(String::from),
        bypass: Some(true),
    });
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
            agent: None,
            bypass: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cmd, "git status");
        assert_eq!(parsed.saved_pct, 83);
    }

    /// Old entries (pre-bypass field) must still deserialize cleanly.
    /// Forward compatibility: `#[serde(default)]` on the field means
    /// missing values become `None` rather than failing the parse.
    #[test]
    fn test_history_entry_legacy_lines_deserialize() {
        let legacy = r#"{"ts":1,"cmd":"git status","in_bytes":100,"out_bytes":20,"saved_pct":80,"ms":5,"cwd":"/p"}"#;
        let parsed: HistoryEntry = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.bypass, None);
        assert_eq!(parsed.agent, None);
    }

    /// Bypass entries: `bypass` field present and set to true,
    /// byte-counts are zero so they don't perturb savings sums.
    #[test]
    fn test_bypass_entry_round_trip() {
        let entry = HistoryEntry {
            ts: 42,
            cmd: "TRS_SKIP=1 git status".to_string(),
            in_bytes: 0,
            out_bytes: 0,
            saved_pct: 0,
            ms: 0,
            cwd: "/p".to_string(),
            agent: Some("claude".into()),
            bypass: Some(true),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"bypass\":true"));
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bypass, Some(true));
        assert_eq!(parsed.agent.as_deref(), Some("claude"));
    }
}
