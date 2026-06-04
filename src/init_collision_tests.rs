use super::*;

#[test]
fn scan_json_flags_rtk_hook() {
    let tmp = std::env::temp_dir().join("trs_collision_test_rtk.json");
    fs::write(
        &tmp,
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"rtk rewrite"}]}]}}"#,
    )
    .unwrap();
    let hits = scan_json(&tmp);
    assert_eq!(hits.len(), 1);
    assert!(matches!(hits[0].kind, CollisionKind::HookBinary { .. }));
    fs::remove_file(&tmp).ok();
}

#[test]
fn scan_json_ignores_trs_hook() {
    let tmp = std::env::temp_dir().join("trs_collision_test_trs.json");
    fs::write(
        &tmp,
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"trs rewrite"}]}]}}"#,
    )
    .unwrap();
    let hits = scan_json(&tmp);
    assert!(hits.is_empty());
    fs::remove_file(&tmp).ok();
}

#[test]
fn scan_text_flags_rtk_rules() {
    let tmp = std::env::temp_dir().join("trs_collision_test_rtk_rules.md");
    fs::write(&tmp, "# My rules\n\nUses rtk rewrite for things.\n").unwrap();
    let mut visited = HashSet::new();
    let hits = scan_text(&tmp, IMPORT_MAX_DEPTH, &mut visited);
    assert_eq!(hits.len(), 1);
    fs::remove_file(&tmp).ok();
}

#[test]
fn scan_text_follows_at_imports() {
    let dir = std::env::temp_dir().join("trs_collision_import_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let main = dir.join("CLAUDE.md");
    let imported = dir.join("RTK.md");
    fs::write(&main, "@RTK.md\n").unwrap();
    fs::write(
        &imported,
        "# RTK - Rust Token Killer\n\nUses rtk rewrite.\n",
    )
    .unwrap();

    let mut visited = HashSet::new();
    let hits = scan_text(&main, IMPORT_MAX_DEPTH, &mut visited);
    assert!(
        !hits.is_empty(),
        "expected imports to be followed: {:?}",
        hits
    );
    assert!(hits.iter().all(|c| c.location == imported));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn scan_text_breaks_import_cycle() {
    let dir = std::env::temp_dir().join("trs_collision_cycle_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let a = dir.join("A.md");
    let b = dir.join("B.md");
    fs::write(&a, "@B.md\nrtk rewrite\n").unwrap();
    fs::write(&b, "@A.md\n").unwrap();
    let mut visited = HashSet::new();
    // Must terminate. Before the visited guard this would recurse
    // until depth runs out.
    let hits = scan_text(&a, IMPORT_MAX_DEPTH, &mut visited);
    assert_eq!(hits.len(), 1);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn resolve_import_handles_home_and_relative() {
    // SAFETY: mutating env in tests is awkward under parallel runs,
    // but this is the only test that touches HOME and it's
    // immediately read back — no cross-test leakage expected.
    std::env::set_var("HOME", "/Users/test");
    let base = PathBuf::from("/Users/test/.claude/CLAUDE.md");
    assert_eq!(
        resolve_import("~/other.md", &base),
        Some(PathBuf::from("/Users/test/other.md"))
    );
    assert_eq!(
        resolve_import("RTK.md", &base),
        Some(PathBuf::from("/Users/test/.claude/RTK.md"))
    );
    assert_eq!(
        resolve_import("/abs/path.md", &base),
        Some(PathBuf::from("/abs/path.md"))
    );
}

#[test]
fn is_competitor_hook_matches_nested() {
    let v: serde_json::Value =
        serde_json::from_str(r#"{"command":"rtk rewrite","description":"x"}"#).unwrap();
    assert!(is_competitor_hook(&v));
}

#[test]
fn is_competitor_hook_rejects_trs() {
    let v: serde_json::Value = serde_json::from_str(r#"{"command":"trs rewrite"}"#).unwrap();
    assert!(!is_competitor_hook(&v));
}
