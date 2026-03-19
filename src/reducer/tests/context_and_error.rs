use super::*;

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
