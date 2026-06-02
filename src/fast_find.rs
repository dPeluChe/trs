//! Fast find using the `ignore` crate (respects .gitignore).
//!
//! Activated with `trs find --gitignore [path] [-name pattern]`.
//! Uses a gitignore-aware walker instead of spawning `find`,
//! which is significantly faster on large repos with build artifacts.

use crate::router::CommandContext;
use crate::OutputFormat;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

/// Run a fast gitignore-aware find.
///
/// Supports a subset of find flags:
///   trs find --gitignore [path] [-name "pattern"] [-type f|d]
pub(crate) fn run(args: &[&str], ctx: &CommandContext) {
    let start = Instant::now();

    // Parse args: extract path, -name pattern, -type filter
    let mut root = ".";
    let mut name_pattern: Option<&str> = None;
    let mut type_filter: Option<&str> = None; // "f" or "d"
    let mut i = 0;

    while i < args.len() {
        match args[i] {
            "-name" if i + 1 < args.len() => {
                name_pattern = Some(args[i + 1]);
                i += 2;
            }
            "-type" if i + 1 < args.len() => {
                type_filter = Some(args[i + 1]);
                i += 2;
            }
            arg if !arg.starts_with('-') => {
                root = arg;
                i += 1;
            }
            _ => {
                i += 1; // skip unknown flags
            }
        }
    }

    // Build glob from -name pattern
    let glob_pattern = name_pattern.map(|p| {
        // Convert find-style patterns to glob: "*.rs" -> "*.rs"
        // Already glob-compatible in most cases
        p.to_string()
    });

    // Walk using ignore crate (respects .gitignore, skip hidden by default)
    let mut builder = ignore::WalkBuilder::new(root);
    builder
        .hidden(false) // show hidden files (like find does)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    // Use types builder for extension-based filtering (much faster than glob per-file)
    let has_ext_pattern = glob_pattern
        .as_ref()
        .map(|p| p.starts_with("*.") && !p[2..].contains('*') && !p[2..].contains('?'))
        .unwrap_or(false);

    let fast_ext = if has_ext_pattern {
        glob_pattern.as_ref().map(|p| &p[2..]) // "*.swift" -> "swift"
    } else {
        None
    };

    let mut files: Vec<String> = Vec::new();
    let mut dirs: Vec<String> = Vec::new();

    for entry in builder.build().flatten() {
        let path = entry.path();

        // Skip the root directory itself
        if path == Path::new(root) {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        // Apply -type filter
        match type_filter {
            Some("f") if is_dir => continue,
            Some("d") if !is_dir => continue,
            _ => {}
        }

        // Apply -name filter
        if let Some(ref pattern) = glob_pattern {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Fast path: simple extension match
                if let Some(ext) = fast_ext {
                    if !file_name.ends_with(ext)
                        || file_name.len() <= ext.len()
                        || file_name.as_bytes()[file_name.len() - ext.len() - 1] != b'.'
                    {
                        continue;
                    }
                } else if !glob_match(pattern, file_name) {
                    continue;
                }
            } else {
                continue;
            }
        }

        let display = crate::path_display::display_path(path);
        if is_dir {
            dirs.push(display);
        } else {
            files.push(display);
        }
    }

    let total_files = files.len();
    let total_dirs = dirs.len();
    let duration_ms = start.elapsed().as_millis() as u64;

    match ctx.format {
        OutputFormat::Json => print_json(&files, &dirs),
        OutputFormat::Raw => print_raw(&files, &dirs),
        _ => print_compact(&files, &dirs, root),
    }

    // Track
    let raw_est = files.iter().map(|f| f.len() + 1).sum::<usize>()
        + dirs.iter().map(|d| d.len() + 1).sum::<usize>();
    let out_est = total_files * 20 + total_dirs * 15; // rough estimate
    let cmd = format!(
        "find --gitignore {} {}",
        root,
        name_pattern
            .map(|p| format!("-name {}", p))
            .unwrap_or_default()
    );
    crate::tracker::log_execution(&cmd, raw_est, out_est.min(raw_est), duration_ms);

    if ctx.stats {
        eprintln!(
            "[trs:find --gitignore] {}F {}D in {}ms",
            total_files, total_dirs, duration_ms
        );
    }
}

/// Print compact tree-style output (default).
fn print_compact(files: &[String], dirs: &[String], _root: &str) {
    // Group files by directory
    let mut tree: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for path in files {
        let p = Path::new(path);
        let dir = p
            .parent()
            .map(crate::path_display::display_path)
            .unwrap_or_else(|| ".".to_string());
        let name = p
            .file_name()
            .map(|n| n.to_str().unwrap_or(path))
            .unwrap_or(path);
        tree.entry(dir).or_default().push(name.to_string());
    }

    println!("{}F {}D:", files.len(), dirs.len());
    println!();

    for (dir, filenames) in &tree {
        print!("{}/", dir);
        if filenames.len() <= 8 {
            // Inline
            println!(" {}", filenames.join(" "));
        } else {
            // Show first 6 + count
            println!(
                " {} +{} more",
                filenames[..6].join(" "),
                filenames.len() - 6
            );
        }
    }
}

/// Print raw paths (one per line, like find).
fn print_raw(files: &[String], dirs: &[String]) {
    for d in dirs {
        println!("{}", d);
    }
    for f in files {
        println!("{}", f);
    }
}

/// Print JSON output.
fn print_json(files: &[String], dirs: &[String]) {
    let obj = serde_json::json!({
        "files": files,
        "directories": dirs,
        "total_files": files.len(),
        "total_dirs": dirs.len(),
    });
    println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
}

/// Simple glob matching (supports * and ? only, no brackets).
fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    glob_match_inner(&p, &n)
}

fn glob_match_inner(p: &[char], n: &[char]) -> bool {
    match (p.first(), n.first()) {
        (None, None) => true,
        (Some('*'), _) => {
            // Collapse consecutive stars to avoid exponential backtracking
            let mut skip = 1;
            while skip < p.len() && p[skip] == '*' {
                skip += 1;
            }
            let p = &p[skip - 1..]; // keep one star
            glob_match_inner(&p[1..], n) || (!n.is_empty() && glob_match_inner(p, &n[1..]))
        }
        (Some('?'), Some(_)) => glob_match_inner(&p[1..], &n[1..]),
        (Some(&pc), Some(&nc)) if pc == nc => glob_match_inner(&p[1..], &n[1..]),
        (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(!glob_match("*.rs", "main.py"));
        assert!(glob_match("test_*", "test_foo"));
        assert!(!glob_match("test_*", "foo_test"));
        assert!(glob_match("?.rs", "a.rs"));
        assert!(!glob_match("?.rs", "ab.rs"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("main.*", "main.rs"));
        assert!(glob_match("main.*", "main.py"));
    }
}
