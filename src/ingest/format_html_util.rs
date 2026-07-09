//! Pure helpers for the `--html` report: escaping, human-readable numbers,
//! the code-extension test, and the gitignore-aware asset scan. Split out of
//! `format_html.rs` to keep the page-assembly function focused.

use std::collections::HashMap;
use std::path::Path;

/// Source-code extensions — the orphan/isolated heuristic only considers code
/// (a stray `.md`/`.json` module isn't "dead code").
pub(super) fn is_code(rel: &str) -> bool {
    let ext = Path::new(rel)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "go"
            | "rb"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "php"
            | "vue"
            | "svelte"
            | "scala"
    )
}

/// A module's leaf name is a conventional entry point (legitimately un-imported).
pub(super) fn is_entry_module(m: &str) -> bool {
    let last = m.rsplit('/').next().unwrap_or(m);
    matches!(
        last,
        "main" | "lib" | "index" | "cli" | "app" | "bin" | "server" | "cmd"
    )
}

pub(super) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn human(n: usize) -> String {
    if n >= 1000 {
        let k = n as f64 / 1000.0;
        let s = format!("{:.1}", k);
        format!("{}k", s.trim_end_matches(".0"))
    } else {
        n.to_string()
    }
}

pub(super) fn human_bytes(b: u64) -> String {
    const U: &[&str] = &["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", b)
    } else {
        format!("{:.1} {}", v, U[i])
    }
}

/// Minimal JSON string encoder for the embedded data literals.
pub(super) fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Image / media / binary assets are skipped by the digest (they aren't
/// code), but they carry real weight in a repo. A second, gitignore-aware
/// walk tallies them by category + size and surfaces the heaviest files.
/// Returns `(section_html, total_count, total_bytes)`.
pub(super) fn scan_assets(root: &Path) -> (String, usize, u64) {
    const CATS: &[(&str, &[&str])] = &[
        (
            "images",
            &[
                "png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "bmp", "avif", "heic",
            ],
        ),
        (
            "media",
            &[
                "mp4", "mov", "mp3", "wav", "avi", "mkv", "webm", "m4a", "flac",
            ],
        ),
        ("fonts", &["woff", "woff2", "ttf", "eot", "otf"]),
        ("archives", &["zip", "tar", "gz", "bz2", "xz", "7z", "rar"]),
        (
            "pdf/office",
            &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"],
        ),
        (
            "data/bin",
            &["sqlite", "sqlite3", "db", "parquet", "bin", "wasm"],
        ),
    ];
    let mut count: HashMap<&str, usize> = HashMap::new();
    let mut bytes: HashMap<&str, u64> = HashMap::new();
    let mut heavy: Vec<(String, u64)> = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);
    for entry in builder.build().flatten() {
        let path = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
            continue;
        }
        if path.components().any(|c| {
            super::SKIP_DIRS
                .iter()
                .any(|d| c.as_os_str().to_str() == Some(*d))
        }) {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        for (cat, exts) in CATS {
            if exts.contains(&ext.as_str()) {
                let sz = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
                *count.entry(cat).or_default() += 1;
                *bytes.entry(cat).or_default() += sz;
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                heavy.push((rel, sz));
                break;
            }
        }
    }
    let total_count: usize = count.values().sum();
    let total_bytes: u64 = bytes.values().sum();
    let mut cats: Vec<&str> = count.keys().copied().collect();
    cats.sort_by_key(|c| std::cmp::Reverse(bytes.get(c).copied().unwrap_or(0)));
    let chips = cats
        .iter()
        .map(|c| {
            format!(
                r#"<span class="chip">{} <b>{}</b> · {}</span>"#,
                c,
                count[c],
                human_bytes(bytes[c])
            )
        })
        .collect::<Vec<_>>()
        .join("");
    heavy.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    let rows = if heavy.is_empty() {
        r#"<div class="row"><span class="p">No image / media / binary assets.</span></div>"#
            .to_string()
    } else {
        heavy
            .iter()
            .take(12)
            .map(|(p, b)| {
                format!(
                    r#"<div class="row"><span class="p">{}</span><span class="loc">{}</span></div>"#,
                    esc(p),
                    human_bytes(*b)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let html = format!(
        r#"<div class="chips" style="margin-bottom:16px">{}</div><div class="rows">{}</div>"#,
        chips, rows
    );
    (html, total_count, total_bytes)
}
