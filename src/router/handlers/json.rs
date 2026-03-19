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

#[path = "json_query.rs"]
mod json_query;

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
    /// Query path to extract (e.g. ".users[0].name")
    pub query: Option<String>,
}

pub(crate) struct JsonHandler;

impl JsonHandler {
    pub(crate) fn execute(&self, input: &JsonInput, ctx: &CommandContext) -> CommandResult {
        let raw = read_json_input(&input.file)?;
        let input_bytes = raw.len();

        let parsed: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| non_json_hint(&input.file, &e))?;

        // Query mode: extract value at path
        if let Some(ref query) = input.query {
            let result = json_query::resolve_query(&parsed, query)?;
            let output = json_query::format_query_result(&result);
            print!("{}", output);
            if ctx.stats {
                CommandStats::new()
                    .with_reducer("json-query")
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(output.len())
                    .print();
            }
            return Ok(());
        }

        // Schema mode (default)
        let max_depth = input.depth.unwrap_or(usize::MAX);

        let output = match ctx.format {
            OutputFormat::Json => {
                let schema = to_schema_json(&parsed, 0, max_depth);
                serde_json::to_string_pretty(&schema).unwrap_or_else(|_| schema.to_string())
            }
            _ => {
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
                "toml" => Some(format!("Not JSON. Try: trs read {}", path.display())),
                "yaml" | "yml" => Some(format!("Not JSON. Try: trs read {}", path.display())),
                "csv" => Some("Not JSON. Try: trs parse deps (for dependency lists)".to_string()),
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
                    buf.push_str(&format!(
                        "{}  ...+{} more keys\n",
                        indent,
                        total - show_count
                    ));
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
fn to_schema_json(value: &serde_json::Value, depth: usize, max_depth: usize) -> serde_json::Value {
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
                        json!(format!(
                            "first {} + last {}",
                            ARRAY_SAMPLE_HEAD, ARRAY_SAMPLE_TAIL
                        )),
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

// ============================================================

#[cfg(test)]
#[path = "json_tests.rs"]
mod tests;
