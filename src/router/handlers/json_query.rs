use super::super::common::{CommandError, CommandResult};
/// A segment of a JSON query path.
#[derive(Debug)]
enum QuerySegment {
    /// Object key: .name
    Key(String),
    /// Array index: [0], [2]
    Index(usize),
    /// Map over array: .[] or .[].field
    Iterate,
}

/// Parse a query string like ".users[0].name" or ".[].id" into segments.
fn parse_query(query: &str) -> Result<Vec<QuerySegment>, CommandError> {
    let mut segments = Vec::new();
    let q = query.trim();
    let q = q.strip_prefix('.').unwrap_or(q);

    if q.is_empty() {
        return Ok(segments);
    }

    let mut chars = q.chars().peekable();
    let mut current_key = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '.' => {
                chars.next();
                if !current_key.is_empty() {
                    segments.push(QuerySegment::Key(current_key.clone()));
                    current_key.clear();
                }
            }
            '[' => {
                chars.next();
                if !current_key.is_empty() {
                    segments.push(QuerySegment::Key(current_key.clone()));
                    current_key.clear();
                }
                // Collect until ]
                let mut idx_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    }
                    idx_str.push(c);
                    chars.next();
                }
                if idx_str.is_empty() {
                    segments.push(QuerySegment::Iterate);
                } else if let Ok(idx) = idx_str.parse::<usize>() {
                    segments.push(QuerySegment::Index(idx));
                } else {
                    // Treat as key (for ["key"] syntax)
                    let key = idx_str.trim_matches('"').trim_matches('\'').to_string();
                    segments.push(QuerySegment::Key(key));
                }
            }
            _ => {
                current_key.push(ch);
                chars.next();
            }
        }
    }
    if !current_key.is_empty() {
        segments.push(QuerySegment::Key(current_key));
    }

    Ok(segments)
}

/// Resolve a query path against a JSON value.
pub(crate) fn resolve_query(
    value: &serde_json::Value,
    query: &str,
) -> CommandResult<serde_json::Value> {
    let segments = parse_query(query)?;
    resolve_segments(value, &segments, 0)
}

/// Recursively resolve query segments.
fn resolve_segments(
    value: &serde_json::Value,
    segments: &[QuerySegment],
    pos: usize,
) -> CommandResult<serde_json::Value> {
    if pos >= segments.len() {
        return Ok(value.clone());
    }

    match &segments[pos] {
        QuerySegment::Key(key) => match value {
            serde_json::Value::Object(map) => {
                let val = map.get(key).ok_or_else(|| {
                    CommandError::InvalidArguments(format!("Key '{}' not found", key))
                })?;
                resolve_segments(val, segments, pos + 1)
            }
            _ => Err(CommandError::InvalidArguments(format!(
                "Cannot access .{} on {}",
                key,
                value_type(value)
            ))),
        },
        QuerySegment::Index(idx) => match value {
            serde_json::Value::Array(arr) => {
                let val = arr.get(*idx).ok_or_else(|| {
                    CommandError::InvalidArguments(format!(
                        "Index [{}] out of bounds (array has {} items)",
                        idx,
                        arr.len()
                    ))
                })?;
                resolve_segments(val, segments, pos + 1)
            }
            _ => Err(CommandError::InvalidArguments(format!(
                "Cannot index [{}] on {}",
                idx,
                value_type(value)
            ))),
        },
        QuerySegment::Iterate => match value {
            serde_json::Value::Array(arr) => {
                let remaining = &segments[pos + 1..];
                if remaining.is_empty() {
                    // Just return the array
                    Ok(value.clone())
                } else {
                    // Map over each element
                    let results: Result<Vec<serde_json::Value>, CommandError> = arr
                        .iter()
                        .map(|item| resolve_segments(item, remaining, 0))
                        .collect();
                    Ok(serde_json::Value::Array(results?))
                }
            }
            _ => Err(CommandError::InvalidArguments(format!(
                "Cannot iterate [] on {}",
                value_type(value)
            ))),
        },
    }
}

/// Format query result compactly.
pub(crate) fn format_query_result(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("{}\n", s),
        serde_json::Value::Array(arr) => {
            // Flat arrays of primitives: one per line
            if arr
                .iter()
                .all(|v| v.is_string() || v.is_number() || v.is_boolean())
            {
                arr.iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n"
            } else {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()) + "\n"
            }
        }
        serde_json::Value::Null => "null\n".to_string(),
        serde_json::Value::Bool(b) => format!("{}\n", b),
        serde_json::Value::Number(n) => format!("{}\n", n),
        serde_json::Value::Object(_) => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()) + "\n"
        }
    }
}

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
