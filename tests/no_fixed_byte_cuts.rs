//! A fixed byte cut like `&msg[..77]` panics when user text has a multibyte
//! char there. New cuts go through `text_util::truncate`, `truncate_chars` or
//! `str::get`; this list only shrinks.

mod support;

const REVIEWED: &[(&str, &str)] = &[
    ("src/anchor_guard.rs", "&h[..7]"),
    ("src/ingest/collect_manifests.rs", "lines[..20]"),
    ("src/ingest/collect_manifests.rs", "lines[..50]"),
    ("src/fast_find.rs", "filenames[..6]"),
    ("src/router/handlers/read_filters.rs", "&trimmed[..3]"),
    ("src/router/handlers/parse/git_status.rs", "chars[..2]"),
];

#[test]
fn no_new_fixed_byte_cuts() {
    let cut = regex::Regex::new(r"&?\w+\[\.\.(\d+|[A-Z_]{3,})\]").unwrap();
    let mut new = Vec::new();
    for (rel, text) in support::production_src_files() {
        for m in cut.find_iter(&text) {
            if !REVIEWED.contains(&(rel.as_str(), m.as_str())) {
                new.push(format!("{rel}: {}", m.as_str()));
            }
        }
    }
    assert!(new.is_empty(), "fixed byte cuts: {new:#?}");
}
