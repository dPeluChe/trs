use std::path::Path;
use std::process::Command;

use super::{
    DigestFile, IngestConfig, IngestLevel, BYTES_PER_TOKEN, MAX_FILE_SIZE, SKIP_DIRS,
    SKIP_EXTENSIONS, SKIP_FILES,
};

use super::collect_compress::read_and_compress;

/// Collect all eligible files from the project.
pub(super) fn collect_files(config: &IngestConfig) -> Vec<DigestFile> {
    let mut builder = ignore::WalkBuilder::new(&config.root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    let mut files: Vec<DigestFile> = Vec::new();

    for entry in builder.build().flatten() {
        let path = entry.path();

        // Skip directories
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
            continue;
        }

        // Skip by extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SKIP_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                continue;
            }
        }

        // Skip by filename
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if SKIP_FILES.contains(&name) {
                continue;
            }
        }

        // Skip known generated/vendor directories
        if path.components().any(|c| {
            SKIP_DIRS
                .iter()
                .any(|d| c.as_os_str().to_str().map_or(false, |s| s == *d))
        }) {
            continue;
        }

        // Skip too large
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > MAX_FILE_SIZE {
                continue;
            }
        }

        // Get relative path
        let rel_path = path
            .strip_prefix(&config.root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Apply exclude patterns
        if config.exclude.iter().any(|ex| rel_path.contains(ex)) {
            continue;
        }

        // Read and compress file content
        let result = match read_and_compress(path, config.level) {
            Some(c) => c,
            None => continue,
        };

        let tokens = (result.content.len() as f64 / BYTES_PER_TOKEN) as usize;

        files.push(DigestFile {
            rel_path,
            content: result.content,
            tokens,
            is_changed: false,
            raw_imports: result.raw_imports,
            module_doc: result.module_doc,
            symbols: result.symbols,
        });
    }

    // Sort: README first, then changed files, then by path
    let is_readme = |p: &str| {
        let lower = p.to_lowercase();
        lower == "readme.md" || lower == "readme" || lower == "readme.txt"
    };
    files.sort_by(|a, b| {
        is_readme(&b.rel_path)
            .cmp(&is_readme(&a.rel_path))
            .then(b.is_changed.cmp(&a.is_changed))
            .then(a.rel_path.cmp(&b.rel_path))
    });

    // Deduplicate: merge identical files into one entry with combined name
    // e.g. AGENTS.md + CLAUDE.md (same content) -> "AGENTS.md & CLAUDE.md"
    let mut seen: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    let mut to_remove: Vec<usize> = Vec::new();
    for i in 0..files.len() {
        if files[i].content.is_empty() || files[i].rel_path.is_empty() {
            continue;
        }
        let hash = files[i]
            .content
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_add(b as u64).wrapping_mul(31));
        if let Some(&original_idx) = seen.get(&hash) {
            // Merge name into original (max 2 names, then count)
            let dup_name = files[i].rel_path.clone();
            let current = &files[original_idx].rel_path;
            if !current.contains(" & ") {
                files[original_idx].rel_path = format!("{} & {}", current, dup_name);
            } else if !current.contains("(+") {
                files[original_idx].rel_path = format!("{} (+1 more)", current);
            } else {
                // Increment counter
                if let Some(start) = current.rfind("(+") {
                    if let Some(end) = current.rfind(" more)") {
                        if let Ok(n) = current[start + 2..end].parse::<usize>() {
                            let base = &current[..start];
                            files[original_idx].rel_path = format!("{}(+{} more)", base, n + 1);
                        }
                    }
                }
            }
            to_remove.push(i);
        } else {
            seen.insert(hash, i);
        }
    }
    // Remove duplicates in reverse order to preserve indices
    for &i in to_remove.iter().rev() {
        files.remove(i);
    }

    files
}

/// Get list of changed files from git.
pub(super) fn get_changed_files(root: &Path, since: Option<&str>) -> Option<Vec<String>> {
    let args = if let Some(ref_spec) = since {
        vec!["diff", "--name-only", ref_spec]
    } else {
        // Uncommitted changes (staged + unstaged + untracked)
        vec!["status", "--porcelain"]
    };

    let output = Command::new("git")
        .args(&args)
        .current_dir(root)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = if since.is_some() {
        // git diff --name-only output
        stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        // git status --porcelain: "XY path" where XY = 2 status chars + 1 space
        // DO NOT trim -- the leading space is part of the format (e.g. " M src/file.rs")
        stdout
            .lines()
            .filter_map(|l| {
                if l.len() > 3 {
                    let path = &l[3..];
                    if let Some(arrow) = path.find(" -> ") {
                        Some(path[arrow + 4..].to_string())
                    } else {
                        Some(path.to_string())
                    }
                } else {
                    None
                }
            })
            .collect()
    };

    Some(files)
}

/// Apply token budget: prioritize changed files, truncate or drop large files.
pub(super) fn apply_budget(
    files: &mut Vec<DigestFile>,
    budget: usize,
    level: IngestLevel,
    root: &Path,
) {
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();

    if total_tokens <= budget {
        return; // Fits within budget
    }

    // Strategy: if over budget, re-compress large files aggressively
    if level != IngestLevel::Aggressive {
        for file in files.iter_mut() {
            if file.tokens > 500 && !file.is_changed {
                // Re-read with ABSOLUTE path from project root
                let path = root.join(&file.rel_path);
                if let Some(result) = read_and_compress(&path, IngestLevel::Aggressive) {
                    file.content = result.content;
                    file.tokens = (file.content.len() as f64 / BYTES_PER_TOKEN) as usize;
                }
            }
        }
    }

    // If still over budget, truncate from the end (least important files)
    let mut used = 0usize;
    let mut keep_count = 0;
    for file in files.iter() {
        if used + file.tokens > budget {
            break;
        }
        used += file.tokens;
        keep_count += 1;
    }

    let dropped = files.len() - keep_count;
    files.truncate(keep_count);

    if dropped > 0 {
        // Add a note about dropped files
        let note = format!("<!-- {} files omitted to fit token budget -->", dropped);
        files.push(DigestFile {
            rel_path: String::new(),
            content: note,
            tokens: 10,
            is_changed: false,
            raw_imports: vec![],
            module_doc: None,
            symbols: vec![],
        });
    }
}
