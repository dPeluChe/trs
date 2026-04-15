use super::*;

// ============================================================
// Helper Function Tests
// ============================================================

#[test]
fn test_format_count_if_positive() {
    assert_eq!(format_count_if_positive("staged", 0), None);
    assert_eq!(
        format_count_if_positive("staged", 3),
        Some("staged=3".to_string())
    );
}

#[test]
fn test_format_list_with_count() {
    let items = vec!["file1.rs".to_string(), "file2.rs".to_string()];
    let output = format_list_with_count("staged", &items);
    assert!(output.contains("staged (2):"));
    assert!(output.contains("file1.rs"));
    assert!(output.contains("file2.rs"));
}

#[test]
fn test_format_list_with_count_empty() {
    let items: Vec<String> = vec![];
    let output = format_list_with_count("staged", &items);
    assert!(output.is_empty());
}

#[test]
fn test_format_key_value() {
    assert_eq!(format_key_value("branch", "main", None), "branch: main\n");
    assert_eq!(
        format_key_value("status", "M", Some("modified")),
        "status [modified]: M\n"
    );
}

#[test]
fn test_format_line() {
    assert_eq!(format_line("branch", "main"), "branch: main\n");
    assert_eq!(format_line("count", 42), "count: 42\n");
}

#[test]
fn test_truncate() {
    assert_eq!(truncate("hello", 10), "hello");
    assert_eq!(truncate("hello world", 8), "hello...");
    assert_eq!(truncate("hi", 3), "hi");
}

#[test]
fn test_truncate_utf8_multibyte() {
    // Emoji: each is 4 bytes. "😀😀😀" = 12 bytes.
    // Truncating at max_len=8 should not panic (would slice mid-emoji at byte 5).
    let emoji_str = "😀😀😀";
    let result = truncate(emoji_str, 8);
    assert!(result.ends_with("..."));
    // Should have sliced at a char boundary (4 bytes for 1 emoji + "...")
    assert_eq!(result, "😀...");

    // 2-byte chars: "ñ" is 2 bytes. "ñañaña" = 12 bytes.
    let result2 = truncate("ñañañañaña", 8);
    assert!(result2.ends_with("..."));
    // Must not panic — that's the main assertion
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(500), "500ms");
    assert_eq!(format_duration(1500), "1.50s");
    assert_eq!(format_duration(90000), "1m 30s");
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500B");
    assert_eq!(format_bytes(1024), "1.00KB");
    assert_eq!(format_bytes(1048576), "1.00MB");
    assert_eq!(format_bytes(1073741824), "1.00GB");
}
