//! Project digest generator for LLM consumption.
//!
//! Walks a project directory, reads files with optional compression,
//! and produces a structured markdown digest optimized for AI context windows.
//!
//! Features:
//! - Budget-aware: auto-fit to token budget (e.g. --budget 128k)
//! - Git-aware: digest only changed files (--changed, --since)
//! - Compression levels: none, minimal (strip comments), aggressive (signatures only)
//! - Ollama integration: optionally format digest with a local LLM

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::router::handlers::read_filters::{
    detect_language, filter_aggressive, filter_minimal, Language,
};

/// Bytes per token estimate (GPT/Claude average).
const BYTES_PER_TOKEN: f64 = 4.0;

/// Files to always skip (binary, generated, large).
const SKIP_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "ico", "svg", "webp", "bmp",
    "woff", "woff2", "ttf", "eot", "otf",
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
    "exe", "dll", "so", "dylib", "bin",
    "pdf", "doc", "docx", "xls", "xlsx",
    "mp3", "mp4", "wav", "avi", "mov", "mkv",
    "db", "sqlite", "sqlite3",
    "pyc", "pyo", "class", "o", "obj",
    "wasm", "map",
];

/// Files to always skip by name.
const SKIP_FILES: &[&str] = &[
    "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
    "Cargo.lock", "Gemfile.lock", "poetry.lock", "composer.lock",
    ".DS_Store", "Thumbs.db",
];

/// Directories to always skip (generated/vendor content).
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", ".next", "__pycache__", ".pytest_cache",
    "dist", "build", "target", ".build", "DerivedData",
    "_generated", ".ruff_cache", ".mypy_cache", "coverage",
    ".turbo", ".nuxt", ".output", ".svelte-kit",
    "vendor", "venv", ".venv", "env",
];

/// Max file size to include (64 KB — large files are usually data, not code).
const MAX_FILE_SIZE: u64 = 64 * 1024;

/// Max size for data files like JSON (truncate with note).
const MAX_DATA_FILE_SIZE: usize = 2048;

/// Compression level for file content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestLevel {
    /// Full file content
    Full,
    /// Strip comments, normalize blanks
    Minimal,
    /// Signatures only (imports + definitions)
    Aggressive,
}

impl IngestLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "minimal" | "min" => Self::Minimal,
            "aggressive" | "agg" | "signatures" => Self::Aggressive,
            _ => Self::Full,
        }
    }
}

/// Configuration for the ingest command.
pub struct IngestConfig {
    pub root: PathBuf,
    pub level: IngestLevel,
    pub budget_tokens: Option<usize>,
    pub changed_only: bool,
    pub since: Option<String>,
    pub exclude: Vec<String>,
    pub output_file: Option<PathBuf>,
    pub ollama_model: Option<String>,
}

/// A file entry in the digest.
struct DigestFile {
    rel_path: String,
    content: String,
    tokens: usize,
    is_changed: bool,
}

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

        eprintln!("trs ingest: warning: {} is not a git repository", abs_path.display());
        Ok(abs_path)
    } else {
        Err(format!("{} is not a directory or git repository", path.display()))
    }
}

/// Run the ingest command.
pub fn run_ingest(config: &IngestConfig) {
    let start = std::time::Instant::now();

    // Collect files
    let mut files = collect_files(config);

    if files.is_empty() {
        eprintln!("trs ingest: no files found");
        return;
    }

    // Get changed files from git (if needed)
    let changed_set = if config.changed_only || config.since.is_some() {
        get_changed_files(&config.root, config.since.as_deref())
    } else {
        None
    };

    // Filter to changed files only
    if let Some(ref changed) = changed_set {
        if config.changed_only || config.since.is_some() {
            files.retain(|f| changed.contains(&f.rel_path));
        }
    }

    // Mark changed files
    if let Some(ref changed) = changed_set {
        for f in &mut files {
            f.is_changed = changed.contains(&f.rel_path);
        }
    }

    // Apply budget if specified
    if let Some(budget) = config.budget_tokens {
        apply_budget(&mut files, budget, config.level, &config.root);
    }

    // Build output
    let output = build_digest(&files, config, &changed_set, start.elapsed().as_millis() as u64);

    // Ollama post-processing
    let final_output = if let Some(ref model) = config.ollama_model {
        match ollama_format(&output, model) {
            Some(formatted) => formatted,
            None => output,
        }
    } else {
        output
    };

    // Save to ~/.trs/ingest/<repo-name>.md (single file per project, overwrites)
    let saved_path = save_to_store(&final_output, config);

    // Also write to explicit output file if requested
    if let Some(ref out_path) = config.output_file {
        if std::fs::write(out_path, &final_output).is_ok() {
            eprintln!("trs ingest: also wrote to {}", out_path.display());
        }
    }

    let tokens = (final_output.len() as f64 / BYTES_PER_TOKEN) as usize;
    if let Some(ref path) = saved_path {
        eprintln!(
            "trs ingest: {} ({} tokens) -> {}",
            format_bytes(final_output.len()),
            format_tokens(tokens),
            path
        );
    }

    // Print to stdout unless -o was specified
    if config.output_file.is_none() {
        print!("{}", final_output);
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
        1 => {
            match std::fs::read_to_string(&matches[0]) {
                Ok(content) => print!("{}", content),
                Err(e) => eprintln!("trs ingest: error reading {}: {}", matches[0].display(), e),
            }
        }
        _ => {
            eprintln!("Multiple matches for '{}'. Be more specific:", search);
            for m in &matches {
                let display = m.strip_prefix(&base).unwrap_or(m);
                eprintln!("  trs ingest --read {}", display.to_string_lossy().trim_end_matches(".md"));
            }
        }
    }
}

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
                // Legacy flat files — treat as "local" owner
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
        println!("  {}  ({}, {} tokens, {})", repo, format_bytes(*size as usize), format_tokens(tokens), modified);
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
fn save_to_store(content: &str, config: &IngestConfig) -> Option<String> {
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

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Collect all eligible files from the project.
fn collect_files(config: &IngestConfig) -> Vec<DigestFile> {
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
        let content = match read_and_compress(path, config.level) {
            Some(c) => c,
            None => continue,
        };

        let tokens = (content.len() as f64 / BYTES_PER_TOKEN) as usize;

        files.push(DigestFile {
            rel_path,
            content,
            tokens,
            is_changed: false,
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
        let hash = files[i].content.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64).wrapping_mul(31));
        if let Some(&original_idx) = seen.get(&hash) {
            // Merge name into original
            let dup_name = files[i].rel_path.clone();
            files[original_idx].rel_path = format!("{} & {}", files[original_idx].rel_path, dup_name);
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

/// Intelligently extract what matters from a file for LLM consumption.
fn read_and_compress(path: &Path, level: IngestLevel) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let lower_name = name.to_lowercase();

    // Skip: CSS, HTML, SVG, lock files, minified, type declarations
    if matches!(ext.as_str(), "css" | "scss" | "less" | "svg" | "html" | "htm") {
        return None;
    }
    if lower_name.ends_with(".lock") || lower_name.ends_with(".min.js")
        || lower_name.ends_with(".min.css") || lower_name.ends_with(".d.ts") {
        return None;
    }

    // Key docs: full content, strip HTML noise
    if matches!(lower_name.as_str(),
        "readme.md" | "claude.md" | "agents.md" | "contributing.md" | "changelog.md"
    ) {
        return Some(strip_html_from_markdown(&content));
    }

    // package.json: compress to name + deps names only
    if lower_name == "package.json" {
        return Some(compress_package_json(&content));
    }

    // JSON/YAML: extract schema (keys + 1 sample), not all data
    let lang = detect_language(&path.to_path_buf());
    if lang == Language::Data {
        return Some(extract_data_schema(&content, &ext));
    }

    // Source code: always extract signatures in minimal/aggressive mode
    // An agent needs to know WHAT exists, not HOW it's implemented
    match level {
        IngestLevel::Full => Some(content),
        _ => Some(extract_signatures(&content, &ext)),
    }
}

/// Extract JSON schema: show structure with 1 sample, count arrays.
fn extract_data_schema(content: &str, ext: &str) -> String {
    if ext == "json" || ext == "jsonl" {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            return summarize_json_value(&val, 0);
        }
    }
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= 20 {
        return content.to_string();
    }
    format!("{}\n... ({} lines)", lines[..20].join("\n"), lines.len())
}

fn summarize_json_value(val: &serde_json::Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    match val {
        serde_json::Value::Array(arr) if arr.is_empty() => "[]".into(),
        serde_json::Value::Array(arr) => {
            let first = summarize_json_value(&arr[0], depth + 1);
            format!("[{}, ...] ({} items)", first, arr.len())
        }
        serde_json::Value::Object(map) if map.is_empty() => "{}".into(),
        serde_json::Value::Object(map) => {
            let mut out = String::from("{\n");
            for (key, val) in map.iter() {
                let v = match val {
                    serde_json::Value::String(s) if s.len() > 40 => format!("\"{}...\"", &s[..37]),
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".into(),
                    serde_json::Value::Array(a) if a.is_empty() => "[]".into(),
                    serde_json::Value::Array(a) => {
                        let inner = summarize_json_value(&a[0], depth + 2);
                        format!("[{}, ...] ({} items)", inner, a.len())
                    }
                    serde_json::Value::Object(_) => summarize_json_value(val, depth + 1),
                };
                out.push_str(&format!("{}  \"{}\": {},\n", indent, key, v));
            }
            out.push_str(&format!("{}}}", indent));
            out
        }
        serde_json::Value::String(s) if s.len() > 40 => format!("\"{}...\"", &s[..37]),
        other => other.to_string(),
    }
}

/// Extract function/class signatures from source code — names without bodies.
fn extract_signatures(content: &str, ext: &str) -> String {
    let mut result = String::new();

    for line in content.lines() {
        let t = line.trim();
        // Keep imports
        if t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") {
            result.push_str(t);
            result.push('\n');
            continue;
        }
        // Keep signatures by language
        let keep = match ext {
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" | "vue" | "svelte" =>
                t.starts_with("export ") || t.starts_with("function ") ||
                t.starts_with("class ") || t.starts_with("interface ") ||
                t.starts_with("type ") || t.starts_with("enum ") ||
                t.starts_with("const ") && (t.contains("= mutation(") || t.contains("= query(") ||
                    t.contains("= action(") || t.contains("= internalMutation(") ||
                    t.contains("=> {") || t.contains("= defineTable(")),
            "rs" =>
                t.starts_with("pub ") || t.starts_with("fn ") || t.starts_with("struct ") ||
                t.starts_with("enum ") || t.starts_with("trait ") || t.starts_with("impl ") ||
                t.starts_with("mod ") || t.starts_with("type "),
            "py" | "pyi" =>
                t.starts_with("def ") || t.starts_with("class ") || t.starts_with("async def "),
            "go" =>
                t.starts_with("func ") || t.starts_with("type ") || t.starts_with("var ") || t.starts_with("const "),
            _ =>
                t.starts_with("export ") || t.starts_with("pub ") || t.starts_with("fn ") ||
                t.starts_with("def ") || t.starts_with("class ") || t.starts_with("function "),
        };
        if keep {
            let cleaned = clean_signature(t);
            if cleaned.len() > 120 {
                result.push_str(&cleaned[..117]);
                result.push_str("...\n");
            } else {
                result.push_str(&cleaned);
                result.push('\n');
            }
        }
    }

    if result.is_empty() {
        let lines: Vec<&str> = content.lines().collect();
        lines.iter().take(10).for_each(|l| { result.push_str(l); result.push('\n'); });
        if lines.len() > 10 { result.push_str(&format!("... ({} lines)\n", lines.len())); }
    }
    result
}

/// Strip trailing noise from a signature line.
/// `export function foo(): string {` -> `export function foo(): string`
/// `export const POINTS = {` -> `export const POINTS`
/// `const handleAnswer = useCallback((index: number) => {` -> `const handleAnswer = useCallback((index: number))`
/// `def merge_blocks(prefix, count, output_file):` -> `def merge_blocks(prefix, count, output_file)`
fn clean_signature(line: &str) -> String {
    let mut s = line.to_string();

    // Strip trailing { and whitespace
    s = s.trim_end().to_string();
    while s.ends_with('{') || s.ends_with("=> {") {
        if s.ends_with("=> {") {
            s = s[..s.len() - 4].trim_end().to_string();
            // Close the paren if we stripped => {
            if !s.ends_with(')') { s.push(')'); }
        } else {
            s = s[..s.len() - 1].trim_end().to_string();
        }
    }

    // Strip trailing = [ or = {
    if s.ends_with("= [") || s.ends_with("= {") {
        s = s[..s.len() - 3].trim_end().to_string();
    }

    // Strip trailing = for const assignments
    if s.ends_with('=') {
        s = s[..s.len() - 1].trim_end().to_string();
    }

    // Strip trailing : (Python)
    if s.ends_with(':') {
        s = s[..s.len() - 1].trim_end().to_string();
    }

    // Strip trailing ;
    if s.ends_with(';') {
        s = s[..s.len() - 1].trim_end().to_string();
    }

    s
}

/// Compress package.json: name, version, scripts names, dep names.
fn compress_package_json(content: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        let mut out = String::new();
        if let Some(n) = val.get("name").and_then(|v| v.as_str()) { out.push_str(&format!("name: {}\n", n)); }
        if let Some(v) = val.get("version").and_then(|v| v.as_str()) { out.push_str(&format!("version: {}\n", v)); }
        if let Some(s) = val.get("scripts").and_then(|v| v.as_object()) {
            out.push_str(&format!("scripts: {}\n", s.keys().cloned().collect::<Vec<_>>().join(", ")));
        }
        for key in &["dependencies", "devDependencies"] {
            if let Some(deps) = val.get(*key).and_then(|v| v.as_object()) {
                let names: Vec<&str> = deps.keys().map(|k| k.as_str()).collect();
                out.push_str(&format!("{}: {}\n", key, names.join(", ")));
            }
        }
        return out;
    }
    content.to_string()
}

/// Get list of changed files from git.
fn get_changed_files(root: &Path, since: Option<&str>) -> Option<Vec<String>> {
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
        stdout.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
    } else {
        // git status --porcelain: "XY path" where XY = 2 status chars + 1 space
        // DO NOT trim — the leading space is part of the format (e.g. " M src/file.rs")
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
fn apply_budget(files: &mut Vec<DigestFile>, budget: usize, level: IngestLevel, root: &Path) {
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
                if let Some(compressed) = read_and_compress(&path, IngestLevel::Aggressive) {
                    file.content = compressed;
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
        });
    }
}

/// Build the final markdown digest.
fn build_digest(
    files: &[DigestFile],
    config: &IngestConfig,
    _changed_set: &Option<Vec<String>>,
    elapsed_ms: u64,
) -> String {
    let mut out = String::new();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_files = files.iter().filter(|f| !f.rel_path.is_empty()).count();

    let project_name = config
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // Header: one compact line
    out.push_str(&format!("# {} ({} files, {} tokens)\n\n", project_name, total_files, format_tokens(total_tokens)));

    // Structure: inline tree grouped by directory
    out.push_str("## Structure\n\n");
    out.push_str(&build_tree(files));
    out.push('\n');

    // Group files by role for the content section
    let mut docs: Vec<&DigestFile> = Vec::new();
    let mut api: Vec<&DigestFile> = Vec::new();
    let mut pages: Vec<&DigestFile> = Vec::new();
    let mut data: Vec<&DigestFile> = Vec::new();
    let mut config_files: Vec<&DigestFile> = Vec::new();
    let mut other: Vec<&DigestFile> = Vec::new();

    for file in files {
        if file.rel_path.is_empty() {
            continue;
        }
        let lower = file.rel_path.to_lowercase();
        let ext = Path::new(&file.rel_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if lower.ends_with("readme.md") || lower.ends_with("claude.md")
            || lower.ends_with("agents.md") || lower.contains("todo")
            || lower.ends_with("contributing.md") || lower.ends_with("changelog.md") {
            docs.push(file);
        } else if lower.contains("convex/") || lower.contains("api/")
            || lower.contains("server/") || lower.contains("backend/")
            || lower.contains("routes/") {
            api.push(file);
        } else if lower.contains("pages/") || lower.contains("views/")
            || lower.contains("screens/") || lower.contains("components/") {
            pages.push(file);
        } else if ext == "json" || ext == "yaml" || ext == "yml" || ext == "csv"
            || lower.contains("data/") || lower.contains("fixtures/") {
            data.push(file);
        } else if lower.contains("config") || lower.ends_with(".toml")
            || lower.ends_with(".env") || lower.ends_with(".env.example")
            || lower.ends_with(".env.local.example") || lower == ".gitignore"
            || lower == "tsconfig.json" || lower == "tsconfig.app.json"
            || lower == "tsconfig.node.json" || lower.ends_with("vite.config.ts") {
            config_files.push(file);
        } else {
            other.push(file);
        }
    }

    // Docs section — full content (already HTML-stripped)
    if !docs.is_empty() {
        for file in &docs {
            out.push_str(&format!("## {}\n\n{}\n\n", file.rel_path, file.content.trim()));
        }
    }

    // API section — signatures only, compact
    if !api.is_empty() {
        out.push_str("## API\n\n");
        for file in &api {
            let content = file.content.trim();
            if content == "(same as AGENTS.md)" || content == "(same as README.md)" || content.is_empty() {
                continue;
            }
            out.push_str(&format!("**{}**\n{}\n\n", file.rel_path, content));
        }
    }

    // Pages/Components — just names and key info
    if !pages.is_empty() {
        out.push_str("## Pages & Components\n\n");
        for file in &pages {
            let content = file.content.trim();
            if content.is_empty() {
                out.push_str(&format!("**{}**\n\n", file.rel_path));
            } else {
                out.push_str(&format!("**{}**\n{}\n\n", file.rel_path, content));
            }
        }
    }

    // Data — schema summaries
    if !data.is_empty() {
        out.push_str("## Data\n\n");
        for file in &data {
            out.push_str(&format!("**{}**\n{}\n\n", file.rel_path, file.content.trim()));
        }
    }

    // Config — compact, only if meaningful
    if !config_files.is_empty() {
        out.push_str("## Config\n\n");
        for file in &config_files {
            let content = file.content.trim();
            // Skip very small or empty configs
            if content.len() < 10 {
                continue;
            }
            out.push_str(&format!("**{}**: {}\n", file.rel_path, summarize_config(content)));
        }
        out.push('\n');
    }

    // Other files
    if !other.is_empty() {
        out.push_str("## Other\n\n");
        for file in &other {
            let content = file.content.trim();
            if content.is_empty() || content.len() < 10 {
                continue;
            }
            if content.starts_with("(same as") {
                out.push_str(&format!("**{}**: {}\n", file.rel_path, content));
            } else {
                out.push_str(&format!("**{}**\n{}\n\n", file.rel_path, content));
            }
        }
    }

    // Footer
    out.push_str(&format!(
        "---\n*trs ingest v{} | {}ms | {}*\n",
        env!("CARGO_PKG_VERSION"),
        elapsed_ms,
        format_bytes(out.len())
    ));

    out
}

/// Summarize a config file to one line.
fn summarize_config(content: &str) -> String {
    let lines: Vec<&str> = content.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && *l != "{" && *l != "}" && *l != "[" && *l != "]")
        .collect();
    if lines.len() <= 3 {
        return lines.join(" | ");
    }
    format!("{} lines", lines.len())
}

/// Build readable tree: directories separated, files wrapped at ~70 chars.
fn build_tree(files: &[DigestFile]) -> String {
    let mut dirs: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for file in files {
        if file.rel_path.is_empty() { continue; }
        let path = Path::new(&file.rel_path);
        let parent = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        dirs.entry(parent).or_default().push(name);
    }

    let mut tree = String::new();
    for (dir, filenames) in &dirs {
        // Directory header
        if dir.is_empty() {
            tree.push_str("/\n");
        } else {
            tree.push_str(&format!("{}/\n", dir));
        }
        // Wrap filenames at ~70 chars with indent
        let mut line = String::from("  ");
        for name in filenames {
            if line.len() + name.len() + 2 > 72 && line.len() > 2 {
                tree.push_str(&line);
                tree.push('\n');
                line = String::from("  ");
            }
            line.push_str(name);
            line.push_str("  ");
        }
        if line.len() > 2 {
            tree.push_str(line.trim_end());
            tree.push('\n');
        }
        tree.push('\n');
    }
    tree
}

/// List available Ollama models.
pub fn list_ollama_models() {
    match get_ollama_models() {
        Some(models) if !models.is_empty() => {
            eprintln!("Ollama models available:");
            for (name, size, family) in &models {
                eprintln!("  {} ({}, {})", name, size, family);
            }
        }
        _ => {
            eprintln!("Ollama not running at localhost:11434");
            eprintln!("Start with: ollama serve");
        }
    }
}

/// Get list of Ollama models: (name, size_display, family).
fn get_ollama_models() -> Option<Vec<(String, String, String)>> {
    let output = Command::new("curl")
        .args(["-s", "http://localhost:11434/api/tags"])
        .output()
        .ok()?;

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let models = response.get("models")?.as_array()?;

    let mut result: Vec<(String, String, String)> = Vec::new();
    for model in models {
        let name = model.get("name")?.as_str()?.to_string();
        let size = model.get("details")
            .and_then(|d| d.get("parameter_size"))
            .and_then(|s| s.as_str())
            .unwrap_or("?")
            .to_string();
        let family = model.get("details")
            .and_then(|d| d.get("family"))
            .and_then(|s| s.as_str())
            .unwrap_or("?")
            .to_string();
        // Skip embedding models
        if name.contains("embed") || name.contains("nomic") {
            continue;
        }
        result.push((name, size, family));
    }

    Some(result)
}

/// Pick the best available local model (prefer larger, local over cloud).
fn pick_default_model() -> Option<String> {
    let models = get_ollama_models()?;
    // Prefer local models (no :cloud suffix) with largest param count
    let local: Vec<&(String, String, String)> = models
        .iter()
        .filter(|(name, _, _)| !name.contains(":cloud"))
        .collect();

    if let Some(best) = local.first() {
        return Some(best.0.clone());
    }
    // Fallback to any model
    models.first().map(|(name, _, _)| name.clone())
}

/// Send digest to Ollama for LLM-formatted summary.
fn ollama_format(digest: &str, model: &str) -> Option<String> {
    // Resolve model: "auto" picks the best available
    let model = if model == "auto" {
        match pick_default_model() {
            Some(m) => {
                eprintln!("trs ingest: using Ollama model: {}", m);
                m
            }
            None => {
                eprintln!("trs ingest: no Ollama models found. Install one with: ollama pull llama3.1");
                return None;
            }
        }
    } else {
        model.to_string()
    };

    // Verify Ollama is running and model exists
    let models = get_ollama_models();
    if models.is_none() {
        eprintln!("trs ingest: Ollama not running at localhost:11434");
        eprintln!("  Start with: ollama serve");
        return None;
    }

    let model_exists = models
        .as_ref()
        .map(|m| m.iter().any(|(n, _, _)| n == &model || n.starts_with(&format!("{}:", model))))
        .unwrap_or(false);

    if !model_exists {
        eprintln!("trs ingest: model '{}' not found in Ollama", model);
        list_ollama_models();
        return None;
    }

    // Extract README from digest if present (for better Ollama context)
    let readme_content = extract_section(digest, "README.md")
        .or_else(|| extract_section(digest, "readme.md"))
        .unwrap_or_default();

    // Truncate digest to fit model context (conservative 24k chars)
    let max_chars = 24_000;
    let input = if digest.len() > max_chars {
        eprintln!("trs ingest: digest truncated to {} for Ollama context", format_bytes(max_chars));
        // Put README first, then as much of the rest as fits
        let readme_section = if !readme_content.is_empty() {
            format!("## README.md\n\n{}\n\n---\n\n", readme_content)
        } else {
            String::new()
        };
        let remaining = max_chars.saturating_sub(readme_section.len());
        let mut combined = readme_section;
        // Find safe char boundary
        let mut cut = remaining.min(digest.len());
        while cut > 0 && !digest.is_char_boundary(cut) { cut -= 1; }
        combined.push_str(&digest[..cut]);
        combined
    } else {
        digest.to_string()
    };
    let input = &input;

    eprintln!("trs ingest: generating summary with {} ({})...", model, format_bytes(input.len()));

    let prompt = format!(
        "Analyze this codebase digest and produce a structured markdown summary.\n\n\
         Output EXACTLY this format:\n\n\
         ## Overview\n\
         [1-2 sentences: what this project does]\n\n\
         ## Tech Stack\n\
         [bullet list: language, framework, database, key dependencies]\n\n\
         ## Architecture\n\
         [bullet list: key directories/modules and their responsibility]\n\n\
         ## Key Files\n\
         [bullet list: 5-10 most important files and what they do]\n\n\
         ## Entry Points\n\
         [bullet list: main entry files, API routes, CLI commands]\n\n\
         Be concise and factual. No explanations of what markdown is. \
         No filler. Just the structured summary.\n\n---\n\n{}",
        input
    );

    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "num_predict": 2048,
        }
    });

    let output = Command::new("curl")
        .args([
            "-s",
            "-X", "POST",
            "http://localhost:11434/api/generate",
            "-H", "Content-Type: application/json",
            "-d", &payload.to_string(),
        ])
        .output()
        .ok()?;

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let formatted = response.get("response")?.as_str()?;

    let duration = response.get("total_duration")
        .and_then(|d| d.as_u64())
        .map(|ns| ns / 1_000_000_000)
        .unwrap_or(0);

    eprintln!("trs ingest: Ollama completed in {}s", duration);

    // Combine: LLM summary at top, then raw digest
    let mut result = String::new();
    result.push_str(&format!("# Project Summary\n\n> Generated by {} via Ollama\n\n", model));
    result.push_str(formatted);
    result.push_str("\n\n---\n\n# Full Digest\n\n");
    result.push_str(digest);

    Some(result)
}

fn format_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{}B", n)
    } else if n < 1024 * 1024 {
        format!("{:.1}KB", n as f64 / 1024.0)
    } else {
        format!("{:.1}MB", n as f64 / (1024.0 * 1024.0))
    }
}

/// Strip HTML tags from markdown content (badges, images, formatting).
/// Preserves markdown content, removes <p>, <a>, <img>, etc.
fn strip_html_from_markdown(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut prev_blank = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Strip code fence markers (```bash, ```json, etc.) — just noise in a digest
        if trimmed.starts_with("```") {
            continue;
        }

        // Skip lines that are pure HTML (start with < and end with > or are self-closing)
        if trimmed.starts_with('<') && (trimmed.ends_with('>') || trimmed.ends_with("/>")) {
            // Keep HTML comments (<!-- -->)
            if trimmed.starts_with("<!--") {
                result.push_str(line);
                result.push('\n');
                prev_blank = false;
            }
            // Skip everything else (badges, images, alignment tags)
            continue;
        }

        // Skip img tags inline
        if trimmed.contains("<img ") && trimmed.contains("src=") {
            continue;
        }

        // Collapse consecutive blank lines
        if trimmed.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
            continue;
        }

        prev_blank = false;
        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Extract a file's content from the digest by filename.
fn extract_section(digest: &str, filename: &str) -> Option<String> {
    let marker = format!("<!-- file: {} -->", filename);
    let start = digest.find(&marker)?;
    // Find the code block after the marker
    let after_marker = &digest[start..];
    let code_start = after_marker.find("```")? + 3;
    let code_content_start = after_marker[code_start..].find('\n')? + code_start + 1;
    let code_end = after_marker[code_content_start..].find("```")? + code_content_start;
    Some(after_marker[code_content_start..code_end].to_string())
}

fn format_modified(entry: &std::fs::DirEntry) -> String {
    let modified = entry.metadata().ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(modified);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        let (y, mo, day) = days_to_date(modified / 86400);
        format!("{:04}-{:02}-{:02}", y, mo, day)
    }
}

fn format_tokens(n: usize) -> String {
    if n < 1000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1500), "1.5k");
        assert_eq!(format_tokens(128000), "128.0k");
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500B");
        assert_eq!(format_bytes(1536), "1.5KB");
        assert_eq!(format_bytes(1_048_576), "1.0MB");
    }

    #[test]
    fn test_ingest_level_from_str() {
        assert_eq!(IngestLevel::from_str("minimal"), IngestLevel::Minimal);
        assert_eq!(IngestLevel::from_str("min"), IngestLevel::Minimal);
        assert_eq!(IngestLevel::from_str("aggressive"), IngestLevel::Aggressive);
        assert_eq!(IngestLevel::from_str("agg"), IngestLevel::Aggressive);
        assert_eq!(IngestLevel::from_str("full"), IngestLevel::Full);
        assert_eq!(IngestLevel::from_str("anything"), IngestLevel::Full);
    }

    #[test]
    fn test_skip_extensions() {
        assert!(SKIP_EXTENSIONS.contains(&"png"));
        assert!(SKIP_EXTENSIONS.contains(&"wasm"));
        assert!(!SKIP_EXTENSIONS.contains(&"rs"));
        assert!(!SKIP_EXTENSIONS.contains(&"ts"));
    }

    #[test]
    fn test_skip_files() {
        assert!(SKIP_FILES.contains(&"package-lock.json"));
        assert!(SKIP_FILES.contains(&"Cargo.lock"));
        assert!(!SKIP_FILES.contains(&"Cargo.toml"));
    }

    #[test]
    fn test_build_tree() {
        let files = vec![
            DigestFile { rel_path: "src/main.rs".into(), content: String::new(), tokens: 0, is_changed: false },
            DigestFile { rel_path: "src/lib.rs".into(), content: String::new(), tokens: 0, is_changed: false },
            DigestFile { rel_path: "README.md".into(), content: String::new(), tokens: 0, is_changed: false },
        ];
        let tree = build_tree(&files);
        assert!(tree.contains("src/"));
        assert!(tree.contains("  main.rs"));
        assert!(tree.contains("README.md"));
    }
}
