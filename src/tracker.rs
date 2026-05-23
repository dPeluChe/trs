//! Execution tracking module.
//!
//! Logs every trs execution to `~/.trs/history.jsonl` for token savings analytics.
//! The tracker is designed to be fire-and-forget: it must never fail or slow down
//! the main command execution.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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
///
/// Rotates the active file into a month-stamped archive
/// (`history.YYYY-MM.jsonl`) before the first append of a new month.
/// `read_history()` transparently reads active + archives so stats
/// stay correct across rotation.
fn append_history_entry(entry: &HistoryEntry) {
    let Some(dir) = dirs_path() else { return };
    if !dir.exists() && fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("history.jsonl");

    maybe_rotate_active(&path, entry.ts);

    let Ok(mut line) = serde_json::to_string(entry) else {
        return;
    };
    line.push('\n');

    let Ok(mut file) = open_user_only(&path) else {
        return;
    };
    let _ = file.write_all(line.as_bytes());
}

/// Rotate `history.jsonl` into `history.YYYY-MM.jsonl` when the
/// month-of-the-first-entry doesn't match the month of `now_ts`.
///
/// Two-tier check keeps the hot path cheap:
///   1. stat `mtime`. If mtime is in the same month as now, the file
///      was touched this month — nothing to rotate.
///   2. Only when mtime is older than the current month, slurp the
///      first valid JSONL line to read the oldest entry's timestamp
///      and use its month for the archive name.
///
/// The first rotation after upgrade can produce an archive that
/// spans multiple months (the legacy file accumulated cross-month
/// data). Subsequent rotations are clean monthly buckets.
fn maybe_rotate_active(path: &Path, now_ts: u64) {
    if !path.exists() {
        return;
    }
    let now_month = month_key_from_ts(now_ts);

    if let Ok(meta) = fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            if let Ok(d) = mtime.duration_since(UNIX_EPOCH) {
                if month_key_from_ts(d.as_secs()) == now_month {
                    return;
                }
            }
        }
    }

    let Some(first_ts) = peek_first_entry_ts(path) else {
        return;
    };
    let first_month = month_key_from_ts(first_ts);
    if first_month == now_month {
        return;
    }
    let archived = path.with_file_name(format!("history.{}.jsonl", first_month));
    // Best-effort: if rename fails, leave the file in place. Better
    // than nuking telemetry on a transient FS error.
    let _ = fs::rename(path, &archived);
}

/// Read just the first valid `HistoryEntry` line's `ts` field. Streams
/// the file rather than loading it entirely, so an 8MB history.jsonl
/// only reads ~200 bytes for the check.
fn peek_first_entry_ts(path: &Path) -> Option<u64> {
    use std::io::BufRead;
    let file = fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
            return Some(entry.ts);
        }
    }
    None
}

/// `YYYY-MM` key derived from a unix timestamp (UTC). Used both for
/// archive filenames and for cheap month-equality comparisons.
fn month_key_from_ts(ts: u64) -> String {
    use time::OffsetDateTime;
    let dt = OffsetDateTime::from_unix_timestamp(ts as i64).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    format!("{:04}-{:02}", dt.year(), u8::from(dt.month()))
}

/// Open a user-private file for append-create. On Unix, sets mode 0600 on
/// first create so a different user on the same machine can't read
/// command lines (which may carry tokens, basic-auth, API keys).
fn open_user_only(path: &Path) -> std::io::Result<fs::File> {
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
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
        cmd: redact_secrets(cmd),
        in_bytes,
        out_bytes,
        saved_pct,
        ms: duration_ms,
        cwd,
        agent,
        bypass: None,
    });
}

/// Replace common credential patterns in a command string with `[REDACTED]`
/// so they don't land in history.jsonl. Patterns covered:
///
/// - `-u user:pass` / `--user user:pass` (curl basic auth) — replace pass.
/// - `--password=...` / `-p ...` (only when followed by non-flag value).
/// - `--token=...`, `--api-key=...`, `--secret-access-key=...`.
/// - Bearer / Basic in inline `Authorization:` headers.
/// - Basic-auth in URLs: `https://user:pass@host`.
/// - Common token prefixes inline: `ghp_`, `gho_`, `ghu_`, `ghs_`, `xoxb-`,
///   `sk-` (OpenAI-shape), `AKIA` (AWS access key id).
///
/// Deliberately conservative — false positives mangle the cmd but never
/// drop signal. False negatives are acceptable (heuristic; not a vault).
pub(crate) fn redact_secrets(cmd: &str) -> String {
    use std::sync::OnceLock;
    static RULES: OnceLock<Vec<(regex::Regex, &'static str)>> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        let raw: &[(&str, &str)] = &[
            // -u user:pass (curl basic auth) — preserve user, redact pass.
            // Stops at whitespace OR quote chars so trailing `'` / `"`
            // from shell-quoted args isn't swallowed.
            (r#"((?:^|\s)-u\s+[^\s:]+:)[^\s'"]+"#, "$1[REDACTED]"),
            (r#"(--user[= ][^\s:]+:)[^\s'"]+"#, "$1[REDACTED]"),
            (
                r#"(--(?:password|token|api[-_]?key|secret[-_]?access[-_]?key|access[-_]?token|client[-_]?secret)[= ])[^\s'"]+"#,
                "$1[REDACTED]",
            ),
            (r#"(?i)(authorization:\s*(?:bearer|basic)\s+)[^\s'"]+"#, "$1[REDACTED]"),
            // Basic-auth embedded in URLs: scheme://user:pass@host.
            (r"(://[^/\s:]+:)[^@\s]+(@)", "$1[REDACTED]$2"),
            // Known token shapes by prefix — opaque secret part only.
            (r"\b(ghp_|gho_|ghu_|ghs_|ghr_)[A-Za-z0-9]{20,}\b", "${1}[REDACTED]"),
            (r"\bsk-[A-Za-z0-9_-]{20,}\b", "sk-[REDACTED]"),
            (r"\bAKIA[0-9A-Z]{16}\b", "AKIA[REDACTED]"),
            (r"\b(xox[baprs])-[A-Za-z0-9-]{10,}\b", "$1-[REDACTED]"),
        ];
        raw.iter()
            .filter_map(|(pat, repl)| regex::Regex::new(pat).ok().map(|r| (r, *repl)))
            .collect()
    });
    let mut out = cmd.to_string();
    for (re, repl) in rules.iter() {
        out = re.replace_all(&out, *repl).into_owned();
    }
    out
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

/// Read all history entries from `~/.trs/history.jsonl` PLUS any
/// monthly archives (`history.YYYY-MM.jsonl`) that share the same
/// directory. Archives come first (chronologically older) and the
/// active file comes last so the merged Vec stays chronologically
/// ordered the same way single-file history did pre-rotation.
///
/// Returns an empty Vec if the dir doesn't exist or can't be read.
/// Malformed lines are silently skipped.
pub fn read_history() -> Vec<HistoryEntry> {
    let Some(active) = history_path() else {
        return Vec::new();
    };
    let dir = match active.parent() {
        Some(p) => p.to_path_buf(),
        None => return Vec::new(),
    };

    let mut archives: Vec<PathBuf> = fs::read_dir(&dir)
        .ok()
        .map(|read| {
            read.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| is_history_archive(p))
                .collect()
        })
        .unwrap_or_default();
    // Filename sort gives chronological order because `YYYY-MM` is
    // lexicographically equivalent to date order.
    archives.sort();

    let mut entries: Vec<HistoryEntry> = Vec::new();
    for archive in archives {
        entries.extend(read_jsonl(&archive));
    }
    entries.extend(read_jsonl(&active));
    entries
}

fn read_jsonl(path: &Path) -> Vec<HistoryEntry> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<HistoryEntry>(line).ok())
        .collect()
}

/// True when `path`'s filename matches `history.YYYY-MM.jsonl` (the
/// monthly archive pattern). The legacy first-rotation archive uses
/// the same pattern so this catches it too. Active `history.jsonl` is
/// excluded — callers add it separately.
fn is_history_archive(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if name == "history.jsonl" {
        return false;
    }
    // history.YYYY-MM.jsonl → "history" + ".YYYY-MM" + ".jsonl"
    if let Some(middle) = name
        .strip_prefix("history.")
        .and_then(|s| s.strip_suffix(".jsonl"))
    {
        // YYYY-MM: 4 digits, dash, 2 digits.
        if middle.len() == 7 && middle.as_bytes()[4] == b'-' {
            let (year, month) = middle.split_at(4);
            let dash_month = &month[1..];
            return year.chars().all(|c| c.is_ascii_digit())
                && dash_month.chars().all(|c| c.is_ascii_digit());
        }
    }
    false
}

/// Delete monthly history archives older than `days` days. Returns the
/// number of files removed plus the total bytes freed. The active
/// `history.jsonl` is never touched — only archives.
///
/// "Older than N days" means: the archive's month ended more than N
/// days before today. So `--older-than 90` on May 22 2026 removes
/// archives for months ending on or before Feb 21 2026 (Feb, Jan, etc.).
pub fn prune_archives(days: u64, dry_run: bool) -> (usize, u64) {
    let Some(active) = history_path() else {
        return (0, 0);
    };
    let dir = match active.parent() {
        Some(p) => p.to_path_buf(),
        None => return (0, 0),
    };
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let cutoff_ts = now_ts.saturating_sub(days * 86_400);
    let cutoff_month = month_key_from_ts(cutoff_ts);

    let Ok(read) = fs::read_dir(&dir) else {
        return (0, 0);
    };
    let mut removed = 0usize;
    let mut freed = 0u64;
    for entry in read.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !is_history_archive(&path) {
            continue;
        }
        let Some(month_key) = path.file_name().and_then(|n| n.to_str()).and_then(|n| {
            n.strip_prefix("history.")
                .and_then(|s| s.strip_suffix(".jsonl"))
        }) else {
            continue;
        };
        // Archive month is strictly older than the cutoff month
        // (lexicographic compare works because `YYYY-MM`).
        if month_key >= cutoff_month.as_str() {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if dry_run {
            removed += 1;
            freed += size;
            continue;
        }
        if fs::remove_file(&path).is_ok() {
            removed += 1;
            freed += size;
        }
    }
    (removed, freed)
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
    fn month_key_from_ts_pads_zero() {
        // 2026-01-15 12:00:00 UTC → ts = 1768564800
        let key = month_key_from_ts(1_768_564_800);
        assert_eq!(key, "2026-01");
    }

    #[test]
    fn month_key_from_ts_handles_december() {
        // 2026-12-31 23:59:59 UTC → ts = 1798761599
        let key = month_key_from_ts(1_798_761_599);
        assert_eq!(key, "2026-12");
    }

    #[test]
    fn is_history_archive_recognizes_monthly_pattern() {
        assert!(is_history_archive(Path::new("/x/history.2026-05.jsonl")));
        assert!(is_history_archive(Path::new("history.2024-12.jsonl")));
        assert!(!is_history_archive(Path::new("history.jsonl")));
        assert!(!is_history_archive(Path::new("history.foo.jsonl")));
        assert!(!is_history_archive(Path::new("history.2026.jsonl")));
        // 8-char middle → 4+1+3 is wrong shape; reject
        assert!(!is_history_archive(Path::new("history.2026-005.jsonl")));
        // Pre-v0.5.1 dump uses a non-month suffix — must NOT be picked
        // up as an archive (we don't want stats to merge it).
        assert!(!is_history_archive(Path::new("history-pre-v0.5.1.jsonl")));
    }

    #[test]
    fn maybe_rotate_active_renames_when_month_differs() {
        let tmp = tempfile::tempdir().unwrap();
        let active = tmp.path().join("history.jsonl");
        // Write one entry timestamped 2026-01-15.
        let mut f = fs::File::create(&active).unwrap();
        let entry = HistoryEntry {
            ts: 1_768_564_800, // 2026-01-15 UTC
            cmd: "git status".into(),
            in_bytes: 100,
            out_bytes: 20,
            saved_pct: 80,
            ms: 5,
            cwd: "/tmp".into(),
            agent: None,
            bypass: None,
        };
        writeln!(f, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
        drop(f);
        // Force mtime to be much earlier than the rotation check by
        // setting it artificially old via filetime — not available in
        // std. Instead, pass a now_ts in a different month so the
        // first-line peek decides.
        let now_in_april = 1_775_385_600; // 2026-04-02 UTC
        maybe_rotate_active(&active, now_in_april);

        // Active file should be gone, replaced by the archive named
        // after the OLDEST entry's month.
        assert!(!active.exists(), "active should have been renamed");
        let archived = tmp.path().join("history.2026-01.jsonl");
        assert!(archived.exists(), "expected {:?}", archived);
    }

    #[test]
    fn maybe_rotate_active_is_noop_in_same_month() {
        let tmp = tempfile::tempdir().unwrap();
        let active = tmp.path().join("history.jsonl");
        let ts = 1_768_564_800; // 2026-01-15
        let entry = HistoryEntry {
            ts,
            cmd: "echo".into(),
            in_bytes: 1,
            out_bytes: 1,
            saved_pct: 0,
            ms: 0,
            cwd: "/".into(),
            agent: None,
            bypass: None,
        };
        let mut f = fs::File::create(&active).unwrap();
        writeln!(f, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
        drop(f);

        // Append two days later — same month, must not rotate.
        let later = ts + 2 * 86_400;
        maybe_rotate_active(&active, later);
        assert!(active.exists());
        assert!(!tmp.path().join("history.2026-01.jsonl").exists());
    }

    #[test]
    fn redact_curl_basic_auth() {
        assert_eq!(
            redact_secrets("curl -u admin:hunter2 https://api.example.com"),
            "curl -u admin:[REDACTED] https://api.example.com"
        );
    }

    #[test]
    fn redact_url_basic_auth() {
        assert_eq!(
            redact_secrets("git push https://oauth2:ghp_AAA@github.com/user/repo"),
            "git push https://oauth2:[REDACTED]@github.com/user/repo"
        );
    }

    #[test]
    fn redact_password_flag() {
        assert_eq!(
            redact_secrets("mysql --password=hunter2 -h localhost"),
            "mysql --password=[REDACTED] -h localhost"
        );
        assert_eq!(
            redact_secrets("foo --api-key=AKIA1234567890ABCDEF"),
            "foo --api-key=[REDACTED]"
        );
    }

    #[test]
    fn redact_authorization_header() {
        assert_eq!(
            redact_secrets("curl -H 'Authorization: Bearer ghp_AAAAAAAAAAAAAAAAAAAA1234' x"),
            "curl -H 'Authorization: Bearer [REDACTED]' x"
        );
    }

    #[test]
    fn redact_token_shapes() {
        let out = redact_secrets("echo ghp_AAAAAAAAAAAAAAAAAAAA1234 sk-test_AAAAAAAAAAAAAAAAAAAA");
        assert!(out.contains("ghp_[REDACTED]"));
        assert!(out.contains("sk-[REDACTED]"));
        assert!(!out.contains("ghp_AAAAAAAAAAAAAAAAAAAA1234"));
    }

    #[test]
    fn redact_leaves_normal_commands_alone() {
        let plain = "git log --oneline main..HEAD";
        assert_eq!(redact_secrets(plain), plain);
    }

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
