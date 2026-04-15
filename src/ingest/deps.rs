//! Dependency graph builder for project digests.
//!
//! Extracts import/use statements from source files, resolves them to project-internal
//! files, and builds a graph used to surface architectural context in the digest header.

use std::collections::HashMap;
use std::path::Path;

use super::DigestFile;

/// Project-internal dependency graph.
pub(super) struct DepGraph {
    /// file rel_path → files it imports (project-internal only)
    pub edges: HashMap<String, Vec<String>>,
    /// file rel_path → files that import it (reverse index)
    pub imported_by: HashMap<String, Vec<String>>,
}

impl DepGraph {
    /// Files sorted by how many other files import them, descending.
    pub fn top_central(&self, n: usize) -> Vec<(&str, &Vec<String>)> {
        let mut entries: Vec<(&str, &Vec<String>)> = self
            .imported_by
            .iter()
            .filter(|(_, importers)| !importers.is_empty())
            .map(|(f, importers)| (f.as_str(), importers))
            .collect();
        entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
        entries.into_iter().take(n).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.imported_by.is_empty()
    }
}

/// Build the dependency graph from collected digest files.
pub(super) fn build_dep_graph(files: &[DigestFile]) -> DepGraph {
    // Fast lookup: module stem → list of rel_paths with that stem
    let mut stem_index: HashMap<String, Vec<&str>> = HashMap::new();
    for file in files {
        if file.rel_path.is_empty() { continue; }
        let path = Path::new(&file.rel_path);
        // Index by file stem (e.g. "config" for "src/config.rs")
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            stem_index.entry(stem.to_lowercase()).or_default().push(&file.rel_path);
        }
        // Also index "mod" files by their parent dir name (e.g. "ingest" for "src/ingest/mod.rs")
        if path.file_stem().and_then(|s| s.to_str()) == Some("mod") {
            if let Some(parent_stem) = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
                stem_index.entry(parent_stem.to_lowercase()).or_default().push(&file.rel_path);
            }
        }
        // Index by index.ts/index.js parent dir too
        if matches!(path.file_stem().and_then(|s| s.to_str()), Some("index")) {
            if let Some(parent_stem) = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
                stem_index.entry(parent_stem.to_lowercase()).or_default().push(&file.rel_path);
            }
        }
    }

    let all_paths: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();

    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut imported_by: HashMap<String, Vec<String>> = HashMap::new();

    for file in files {
        if file.rel_path.is_empty() || file.raw_imports.is_empty() { continue; }

        let resolved = resolve_imports(&file.raw_imports, &file.rel_path, &stem_index, &all_paths);

        for dep in &resolved {
            imported_by
                .entry(dep.clone())
                .or_default()
                .push(file.rel_path.clone());
        }
        if !resolved.is_empty() {
            edges.insert(file.rel_path.clone(), resolved);
        }
    }

    DepGraph { edges, imported_by }
}

/// Extract raw import tokens from file content (before compression).
/// Returns module names or relative paths — not yet resolved to project files.
pub(crate) fn extract_raw_imports(content: &str, ext: &str) -> Vec<String> {
    let mut imports = match ext {
        "rs" => extract_rust(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" => extract_ts(content),
        "py" => extract_python(content),
        "go" => extract_go(content),
        _ => return vec![],
    };
    imports.sort();
    imports.dedup();
    imports
}

fn extract_rust(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        // mod foo; — submodule declaration
        if let Some(rest) = t.strip_prefix("mod ") {
            let name = rest.trim_end_matches(';').trim();
            if !name.is_empty()
                && !name.starts_with('{')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                out.push(name.to_string());
            }
            continue;
        }
        // use crate::X or use crate::X::Y
        if let Some(rest) = t.strip_prefix("use crate::") {
            if let Some(first) = rest.split("::").next() {
                let name = first.split('{').next().unwrap_or("").trim().trim_end_matches(';');
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
            continue;
        }
        // use super::X
        if let Some(rest) = t.strip_prefix("use super::") {
            if let Some(first) = rest.split("::").next() {
                let name = first.split('{').next().unwrap_or("").trim().trim_end_matches(';');
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
        // Stop scanning after the import block (first non-import line that isn't a comment/attr)
        // This keeps it fast on large files
        if !t.is_empty()
            && !t.starts_with("use ")
            && !t.starts_with("mod ")
            && !t.starts_with("//")
            && !t.starts_with("#[")
            && !t.starts_with("pub mod")
            && !t.starts_with("extern crate")
        {
            // Give up early — imports are at the top in Rust
            if out.len() > 0 { break; }
        }
    }
    out
}

fn extract_ts(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if !t.starts_with("import ") && !t.starts_with("export ") { continue; }

        // Find `from '...'` or `from "..."`
        let path = extract_from_path(t);
        if let Some(p) = path {
            // Relative imports: ./foo or ../bar
            // Alias imports: @/lib/foo (Next.js, Vite, etc.)
            if p.starts_with("./") || p.starts_with("../") || p.starts_with("@/") {
                out.push(p);
            }
        }
    }
    out
}

fn extract_from_path(line: &str) -> Option<String> {
    // Try single quotes first, then double quotes
    for (open, close) in [("from '", "'"), ("from \"", "\"")] {
        if let Some(pos) = line.rfind(open) {
            let after = &line[pos + open.len()..];
            if let Some(end) = after.find(close) {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

fn extract_python(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        // from .foo import ...  (relative imports only)
        if let Some(rest) = t.strip_prefix("from .") {
            let module = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('.');
            if !module.is_empty() && module != "import" {
                out.push(module.to_string());
            }
        }
        // Stop after class/def definitions start — imports are at top
        if t.starts_with("def ") || t.starts_with("class ") || t.starts_with("if __name__") {
            if !out.is_empty() { break; }
        }
    }
    out
}

fn extract_go(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_import_block = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("import (") || t == "import(" { in_import_block = true; continue; }
        if in_import_block && t == ")" { break; }
        // Single import: import "path"
        if let Some(rest) = t.strip_prefix("import \"") {
            let path = rest.trim_end_matches('"');
            if path.starts_with("./") || path.starts_with("../") {
                out.push(path.to_string());
            }
        }
        if in_import_block {
            let path = t.trim_matches('"').trim_matches('`');
            if path.starts_with("./") || path.starts_with("../") {
                out.push(path.to_string());
            }
        }
    }
    out
}

/// Resolve raw import tokens to actual project file rel_paths.
fn resolve_imports(
    raw: &[String],
    importer: &str,
    stem_index: &HashMap<String, Vec<&str>>,
    all_paths: &[&str],
) -> Vec<String> {
    let importer_dir = Path::new(importer)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut resolved = Vec::new();

    for import in raw {
        let found = if import.starts_with("./") || import.starts_with("../") || import.starts_with("@/") {
            // Relative/alias path resolution (TS/Go, @/ Next.js/Vite aliases)
            resolve_relative(import, &importer_dir, all_paths)
        } else {
            // Module name resolution (Rust/Python)
            resolve_by_stem(import, stem_index)
        };

        if let Some(dep) = found {
            if dep != importer {
                resolved.push(dep);
            }
        }
    }

    resolved.sort();
    resolved.dedup();
    resolved
}

fn resolve_relative(import: &str, importer_dir: &str, all_paths: &[&str]) -> Option<String> {
    // @/ alias (Next.js, Vite, etc.) — strip prefix and match by path suffix
    if let Some(alias_path) = import.strip_prefix("@/") {
        return resolve_by_suffix(alias_path, all_paths);
    }

    let normalized = normalize_path(importer_dir, import);

    // Try exact match first, then with common extensions
    for path in all_paths {
        // Exact (e.g. import has extension)
        if *path == normalized {
            return Some(path.to_string());
        }
        // Without extension match
        let path_no_ext = Path::new(path)
            .with_extension("")
            .to_string_lossy()
            .to_string();
        if path_no_ext == normalized {
            return Some(path.to_string());
        }
        // index file (e.g. './router' → 'src/router/index.ts')
        let index_path = format!("{}/index", normalized);
        if path_no_ext == index_path {
            return Some(path.to_string());
        }
    }
    None
}

/// Resolve an alias path like `lib/cn` by finding any project file whose
/// path ends with that suffix (e.g. `packages/web/src/lib/cn.ts`).
fn resolve_by_suffix(suffix: &str, all_paths: &[&str]) -> Option<String> {
    let suffix_no_ext = Path::new(suffix)
        .with_extension("")
        .to_string_lossy()
        .to_string();

    // Score: prefer shorter paths (closer to root match), then alphabetical
    let mut best: Option<(&str, usize)> = None;
    for path in all_paths {
        let path_no_ext = Path::new(path)
            .with_extension("")
            .to_string_lossy()
            .to_string();

        let matches = path_no_ext.ends_with(&suffix_no_ext)
            && (path_no_ext.len() == suffix_no_ext.len()
                || path_no_ext[..path_no_ext.len() - suffix_no_ext.len()].ends_with('/'));

        if matches {
            let score = path.len(); // shorter = better match
            if best.map_or(true, |(_, s)| score < s) {
                best = Some((path, score));
            }
        }
    }
    best.map(|(p, _)| p.to_string())
}

fn resolve_by_stem(name: &str, stem_index: &HashMap<String, Vec<&str>>) -> Option<String> {
    // Try lowercase stem match
    let candidates = stem_index.get(&name.to_lowercase())?;
    // Prefer src/ files, then first match
    candidates
        .iter()
        .find(|p| p.starts_with("src/"))
        .or_else(|| candidates.first())
        .map(|p| p.to_string())
}

fn normalize_path(base: &str, import: &str) -> String {
    let mut parts: Vec<&str> = if base.is_empty() {
        vec![]
    } else {
        base.split('/').collect()
    };
    for segment in import.split('/') {
        match segment {
            "." | "" => {}
            ".." => { parts.pop(); }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Compact summary for injection into the digest header.
/// Only shown when ≥2 files are imported by ≥2 others.
pub(super) fn format_dep_summary(graph: &DepGraph) -> String {
    let top = graph.top_central(5);
    // Skip if nothing meaningful
    if top.is_empty() || top[0].1.len() < 2 {
        return String::new();
    }

    // Collect all rel_paths that will be displayed (both as targets and importers)
    // to decide when we need parent dir context for disambiguation
    let all_display: Vec<&str> = top.iter().map(|(f, _)| *f)
        .chain(top.iter().flat_map(|(_, imp)| imp.iter().map(|s| s.as_str())))
        .collect();

    let mut out = String::from("## Key Dependencies\n\n");
    for (file, importers) in &top {
        if importers.len() < 2 { break; } // stop at singletons
        let file_label = short_label(file, &all_display);
        let n = importers.len();
        let shown: Vec<String> = importers
            .iter()
            .take(4)
            .map(|s| short_label(s, &all_display))
            .collect();
        if n > 4 {
            out.push_str(&format!("  {} ← {} (+{})\n", file_label, shown.join(", "), n - 4));
        } else {
            out.push_str(&format!("  {} ← {}\n", file_label, shown.join(", ")));
        }
    }
    out.push('\n');
    out
}

/// Return "filename" if unique among paths, else "parent/filename".
fn short_label(rel_path: &str, all_paths: &[&str]) -> String {
    let filename = Path::new(rel_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(rel_path);
    // Count how many paths share this filename
    let count = all_paths.iter().filter(|p| {
        Path::new(p).file_name().and_then(|n| n.to_str()) == Some(filename)
    }).count();
    if count <= 1 {
        return filename.to_string();
    }
    // Show parent dir for context
    let parent = Path::new(rel_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());
    match parent {
        Some(p) if !p.is_empty() => format!("{}/{}", p, filename),
        _ => rel_path.to_string(),
    }
}

/// Full dependency graph output for `--deps` mode (no file content).
pub(super) fn format_dep_full(graph: &DepGraph, project_name: &str) -> String {
    let mut out = format!("# {} — import graph\n\n", project_name);

    // Build full path list for disambiguation
    let all_paths: Vec<&str> = graph.imported_by.keys()
        .chain(graph.edges.keys())
        .map(|s| s.as_str())
        .collect();

    let top = graph.top_central(30);
    if !top.is_empty() {
        out.push_str("## Most imported\n\n");
        for (file, importers) in &top {
            let n = importers.len();
            let file_label = short_label(file, &all_paths);
            let shown: Vec<String> = importers
                .iter()
                .take(5)
                .map(|s| short_label(s, &all_paths))
                .collect();
            if n > 5 {
                out.push_str(&format!(
                    "{} ({} files)\n  ← {}, +{} more\n",
                    file_label, n, shown.join(", "), n - 5
                ));
            } else {
                out.push_str(&format!(
                    "{} ({} file{})\n  ← {}\n",
                    file_label, n, if n == 1 { "" } else { "s" }, shown.join(", ")
                ));
            }
        }
        out.push('\n');
    }

    // Full adjacency list — only files that import something
    let mut all_importers: Vec<(&String, &Vec<String>)> = graph.edges.iter().collect();
    all_importers.sort_by_key(|(k, _)| k.as_str());

    if !all_importers.is_empty() {
        out.push_str("## All imports\n\n");
        for (file, deps) in all_importers {
            if deps.is_empty() { continue; }
            let shown: Vec<String> = deps
                .iter()
                .map(|d| short_label(d, &all_paths))
                .collect();
            out.push_str(&format!("{} → {}\n", file, shown.join(", ")));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_imports() {
        let src = r#"
use crate::router;
use crate::config::Config;
use crate::tracker;
use std::collections::HashMap;

fn main() {}
"#;
        let imports = extract_raw_imports(src, "rs");
        assert!(imports.contains(&"router".to_string()));
        assert!(imports.contains(&"config".to_string()));
        assert!(imports.contains(&"tracker".to_string()));
        // std:: should not appear
        assert!(!imports.iter().any(|i| i == "std"));
    }

    #[test]
    fn test_extract_rust_mod() {
        let src = "mod collect;\nmod format;\nmod store;\n\npub fn run() {}\n";
        let imports = extract_raw_imports(src, "rs");
        assert!(imports.contains(&"collect".to_string()));
        assert!(imports.contains(&"format".to_string()));
        assert!(imports.contains(&"store".to_string()));
    }

    #[test]
    fn test_extract_ts_imports() {
        let src = r#"
import { foo } from './utils';
import Bar from '../components/Bar';
import React from 'react';
"#;
        let imports = extract_raw_imports(src, "ts");
        assert!(imports.contains(&"./utils".to_string()));
        assert!(imports.contains(&"../components/Bar".to_string()));
        // External package — should not appear
        assert!(!imports.iter().any(|i| i == "react"));
    }

    #[test]
    fn test_extract_python_imports() {
        let src = "from .models import User\nfrom .utils import helper\nimport os\n";
        let imports = extract_raw_imports(src, "py");
        assert!(imports.contains(&"models".to_string()));
        assert!(imports.contains(&"utils".to_string()));
        assert!(!imports.contains(&"os".to_string()));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("src/ingest", "./format"), "src/ingest/format");
        assert_eq!(normalize_path("src/ingest", "../router"), "src/router");
        assert_eq!(normalize_path("", "./foo"), "foo");
    }

    #[test]
    fn test_build_dep_graph_empty() {
        let files: Vec<DigestFile> = vec![];
        let graph = build_dep_graph(&files);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_dep_summary_skips_singletons() {
        // A file imported by only 1 other file should not appear in summary
        let graph = DepGraph {
            edges: HashMap::new(),
            imported_by: {
                let mut m = HashMap::new();
                m.insert("src/utils.rs".to_string(), vec!["src/main.rs".to_string()]);
                m
            },
        };
        let summary = format_dep_summary(&graph);
        assert!(summary.is_empty(), "Singleton should be excluded from summary");
    }
}
