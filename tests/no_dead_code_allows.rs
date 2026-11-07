//! `allow(dead_code)` hid about 8,000 unused lines (whole modules) from the
//! compiler. In a binary crate the unused lints are the only unused-code
//! signal, so shipped code keeps them on: delete what is unused instead.

mod support;

#[test]
fn production_code_does_not_silence_unused_lints() {
    let attr = regex::Regex::new(r"#!?\[\s*(allow|expect)\s*\(([^\]]*)\)\s*\]").unwrap();
    let lint = regex::Regex::new(r"\b(dead_code|unused|unused_imports)\b").unwrap();
    let mut hits = Vec::new();
    for (rel, text) in support::production_src_files() {
        for m in attr.captures_iter(&text) {
            if lint.is_match(&m[2]) {
                let line = text[..m.get(0).unwrap().start()].lines().count() + 1;
                hits.push(format!("{rel}:{line}: {}", &m[0]));
            }
        }
    }
    assert!(hits.is_empty(), "unused lints silenced in: {hits:#?}");
}
