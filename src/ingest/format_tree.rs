//! Digest tree + symbol-index builders for ingest. The top-level digest
//! assembly and the byte/token/html helpers live in `format.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use super::DigestFile;

/// Build readable tree: directories separated, files wrapped at ~70 chars.
pub(crate) fn build_tree(files: &[DigestFile]) -> String {
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

    // Pre-index module docs so we can annotate directories in one pass.
    // Key: parent directory path, Value: summary from mod.rs / __init__.py / etc.
    let dir_annotations = collect_dir_annotations(files);

    let mut tree = String::new();
    for (dir, filenames) in &dirs {
        // Directory header with optional annotation
        let header_dir = if dir.is_empty() { "/" } else { dir.as_str() };
        match dir_annotations.get(dir) {
            Some(annotation) => {
                tree.push_str(&format!("{}/: {}\n", header_dir, annotation));
            }
            None => {
                tree.push_str(&format!("{}/\n", header_dir));
            }
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

/// Build the Symbols section: flat `name → path` list sorted alphabetically.
/// Caps at 200 entries — agents can use trs ingest --deps + file sections
/// for repos larger than that.
pub(crate) fn build_symbol_index(files: &[DigestFile]) -> String {
    let mut entries: Vec<(String, String)> = Vec::new();
    for file in files {
        if file.rel_path.is_empty() {
            continue;
        }
        for name in &file.symbols {
            entries.push((name.clone(), file.rel_path.clone()));
        }
    }
    if entries.is_empty() {
        return String::new();
    }
    entries.sort_by(|a, b| {
        a.0.to_lowercase()
            .cmp(&b.0.to_lowercase())
            .then(a.1.cmp(&b.1))
    });
    let total = entries.len();
    let cap = 200;

    // Column-align the arrow: widest name (within the capped set) + 2 spaces.
    let shown: &[(String, String)] = if total > cap {
        &entries[..cap]
    } else {
        &entries
    };
    let col = shown
        .iter()
        .map(|(n, _)| n.len())
        .max()
        .unwrap_or(0)
        .min(40);

    // Header always carries the totals so agents know at a glance whether
    // the list is complete or truncated.
    let header = if total > cap {
        format!("## Symbols ({} of {} shown)\n\n", shown.len(), total)
    } else {
        format!("## Symbols ({})\n\n", total)
    };

    let mut out = header;
    for (name, path) in shown {
        out.push_str(&format!("  {:<col$}  → {}\n", name, path, col = col));
    }
    if total > cap {
        out.push_str(&format!("  ... ({} more)\n", total - cap));
    }
    out.push('\n');
    out
}

/// Collect directory annotations from anchor files (mod.rs / lib.rs / __init__.py / index.ts).
/// Returns a map keyed by the parent directory path.
fn collect_dir_annotations(files: &[DigestFile]) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for file in files {
        let Some(doc) = file.module_doc.as_ref() else {
            continue;
        };
        let parent = Path::new(&file.rel_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        // Only insert if not already set (first anchor wins).
        out.entry(parent).or_insert_with(|| doc.clone());
    }
    out
}
