use super::*;
use std::io::Write;

#[test]
fn scrub_legacy_codex_hook_removes_trs_only_event() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(
            tmp,
            r#"{{"hooks":{{"PreToolUse":[{{"hooks":[{{"command":"trs rewrite","type":"command"}}],"matcher":".*"}}],"SessionStart":[{{"hooks":[{{"command":"notify","type":"command"}}]}}]}}}}"#
        )
        .unwrap();
    let result = scrub_legacy_codex_hook(tmp.path(), false).unwrap();
    assert!(result.is_some(), "scrub should report a change");
    let after: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path()).unwrap()).unwrap();
    // PreToolUse was trs-only → key removed entirely.
    assert!(after["hooks"]["PreToolUse"].is_null());
    // SessionStart preserved verbatim.
    assert_eq!(
        after["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        serde_json::json!("notify")
    );
}

#[test]
fn scrub_legacy_codex_hook_preserves_user_entries_in_same_event() {
    // PreToolUse has trs AND a user-added entry — only trs is dropped,
    // the event survives.
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(
            tmp,
            r#"{{"hooks":{{"PreToolUse":[{{"hooks":[{{"command":"trs rewrite","type":"command"}}]}},{{"hooks":[{{"command":"my-audit","type":"command"}}]}}]}}}}"#
        )
        .unwrap();
    scrub_legacy_codex_hook(tmp.path(), false).unwrap();
    let after: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path()).unwrap()).unwrap();
    let arr = after["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["hooks"][0]["command"], serde_json::json!("my-audit"));
}

#[test]
fn scrub_legacy_codex_hook_is_noop_when_clean() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(
        tmp,
        r#"{{"hooks":{{"SessionStart":[{{"hooks":[{{"command":"notify","type":"command"}}]}}]}}}}"#
    )
    .unwrap();
    let result = scrub_legacy_codex_hook(tmp.path(), false).unwrap();
    assert!(result.is_none(), "no trs entry → no-op");
}

#[test]
fn scrub_legacy_codex_hook_dry_run_does_not_write() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    let original = r#"{"hooks":{"PreToolUse":[{"hooks":[{"command":"trs rewrite","type":"command"}],"matcher":".*"}]}}"#;
    write!(tmp, "{}", original).unwrap();
    let result = scrub_legacy_codex_hook(tmp.path(), true).unwrap();
    assert!(result.is_some());
    let after = std::fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(after, original, "dry-run must not modify file");
}

#[test]
fn refresh_sentinel_block_leaves_everything_outside_untouched() {
    // The file holds two trs blocks plus the user's own rules. A refresh that
    // ate the blank line between them would run three separate sets of
    // instructions together into one wall of prose, which is the opposite of
    // what a rules file is for.
    let content = "<!-- s -->\nold\n<!-- e -->\n\n## User rules\nkeep me\n";
    let out = refresh_sentinel_block(
        content,
        "<!-- s -->",
        "<!-- e -->",
        "<!-- s -->\nnew\n<!-- e -->",
    )
    .expect("sentinels are present");
    assert_eq!(
        out,
        "<!-- s -->\nnew\n<!-- e -->\n\n## User rules\nkeep me\n"
    );
}

#[test]
fn refresh_sentinel_block_declines_without_a_pair() {
    assert!(refresh_sentinel_block("no sentinels here", "<!-- s -->", "<!-- e -->", "x").is_none());
    // Opening sentinel but no close: replacing would swallow the rest of the
    // file, so decline rather than guess where the block ends.
    assert!(refresh_sentinel_block("<!-- s -->\nbody", "<!-- s -->", "<!-- e -->", "x").is_none());
}

#[test]
fn the_trs_marker_catches_the_backticked_spelling() {
    // The codex rules prose writes it as `trs` with backticks. Missing that
    // is what let a second copy of the block get appended to an AGENTS.md
    // that already had one, costing 2189 bytes on every session.
    use crate::init::file_has_any_trs_marker;
    assert!(file_has_any_trs_marker(
        "uses `trs` (Token-Reducing Shell) for output"
    ));
    assert!(file_has_any_trs_marker(
        "uses trs (Token-Reducing Shell) for output"
    ));
    assert!(!file_has_any_trs_marker("nothing to do with the tool"));
}
