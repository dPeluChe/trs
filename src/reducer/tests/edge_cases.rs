use super::*;

// ============================================================
// Additional Edge Cases
// ============================================================

#[test]
fn test_truncation_info_limited_saturating_sub() {
    // Test that saturating_sub works correctly when shown > total
    let info = TruncationInfo::limited(5, 10, 20);

    assert_eq!(info.items_hidden, Some(0));
    assert!(!info.is_truncated);
}

#[test]
fn test_truncation_info_size_threshold_saturating_sub() {
    let info = TruncationInfo::size_threshold(100, 200, 200);

    assert_eq!(info.items_hidden, Some(0));
}

#[test]
fn test_reducer_output_serialization() {
    let output = ReducerOutput::new(vec![1, 2, 3]).with_summary("Test summary");

    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"data\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("Test summary"));
}

#[test]
fn test_reducer_item_serialization() {
    let item = ReducerItem::new("key", "value").with_label("label");

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"key\""));
    assert!(json.contains("\"value\""));
    assert!(json.contains("\"label\""));
}

#[test]
fn test_reducer_item_serialization_skip_none() {
    let item = ReducerItem::new("key", "value");

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"key\""));
    assert!(!json.contains("\"label\""));
    assert!(!json.contains("\"data\""));
}

#[test]
fn test_reducer_section_serialization() {
    let section = ReducerSection::new("Test")
        .with_count(5)
        .with_items(vec![ReducerItem::new("a", "1")]);

    let json = serde_json::to_string(&section).unwrap();
    assert!(json.contains("\"name\""));
    assert!(json.contains("Test"));
    assert!(json.contains("\"count\""));
}

#[test]
fn test_reducer_metadata_serialization() {
    let mut custom = HashMap::new();
    custom.insert("version".to_string(), "1.0".to_string());

    let metadata = ReducerMetadata {
        reducer: "test".to_string(),
        items_processed: 100,
        items_filtered: 10,
        duration_ms: 50,
        custom: Some(custom),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"reducer\""));
    assert!(json.contains("\"custom\""));
}

#[test]
fn test_reducer_metadata_skip_none_custom() {
    let metadata = ReducerMetadata {
        reducer: "test".to_string(),
        items_processed: 100,
        items_filtered: 10,
        duration_ms: 50,
        custom: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(!json.contains("\"custom\""));
}

#[test]
fn test_reducer_stats_serialization() {
    let stats = ReducerStats::new(1000, 500, 100, 50);

    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("\"input_bytes\""));
    assert!(json.contains("\"output_bytes\""));
    assert!(json.contains("\"reduction_ratio\""));
    assert!(json.contains("\"input_tokens\""));
}

#[test]
fn test_truncation_info_serialization() {
    let info = TruncationInfo::limited(100, 50, 50);

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"is_truncated\""));
    assert!(json.contains("\"total_available\""));
}

#[test]
fn test_truncation_info_serialization_skip_none() {
    let info = TruncationInfo::none();

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"is_truncated\""));
    // These should be skipped due to skip_serializing_if
    assert!(!json.contains("\"total_available\""));
    assert!(!json.contains("\"warning\""));
}

#[test]
fn test_reducer_error_from_processing() {
    let err = ReducerError::ProcessingError {
        message: "Something went wrong".to_string(),
    };

    assert!(err.is_processing_error());
    assert!(!err.is_not_implemented());
}

#[test]
fn test_reducer_output_with_exit_code() {
    let output = ReducerOutput {
        data: serde_json::Value::Null,
        metadata: None,
        stats: None,
        is_empty: false,
        summary: None,
        items: vec![],
        sections: vec![],
        exit_code: Some(0),
    };

    assert_eq!(output.exit_code, Some(0));
}

#[test]
fn test_format_csv_fallback_to_json() {
    // When no items, format_csv falls back to JSON
    let output = ReducerOutput::new(vec![1, 2, 3]);
    let csv = output.format_csv();

    // Should be JSON format since items is empty
    assert!(csv.starts_with('{'));
}

#[test]
fn test_format_tsv_fallback_to_json() {
    // When no items, format_tsv falls back to JSON
    let output = ReducerOutput::new(vec![1, 2, 3]);
    let tsv = output.format_tsv();

    // Should be JSON format since items is empty
    assert!(tsv.starts_with('{'));
}

#[test]
fn test_format_raw_with_sections() {
    let output =
        ReducerOutput::new(Vec::<i32>::new()).with_sections(vec![ReducerSection::new("Files")
            .with_items(vec![
                ReducerItem::new("file1.txt", "100"),
                ReducerItem::new("file2.txt", "200"),
            ])]);

    let raw = output.format_raw();
    assert!(raw.contains("file1.txt"));
    assert!(raw.contains("file2.txt"));
}

#[test]
fn test_reducer_output_format_agent_with_metadata() {
    let output = ReducerOutput::new(Vec::<i32>::new()).with_metadata(ReducerMetadata {
        reducer: "test".to_string(),
        items_processed: 10,
        items_filtered: 2,
        duration_ms: 5,
        custom: None,
    });

    let agent = output.format_agent();
    assert!(agent.contains("## Metadata"));
    assert!(agent.contains("reducer: test"));
}

#[test]
fn test_format_compact_items_without_label() {
    let output =
        ReducerOutput::new(Vec::<i32>::new()).with_items(vec![ReducerItem::new("key1", "value1")]);

    let compact = output.format_compact();
    assert!(compact.contains("key1: value1"));
}

#[test]
fn test_format_compact_section_without_count() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_sections(vec![
            ReducerSection::new("Files").with_items(vec![ReducerItem::new("file.txt", "100")])
        ]);

    let compact = output.format_compact();
    assert!(compact.contains("Files:"));
    assert!(!compact.contains("Files ("));
}

#[test]
fn test_format_agent_items_without_sections() {
    let output = ReducerOutput::new(Vec::<i32>::new()).with_items(vec![
        ReducerItem::new("key1", "value1").with_label("label1"),
        ReducerItem::new("key2", "value2"),
    ]);

    let agent = output.format_agent();
    assert!(agent.contains("## Items"));
    assert!(agent.contains("key1 [label1]: value1"));
    assert!(agent.contains("key2: value2"));
}
