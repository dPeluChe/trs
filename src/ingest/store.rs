use std::path::{Path, PathBuf};
use std::process::Command;

use super::format::{format_bytes, format_modified, format_tokens};
use super::{IngestConfig, BYTES_PER_TOKEN};

/// List saved ingest digests.
pub fn list_ingests() {
    let Some(base) = ingest_store_dir() else {
        println!("No ingests found");
        println!("  storage: ~/.trs/ingest/");
        return;
    };

    if !base.exists() {
        println!("No ingests found");
        println!("  storage: ~/.trs/ingest/");
        println!("  run: trs ingest [path]");
        return;
    }

    // (owner, repo, size, modified)
    let mut entries: Vec<(String, String, u64, String)> = Vec::new();

    if let Ok(owners) = std::fs::read_dir(&base) {
        for owner_entry in owners.flatten() {
            if !owner_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                // Legacy flat files -- treat as "local" owner
                let name = owner_entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") {
                    let size = owner_entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let modified = format_modified(&owner_entry);
                    let repo = name.strip_suffix(".md").unwrap_or(&name).to_string();
                    entries.push(("local".to_string(), repo, size, modified));
                }
                continue;
            }
            let owner_name = owner_entry.file_name().to_string_lossy().to_string();
            if let Ok(files) = std::fs::read_dir(owner_entry.path()) {
                for file in files.flatten() {
                    let name = file.file_name().to_string_lossy().to_string();
                    if name.ends_with(".md") {
                        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
                        let modified = format_modified(&file);
                        let repo = name.strip_suffix(".md").unwrap_or(&name).to_string();
                        entries.push((owner_name.clone(), repo, size, modified));
                    }
                }
            }
        }
    }

    if entries.is_empty() {
        println!("No ingests found");
        println!("  run: trs ingest [path]");
        return;
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    println!("Saved digests ({}):\n", base.display());
    let mut current_owner = String::new();
    for (owner, repo, size, modified) in &entries {
        if *owner != current_owner {
            println!("{}:", owner);
            current_owner = owner.clone();
        }
        let tokens = (*size as f64 / BYTES_PER_TOKEN) as usize;
        println!(
            "  {}  ({}, {} tokens, {})",
            repo,
            format_bytes(*size as usize),
            format_tokens(tokens),
            modified
        );
    }
}

/// Read a saved digest by project name (fuzzy match across all owners).
/// If no name given, uses the current repo name.
pub fn read_digest(name: Option<&str>, project_path: &Path) {
    let Some(base) = ingest_store_dir() else {
        eprintln!("No ingests found. Run: trs ingest [path]");
        return;
    };

    let search = match name {
        Some(n) => n.to_string(),
        None => get_repo_name(project_path),
    };

    // Search across all owner dirs: owner/repo.md
    let mut matches: Vec<PathBuf> = Vec::new();

    if let Ok(owners) = std::fs::read_dir(&base) {
        for owner in owners.flatten() {
            if owner.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Ok(files) = std::fs::read_dir(owner.path()) {
                    for file in files.flatten() {
                        let name = file.file_name().to_string_lossy().to_string();
                        if name.ends_with(".md") {
                            let stem = name.strip_suffix(".md").unwrap_or(&name);
                            if stem == search || stem.contains(&search) || search.contains(stem) {
                                matches.push(file.path());
                            }
                        }
                    }
                }
            } else {
                // Legacy flat files
                let name = owner.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") {
                    let stem = name.strip_suffix(".md").unwrap_or(&name);
                    if stem == search || stem.contains(&search) || search.contains(stem) {
                        matches.push(owner.path());
                    }
                }
            }
        }
    }

    match matches.len() {
        0 => {
            eprintln!("No digest found for '{}'. Available:", search);
            list_ingests();
        }
        1 => match std::fs::read_to_string(&matches[0]) {
            Ok(content) => print!("{}", content),
            Err(e) => eprintln!("trs ingest: error reading {}: {}", matches[0].display(), e),
        },
        _ => {
            eprintln!("Multiple matches for '{}'. Be more specific:", search);
            for m in &matches {
                let display = m.strip_prefix(&base).unwrap_or(m);
                eprintln!(
                    "  trs ingest --read {}",
                    display.to_string_lossy().trim_end_matches(".md")
                );
            }
        }
    }
}

/// Get the base ingest storage directory: ~/.trs/ingest/
fn ingest_store_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".trs").join("ingest"))
}

/// Get owner/repo from git remote origin URL.
/// "git@github.com:dPeluChe/trs.git" -> ("dPeluChe", "trs")
/// "https://github.com/user/my-repo.git" -> ("user", "my-repo")
/// Fallback: parent folder name + folder name
fn get_repo_identity(root: &Path) -> (String, String) {
    let remote = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(url) = remote {
        // Parse SSH: "git@github.com:owner/repo.git"
        if let Some(path) = url.split(':').last() {
            let clean = path.trim_end_matches(".git");
            let parts: Vec<&str> = clean.split('/').collect();
            if parts.len() >= 2 {
                let owner = parts[parts.len() - 2].to_string();
                let repo = parts[parts.len() - 1].to_string();
                if !owner.is_empty() && !repo.is_empty() {
                    return (owner, repo);
                }
            }
        }
    }

    // Fallback: parent_dir/folder_name
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let repo = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    let owner = canonical
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "local".to_string());
    (owner, repo)
}

/// Get just the repo name (for backward compat with read_digest).
fn get_repo_name(root: &Path) -> String {
    let (_, repo) = get_repo_identity(root);
    repo
}

/// Save digest to ~/.trs/ingest/<owner>/<repo>.md (single file, overwrites)
/// Also cleans up old digest if repo moved to a different owner (e.g. added remote).
pub(super) fn save_to_store(content: &str, config: &IngestConfig) -> Option<String> {
    let base = ingest_store_dir()?;
    let (owner, repo) = get_repo_identity(&config.root);

    let owner_dir = base.join(&owner);
    std::fs::create_dir_all(&owner_dir).ok()?;

    let filename = format!("{}.md", repo);
    let filepath = owner_dir.join(&filename);

    // Clean up old digest under a different owner (e.g. repo got a remote)
    if let Ok(owners) = std::fs::read_dir(&base) {
        for old_owner in owners.flatten() {
            if !old_owner.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let old_owner_name = old_owner.file_name().to_string_lossy().to_string();
            if old_owner_name == owner {
                continue; // same owner, skip
            }
            let old_path = old_owner.path().join(&filename);
            if old_path.exists() {
                eprintln!(
                    "trs ingest: migrating {}/{} -> {}/{}",
                    old_owner_name, repo, owner, repo
                );
                let _ = std::fs::remove_file(&old_path);
                // Remove empty owner dir
                if std::fs::read_dir(old_owner.path())
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false)
                {
                    let _ = std::fs::remove_dir(old_owner.path());
                }
            }
        }
    }

    std::fs::write(&filepath, content).ok()?;
    Some(filepath.to_string_lossy().to_string())
}
