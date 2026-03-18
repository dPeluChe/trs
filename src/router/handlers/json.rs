//! JSON structure handler for `trs json` command.
//!
//! Reads JSON from a file or stdin and outputs the structure (keys + types + array lengths)
//! without values. Reduces large JSON payloads to a compact schema overview.
//!
//! Features:
//! - Array sampling: large arrays (>5 items) show first 3 + last 3, preserving error items
//! - ID field detection: keys like "id", "uuid", "_id", "*_id" annotated with `(id)`
//! - Non-JSON rejection: suggests alternative trs commands for .toml, .yaml, .csv files
//!
//! Example:
//! ```text
//! $ cat api-response.json | trs json
//! {
//!   "data": Array[42] of {
//!     "user_id": Number (id),
//!     "name": String
//!   } (sampled: first 3 + last 3),
//!   "meta": {
//!     "page": Number,
//!     "total": Number,
//!     "cursor": String
//!   },
//!   "errors": Array[0]
//! }
//! ```

use std::path::PathBuf;

use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use crate::OutputFormat;

/// Threshold above which arrays are sampled instead of fully inspected.
const ARRAY_SAMPLE_THRESHOLD: usize = 5;
/// Number of items to keep from the start of a large array.
const ARRAY_SAMPLE_HEAD: usize = 3;
/// Number of items to keep from the end of a large array.
const ARRAY_SAMPLE_TAIL: usize = 3;

/// Keys in an object that indicate error-related data worth preserving.
const ERROR_KEYS: &[&str] = &["error", "exception", "failed", "failure"];

/// Input for the json command.
pub(crate) struct JsonInput {
    /// Input file (stdin if None)
    pub file: Option<PathBuf>,
    /// Maximum depth to display (None = unlimited)
    pub depth: Option<usize>,
}

pub(crate) struct JsonHandler;

impl JsonHandler {
    pub(crate) fn execute(&self, input: &JsonInput, ctx: &CommandContext) -> CommandResult {
        let raw = read_json_input(&input.file)?;
        let input_bytes = raw.len();

        let parsed: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            non_json_hint(&input.file, &e)
        })?;

        let max_depth = input.depth.unwrap_or(usize::MAX);

        let output = match ctx.format {
            OutputFormat::Json => {
                // JSON output: schema as JSON
                let schema = to_schema_json(&parsed, 0, max_depth);
                serde_json::to_string_pretty(&schema).unwrap_or_else(|_| schema.to_string())
            }
            _ => {
                // Compact/Agent: human-readable structure
                let mut buf = String::new();
                format_structure(&parsed, &mut buf, 0, max_depth);
                buf
            }
        };

        print!("{}", output);

        if ctx.stats {
            CommandStats::new()
                .with_reducer("json")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }

        Ok(())
    }
}

/// Produce a helpful error when JSON parsing fails.
///
/// If a file was provided, check its extension and suggest the right trs command.
/// Otherwise fall back to the raw parse error.
fn non_json_hint(file: &Option<PathBuf>, err: &serde_json::Error) -> CommandError {
    if let Some(path) = file {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let hint = match ext.to_lowercase().as_str() {
                "toml" => Some(format!(
                    "Not JSON. Try: trs read {}",
                    path.display()
                )),
                "yaml" | "yml" => Some(format!(
                    "Not JSON. Try: trs read {}",
                    path.display()
                )),
                "csv" => Some(
                    "Not JSON. Try: trs parse deps (for dependency lists)".to_string(),
                ),
                _ => None,
            };
            if let Some(msg) = hint {
                return CommandError::InvalidArguments(msg);
            }
        }
    }
    CommandError::InvalidArguments(format!("Invalid JSON: {}", err))
}

/// Read JSON input from file or stdin.
fn read_json_input(file: &Option<PathBuf>) -> CommandResult<String> {
    use std::io::{self, Read};

    if let Some(path) = file {
        std::fs::read_to_string(path)
            .map_err(|e| CommandError::IoError(format!("{}: {}", path.display(), e)))
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CommandError::IoError(e.to_string()))?;
        Ok(buf)
    }
}

// ============================================================
// ID field detection
// ============================================================

/// Check whether a key name looks like an identifier/ID field.
///
/// Matches: "id", "uuid", "_id", or any key ending with "_id".
fn is_id_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower == "id" || lower == "uuid" || lower == "_id" || lower.ends_with("_id")
}

// ============================================================
// Array sampling helpers
// ============================================================

/// Check if any object in the array contains error-related keys.
fn has_error_keys(value: &serde_json::Value) -> bool {
    if let serde_json::Value::Object(map) = value {
        map.keys().any(|k| {
            let lower = k.to_lowercase();
            ERROR_KEYS.iter().any(|ek| lower.contains(ek))
        })
    } else {
        false
    }
}

/// Build a sampled subset of array items for large arrays.
///
/// Returns the sampled items and whether sampling was applied.
fn sample_array(arr: &[serde_json::Value]) -> (Vec<&serde_json::Value>, bool) {
    if arr.len() <= ARRAY_SAMPLE_THRESHOLD {
        return (arr.iter().collect(), false);
    }

    let mut indices = std::collections::BTreeSet::new();

    // First N
    for i in 0..ARRAY_SAMPLE_HEAD.min(arr.len()) {
        indices.insert(i);
    }
    // Last N
    for i in arr.len().saturating_sub(ARRAY_SAMPLE_TAIL)..arr.len() {
        indices.insert(i);
    }
    // Error items
    for (i, item) in arr.iter().enumerate() {
        if has_error_keys(item) {
            indices.insert(i);
        }
    }

    let sampled: Vec<&serde_json::Value> = indices.iter().map(|&i| &arr[i]).collect();
    (sampled, true)
}

// ============================================================
// Human-readable (compact) formatting
// ============================================================

/// Format JSON value as human-readable structure (compact output).
fn format_structure(value: &serde_json::Value, buf: &mut String, depth: usize, max_depth: usize) {
    let indent = "  ".repeat(depth);

    match value {
        serde_json::Value::Null => buf.push_str("Null"),
        serde_json::Value::Bool(_) => buf.push_str("Bool"),
        serde_json::Value::Number(n) => {
            if n.is_f64() && n.as_f64().map_or(false, |f| f.fract() != 0.0) {
                buf.push_str("Float");
            } else {
                buf.push_str("Number");
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                buf.push_str(&format!("String[{}]", s.len()));
            } else {
                buf.push_str("String");
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                buf.push_str("Array[0]");
            } else if depth >= max_depth {
                buf.push_str(&format!("Array[{}]", arr.len()));
            } else {
                let (sampled, was_sampled) = sample_array(arr);

                // Check type homogeneity across *all* items (not just sampled)
                if all_same_type(arr) {
                    buf.push_str(&format!("Array[{}] of ", arr.len()));
                    // Use first sampled item as representative structure
                    format_structure(sampled[0], buf, depth + 1, max_depth);
                } else {
                    buf.push_str(&format!("Array[{}] of Mixed", arr.len()));
                }

                if was_sampled {
                    buf.push_str(&format!(
                        " (sampled: first {} + last {})",
                        ARRAY_SAMPLE_HEAD, ARRAY_SAMPLE_TAIL
                    ));
                }
            }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                buf.push_str("{}");
            } else if depth >= max_depth {
                buf.push_str(&format!("Object{{{} keys}}", map.len()));
            } else {
                buf.push_str("{\n");
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let show_count = if depth == 0 { 30 } else { 15 };
                let total = keys.len();
                for (i, key) in keys.iter().take(show_count).enumerate() {
                    let val = &map[*key];
                    buf.push_str(&format!("{}  \"{}\": ", indent, key));
                    format_structure(val, buf, depth + 1, max_depth);
                    if is_id_key(key) {
                        buf.push_str(" (id)");
                    }
                    if i < total.min(show_count) - 1 {
                        buf.push(',');
                    }
                    buf.push('\n');
                }
                if total > show_count {
                    buf.push_str(&format!("{}  ...+{} more keys\n", indent, total - show_count));
                }
                buf.push_str(&format!("{}}}", indent));
            }
        }
    }
}

/// Check if all elements in a JSON array have the same type.
fn all_same_type(arr: &[serde_json::Value]) -> bool {
    if arr.len() <= 1 {
        return true;
    }
    let first_type = value_type(&arr[0]);
    arr.iter().all(|v| value_type(v) == first_type)
}

/// Get a simple type tag for a JSON value.
fn value_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ============================================================
// JSON schema output
// ============================================================

/// Convert JSON value to a schema-like JSON representation.
fn to_schema_json(
    value: &serde_json::Value,
    depth: usize,
    max_depth: usize,
) -> serde_json::Value {
    use serde_json::json;

    match value {
        serde_json::Value::Null => json!("Null"),
        serde_json::Value::Bool(_) => json!("Bool"),
        serde_json::Value::Number(_) => json!("Number"),
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                json!(format!("String[{}]", s.len()))
            } else {
                json!("String")
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                json!("Array[0]")
            } else if depth >= max_depth {
                json!(format!("Array[{}]", arr.len()))
            } else {
                let (sampled, was_sampled) = sample_array(arr);
                let mut obj = serde_json::Map::new();
                obj.insert("_type".to_string(), json!("Array"));
                obj.insert("_length".to_string(), json!(arr.len()));

                if all_same_type(arr) {
                    obj.insert(
                        "_items".to_string(),
                        to_schema_json(sampled[0], depth + 1, max_depth),
                    );
                } else {
                    obj.insert("_items".to_string(), json!("Mixed"));
                }

                if was_sampled {
                    obj.insert(
                        "_sampled".to_string(),
                        json!(format!("first {} + last {}", ARRAY_SAMPLE_HEAD, ARRAY_SAMPLE_TAIL)),
                    );
                }

                serde_json::Value::Object(obj)
            }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                json!({})
            } else if depth >= max_depth {
                json!(format!("Object{{{} keys}}", map.len()))
            } else {
                let mut schema = serde_json::Map::new();
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for key in keys.iter().take(30) {
                    let val_schema = to_schema_json(&map[*key], depth + 1, max_depth);
                    if is_id_key(key) {
                        // Annotate ID fields in JSON output
                        let annotated = match val_schema {
                            serde_json::Value::String(ref s) => {
                                json!(format!("{} (id)", s))
                            }
                            other => other,
                        };
                        schema.insert((*key).clone(), annotated);
                    } else {
                        schema.insert((*key).clone(), val_schema);
                    }
                }
                if keys.len() > 30 {
                    schema.insert(
                        "_truncated".to_string(),
                        json!(format!("+{} more keys", keys.len() - 30)),
                    );
                }
                serde_json::Value::Object(schema)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"name":"Alice","age":30,"active":true}"#).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("\"name\": String"));
        assert!(buf.contains("\"age\": Number"));
        assert!(buf.contains("\"active\": Bool"));
    }

    #[test]
    fn test_array_of_objects() {
        let json: serde_json::Value =
            serde_json::from_str(r#"[{"id":1},{"id":2},{"id":3}]"#).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[3]"));
    }

    #[test]
    fn test_empty_structures() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"items":[],"meta":{}}"#).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[0]"));
        assert!(buf.contains("{}"));
    }

    #[test]
    fn test_nested_depth_limit() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"a":{"b":{"c":{"d":"deep"}}}}"#).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, 2);
        assert!(buf.contains("Object{1 keys}"));
    }

    #[test]
    fn test_long_string_shows_length() {
        let long = "x".repeat(100);
        let json: serde_json::Value =
            serde_json::from_str(&format!(r#"{{"content":"{}"}}"#, long)).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("String[100]"));
    }

    #[test]
    fn test_null_value() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"value":null}"#).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Null"));
    }

    #[test]
    fn test_schema_json_output() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"name":"test","count":5}"#).unwrap();
        let schema = to_schema_json(&json, 0, usize::MAX);
        assert_eq!(schema["name"], "String");
        assert_eq!(schema["count"], "Number");
    }

    // ============================================================
    // Array sampling tests
    // ============================================================

    #[test]
    fn test_small_array_no_sampling() {
        // Arrays with <=5 items should NOT be sampled
        let json: serde_json::Value = serde_json::from_str(
            r#"[{"x":1},{"x":2},{"x":3},{"x":4},{"x":5}]"#
        ).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[5]"));
        assert!(!buf.contains("sampled"));
    }

    #[test]
    fn test_large_array_sampling() {
        // Arrays with >5 items should show sampling annotation
        let items: Vec<String> = (0..10)
            .map(|i| format!(r#"{{"val":{}}}"#, i))
            .collect();
        let json_str = format!("[{}]", items.join(","));
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[10]"));
        assert!(buf.contains("sampled: first 3 + last 3"));
    }

    #[test]
    fn test_large_array_sampling_numbers() {
        // Homogeneous number array - still sampled
        let json: serde_json::Value =
            serde_json::from_str("[1,2,3,4,5,6,7,8,9,10]").unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[10] of Number"));
        assert!(buf.contains("sampled: first 3 + last 3"));
    }

    #[test]
    fn test_sample_array_preserves_error_items() {
        // Error items in the middle should be preserved during sampling
        let json_str = r#"[
            {"status":"ok"},
            {"status":"ok"},
            {"status":"ok"},
            {"status":"ok"},
            {"error":"something broke","status":"fail"},
            {"status":"ok"},
            {"status":"ok"},
            {"status":"ok"},
            {"status":"ok"},
            {"status":"ok"}
        ]"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        if let serde_json::Value::Array(arr) = &json {
            let (sampled, was_sampled) = sample_array(arr);
            assert!(was_sampled);
            // Should include first 3 + error item (index 4) + last 3
            // first 3: indices 0,1,2; last 3: indices 7,8,9; error: index 4
            assert!(sampled.len() >= 7); // 3 + 3 + 1 error
            // Verify the error item is in the sample
            let has_error = sampled.iter().any(|v| has_error_keys(v));
            assert!(has_error, "Error item should be preserved in sample");
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_sample_array_exactly_at_threshold() {
        // Exactly 5 items: no sampling
        let json: serde_json::Value =
            serde_json::from_str("[1,2,3,4,5]").unwrap();
        if let serde_json::Value::Array(arr) = &json {
            let (_sampled, was_sampled) = sample_array(arr);
            assert!(!was_sampled);
        }
    }

    #[test]
    fn test_sample_array_just_above_threshold() {
        // 6 items: sampling kicks in
        let json: serde_json::Value =
            serde_json::from_str("[1,2,3,4,5,6]").unwrap();
        if let serde_json::Value::Array(arr) = &json {
            let (sampled, was_sampled) = sample_array(arr);
            assert!(was_sampled);
            // first 3 + last 3 = all 6 (overlap), so all items present
            assert_eq!(sampled.len(), 6);
        }
    }

    #[test]
    fn test_large_array_json_schema_output() {
        let items: Vec<String> = (0..10)
            .map(|i| format!(r#"{{"val":{}}}"#, i))
            .collect();
        let json_str = format!("[{}]", items.join(","));
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let schema = to_schema_json(&json, 0, usize::MAX);
        assert_eq!(schema["_type"], "Array");
        assert_eq!(schema["_length"], 10);
        assert!(schema["_sampled"].as_str().unwrap().contains("first 3 + last 3"));
    }

    // ============================================================
    // ID field detection tests
    // ============================================================

    #[test]
    fn test_id_key_detection() {
        assert!(is_id_key("id"));
        assert!(is_id_key("Id"));
        assert!(is_id_key("ID"));
        assert!(is_id_key("uuid"));
        assert!(is_id_key("UUID"));
        assert!(is_id_key("_id"));
        assert!(is_id_key("user_id"));
        assert!(is_id_key("User_Id"));
        assert!(is_id_key("account_id"));
        // Should NOT match
        assert!(!is_id_key("name"));
        assert!(!is_id_key("identity"));
        assert!(!is_id_key("video"));
        assert!(!is_id_key("ideas"));
    }

    #[test]
    fn test_id_annotation_in_structure() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"user_id":123,"name":"Alice","uuid":"abc-def"}"#
        ).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("\"user_id\": Number (id)"));
        assert!(buf.contains("\"uuid\": String (id)"));
        // "name" should NOT have (id)
        assert!(buf.contains("\"name\": String"));
        assert!(!buf.contains("\"name\": String (id)"));
    }

    #[test]
    fn test_id_annotation_nested_object() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"data":{"_id":"abc","title":"test"}}"#
        ).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("\"_id\": String (id)"));
        assert!(!buf.contains("\"title\": String (id)"));
    }

    #[test]
    fn test_id_annotation_json_schema() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"id":1,"name":"test","account_id":42}"#
        ).unwrap();
        let schema = to_schema_json(&json, 0, usize::MAX);
        assert_eq!(schema["id"], "Number (id)");
        assert_eq!(schema["account_id"], "Number (id)");
        assert_eq!(schema["name"], "String");
    }

    // ============================================================
    // Non-JSON rejection with hints
    // ============================================================

    #[test]
    fn test_non_json_hint_toml() {
        let err = serde_json::from_str::<serde_json::Value>("[package]\nname = \"foo\"")
            .unwrap_err();
        let path = Some(PathBuf::from("Cargo.toml"));
        let cmd_err = non_json_hint(&path, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Not JSON"), "got: {}", msg);
                assert!(msg.contains("trs read"), "got: {}", msg);
                assert!(msg.contains("Cargo.toml"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_json_hint_yaml() {
        let err = serde_json::from_str::<serde_json::Value>("key: value\n")
            .unwrap_err();
        let path = Some(PathBuf::from("config.yaml"));
        let cmd_err = non_json_hint(&path, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Not JSON"), "got: {}", msg);
                assert!(msg.contains("trs read"), "got: {}", msg);
                assert!(msg.contains("config.yaml"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_json_hint_yml() {
        let err = serde_json::from_str::<serde_json::Value>("key: value\n")
            .unwrap_err();
        let path = Some(PathBuf::from("config.yml"));
        let cmd_err = non_json_hint(&path, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Not JSON"), "got: {}", msg);
                assert!(msg.contains("trs read"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_json_hint_csv() {
        let err = serde_json::from_str::<serde_json::Value>("a,b,c\n1,2,3\n")
            .unwrap_err();
        let path = Some(PathBuf::from("data.csv"));
        let cmd_err = non_json_hint(&path, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Not JSON"), "got: {}", msg);
                assert!(msg.contains("trs parse deps"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_json_hint_unknown_ext() {
        let err = serde_json::from_str::<serde_json::Value>("not json at all")
            .unwrap_err();
        let path = Some(PathBuf::from("data.txt"));
        let cmd_err = non_json_hint(&path, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Invalid JSON"), "got: {}", msg);
                assert!(!msg.contains("Not JSON"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_json_hint_no_file() {
        let err = serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err();
        let cmd_err = non_json_hint(&None, &err);
        match cmd_err {
            CommandError::InvalidArguments(msg) => {
                assert!(msg.contains("Invalid JSON"), "got: {}", msg);
            }
            other => panic!("Expected InvalidArguments, got: {:?}", other),
        }
    }

    // ============================================================
    // Error key detection tests
    // ============================================================

    #[test]
    fn test_has_error_keys() {
        let obj: serde_json::Value =
            serde_json::from_str(r#"{"error":"bad","code":500}"#).unwrap();
        assert!(has_error_keys(&obj));

        let obj: serde_json::Value =
            serde_json::from_str(r#"{"exception":"NPE","trace":"..."}"#).unwrap();
        assert!(has_error_keys(&obj));

        let obj: serde_json::Value =
            serde_json::from_str(r#"{"failed":true}"#).unwrap();
        assert!(has_error_keys(&obj));

        let obj: serde_json::Value =
            serde_json::from_str(r#"{"failure_reason":"timeout"}"#).unwrap();
        assert!(has_error_keys(&obj));

        let obj: serde_json::Value =
            serde_json::from_str(r#"{"name":"ok","status":"good"}"#).unwrap();
        assert!(!has_error_keys(&obj));

        // Non-objects never match
        let arr: serde_json::Value = serde_json::from_str("[1,2,3]").unwrap();
        assert!(!has_error_keys(&arr));
    }

    // ============================================================
    // Combined features test
    // ============================================================

    #[test]
    fn test_combined_large_array_with_id_fields() {
        // Large array of objects with ID fields: should sample + annotate IDs
        let items: Vec<String> = (0..10)
            .map(|i| format!(r#"{{"user_id":{},"name":"user{}"}}"#, i, i))
            .collect();
        let json_str = format!(r#"{{"users":{}}}"#, format!("[{}]", items.join(",")));
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let mut buf = String::new();
        format_structure(&json, &mut buf, 0, usize::MAX);
        assert!(buf.contains("Array[10]"), "should show array length");
        assert!(buf.contains("sampled"), "should indicate sampling");
        assert!(buf.contains("(id)"), "should annotate id fields");
    }
}
