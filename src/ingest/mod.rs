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

mod collect;
mod collect_compress;
mod collect_index;
mod collect_manifests;
mod deps;
mod deps_extract;
mod dupes;
mod format;
mod format_html;
mod format_html_util;
mod format_tree;
mod meta;
mod mod_html;
mod ollama;
mod purpose;
mod remote;
mod resolve;
mod store;

pub use remote::{is_remote_ref, resolve_remote, TmpMode};
pub use resolve::resolve_project_root;

use std::path::PathBuf;

use collect::{apply_budget, collect_files, get_changed_files};
use deps::{build_dep_graph, format_dep_full, format_dep_summary};
use format::{build_digest, format_bytes, format_tokens};
use ollama::ollama_format;
use store::save_to_store;

pub use store::{list_ingests, read_digest};

/// Bytes per token estimate (GPT/Claude average).
pub(crate) const BYTES_PER_TOKEN: f64 = 4.0;

/// Files to always skip (binary, generated, large).
pub(crate) const SKIP_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "ico", "svg", "webp", "bmp", "woff", "woff2", "ttf", "eot", "otf",
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "exe", "dll", "so", "dylib", "bin", "pdf", "doc",
    "docx", "xls", "xlsx", "mp3", "mp4", "wav", "avi", "mov", "mkv", "db", "sqlite", "sqlite3",
    "pyc", "pyo", "class", "o", "obj", "wasm", "map",
];

/// Files to always skip by name.
pub(crate) const SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "Gemfile.lock",
    "poetry.lock",
    "composer.lock",
    ".DS_Store",
    "Thumbs.db",
];

/// Directories to always skip (generated/vendor/historical content).
pub(crate) const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".next",
    "__pycache__",
    ".pytest_cache",
    "dist",
    "build",
    "target",
    ".build",
    "DerivedData",
    "_generated",
    ".ruff_cache",
    ".mypy_cache",
    "coverage",
    ".turbo",
    ".nuxt",
    ".output",
    ".svelte-kit",
    "vendor",
    "venv",
    ".venv",
    "env",
    "archived",
    "archives",
    "archive",
    "old",
    "legacy",
    "deprecated",
    "TASK_COMPLETED",
    "tests",
    "test",
    "__tests__",
    "spec",
    "specs",
    "fixtures",
    "testdata",
    "test_data",
    ".claude",
    ".cursor",
    ".windsurf",
    ".copilot",
    ".codeium",
    ".agents",
    ".claude-plugin",
    ".codex-plugin",
    ".codex",
    ".codebuddy",
    ".kiro",
    ".gemini",
    ".goose",
    ".kilocode",
    ".trae",
    ".qoder",
    ".vscode",
    ".idea",
    ".fleet",
];

/// Max file size to include (64 KB -- large files are usually data, not code).
pub(crate) const MAX_FILE_SIZE: u64 = 64 * 1024;

/// Max size for data files like JSON (truncate with note).
#[allow(dead_code)]
pub(crate) const MAX_DATA_FILE_SIZE: usize = 2048;

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
    /// Output only the dependency graph, no file content.
    pub deps_only: bool,
    /// Use stored HEAD from last ingest as --since reference.
    pub since_last: bool,
    /// Skip regeneration if HEAD unchanged since last ingest.
    pub fresh_check: bool,
    /// Force regeneration, bypassing fresh check.
    pub force: bool,
    /// Print digest contents to stdout instead of just the saved path.
    pub print_content: bool,
    /// Warn on stderr when the digest exceeds this many tokens. Pass None
    /// (or 0 via CLI) to disable. Default: 40k — fine for GPT-4 / Claude
    /// 200k / etc., but a signal that `--budget` may be warranted.
    pub warn_at_tokens: Option<usize>,
    /// Emit a flat symbol → file index after the Structure section. Lets
    /// agents resolve "where is X?" in a single scan without reading any file.
    pub symbols_index: bool,
    /// Emit a self-contained visual HTML report instead of the markdown digest.
    pub html: bool,
    /// LOC threshold for flagging oversized files in the HTML report.
    pub max_loc: usize,
}

/// A file entry in the digest.
pub(crate) struct DigestFile {
    pub(crate) rel_path: String,
    pub(crate) content: String,
    pub(crate) tokens: usize,
    /// Raw line count of the original file (before compression). Feeds the
    /// HTML report's LOC-by-module bars and oversized-file flags.
    pub(crate) loc: usize,
    pub(crate) is_changed: bool,
    /// Raw import tokens extracted from original file content (before compression).
    pub(crate) raw_imports: Vec<String>,
    /// Module-level docstring pulled from mod.rs / lib.rs / __init__.py.
    /// Used to annotate directory headers in the structure tree.
    pub(crate) module_doc: Option<String>,
    /// Public / exported symbol names declared in this file.
    pub(crate) symbols: Vec<String>,
}

/// Run the ingest command.
pub fn run_ingest(config: &IngestConfig) {
    let start = std::time::Instant::now();

    // Resolve --since-last: look up stored HEAD from last ingest
    let effective_since = if config.since_last {
        match store::stored_head_for(&config.root) {
            Some(sha) => {
                eprintln!("trs ingest: --since-last using stored HEAD {}", sha);
                Some(sha)
            }
            None => {
                eprintln!("trs ingest: --since-last: no previous ingest found, using full mode");
                None
            }
        }
    } else {
        config.since.clone()
    };

    // Fresh check: skip if HEAD unchanged since last ingest
    if config.fresh_check && !config.force && !config.deps_only {
        if let Some(current) = meta::get_head_sha(&config.root) {
            if let Some(prev) = store::stored_head_for(&config.root) {
                if current == prev {
                    if let Some(path) = store::digest_path_for(&config.root) {
                        eprintln!(
                            "trs ingest: HEAD unchanged ({}), reusing cached digest at {}",
                            current,
                            path.display()
                        );
                        if config.print_content {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                print!("{}", content);
                            }
                        } else {
                            println!("{}", path.display());
                        }
                        return;
                    }
                }
            }
        }
    }

    // Collect files
    let mut files = collect_files(config);

    if files.is_empty() {
        eprintln!("trs ingest: no files found");
        return;
    }

    // Get changed files from git (if needed)
    let changed_set = if config.changed_only || effective_since.is_some() {
        get_changed_files(&config.root, effective_since.as_deref())
    } else {
        None
    };

    // Filter to changed files only
    if let Some(ref changed) = changed_set {
        if config.changed_only || effective_since.is_some() {
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

    // Build dependency graph
    let graph = build_dep_graph(&files);
    let project_name = config
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // --deps mode: output only the graph, no file content
    if config.deps_only {
        let output = format_dep_full(&graph, project_name);
        let tokens = (output.len() as f64 / BYTES_PER_TOKEN) as usize;
        eprintln!(
            "trs ingest --deps: {} ({} tokens)",
            format_bytes(output.len()),
            format_tokens(tokens),
        );
        print!("{}", output);
        return;
    }

    // --html mode: emit a self-contained visual report instead of markdown.
    if config.html {
        let root_display = config.root.display().to_string();
        let output = format_html::format_html(&files, project_name, &root_display, config.max_loc);
        let out_path = config.output_file.clone().unwrap_or_else(|| {
            let safe: String = project_name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect();
            std::path::PathBuf::from(format!("{}-report.html", safe))
        });
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        match std::fs::write(&out_path, &output) {
            Ok(()) => {
                eprintln!(
                    "trs ingest --html: {} -> {}",
                    format_bytes(output.len()),
                    out_path.display()
                );
                println!("{}", out_path.display());
            }
            Err(e) => {
                eprintln!("trs ingest: write failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Build the dep summary for header injection
    let dep_summary = if graph.is_empty() {
        String::new()
    } else {
        format_dep_summary(&graph)
    };

    // Build output
    let output = build_digest(
        &files,
        config,
        &changed_set,
        &dep_summary,
        start.elapsed().as_millis() as u64,
    );

    // Ollama post-processing
    let final_output = if let Some(ref model) = config.ollama_model {
        match ollama_format(&output, model) {
            Some(formatted) => formatted,
            None => output,
        }
    } else {
        output
    };

    let total_tokens = (final_output.len() as f64 / BYTES_PER_TOKEN) as usize;
    let file_count = files.iter().filter(|f| !f.rel_path.is_empty()).count();

    // stdout contract:
    //   default                → print the saved path (cheap for callers)
    //   --print                → print the digest content (legacy behavior)
    // stderr always gets a one-line summary with bytes/tokens/file counts.

    let digest_meta = meta::IngestMeta {
        head_sha: meta::get_head_sha(&config.root),
        timestamp: meta::now_unix(),
        file_count,
        tokens: total_tokens,
        project_root: config.root.display().to_string(),
        trs_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Resolve the output destination:
    //   --output <path>  → write there, no shadow save
    //   no flag          → shadow save to ~/.trs/ingest/<owner>/<repo>.md
    let written_path: Option<String> = if let Some(ref out_path) = config.output_file {
        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(out_path, &final_output) {
            Ok(()) => Some(out_path.display().to_string()),
            Err(e) => {
                eprintln!("trs ingest: write failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let saved_path = save_to_store(&final_output, config);
        if let Some(ref p) = saved_path {
            let digest_path = std::path::PathBuf::from(p);
            let _ = meta::save_meta(&digest_path, &digest_meta);
        }
        saved_path
    };

    if let Some(ref p) = written_path {
        eprintln!(
            "trs ingest: {} ({} tokens, {} files) -> {}",
            format_bytes(final_output.len()),
            format_tokens(total_tokens),
            file_count,
            p,
        );

        // Warn when the digest gets large enough that it will dominate an
        // agent's context window. Go to stderr only so spark / pipes stay
        // clean.
        if let Some(threshold) = config.warn_at_tokens {
            if threshold > 0 && total_tokens > threshold {
                let suggested = resolve::suggest_budget(total_tokens);
                eprintln!(
                    "  ⚠  {} tokens exceeds threshold ({}) — consider: trs ingest --budget {}",
                    format_tokens(total_tokens),
                    format_tokens(threshold),
                    suggested,
                );
            }
        }
    }

    if config.print_content {
        print!("{}", final_output);
    } else if let Some(ref p) = written_path {
        // Default: emit just the path on stdout for easy capture.
        println!("{}", p);
    }
}

#[cfg(test)]
mod tests;
