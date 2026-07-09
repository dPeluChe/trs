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
            loc: 0,
            is_changed: false,
            raw_imports: vec![],
            module_doc: None,
            symbols: vec![],
        },
        DigestFile {
            rel_path: "src/lib.rs".into(),
            content: String::new(),
            tokens: 0,
            loc: 0,
            is_changed: false,
            raw_imports: vec![],
            module_doc: None,
            symbols: vec![],
        },
        DigestFile {
            rel_path: "README.md".into(),
            content: String::new(),
            tokens: 0,
            loc: 0,
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
