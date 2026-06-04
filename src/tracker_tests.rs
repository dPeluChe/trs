use super::*;

#[test]
fn month_key_from_ts_pads_zero() {
    // 2026-01-15 12:00:00 UTC → ts = 1768564800
    let key = month_key_from_ts(1_768_564_800);
    assert_eq!(key, "2026-01");
}

#[test]
fn month_key_from_ts_handles_december() {
    // 2026-12-31 23:59:59 UTC → ts = 1798761599
    let key = month_key_from_ts(1_798_761_599);
    assert_eq!(key, "2026-12");
}

#[test]
fn is_history_archive_recognizes_monthly_pattern() {
    assert!(is_history_archive(Path::new("/x/history.2026-05.jsonl")));
    assert!(is_history_archive(Path::new("history.2024-12.jsonl")));
    assert!(!is_history_archive(Path::new("history.jsonl")));
    assert!(!is_history_archive(Path::new("history.foo.jsonl")));
    assert!(!is_history_archive(Path::new("history.2026.jsonl")));
    // 8-char middle → 4+1+3 is wrong shape; reject
    assert!(!is_history_archive(Path::new("history.2026-005.jsonl")));
    // Pre-v0.5.1 dump uses a non-month suffix — must NOT be picked
    // up as an archive (we don't want stats to merge it).
    assert!(!is_history_archive(Path::new("history-pre-v0.5.1.jsonl")));
}

#[test]
fn maybe_rotate_active_renames_when_month_differs() {
    let tmp = tempfile::tempdir().unwrap();
    let active = tmp.path().join("history.jsonl");
    // Write one entry timestamped 2026-01-15.
    let mut f = fs::File::create(&active).unwrap();
    let entry = HistoryEntry {
        ts: 1_768_564_800, // 2026-01-15 UTC
        cmd: "git status".into(),
        in_bytes: 100,
        out_bytes: 20,
        saved_pct: 80,
        ms: 5,
        cwd: "/tmp".into(),
        agent: None,
        bypass: None,
    };
    writeln!(f, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
    drop(f);
    // Force mtime to be much earlier than the rotation check by
    // setting it artificially old via filetime — not available in
    // std. Instead, pass a now_ts in a different month so the
    // first-line peek decides.
    let now_in_april = 1_775_385_600; // 2026-04-02 UTC
    maybe_rotate_active(&active, now_in_april);

    // Active file should be gone, replaced by the archive named
    // after the OLDEST entry's month.
    assert!(!active.exists(), "active should have been renamed");
    let archived = tmp.path().join("history.2026-01.jsonl");
    assert!(archived.exists(), "expected {:?}", archived);
}

#[test]
fn maybe_rotate_active_is_noop_in_same_month() {
    let tmp = tempfile::tempdir().unwrap();
    let active = tmp.path().join("history.jsonl");
    let ts = 1_768_564_800; // 2026-01-15
    let entry = HistoryEntry {
        ts,
        cmd: "echo".into(),
        in_bytes: 1,
        out_bytes: 1,
        saved_pct: 0,
        ms: 0,
        cwd: "/".into(),
        agent: None,
        bypass: None,
    };
    let mut f = fs::File::create(&active).unwrap();
    writeln!(f, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
    drop(f);

    // Append two days later — same month, must not rotate.
    let later = ts + 2 * 86_400;
    maybe_rotate_active(&active, later);
    assert!(active.exists());
    assert!(!tmp.path().join("history.2026-01.jsonl").exists());
}

#[test]
fn redact_curl_basic_auth() {
    assert_eq!(
        redact_secrets("curl -u admin:hunter2 https://api.example.com"),
        "curl -u admin:[REDACTED] https://api.example.com"
    );
}

#[test]
fn redact_url_basic_auth() {
    assert_eq!(
        redact_secrets("git push https://oauth2:ghp_AAA@github.com/user/repo"),
        "git push https://oauth2:[REDACTED]@github.com/user/repo"
    );
}

#[test]
fn redact_password_flag() {
    assert_eq!(
        redact_secrets("mysql --password=hunter2 -h localhost"),
        "mysql --password=[REDACTED] -h localhost"
    );
    assert_eq!(
        redact_secrets("foo --api-key=AKIA1234567890ABCDEF"),
        "foo --api-key=[REDACTED]"
    );
}

#[test]
fn redact_authorization_header() {
    assert_eq!(
        redact_secrets("curl -H 'Authorization: Bearer ghp_AAAAAAAAAAAAAAAAAAAA1234' x"),
        "curl -H 'Authorization: Bearer [REDACTED]' x"
    );
}

#[test]
fn redact_token_shapes() {
    let out = redact_secrets("echo ghp_AAAAAAAAAAAAAAAAAAAA1234 sk-test_AAAAAAAAAAAAAAAAAAAA");
    assert!(out.contains("ghp_[REDACTED]"));
    assert!(out.contains("sk-[REDACTED]"));
    assert!(!out.contains("ghp_AAAAAAAAAAAAAAAAAAAA1234"));
}

#[test]
fn redact_leaves_normal_commands_alone() {
    let plain = "git log --oneline main..HEAD";
    assert_eq!(redact_secrets(plain), plain);
}

#[test]
fn test_format_bytes_human() {
    assert_eq!(format_bytes_human(0), "0");
    assert_eq!(format_bytes_human(500), "500");
    assert_eq!(format_bytes_human(1000), "1.0K");
    assert_eq!(format_bytes_human(12400), "12.4K");
    assert_eq!(format_bytes_human(1_500_000), "1.5M");
}

#[test]
fn test_saved_pct_calculation() {
    // 0 input -> 0%
    let in_b = 0usize;
    let out_b = 0usize;
    let pct = if in_b == 0 {
        0u8
    } else {
        (((in_b - out_b) as f64 / in_b as f64) * 100.0) as u8
    };
    assert_eq!(pct, 0);

    // 100 input, 20 output -> 80%
    let in_b = 100usize;
    let out_b = 20usize;
    let pct = (((in_b - out_b) as f64 / in_b as f64) * 100.0) as u8;
    assert_eq!(pct, 80);
}

#[test]
fn test_history_entry_serialization() {
    let entry = HistoryEntry {
        ts: 1773771663,
        cmd: "git status".to_string(),
        in_bytes: 497,
        out_bytes: 81,
        saved_pct: 83,
        ms: 12,
        cwd: "/path/to/project".to_string(),
        agent: None,
        bypass: None,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.cmd, "git status");
    assert_eq!(parsed.saved_pct, 83);
}

/// Old entries (pre-bypass field) must still deserialize cleanly.
/// Forward compatibility: `#[serde(default)]` on the field means
/// missing values become `None` rather than failing the parse.
#[test]
fn test_history_entry_legacy_lines_deserialize() {
    let legacy = r#"{"ts":1,"cmd":"git status","in_bytes":100,"out_bytes":20,"saved_pct":80,"ms":5,"cwd":"/p"}"#;
    let parsed: HistoryEntry = serde_json::from_str(legacy).unwrap();
    assert_eq!(parsed.bypass, None);
    assert_eq!(parsed.agent, None);
}

/// Bypass entries: `bypass` field present and set to true,
/// byte-counts are zero so they don't perturb savings sums.
#[test]
fn test_bypass_entry_round_trip() {
    let entry = HistoryEntry {
        ts: 42,
        cmd: "TRS_SKIP=1 git status".to_string(),
        in_bytes: 0,
        out_bytes: 0,
        saved_pct: 0,
        ms: 0,
        cwd: "/p".to_string(),
        agent: Some("claude".into()),
        bypass: Some(true),
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"bypass\":true"));
    let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.bypass, Some(true));
    assert_eq!(parsed.agent.as_deref(), Some("claude"));
}
