//! Shared helper functions for formatters.

/// Format a count with label, only showing if count > 0.
#[allow(dead_code)]
pub fn format_count_if_positive(label: &str, count: usize) -> Option<String> {
    if count > 0 {
        Some(format!("{}={}", label, count))
    } else {
        None
    }
}

/// Format a list of items with a header and count.
#[allow(dead_code)]
pub fn format_list_with_count(label: &str, items: &[String]) -> String {
    let mut output = String::new();
    if !items.is_empty() {
        output.push_str(&format!("{} ({}):\n", label, items.len()));
        for item in items {
            output.push_str(&format!("  {}\n", item));
        }
    }
    output
}

/// Format a key-value pair with optional label.
#[allow(dead_code)]
pub fn format_key_value(key: &str, value: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{} [{}]: {}\n", key, l, value),
        None => format!("{}: {}\n", key, value),
    }
}

/// Format a simple key-value line.
#[allow(dead_code)]
pub fn format_line(key: &str, value: impl std::fmt::Display) -> String {
    format!("{}: {}\n", key, value)
}

/// Truncate a string to a maximum length with ellipsis.
#[allow(dead_code)]
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format a duration in human-readable form.
#[allow(dead_code)]
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Format a byte count in human-readable form.
#[allow(dead_code)]
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
