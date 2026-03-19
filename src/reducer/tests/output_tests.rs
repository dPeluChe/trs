use super::*;

// ============================================================
// ReducerOutput Tests
// ============================================================

#[test]
fn test_reducer_output_new() {
    let data = vec!["item1", "item2", "item3"];
    let output = ReducerOutput::new(&data);

    assert!(!output.is_empty);
    assert!(output.metadata.is_none());
    assert!(output.stats.is_none());
}

#[test]
fn test_reducer_output_empty() {
    let output = ReducerOutput::empty();

    assert!(output.is_empty);
    assert!(output.data.is_null());
}

#[test]
fn test_reducer_output_with_metadata() {
    let metadata = ReducerMetadata {
        reducer: "test".to_string(),
        items_processed: 10,
        items_filtered: 2,
        duration_ms: 50,
        custom: None,
    };

    let output = ReducerOutput::new(vec![1, 2, 3]).with_metadata(metadata);

    assert!(output.metadata.is_some());
    let meta = output.metadata.unwrap();
    assert_eq!(meta.reducer, "test");
    assert_eq!(meta.items_processed, 10);
    assert_eq!(meta.items_filtered, 2);
}

#[test]
fn test_reducer_output_with_stats() {
    let stats = ReducerStats::new(1000, 500, 100, 50);

    let output = ReducerOutput::new(vec![1, 2, 3]).with_stats(stats);

    assert!(output.stats.is_some());
    let s = output.stats.unwrap();
    assert_eq!(s.input_bytes, 1000);
    assert_eq!(s.output_bytes, 500);
    assert_eq!(s.reduction_ratio, 0.5);
    assert_eq!(s.input_tokens, 250);  // 1000 / 4
    assert_eq!(s.output_tokens, 125); // 500 / 4
    assert_eq!(s.token_reduction_ratio, 0.5);
}

#[test]
fn test_reducer_output_with_summary() {
    let output = ReducerOutput::new(vec![1, 2, 3]).with_summary("3 items processed");

    assert_eq!(output.summary, Some("3 items processed".to_string()));
}

#[test]
fn test_reducer_output_with_items() {
    let items = vec![
        ReducerItem::new("key1", "value1"),
        ReducerItem::new("key2", "value2"),
    ];

    let output = ReducerOutput::new(Vec::<i32>::new()).with_items(items);

    assert_eq!(output.items.len(), 2);
    assert_eq!(output.items[0].key, "key1");
    assert_eq!(output.items[1].value, "value2");
}

#[test]
fn test_reducer_output_with_sections() {
    let section1 = ReducerSection::new("Section 1")
        .with_items(vec![ReducerItem::new("a", "1")]);

    let section2 = ReducerSection::new("Section 2")
        .with_count(5)
        .with_items(vec![ReducerItem::new("b", "2")]);

    let output = ReducerOutput::new(Vec::<i32>::new()).with_sections(vec![section1, section2]);

    assert_eq!(output.sections.len(), 2);
    assert_eq!(output.sections[0].name, "Section 1");
    assert_eq!(output.sections[1].count, Some(5));
}

#[test]
fn test_reducer_output_format_json() {
    let output = ReducerOutput::new(vec![1, 2, 3]);
    let context = ReducerContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("\"data\""));
    // JSON may have spaces in array output
    assert!(formatted.contains("1") && formatted.contains("2") && formatted.contains("3"));
}

#[test]
fn test_reducer_output_format_compact() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_summary("test summary")
        .with_sections(vec![
            ReducerSection::new("Files")
                .with_count(3)
                .with_items(vec![
                    ReducerItem::new("file1.txt", "100"),
                    ReducerItem::new("file2.txt", "200"),
                ]),
        ]);

    let context = ReducerContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("test summary"));
    assert!(formatted.contains("Files (3):"));
    assert!(formatted.contains("file1.txt"));
}

#[test]
fn test_reducer_output_format_raw() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_items(vec![
            ReducerItem::new("item1", "value1"),
            ReducerItem::new("item2", "value2"),
        ]);

    let context = ReducerContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("item1"));
    assert!(formatted.contains("item2"));
}

#[test]
fn test_reducer_output_format_agent() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_summary("Agent summary")
        .with_sections(vec![
            ReducerSection::new("Results")
                .with_items(vec![
                    ReducerItem::new("result1", "value1").with_label("label1"),
                ]),
        ]);

    let context = ReducerContext {
        format: OutputFormat::Agent,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("SUMMARY: Agent summary"));
    assert!(formatted.contains("## Results"));
    assert!(formatted.contains("[label1]"));
}

#[test]
fn test_reducer_output_format_csv() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_items(vec![
            ReducerItem::new("key1", "value1").with_label("label1"),
            ReducerItem::new("key2", "value2"),
        ]);

    let context = ReducerContext {
        format: OutputFormat::Csv,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("key,value,label"));
    assert!(formatted.contains("key1,value1,label1"));
    assert!(formatted.contains("key2,value2,"));
}

#[test]
fn test_reducer_output_format_tsv() {
    let output = ReducerOutput::new(Vec::<i32>::new())
        .with_items(vec![
            ReducerItem::new("key1", "value1"),
            ReducerItem::new("key2", "value2"),
        ]);

    let context = ReducerContext {
        format: OutputFormat::Tsv,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("key\tvalue\tlabel"));
    assert!(formatted.contains("key1\tvalue1\t"));
}

#[test]
fn test_reducer_output_empty_format() {
    let output = ReducerOutput::empty();
    let context = ReducerContext {
        format: OutputFormat::Compact,
        stats: false,
        enabled_formats: vec![],
    };

    let formatted = output.format(&context);
    assert!(formatted.contains("(empty)"));
}
