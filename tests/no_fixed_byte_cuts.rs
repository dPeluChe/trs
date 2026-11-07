//! A fixed byte cut like `&msg[..77]` panics when user text has a multibyte
//! char there. New cuts go through `text_util::truncate`, `truncate_chars` or
//! `str::get`; this list only shrinks.

use std::path::Path;

const REVIEWED: &[(&str, &str)] = &[
    ("src/anchor_guard.rs", "&h[..7]"),
    ("src/ingest/collect_manifests.rs", "lines[..20]"),
    ("src/ingest/collect_manifests.rs", "lines[..50]"),
    ("src/fast_find.rs", "filenames[..6]"),
    ("src/router/handlers/read_filters.rs", "&trimmed[..3]"),
    ("src/router/handlers/parse/git_status.rs", "chars[..2]"),
];

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_new_fixed_byte_cuts() {
    let cut = regex::Regex::new(r"&?\w+\[\.\.(\d+|[A-Z_]{3,})\]").unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut new = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.ends_with("_tests.rs") || rel.contains("/tests/") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for m in cut.find_iter(&text) {
            if !REVIEWED.contains(&(rel.as_str(), m.as_str())) {
                new.push(format!("{rel}: {}", m.as_str()));
            }
        }
    }
    assert!(new.is_empty(), "fixed byte cuts: {new:#?}");
}
