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
mod format;
mod meta;
mod ollama;
mod remote;
mod store;

pub use remote::{is_remote_ref, resolve_remote, TmpMode};

use std::path::{Path, PathBuf};
use std::process::Command;

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
}

/// A file entry in the digest.
pub(crate) struct DigestFile {
    pub(crate) rel_path: String,
    pub(crate) content: String,
    pub(crate) tokens: usize,
    pub(crate) is_changed: bool,
    /// Raw import tokens extracted from original file content (before compression).
    pub(crate) raw_imports: Vec<String>,
    /// Module-level docstring pulled from mod.rs / lib.rs / __init__.py.
    /// Used to annotate directory headers in the structure tree.
    pub(crate) module_doc: Option<String>,
    /// Public / exported symbol names declared in this file.
    pub(crate) symbols: Vec<String>,
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
fn suggest_budget(n: usize) -> &'static str {
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
                let suggested = suggest_budget(total_tokens);
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
            DigestFile {
                rel_path: "src/main.rs".into(),
                content: String::new(),
                tokens: 0,
                is_changed: false,
                raw_imports: vec![],
                module_doc: None,
                symbols: vec![],
            },
            DigestFile {
                rel_path: "src/lib.rs".into(),
                content: String::new(),
                tokens: 0,
                is_changed: false,
                raw_imports: vec![],
                module_doc: None,
                symbols: vec![],
            },
            DigestFile {
                rel_path: "README.md".into(),
                content: String::new(),
                tokens: 0,
                is_changed: false,
                raw_imports: vec![],
                module_doc: None,
                symbols: vec![],
            },
        ];
        let tree = format::build_tree(&files);
        assert!(tree.contains("src/"));
        assert!(tree.contains("  main.rs"));
        assert!(tree.contains("README.md"));
    }
}
