//! JSON structure handler for `trs json` command.
//!
//! Reads JSON from a file or stdin and outputs the structure (keys + types + array lengths)
//! without values. Reduces large JSON payloads to a compact schema overview.
//!
//! Example:
//! ```text
//! $ cat api-response.json | trs json
//! {
//!   "data": Array[42],
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
            CommandError::InvalidArguments(format!("Invalid JSON: {}", e))
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
                // Show the type of the first element as representative
                buf.push_str(&format!("Array[{}] of ", arr.len()));
                // Check if all elements have the same type
                if all_same_type(arr) {
                    format_structure(&arr[0], buf, depth + 1, max_depth);
                } else {
                    buf.push_str("Mixed");
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
            } else if all_same_type(arr) {
                json!({
                    "_type": "Array",
                    "_length": arr.len(),
                    "_items": to_schema_json(&arr[0], depth + 1, max_depth)
                })
            } else {
                json!({
                    "_type": "Array",
                    "_length": arr.len(),
                    "_items": "Mixed"
                })
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
                    schema.insert(
                        (*key).clone(),
                        to_schema_json(&map[*key], depth + 1, max_depth),
                    );
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
}
