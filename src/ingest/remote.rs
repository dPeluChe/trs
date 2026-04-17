//! URL / remote-repo resolution for `trs ingest`.
//!
//! Supports three inputs beyond a local path:
//!   - `https://github.com/owner/repo` (canonical URL)
//!   - `github.com/owner/repo` (host-prefixed shorthand)
//!   - `owner/repo` (assumed github.com)
//!
//! When the input is remote and spark is installed, we delegate the clone to
//! `spark clone` so the repo lands in the user's spark-managed directory and
//! future `trs ingest owner/repo` calls are free. When spark is missing (or
//! `--tmp` is passed), we shallow-clone into a tempdir that disappears with
//! the returned `ResolvedSource`.
//!
//! The resolved source always yields a plain local path — the rest of the
//! ingest pipeline stays unchanged.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

const SPARK_INSTALL_URL: &str = "https://github.com/dPeluChe/spark";

/// A usable local path for the ingest pipeline. The optional `TempDir` guard
/// keeps an ephemeral clone alive until this value is dropped.
pub struct ResolvedSource {
    pub path: PathBuf,
    _guard: Option<TempDir>,
}

/// True if `s` should be treated as a remote ref (URL or shorthand) rather
/// than a local filesystem path. Local paths always win when they exist —
/// `trs ingest owner/repo` on a directory named `owner/repo` uses the local
/// directory (matches `git clone` precedence).
pub fn is_remote_ref(s: &str) -> bool {
    if Path::new(s).exists() {
        return false;
    }
    if s.starts_with("https://") || s.starts_with("http://") || s.starts_with("git@") {
        return true;
    }
    if s.starts_with("github.com/") || s.starts_with("gitlab.com/") {
        return true;
    }
    is_owner_repo_shorthand(s)
}

/// `owner/repo` shorthand: exactly one `/`, no leading `.` or `/`, each side
/// looks like an identifier (`a-zA-Z0-9._-`). Intentionally strict — we don't
/// want to clone if the user typoed a local path.
fn is_owner_repo_shorthand(s: &str) -> bool {
    if s.starts_with('.') || s.starts_with('/') {
        return false;
    }
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(is_ident_char))
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'
}

/// Whether the user asked for an ephemeral clone explicitly.
#[derive(Clone, Copy)]
pub enum TmpMode {
    /// Default: use spark if available, fall back to ephemeral.
    Auto,
    /// `--tmp` passed: ephemeral regardless of spark.
    Force,
}

/// Resolve a remote ref to a local path usable by the rest of the pipeline.
/// Emits a one-line stderr hint describing what happened so the user knows
/// whether the clone is persistent.
pub fn resolve_remote(input: &str, tmp: TmpMode) -> Result<ResolvedSource, String> {
    let url = normalize_to_url(input)?;
    let (_, repo) = owner_repo_from(&url)?;

    let use_tmp = matches!(tmp, TmpMode::Force) || !has_spark();

    if !use_tmp {
        // Spark path: search first, clone only if not present. Avoids the
        // failure mode where `spark clone` exits 1 on an already-cloned repo,
        // and saves one subprocess when the repo is already local.
        let owner_repo = owner_repo_slash(&url)?;
        let (path, cloned) = match spark_search_first(&owner_repo) {
            Ok(p) => (p, false),
            Err(_) => {
                spark_clone(&url)?;
                (spark_search_first(&owner_repo)?, true)
            }
        };
        let verb = if cloned { "cloned via" } else { "found via" };
        eprintln!(
            "trs ingest: {} spark → {} (use `spark ingest {}` to regenerate)",
            verb,
            path.display(),
            repo,
        );
        return Ok(ResolvedSource { path, _guard: None });
    }

    // Ephemeral path: shallow clone into a tempdir that we own.
    let guard = git_shallow_clone(&url, &repo)?;
    let path = guard.path().to_path_buf();
    if matches!(tmp, TmpMode::Force) {
        eprintln!(
            "trs ingest: ephemeral clone (not saved) — to persist: `spark clone {}`",
            url
        );
    } else {
        eprintln!(
            "trs ingest: ephemeral clone — install spark to manage repos permanently: {}",
            SPARK_INSTALL_URL
        );
    }
    Ok(ResolvedSource {
        path,
        _guard: Some(guard),
    })
}

/// Normalize any accepted input to a canonical https URL.
fn normalize_to_url(input: &str) -> Result<String, String> {
    if input.starts_with("https://") || input.starts_with("http://") || input.starts_with("git@") {
        return Ok(input.to_string());
    }
    if let Some(rest) = input.strip_prefix("github.com/") {
        return Ok(format!("https://github.com/{}", rest));
    }
    if let Some(rest) = input.strip_prefix("gitlab.com/") {
        return Ok(format!("https://gitlab.com/{}", rest));
    }
    if is_owner_repo_shorthand(input) {
        return Ok(format!("https://github.com/{}", input));
    }
    Err(format!("cannot parse {} as a repo URL", input))
}

/// Extract `(owner, repo)` from a normalized URL. Strips `.git` suffix.
fn owner_repo_from(url: &str) -> Result<(String, String), String> {
    // Strip scheme + host
    let after_host = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split_once('/')
        .map(|(_, rest)| rest)
        .unwrap_or(url);
    let path = after_host.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = path.splitn(3, '/');
    let owner = parts.next().unwrap_or("").to_string();
    let repo = parts.next().unwrap_or("").to_string();
    if owner.is_empty() || repo.is_empty() {
        return Err(format!("cannot extract owner/repo from {}", url));
    }
    Ok((owner, repo))
}

/// `"owner/repo"` form for spark search queries.
fn owner_repo_slash(url: &str) -> Result<String, String> {
    let (o, r) = owner_repo_from(url)?;
    Ok(format!("{}/{}", o, r))
}

pub fn has_spark() -> bool {
    Command::new("which")
        .arg("spark")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn spark_clone(url: &str) -> Result<(), String> {
    let status = Command::new("spark")
        .args(["clone", url])
        .status()
        .map_err(|e| format!("failed to run spark clone: {}", e))?;
    if !status.success() {
        return Err(format!("spark clone exited with status {}", status));
    }
    Ok(())
}

fn spark_search_first(owner_repo: &str) -> Result<PathBuf, String> {
    let output = Command::new("spark")
        .args(["search", owner_repo, "--first"])
        .output()
        .map_err(|e| format!("failed to run spark search: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "spark search could not locate {} after clone",
            owner_repo
        ));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err(format!(
            "spark search returned empty path for {}",
            owner_repo
        ));
    }
    Ok(PathBuf::from(path))
}

fn git_shallow_clone(url: &str, repo: &str) -> Result<TempDir, String> {
    // Prefix the tempdir with a human-readable repo name so ps/lsof output is
    // obvious if an orphaned process keeps one around. `git clone URL DEST`
    // accepts an existing empty directory, so we clone into the tempdir root.
    let tmp = tempfile::Builder::new()
        .prefix(&format!("trs-{}-", repo))
        .tempdir()
        .map_err(|e| format!("cannot create tempdir: {}", e))?;
    let status = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(tmp.path())
        .status()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                "git not found — install git to use `trs ingest` with URLs".to_string()
            }
            _ => format!("failed to run git clone: {}", e),
        })?;
    if !status.success() {
        return Err(format!("git clone exited with status {}", status));
    }
    Ok(tmp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_remote_refs() {
        assert!(is_remote_ref("https://github.com/owner/repo"));
        assert!(is_remote_ref("git@github.com:owner/repo.git"));
        assert!(is_remote_ref("github.com/owner/repo"));
        assert!(is_remote_ref("owner/repo"));
        assert!(is_remote_ref("gitlab.com/owner/repo"));
    }

    #[test]
    fn rejects_local_paths() {
        assert!(!is_remote_ref("."));
        assert!(!is_remote_ref("./foo"));
        assert!(!is_remote_ref("/tmp"));
        assert!(!is_remote_ref("src/main.rs"));
        assert!(!is_remote_ref("owner/repo/extra"));
    }

    #[test]
    fn normalizes_shorthand() {
        assert_eq!(
            normalize_to_url("owner/repo").unwrap(),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            normalize_to_url("github.com/o/r").unwrap(),
            "https://github.com/o/r"
        );
        assert_eq!(
            normalize_to_url("gitlab.com/o/r").unwrap(),
            "https://gitlab.com/o/r"
        );
    }

    #[test]
    fn extracts_owner_repo() {
        assert_eq!(
            owner_repo_from("https://github.com/owner/repo").unwrap(),
            ("owner".into(), "repo".into())
        );
        assert_eq!(
            owner_repo_from("https://github.com/owner/repo.git").unwrap(),
            ("owner".into(), "repo".into())
        );
    }
}
