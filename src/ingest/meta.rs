//! Ingest metadata: sidecar JSON files tracking HEAD sha, timestamp, and stats.
//!
//! Each digest `<owner>/<repo>.md` gets a companion `<owner>/<repo>.json`
//! with fields used for stale detection, --since-last, and richer --list output.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct IngestMeta {
    /// Commit SHA at the time the digest was generated (short form, 7 chars)
    pub head_sha: Option<String>,
    /// Unix timestamp when the digest was written
    pub timestamp: u64,
    /// Number of files included
    pub file_count: usize,
    /// Estimated token count
    pub tokens: usize,
    /// Absolute path to the project root (for reverse lookup)
    pub project_root: String,
    /// trs version that generated this digest
    pub trs_version: String,
}

/// Get the short HEAD sha (7 chars) from a git repo, or None if not a repo.
pub(crate) fn get_head_sha(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

/// Load metadata sidecar for a digest. Returns None if missing or invalid.
pub(crate) fn load_meta(digest_path: &Path) -> Option<IngestMeta> {
    let meta_path = meta_path_for(digest_path);
    let content = std::fs::read_to_string(&meta_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save metadata sidecar next to the digest file.
pub(crate) fn save_meta(digest_path: &Path, meta: &IngestMeta) -> std::io::Result<()> {
    let meta_path = meta_path_for(digest_path);
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(meta_path, json)
}

/// Convert `<path>.md` -> `<path>.json`.
fn meta_path_for(digest_path: &Path) -> PathBuf {
    digest_path.with_extension("json")
}

/// Check if a repo's HEAD has moved since the given stored sha.
/// Returns Some(commits_ahead) if stale, None if fresh or can't compare.
pub(crate) fn commits_since(root: &Path, stored_sha: &str) -> Option<usize> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("{}..HEAD", stored_sha)])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let n_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    n_str.parse::<usize>().ok()
}

/// Current unix timestamp in seconds.
pub(crate) fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
