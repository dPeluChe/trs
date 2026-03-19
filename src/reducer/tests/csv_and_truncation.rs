use super::*;

// ============================================================
// Escape CSV Tests
// ============================================================

#[test]
fn test_escape_csv_simple() {
    assert_eq!(output::escape_csv("simple"), "simple");
}

#[test]
fn test_escape_csv_with_comma() {
    assert_eq!(output::escape_csv("hello,world"), "\"hello,world\"");
}

#[test]
fn test_escape_csv_with_quotes() {
    assert_eq!(output::escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
}

#[test]
fn test_escape_csv_with_newline() {
    assert_eq!(output::escape_csv("line1\nline2"), "\"line1\nline2\"");
}

// ============================================================
// TruncationInfo Tests
// ============================================================

#[test]
fn test_truncation_info_none() {
    let info = TruncationInfo::none();

    assert!(!info.is_truncated);
    assert!(info.total_available.is_none());
    assert!(info.items_shown.is_none());
    assert!(info.items_hidden.is_none());
    assert!(info.reason.is_none());
    assert!(info.threshold.is_none());
    assert!(info.warning.is_none());
}

#[test]
fn test_truncation_info_limited_no_hidden() {
    let info = TruncationInfo::limited(10, 10, 20);

    assert!(!info.is_truncated);
    assert_eq!(info.total_available, Some(10));
    assert_eq!(info.items_shown, Some(10));
    assert_eq!(info.items_hidden, Some(0));
    assert_eq!(info.reason, Some("limit".to_string()));
    assert_eq!(info.threshold, Some(20));
    assert!(info.warning.is_none());
}

#[test]
fn test_truncation_info_limited_with_hidden() {
    let info = TruncationInfo::limited(100, 20, 20);

    assert!(info.is_truncated);
    assert_eq!(info.total_available, Some(100));
    assert_eq!(info.items_shown, Some(20));
    assert_eq!(info.items_hidden, Some(80));
    assert_eq!(info.reason, Some("limit".to_string()));
    assert_eq!(info.threshold, Some(20));
    assert!(info.warning.is_some());
    assert!(info.warning.unwrap().contains("20 of 100"));
}

#[test]
fn test_truncation_info_size_threshold() {
    let info = TruncationInfo::size_threshold(10000, 5000, 5000);

    assert!(info.is_truncated);
    assert_eq!(info.total_available, Some(10000));
    assert_eq!(info.items_shown, Some(5000));
    assert_eq!(info.items_hidden, Some(5000));
    assert_eq!(info.reason, Some("size_threshold".to_string()));
    assert_eq!(info.threshold, Some(5000));
    assert!(info.warning.is_some());
    assert!(info.warning.unwrap().contains("5000 of 10000 bytes"));
}

#[test]
fn test_truncation_info_detected() {
    let info = TruncationInfo::detected("incomplete_json", 500);

    assert!(info.is_truncated);
    assert!(info.total_available.is_none());
    assert_eq!(info.items_shown, Some(500));
    assert!(info.items_hidden.is_none());
    assert_eq!(info.reason, Some("detected".to_string()));
    assert!(info.threshold.is_none());
    assert!(info.warning.is_some());
    assert!(info.warning.as_ref().unwrap().contains("incomplete_json"));
}

#[test]
fn test_truncation_info_detect_from_output_no_truncation() {
    let output = "This is normal output\nWith multiple lines\nNo truncation here";
    let info = TruncationInfo::detect_from_output(output);

    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_truncated_marker() {
    let output = "Some output\n... truncated";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
    assert_eq!(info.reason, Some("detected".to_string()));
}

#[test]
fn test_truncation_info_detect_from_output_truncated_brackets() {
    let output = "Results [truncated]";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_showing_first() {
    let output = "Showing first 10 results...";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_more_results() {
    let output = "10 items shown, more results available";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_incomplete_json_array() {
    let output = "[1, 2, 3,";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
    assert!(info.warning.as_ref().unwrap().contains("incomplete_json"));
}

#[test]
fn test_truncation_info_detect_from_output_incomplete_json_object() {
    let output = r#"{"key": "value""#;
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_complete_json() {
    let output = r#"{"key": "value"}"#;
    let info = TruncationInfo::detect_from_output(output);

    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_complete_array() {
    let output = "[1, 2, 3]";
    let info = TruncationInfo::detect_from_output(output);

    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_cutoff_line_ellipsis() {
    // Last line ends with ... (more than 3 chars total)
    let output = "Some text here\nAnd more content...";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_detect_from_output_cutoff_and() {
    let output = "Item 1\nItem 2\n and";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

#[test]
fn test_truncation_info_is_truncated_method() {
    let truncated = TruncationInfo::detected("test", 100);
    assert!(truncated.is_truncated());

    let not_truncated = TruncationInfo::none();
    assert!(!not_truncated.is_truncated());
}

#[test]
fn test_truncation_info_summary() {
    let info = TruncationInfo::limited(100, 20, 20);
    let summary = info.summary();

    assert!(summary.is_some());
    assert!(summary.unwrap().contains("20"));
}

#[test]
fn test_truncation_info_summary_none() {
    let info = TruncationInfo::none();
    let summary = info.summary();

    assert!(summary.is_none());
}

#[test]
fn test_truncation_info_summary_minimal() {
    let mut info = TruncationInfo::default();
    info.is_truncated = true;
    info.items_shown = Some(10);
    info.items_hidden = None;
    info.warning = None;

    let summary = info.summary();
    assert!(summary.is_some());
    assert_eq!(summary.unwrap(), "Output was truncated");
}

#[test]
fn test_truncation_info_summary_with_counts() {
    let mut info = TruncationInfo::default();
    info.is_truncated = true;
    info.items_shown = Some(10);
    info.items_hidden = Some(5);
    info.warning = None;

    let summary = info.summary();
    assert!(summary.is_some());
    assert!(summary.unwrap().contains("10 items shown"));
}

#[test]
fn test_truncation_info_detect_case_insensitive() {
    let output = "OUTPUT TRUNCATED due to size";
    let info = TruncationInfo::detect_from_output(output);

    assert!(info.is_truncated);
}

// ============================================================
// TruncationConfig Tests
// ============================================================

#[test]
fn test_truncation_config_default() {
    let config = TruncationConfig::default();

    assert!(config.max_items.is_none());
    assert!(config.max_bytes.is_none());
    assert!(config.detect_patterns);
    assert!(config.include_warnings);
}

#[test]
fn test_truncation_config_with_max_items() {
    let config = TruncationConfig::with_max_items(50);

    assert_eq!(config.max_items, Some(50));
    assert!(config.max_bytes.is_none());
    assert!(config.detect_patterns);
}

#[test]
fn test_truncation_config_with_max_bytes() {
    let config = TruncationConfig::with_max_bytes(1024);

    assert!(config.max_items.is_none());
    assert_eq!(config.max_bytes, Some(1024));
    assert!(config.detect_patterns);
}

#[test]
fn test_truncation_config_truncate_items_no_limit() {
    let config = TruncationConfig::default();
    let items = vec![1, 2, 3, 4, 5];
    let (result, info) = config.truncate_items(items);

    assert_eq!(result.len(), 5);
    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_config_truncate_items_with_limit() {
    let config = TruncationConfig::with_max_items(3);
    let items = vec![1, 2, 3, 4, 5];
    let (result, info) = config.truncate_items(items);

    assert_eq!(result.len(), 3);
    assert!(info.is_truncated);
    assert_eq!(info.total_available, Some(5));
    assert_eq!(info.items_shown, Some(3));
    assert_eq!(info.items_hidden, Some(2));
}

#[test]
fn test_truncation_config_truncate_items_within_limit() {
    let config = TruncationConfig::with_max_items(10);
    let items = vec![1, 2, 3];
    let (result, info) = config.truncate_items(items);

    assert_eq!(result.len(), 3);
    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_config_truncate_output_no_limit() {
    let config = TruncationConfig {
        detect_patterns: false,
        ..Default::default()
    };
    let output = "Hello, world!".to_string();
    let (result, info) = config.truncate_output(output);

    assert_eq!(result, "Hello, world!");
    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_config_truncate_output_with_byte_limit() {
    let config = TruncationConfig {
        max_bytes: Some(5),
        detect_patterns: false,
        ..Default::default()
    };
    let output = "Hello, world!".to_string();
    let (result, info) = config.truncate_output(output);

    assert_eq!(result.len(), 5);
    assert!(info.is_truncated);
    assert_eq!(info.reason, Some("size_threshold".to_string()));
}

#[test]
fn test_truncation_config_truncate_output_detect_patterns() {
    let config = TruncationConfig {
        detect_patterns: true,
        ..Default::default()
    };
    let output = "Some data\n... truncated".to_string();
    let (result, info) = config.truncate_output(output);

    assert_eq!(result, "Some data\n... truncated");
    assert!(info.is_truncated);
    assert_eq!(info.reason, Some("detected".to_string()));
}

#[test]
fn test_truncation_config_truncate_output_no_detect_patterns() {
    let config = TruncationConfig {
        detect_patterns: false,
        ..Default::default()
    };
    let output = "Some data\n... truncated".to_string();
    let (result, info) = config.truncate_output(output);

    assert_eq!(result, "Some data\n... truncated");
    assert!(!info.is_truncated);
}
