//! Project-root resolution and budget suggestion for `trs ingest`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the project root: find git root or use the given path.
pub fn resolve_project_root(path: &Path) -> Result<PathBuf, String> {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cannot get current dir: {}", e))?
            .join(path)
    };

    // If path is "." or doesn't exist as-is, try to find git root
    let check_path = if abs_path.to_str() == Some(".") || path.to_str() == Some(".") {
        std::env::current_dir().unwrap_or(abs_path.clone())
    } else {
        abs_path.clone()
    };

    // Try to find git root from the given path
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&check_path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    if let Some(root) = git_root {
        let root_path = PathBuf::from(&root);

        // Even if this is a git repo, check if it contains many sub-repos
        // (common pattern: workspace directory with .git tracking many projects)
        let sub_repos: Vec<String> = std::fs::read_dir(&root_path)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .filter(|e| e.path().join(".git").exists())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();

        if sub_repos.len() > 5 {
            let mut msg = format!(
                "{} contains {} sub-repositories. Specify one:\n",
                root_path.display(),
                sub_repos.len()
            );
            for repo in sub_repos.iter().take(10) {
                msg.push_str(&format!("  trs ingest {}/{}\n", root_path.display(), repo));
            }
            if sub_repos.len() > 10 {
                msg.push_str(&format!("  ... and {} more\n", sub_repos.len() - 10));
            }
            return Err(msg);
        }

        Ok(root_path)
    } else if abs_path.is_dir() {
        // Check if this is a folder containing multiple repos
        let sub_repos: Vec<String> = std::fs::read_dir(&abs_path)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().join(".git").exists())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();

        if sub_repos.len() > 1 {
            let mut msg = format!(
                "{} contains {} repositories. Specify one:\n",
                abs_path.display(),
                sub_repos.len()
            );
            for repo in &sub_repos {
                msg.push_str(&format!("  trs ingest {}/{}\n", path.display(), repo));
            }
            return Err(msg);
        }

        eprintln!(
            "trs ingest: warning: {} is not a git repository",
            abs_path.display()
        );
        Ok(abs_path)
    } else {
        Err(format!(
            "{} is not a directory or git repository",
            path.display()
        ))
    }
}

/// Return a human-friendly budget suggestion for a digest of `n` tokens.
/// Picks a round budget that roughly halves the current output — enough
/// compression pressure to matter, but not so aggressive it empties the digest.
pub(super) fn suggest_budget(n: usize) -> &'static str {
    if n > 200_000 {
        "128k"
    } else if n > 80_000 {
        "64k"
    } else if n > 40_000 {
        "32k"
    } else {
        "16k"
    }
}
