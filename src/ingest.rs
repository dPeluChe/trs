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
        apply_budget(&mut files, budget, config.level);
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

/// Read a saved digest by project name (fuzzy match).
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

    // Try exact match first
    let exact = base.join(format!("{}.md", search));
    if exact.exists() {
        match std::fs::read_to_string(&exact) {
            Ok(content) => print!("{}", content),
            Err(e) => eprintln!("trs ingest: error reading {}: {}", exact.display(), e),
        }
        return;
    }

    // Fuzzy match: find digests containing the search term
    let mut matches: Vec<String> = Vec::new();
    if let Ok(files) = std::fs::read_dir(&base) {
        for file in files.flatten() {
            let name = file.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let stem = name.strip_suffix(".md").unwrap_or(&name);
                if stem.contains(&search) || search.contains(stem) {
                    matches.push(name);
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
            let path = base.join(&matches[0]);
            match std::fs::read_to_string(&path) {
                Ok(content) => print!("{}", content),
                Err(e) => eprintln!("trs ingest: error reading {}: {}", path.display(), e),
            }
        }
        _ => {
            eprintln!("Multiple matches for '{}'. Be more specific:", search);
            for m in &matches {
                eprintln!("  trs ingest --read {}", m.strip_suffix(".md").unwrap_or(m));
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

    let mut entries: Vec<(String, u64, String)> = Vec::new(); // (name, size, modified)

    if let Ok(files) = std::fs::read_dir(&base) {
        for file in files.flatten() {
            let name = file.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let size = file.metadata().map(|m| m.len()).unwrap_or(0);
                let modified = file.metadata().ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        let secs = d.as_secs();
                        let (y, m, day) = days_to_date(secs / 86400);
                        let h = (secs % 86400) / 3600;
                        let min = (secs % 3600) / 60;
                        format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, day, h, min)
                    })
                    .unwrap_or_default();
                entries.push((name, size, modified));
            }
        }
    }

    if entries.is_empty() {
        println!("No ingests found");
        println!("  run: trs ingest [path]");
        return;
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    println!("Saved digests ({}):", base.display());
    println!();
    for (name, size, modified) in &entries {
        let tokens = (*size as f64 / BYTES_PER_TOKEN) as usize;
        let display_name = name.strip_suffix(".md").unwrap_or(name);
        println!("  {}  ({}, {} tokens, {})", display_name, format_bytes(*size as usize), format_tokens(tokens), modified);
    }
}

/// Get the base ingest storage directory: ~/.trs/ingest/
fn ingest_store_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".trs").join("ingest"))
}

/// Get the repo name from git remote origin URL, or fallback to folder name.
/// Examples: "git@github.com:user/my-repo.git" -> "my-repo"
///           "https://github.com/user/my-repo.git" -> "my-repo"
fn get_repo_name(root: &Path) -> String {
    // Try git remote
    let remote = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(url) = remote {
        // Parse: "git@github.com:user/repo.git" or "https://github.com/.../repo.git"
        let name = url
            .rsplit('/')
            .next()
            .or_else(|| url.rsplit(':').next())
            .unwrap_or(&url)
            .trim_end_matches(".git")
            .to_string();
        if !name.is_empty() {
            return name;
        }
    }

    // Fallback to folder name
    root.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string())
}

/// Save digest to ~/.trs/ingest/<repo-name>.md (single file, overwrites)
fn save_to_store(content: &str, config: &IngestConfig) -> Option<String> {
    let base = ingest_store_dir()?;
    std::fs::create_dir_all(&base).ok()?;

    let repo_name = get_repo_name(&config.root);
    let filename = format!("{}.md", repo_name);
    let filepath = base.join(&filename);

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
    files
}

/// Read a file and apply compression level.
fn read_and_compress(path: &Path, level: IngestLevel) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    // Skip binary-looking files
    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    // Never compress key documentation files — LLMs need these in full
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let lower = name.to_lowercase();
        if lower == "readme.md"
            || lower == "claude.md"
            || lower == "agents.md"
            || lower == "contributing.md"
            || lower == "changelog.md"
        {
            return Some(content);
        }
    }

    let lang = detect_language(&path.to_path_buf());

    // Data files: truncate if large (JSON fixtures, match data, etc.)
    if lang == Language::Data {
        if content.len() > MAX_DATA_FILE_SIZE {
            // Find a safe char boundary for truncation
            let mut cut_at = MAX_DATA_FILE_SIZE.min(content.len());
            while cut_at > 0 && !content.is_char_boundary(cut_at) {
                cut_at -= 1;
            }
            // Cut at last newline to avoid broken lines
            if let Some(nl) = content[..cut_at].rfind('\n') {
                cut_at = nl;
            }
            return Some(format!(
                "{}\n... [truncated: {} total, showing first {}]",
                &content[..cut_at],
                format_bytes_static(content.len()),
                format_bytes_static(cut_at)
            ));
        }
        return Some(content);
    }

    match level {
        IngestLevel::Full => Some(content),
        IngestLevel::Minimal => Some(filter_minimal(&content, lang)),
        IngestLevel::Aggressive => Some(filter_aggressive(&content, lang)),
    }
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
fn apply_budget(files: &mut Vec<DigestFile>, budget: usize, level: IngestLevel) {
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();

    if total_tokens <= budget {
        return; // Fits within budget
    }

    // Strategy: if over budget, re-compress large files aggressively
    if level != IngestLevel::Aggressive {
        for file in files.iter_mut() {
            if file.tokens > 500 && !file.is_changed {
                // Re-read with aggressive compression
                let path = PathBuf::from(&file.rel_path);
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

    // Header
    let project_name = config
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    out.push_str(&format!("# {}\n\n", project_name));

    // Metadata
    out.push_str(&format!(
        "> {} files | {} tokens | {} | {}\n",
        total_files,
        format_tokens(total_tokens),
        match config.level {
            IngestLevel::Full => "full content",
            IngestLevel::Minimal => "comments stripped",
            IngestLevel::Aggressive => "signatures only",
        },
        if config.changed_only || config.since.is_some() {
            format!(
                "changed files only{}",
                config
                    .since
                    .as_ref()
                    .map(|s| format!(" (since {})", s))
                    .unwrap_or_default()
            )
        } else {
            "full project".to_string()
        }
    ));

    if let Some(budget) = config.budget_tokens {
        out.push_str(&format!(
            "> budget: {}/{} tokens ({}% used)\n",
            format_tokens(total_tokens),
            format_tokens(budget),
            total_tokens * 100 / budget.max(1)
        ));
    }

    out.push_str("\n");

    // Directory tree
    out.push_str("## File tree\n\n```\n");
    let tree = build_tree(files);
    out.push_str(&tree);
    out.push_str("```\n\n");

    // File contents
    out.push_str("## Files\n\n");

    for file in files {
        if file.rel_path.is_empty() {
            out.push_str(&file.content);
            out.push('\n');
            continue;
        }

        let ext = Path::new(&file.rel_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let changed_marker = if file.is_changed { " (changed)" } else { "" };

        // Use <!-- file: path --> comment to avoid header conflicts with file content
        out.push_str(&format!(
            "<!-- file: {} -->\n### `{}`{}\n\n```{}\n{}\n```\n\n",
            file.rel_path, file.rel_path, changed_marker, ext, file.content.trim_end()
        ));
    }

    // Footer
    out.push_str(&format!(
        "---\n*Generated by trs ingest v{} in {}ms*\n",
        env!("CARGO_PKG_VERSION"),
        elapsed_ms
    ));

    out
}

/// Build a compact directory tree from file paths.
fn build_tree(files: &[DigestFile]) -> String {
    let mut dirs: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for file in files {
        if file.rel_path.is_empty() {
            continue;
        }
        let path = Path::new(&file.rel_path);
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        dirs.entry(parent).or_default().push(name);
    }

    let mut tree = String::new();
    for (dir, filenames) in &dirs {
        if dir.is_empty() {
            for f in filenames {
                tree.push_str(&format!("{}\n", f));
            }
        } else {
            tree.push_str(&format!("{}/\n", dir));
            for f in filenames {
                tree.push_str(&format!("  {}\n", f));
            }
        }
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

fn format_bytes_static(n: usize) -> String {
    format_bytes(n)
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
