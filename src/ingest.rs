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

/// Max file size to include (256 KB).
const MAX_FILE_SIZE: u64 = 256 * 1024;

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

    // Write to file or stdout
    if let Some(ref out_path) = config.output_file {
        match std::fs::write(out_path, &final_output) {
            Ok(_) => {
                let tokens = (final_output.len() as f64 / BYTES_PER_TOKEN) as usize;
                eprintln!(
                    "trs ingest: wrote {} ({} tokens) to {}",
                    format_bytes(final_output.len()),
                    format_tokens(tokens),
                    out_path.display()
                );
            }
            Err(e) => eprintln!("trs ingest: failed to write {}: {}", out_path.display(), e),
        }
    } else {
        print!("{}", final_output);
    }
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

    // Sort: changed files first, then by path
    files.sort_by(|a, b| b.is_changed.cmp(&a.is_changed).then(a.rel_path.cmp(&b.rel_path)));
    files
}

/// Read a file and apply compression level.
fn read_and_compress(path: &Path, level: IngestLevel) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    // Skip binary-looking files
    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    let lang = detect_language(&path.to_path_buf());

    // Data files: never compress
    if lang == Language::Data {
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

        out.push_str(&format!(
            "### {}{}\n\n```{}\n{}\n```\n\n",
            file.rel_path, changed_marker, ext, file.content
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

/// Send digest to Ollama for LLM-formatted summary.
fn ollama_format(digest: &str, model: &str) -> Option<String> {
    // Check if Ollama is running
    let check = Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://localhost:11434/api/tags"])
        .output()
        .ok()?;

    let status = String::from_utf8_lossy(&check.stdout);
    if status.trim() != "200" {
        eprintln!("trs ingest: Ollama not running at localhost:11434 (skipping --ollama)");
        return None;
    }

    eprintln!("trs ingest: sending to Ollama ({})...", model);

    // Truncate digest if too large for Ollama context
    let max_chars = 32_000; // conservative for most models
    let input = if digest.len() > max_chars {
        &digest[..max_chars]
    } else {
        digest
    };

    let prompt = format!(
        "You are a technical documentation assistant. Given this codebase digest, produce a clean, \
         structured markdown summary that helps a developer or AI agent understand the project quickly.\n\n\
         Include:\n\
         1. Project overview (1-2 sentences)\n\
         2. Architecture (key modules and their responsibilities)\n\
         3. Key files and what they do\n\
         4. Dependencies and tech stack\n\
         5. Entry points\n\n\
         Keep it concise and factual. No fluff.\n\n\
         ---\n\n{}",
        input
    );

    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
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

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).ok()?;

    let formatted = response.get("response")?.as_str()?;

    // Combine: LLM summary at top, then the raw digest below
    let mut result = String::new();
    result.push_str("# Project Summary (LLM-generated)\n\n");
    result.push_str(formatted);
    result.push_str("\n\n---\n\n# Raw Digest\n\n");
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
