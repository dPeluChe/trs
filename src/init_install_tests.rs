
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
