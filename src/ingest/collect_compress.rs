//! File reading and compression for the ingest pipeline.
//!
//! Reads source files and extracts what matters for LLM consumption:
//! signatures from code, schemas from data, summaries from docs.

use std::path::Path;

use crate::router::handlers::read_filters::{detect_language, filter_minimal, Language};

use super::collect_index::{extract_module_doc, extract_symbols, is_module_anchor};
use super::collect_manifests::{
    compress_jupyter_notebook, compress_package_json, compress_toml_manifest, extract_data_schema,
};
use super::signatures::extract_signatures;
use super::IngestLevel;

/// Output of a single-file compression pass.
pub(super) struct CompressResult {
    pub content: String,
    pub raw_imports: Vec<String>,
    /// Module-level doc for directory annotations (mod.rs / lib.rs / __init__.py).
    pub module_doc: Option<String>,
    /// Public / exported symbol names for the symbol index.
    pub symbols: Vec<String>,
    /// Raw line count of the original (uncompressed) file.
    pub loc: usize,
}

/// Intelligently extract what matters from a file for LLM consumption.
/// Returns a `CompressResult` with the compressed content plus metadata
/// (raw imports, module-level docstring, exported symbol names).
pub(super) fn read_and_compress(path: &Path, level: IngestLevel) -> Option<CompressResult> {
    let content = std::fs::read_to_string(path).ok()?;

    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    let mut ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let lower_name = name.to_lowercase();

    // Skip: CSS, HTML, SVG, XML, plist, lock files, minified, type declarations
    if matches!(
        ext.as_str(),
        "css" | "scss" | "less" | "svg" | "html" | "htm" | "xml" | "plist" | "xib" | "storyboard"
    ) {
        return None;
    }
    if lower_name.ends_with(".lock")
        || lower_name.ends_with(".min.js")
        || lower_name.ends_with(".min.css")
        || lower_name.ends_with(".d.ts")
        || lower_name == "__init__.py"
        || lower_name == ".gitkeep"
        || lower_name == ".nojekyll"
        || lower_name == ".gitattributes"
        || lower_name == "py.typed"
    {
        return None;
    }
    // Skip empty or near-empty files
    if content.trim().len() < 5 {
        return None;
    }

    // No extension but has a shebang → infer language from the interpreter.
    // This matters for files like `tmux-bridge`, `hooks/pre-commit`, etc.
    if ext.is_empty() {
        if let Some(first) = content.lines().next() {
            if first.starts_with("#!") {
                if first.contains("bash") || first.contains("/sh") || first.contains("zsh") {
                    ext = "sh".to_string();
                } else if first.contains("python") {
                    ext = "py".to_string();
                } else if first.contains("node") {
                    ext = "js".to_string();
                } else if first.contains("ruby") {
                    ext = "rb".to_string();
                }
            }
        }
    }

    let raw_imports = super::deps_extract::extract_raw_imports(&content, &ext);
    let module_doc = if is_module_anchor(&lower_name) {
        extract_module_doc(&content, &ext)
    } else {
        None
    };
    let symbols = extract_symbols(&content, &ext);

    // Wrap the compressed content in a CompressResult carrying the
    // already-extracted metadata. Each early-return branch calls `ok` exactly
    // once, so the metadata is moved (not cloned) into the result via
    // interior mutability on an Option.
    let loc = content.lines().count();
    let mut meta = Some((raw_imports, module_doc, symbols));
    let mut ok = |s: String| {
        let (imports, doc, syms) = meta.take().expect("CompressResult built twice");
        Some(CompressResult {
            content: s,
            raw_imports: imports,
            module_doc: doc,
            symbols: syms,
            loc,
        })
    };

    // Jupyter notebooks: parse JSON, show code + markdown cells.
    if ext == "ipynb" {
        return ok(compress_jupyter_notebook(&content));
    }

    // All markdown files: treat as docs, never extract signatures
    if ext == "md" || ext == "mdx" {
        // Translated READMEs (README_es.md, README_fr.md, etc.): skip content
        // They're variants of README.md -- just mention they exist
        if lower_name.starts_with("readme_") || lower_name.starts_with("readme-") {
            return ok(format!(
                "(translation of README.md, {} lines)",
                content.lines().count()
            ));
        }

        let cleaned = super::format::strip_html_from_markdown(&content);
        let is_key = matches!(
            lower_name.as_str(),
            "readme.md" | "claude.md" | "agents.md" | "contributing.md" | "changelog.md"
        );
        let max_lines = if is_key { 40 } else { 20 };
        let lines: Vec<&str> = cleaned.lines().collect();
        if lines.len() > max_lines + 5 {
            let preview: String = lines[..max_lines].join("\n");
            // Collect headings in the TRUNCATED portion so the reader knows
            // what the truncation hid.
            let tail_headings: Vec<String> = lines[max_lines..]
                .iter()
                .filter_map(|l| {
                    let t = l.trim_start();
                    t.strip_prefix("## ")
                        .or_else(|| t.strip_prefix("# "))
                        .map(|rest| rest.trim().to_string())
                })
                .take(8)
                .collect();
            let suffix = if tail_headings.is_empty() {
                format!("\n... ({} lines)", lines.len())
            } else {
                format!(
                    "\n... ({} lines, hidden sections: {})",
                    lines.len(),
                    tail_headings.join(" · ")
                )
            };
            return ok(format!("{}{}", preview, suffix));
        }
        return ok(cleaned);
    }

    // SKILL files and similar: cap at ~1KB
    if lower_name.contains("skill") || lower_name.ends_with(".prompt") {
        if content.len() > 1024 {
            let mut cut = 1024;
            while cut > 0 && !content.is_char_boundary(cut) {
                cut -= 1;
            }
            if let Some(nl) = content[..cut].rfind('\n') {
                cut = nl;
            }
            return ok(format!(
                "{}\n... ({} lines)",
                &content[..cut],
                content.lines().count()
            ));
        }
        return ok(content);
    }

    // Manifest files: compact + dependency-complete view. Never truncate the
    // dep list — it's the most informative part of the manifest for an LLM.
    if lower_name == "package.json" {
        return ok(compress_package_json(&content));
    }
    if lower_name == "pyproject.toml" || lower_name == "cargo.toml" {
        return ok(compress_toml_manifest(&content));
    }
    if lower_name == "requirements.txt" {
        // requirements are already a list — pass through unchanged, it's small.
        return ok(content);
    }

    // JSON/YAML: extract schema (keys + 1 sample), not all data
    let lang = detect_language(&path.to_path_buf());
    if lang == Language::Data {
        return ok(extract_data_schema(&content, &ext));
    }

    // Source code compression ladder:
    //   Full       — raw content, nothing removed
    //   Minimal    — strip comments + normalize blank lines (shared with `trs read`)
    //   Aggressive — signatures only (imports + defs), no function bodies
    match level {
        IngestLevel::Full => ok(content),
        IngestLevel::Minimal => ok(filter_minimal(&content, lang)),
        IngestLevel::Aggressive => ok(extract_signatures(&content, &ext)),
    }
}
