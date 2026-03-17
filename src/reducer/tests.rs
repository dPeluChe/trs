#[cfg(test)]
mod tests {
    use crate::reducer::*;
    use crate::reducer::output;
    use crate::OutputFormat;
    use serde::Serialize;
    use std::collections::HashMap;

    #[test]
    fn test_reducer_context_creation() {
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: true,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert_eq!(context.format, OutputFormat::Json);
        assert!(context.stats);
        assert_eq!(context.enabled_formats.len(), 1);
    }

    #[test]
    fn test_reducer_context_has_conflicting_formats() {
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json, OutputFormat::Csv],
        };

        assert!(context.has_conflicting_formats());

        let context_no_conflict = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![OutputFormat::Json],
        };

        assert!(!context_no_conflict.has_conflicting_formats());
    }

    #[test]
    fn test_reducer_error_display() {
        let err = ReducerError::NotImplemented("test".to_string());
        assert_eq!(format!("{}", err), "Not implemented: test");

        let err = ReducerError::ProcessingError {
            message: "failed".to_string(),
        };
        assert_eq!(format!("{}", err), "Processing error: failed");

        let err = ReducerError::InvalidInput("bad input".to_string());
        assert_eq!(format!("{}", err), "Invalid input: bad input");

        let err = ReducerError::IoError("file not found".to_string());
        assert_eq!(format!("{}", err), "I/O error: file not found");
    }

    #[test]
    fn test_reducer_error_helpers() {
        let err = ReducerError::NotImplemented("test".to_string());
        assert!(err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::InvalidInput("test".to_string());
        assert!(!err.is_not_implemented());
        assert!(err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::IoError("test".to_string());
        assert!(!err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(err.is_io_error());
        assert!(!err.is_processing_error());

        let err = ReducerError::ProcessingError {
            message: "test".to_string(),
        };
        assert!(!err.is_not_implemented());
        assert!(!err.is_invalid_input());
        assert!(!err.is_io_error());
        assert!(err.is_processing_error());
    }

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

    // ============================================================
    // ReducerItem Tests
    // ============================================================

    #[test]
    fn test_reducer_item_new() {
        let item = ReducerItem::new("key", "value");

        assert_eq!(item.key, "key");
        assert_eq!(item.value, "value");
        assert!(item.label.is_none());
        assert!(item.data.is_none());
    }

    #[test]
    fn test_reducer_item_with_label() {
        let item = ReducerItem::new("key", "value").with_label("label");

        assert_eq!(item.label, Some("label".to_string()));
    }

    #[test]
    fn test_reducer_item_with_data() {
        let data = serde_json::json!({"extra": "info"});
        let item = ReducerItem::new("key", "value").with_data(data.clone());

        assert_eq!(item.data, Some(data));
    }

    // ============================================================
    // ReducerSection Tests
    // ============================================================

    #[test]
    fn test_reducer_section_new() {
        let section = ReducerSection::new("Test Section");

        assert_eq!(section.name, "Test Section");
        assert!(section.count.is_none());
        assert!(section.items.is_empty());
    }

    #[test]
    fn test_reducer_section_with_count() {
        let section = ReducerSection::new("Test").with_count(10);

        assert_eq!(section.count, Some(10));
    }

    #[test]
    fn test_reducer_section_with_items() {
        let items = vec![
            ReducerItem::new("a", "1"),
            ReducerItem::new("b", "2"),
        ];

        let section = ReducerSection::new("Test").with_items(items);

        assert_eq!(section.items.len(), 2);
    }

    #[test]
    fn test_reducer_section_add_item() {
        let mut section = ReducerSection::new("Test");
        section.add_item(ReducerItem::new("a", "1"));

        assert_eq!(section.items.len(), 1);
        assert_eq!(section.items[0].key, "a");
    }

    // ============================================================
    // ReducerMetadata Tests
    // ============================================================

    #[test]
    fn test_reducer_metadata_default() {
        let metadata = ReducerMetadata::default();

        assert!(metadata.reducer.is_empty());
        assert_eq!(metadata.items_processed, 0);
        assert_eq!(metadata.items_filtered, 0);
        assert_eq!(metadata.duration_ms, 0);
        assert!(metadata.custom.is_none());
    }

    // ============================================================
    // ReducerStats Tests
    // ============================================================

    #[test]
    fn test_reducer_stats_default() {
        let stats = ReducerStats::default();

        assert_eq!(stats.input_bytes, 0);
        assert_eq!(stats.output_bytes, 0);
        assert_eq!(stats.reduction_ratio, 0.0);
        assert_eq!(stats.input_lines, 0);
        assert_eq!(stats.output_lines, 0);
        assert_eq!(stats.input_tokens, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.token_reduction_ratio, 0.0);
    }

    #[test]
    fn test_reducer_stats_new() {
        let stats = ReducerStats::new(4000, 1000, 200, 50);

        assert_eq!(stats.input_bytes, 4000);
        assert_eq!(stats.output_bytes, 1000);
        assert_eq!(stats.reduction_ratio, 0.25);
        assert_eq!(stats.input_lines, 200);
        assert_eq!(stats.output_lines, 50);
        assert_eq!(stats.input_tokens, 1000);  // 4000 / 4
        assert_eq!(stats.output_tokens, 250);  // 1000 / 4
        assert_eq!(stats.token_reduction_ratio, 0.25);
    }

    #[test]
    fn test_reducer_stats_zero_input() {
        let stats = ReducerStats::new(0, 0, 0, 0);

        assert_eq!(stats.reduction_ratio, 0.0);
        assert_eq!(stats.token_reduction_ratio, 0.0);
    }

    // ============================================================
    // BaseReducer Tests
    // ============================================================

    #[test]
    fn test_base_reducer_creation() {
        use serde::Deserialize;

        #[derive(Serialize, Deserialize, Debug)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        assert_eq!(reducer.name(), "test");
    }

    #[test]
    fn test_base_reducer_valid_json_input() {
        use serde::Deserialize;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = r#"{"value": 42}"#.to_string();
        let result = reducer.reduce(&input, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestData { value: 42 });
    }

    #[test]
    fn test_base_reducer_invalid_json_input() {
        use serde::Deserialize;

        #[derive(Serialize, Deserialize, Debug)]
        struct TestData {
            value: i32,
        }

        let reducer = BaseReducer::<TestData>::new("test");
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let input = "not valid json".to_string();
        let result = reducer.reduce(&input, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_invalid_input());
    }

    // ============================================================
    // ReducerRegistry Tests
    // ============================================================

    #[test]
    fn test_reducer_registry_new() {
        let registry = ReducerRegistry::new();
        assert!(registry.reducer_names().is_empty());
    }

    #[test]
    fn test_reducer_registry_default() {
        let registry = ReducerRegistry::default();
        assert!(registry.reducer_names().is_empty());
    }

    #[test]
    fn test_reducer_registry_register() {
        struct TestReducer;

        impl Reducer for TestReducer {
            type Input = String;
            type Output = ReducerOutput;

            fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
                Ok(ReducerOutput::new(input.clone()))
            }

            fn name(&self) -> &'static str {
                "test_reducer"
            }
        }

        let mut registry = ReducerRegistry::new();
        registry.register(TestReducer);

        assert_eq!(registry.reducer_names().len(), 1);
        assert!(registry.reducer_names().contains(&"test_reducer"));
    }

    #[test]
    fn test_reducer_registry_execute() {
        struct TestReducer;

        impl Reducer for TestReducer {
            type Input = String;
            type Output = ReducerOutput;

            fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
                Ok(ReducerOutput::new(input.clone())
                    .with_summary("Processed"))
            }

            fn name(&self) -> &'static str {
                "test_reducer"
            }
        }

        let mut registry = ReducerRegistry::new();
        registry.register(TestReducer);

        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let result = registry.execute("test_reducer", "test input", &context);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.summary, Some("Processed".to_string()));
    }

    #[test]
    fn test_reducer_registry_execute_not_found() {
        let registry = ReducerRegistry::new();
        let context = ReducerContext {
            format: OutputFormat::Json,
            stats: false,
            enabled_formats: vec![],
        };

        let result = registry.execute("nonexistent", "input", &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_not_implemented());
    }

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
        let output = ReducerOutput::new(vec![1, 2, 3])
            .with_summary("Test summary");

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"summary\""));
        assert!(json.contains("Test summary"));
    }

    #[test]
    fn test_reducer_item_serialization() {
        let item = ReducerItem::new("key", "value")
            .with_label("label");

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
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_sections(vec![
                ReducerSection::new("Files")
                    .with_items(vec![
                        ReducerItem::new("file1.txt", "100"),
                        ReducerItem::new("file2.txt", "200"),
                    ]),
            ]);

        let raw = output.format_raw();
        assert!(raw.contains("file1.txt"));
        assert!(raw.contains("file2.txt"));
    }

    #[test]
    fn test_reducer_output_format_agent_with_metadata() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_metadata(ReducerMetadata {
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
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1"),
            ]);

        let compact = output.format_compact();
        assert!(compact.contains("key1: value1"));
    }

    #[test]
    fn test_format_compact_section_without_count() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_sections(vec![
                ReducerSection::new("Files")
                    .with_items(vec![ReducerItem::new("file.txt", "100")]),
            ]);

        let compact = output.format_compact();
        assert!(compact.contains("Files:"));
        assert!(!compact.contains("Files ("));
    }

    #[test]
    fn test_format_agent_items_without_sections() {
        let output = ReducerOutput::new(Vec::<i32>::new())
            .with_items(vec![
                ReducerItem::new("key1", "value1").with_label("label1"),
                ReducerItem::new("key2", "value2"),
            ]);

        let agent = output.format_agent();
        assert!(agent.contains("## Items"));
        assert!(agent.contains("key1 [label1]: value1"));
        assert!(agent.contains("key2: value2"));
    }
}
