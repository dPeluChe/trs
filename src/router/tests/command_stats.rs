use super::*;

// ============================================================
// CommandStats Tests
// ============================================================

#[test]
fn test_command_stats_with_reducer() {
    let stats = CommandStats::new().with_reducer("search");

    assert_eq!(stats.reducer, Some("search".to_string()));
}

#[test]
fn test_command_stats_with_output_mode() {
    let stats = CommandStats::new().with_output_mode(OutputFormat::Json);

    assert_eq!(stats.output_mode, Some(OutputFormat::Json));
}

#[test]
fn test_command_stats_with_all_fields() {
    let stats = CommandStats::new()
        .with_reducer("git-status")
        .with_output_mode(OutputFormat::Compact)
        .with_input_bytes(1000)
        .with_output_bytes(500)
        .with_items_processed(10);

    assert_eq!(stats.reducer, Some("git-status".to_string()));
    assert_eq!(stats.output_mode, Some(OutputFormat::Compact));
    assert_eq!(stats.input_bytes, 1000);
    assert_eq!(stats.output_bytes, 500);
    assert_eq!(stats.items_processed, 10);
}

#[test]
fn test_command_stats_format_output_mode() {
    assert_eq!(CommandStats::format_output_mode(OutputFormat::Raw), "raw");
    assert_eq!(
        CommandStats::format_output_mode(OutputFormat::Compact),
        "compact"
    );
    assert_eq!(CommandStats::format_output_mode(OutputFormat::Json), "json");
    assert_eq!(CommandStats::format_output_mode(OutputFormat::Csv), "csv");
    assert_eq!(CommandStats::format_output_mode(OutputFormat::Tsv), "tsv");
    assert_eq!(
        CommandStats::format_output_mode(OutputFormat::Agent),
        "agent"
    );
}

#[test]
fn test_command_stats_default() {
    let stats = CommandStats::default();

    assert!(stats.reducer.is_none());
    assert!(stats.output_mode.is_none());
    assert_eq!(stats.input_bytes, 0);
    assert_eq!(stats.output_bytes, 0);
}

#[test]
fn test_command_stats_reduction_percent() {
    let stats = CommandStats::new()
        .with_input_bytes(1000)
        .with_output_bytes(500);

    assert_eq!(stats.reduction_percent(), 50.0);
}

#[test]
fn test_command_stats_no_reduction_when_output_larger() {
    let stats = CommandStats::new()
        .with_input_bytes(500)
        .with_output_bytes(1000);

    assert_eq!(stats.reduction_percent(), 0.0);
}

#[test]
fn test_command_error_display() {
    let err = CommandError::NotImplemented("test command".to_string());
    assert_eq!(format!("{}", err), "Not implemented: test command");

    let err = CommandError::ExecutionError {
        message: "failed".to_string(),
        exit_code: Some(1),
    };
    assert_eq!(format!("{}", err), "Execution error: failed");

    let err = CommandError::InvalidArguments("bad args".to_string());
    assert_eq!(format!("{}", err), "Invalid arguments: bad args");

    let err = CommandError::IoError("file not found".to_string());
    assert_eq!(format!("{}", err), "I/O error: file not found");
}
