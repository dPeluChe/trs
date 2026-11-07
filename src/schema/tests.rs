use crate::schema::*;

// ============================================================
// Schema Version Tests
// ============================================================

#[test]
fn test_schema_version() {
    let version = SchemaVersion::new("test_type");
    assert_eq!(version.version, SCHEMA_VERSION);
    assert_eq!(version.schema_type, "test_type");
}

#[test]
fn test_schema_version_serialization() {
    let version = SchemaVersion::new("grep_output");
    let json = serde_json::to_string(&version).unwrap();
    assert!(json.contains("\"version\":\"1.0.0\""));
    assert!(json.contains("\"type\":\"grep_output\""));
}

// ============================================================
// Grep Output Schema Tests
// ============================================================

#[test]
fn test_grep_output_schema_new() {
    let schema = GrepOutputSchema::new();
    assert!(schema.is_empty);
    assert!(!schema.is_truncated);
    assert!(schema.files.is_empty());
}

// ============================================================
// Replace Output Schema Tests
// ============================================================

#[test]
fn test_replace_output_schema_new() {
    let schema = ReplaceOutputSchema::new("old", "new", false);
    assert!(!schema.dry_run);
    assert_eq!(schema.search_pattern, "old");
    assert_eq!(schema.replacement, "new");
    assert!(schema.files.is_empty());
    assert_eq!(schema.counts.files_affected, 0);
    assert_eq!(schema.counts.total_replacements, 0);
}

#[test]
fn test_replace_output_schema_dry_run() {
    let schema = ReplaceOutputSchema::new("pattern", "replacement", true);
    assert!(schema.dry_run);
}

#[test]
fn test_replace_match_new() {
    let m = ReplaceMatch::new(10, "old_function()", "new_function()");
    assert_eq!(m.line_number, 10);
    assert_eq!(m.original, "old_function()");
    assert_eq!(m.replaced, "new_function()");
}

#[test]
fn test_replace_output_schema_serialization() {
    let schema = ReplaceOutputSchema::new("old", "new", true);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("\"dry_run\":true"));
    assert!(json.contains("\"search_pattern\":\"old\""));
    assert!(json.contains("\"replacement\":\"new\""));
    assert!(json.contains("\"type\":\"replace_output\""));
}

#[test]
fn test_replace_counts_default() {
    let counts = ReplaceCounts::default();
    assert_eq!(counts.files_affected, 0);
    assert_eq!(counts.total_replacements, 0);
}

#[test]
fn test_replace_output_schema_with_files() {
    let file = ReplaceFile {
        path: "test.rs".to_string(),
        matches: vec![ReplaceMatch::new(1, "old", "new")],
    };

    let schema = ReplaceOutputSchema::new("old", "new", false)
        .with_files(vec![file])
        .with_counts(ReplaceCounts {
            files_affected: 1,
            total_replacements: 1,
        });

    assert_eq!(schema.files.len(), 1);
    assert_eq!(schema.files[0].path, "test.rs");
    assert_eq!(schema.counts.files_affected, 1);
    assert_eq!(schema.counts.total_replacements, 1);
}

#[test]
fn test_replace_output_round_trip() {
    let file = ReplaceFile {
        path: "src/lib.rs".to_string(),
        matches: vec![
            ReplaceMatch::new(5, "foo", "bar"),
            ReplaceMatch::new(10, "foo", "bar"),
        ],
    };

    let original = ReplaceOutputSchema::new("foo", "bar", true)
        .with_files(vec![file])
        .with_counts(ReplaceCounts {
            files_affected: 1,
            total_replacements: 2,
        });

    let json = serde_json::to_string(&original).unwrap();
    let parsed: ReplaceOutputSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(original, parsed);
}
