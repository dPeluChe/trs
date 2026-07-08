use super::*;

#[test]
fn replace_between_swaps_segment() {
    let before =
        "pre\n<!-- trs:output-saver:start v1 -->\nold\n<!-- trs:output-saver:end -->\npost";
    let out = replace_between(before, SENTINEL_START, SENTINEL_END, "NEW_BLOCK");
    assert!(out.contains("NEW_BLOCK"));
    assert!(out.contains("pre"));
    assert!(out.contains("post"));
    assert!(!out.contains("old"));
}

#[test]
fn standalone_file_contains_block() {
    let s = standalone_file();
    assert!(s.contains("Output saver"));
    assert!(s.contains("Open with the answer"));
}

/// Regression guard: hook-context template must NOT advertise
/// bypass mechanisms to agents. They reached for them defensively
/// on routine commands and burned the savings the hook just
/// bought. If you're tempted to add usage docs back here, read
/// the comment on `standalone_file()` first — bypass docs belong
/// in human-facing channels (`trs --help`, public docs), not in
/// the agent's prompt.
#[test]
fn standalone_file_does_not_promote_bypass_mechanisms() {
    let s = standalone_file();
    assert!(
        !s.contains("TRS_SKIP"),
        "hook-context template must not mention TRS_SKIP — see fn comment"
    );
    assert!(
        !s.contains("trs raw"),
        "hook-context template must not mention `trs raw` — see fn comment"
    );
}

#[test]
fn sentinel_wrapped_is_idempotent_on_replace() {
    // Running replace_between with freshly-wrapped content should
    // preserve the block unchanged.
    let wrapped = sentinel_wrapped();
    let out = replace_between(&wrapped, SENTINEL_START, SENTINEL_END, &sentinel_wrapped());
    // Both retain the sentinels and the block.
    assert!(out.contains(SENTINEL_START));
    assert!(out.contains(SENTINEL_END));
    assert!(out.contains("Open with the answer"));
}

#[test]
fn scan_unknown_agent_returns_unsupported() {
    let s = scan_agent("bogus");
    matches!(s, Status::Unsupported { .. });
}

#[test]
fn install_and_remove_imported_agent_roundtrip() {
    let dir = std::env::temp_dir().join("trs_os_roundtrip_imported");
    let _ = fs::remove_dir_all(&dir);
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    let res = install_agent_with_home("claude", Some(&home)).unwrap();
    // Message now names just the trs.md it wrote (the @import into CLAUDE.md
    // is verified behaviorally below); it no longer echoes the root config.
    assert!(res.contains("trs.md"));
    let claude_md = home.join(".claude/CLAUDE.md");
    let saver = home.join(".claude/trs.md");
    assert!(claude_md.exists());
    assert!(saver.exists());
    assert!(fs::read_to_string(&claude_md).unwrap().contains("@trs.md"));

    remove_agent_with_home("claude", Some(&home)).unwrap();
    assert!(!saver.exists());
    assert!(!fs::read_to_string(&claude_md).unwrap().contains("@trs.md"));

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn install_migrates_legacy_file() {
    let dir = std::env::temp_dir().join("trs_os_migration");
    let _ = fs::remove_dir_all(&dir);
    let home = dir.join("home");
    let claude_dir = home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Simulate a legacy install: old file + old @import line
    let legacy = claude_dir.join(IMPORT_FILENAME_LEGACY);
    let claude_md = claude_dir.join("CLAUDE.md");
    fs::write(&legacy, "old content").unwrap();
    fs::write(&claude_md, "@trs-output-saver.md\n").unwrap();

    // Confirm scan sees it as installed
    let status = scan_agent_with_home("claude", Some(&home));
    assert!(matches!(status, Status::AlreadyInstalled));

    // Install migrates: old file gone, new file present, import updated
    install_agent_with_home("claude", Some(&home)).unwrap();
    assert!(!legacy.exists(), "legacy file should be deleted");
    assert!(
        claude_dir.join(IMPORT_FILENAME).exists(),
        "new trs.md missing"
    );
    let root = fs::read_to_string(&claude_md).unwrap();
    assert!(
        !root.contains("@trs-output-saver.md"),
        "legacy import not removed"
    );
    assert!(root.contains("@trs.md"), "new import not added");

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn install_inline_file_is_idempotent() {
    let dir = std::env::temp_dir().join("trs_os_inline_idem");
    let _ = fs::remove_dir_all(&dir);
    let home = dir.join("home");
    fs::create_dir_all(home.join(".codex")).unwrap();
    let agents_path = home.join(".codex/AGENTS.md");
    fs::write(&agents_path, "# User agents\n\nCustom rules.\n").unwrap();

    install_agent_with_home("codex", Some(&home)).unwrap();
    let after1 = fs::read_to_string(&agents_path).unwrap();
    install_agent_with_home("codex", Some(&home)).unwrap();
    let after2 = fs::read_to_string(&agents_path).unwrap();
    assert_eq!(after1, after2, "second install mutated the file");
    assert!(after1.contains("Custom rules."));
    assert!(after1.contains("Open with the answer"));
    assert_eq!(
        after1.matches(SENTINEL_START).count(),
        1,
        "sentinel duplicated on re-install"
    );
    fs::remove_dir_all(&dir).ok();
}
