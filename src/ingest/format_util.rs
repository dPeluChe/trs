//! Presentation helpers for the ingest digest: byte and token counts,
//! dates, and the HTML strip that keeps embedded markup out of the
//! markdown. Split out of `format.rs`, which is about assembling the
//! digest, not about rendering individual values.

/// Strip HTML tags from markdown content (badges, images, formatting).
/// Preserves markdown content, removes <p>, <a>, <img>, etc.
pub(crate) fn strip_html_from_markdown(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut prev_blank = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Strip code fence markers (```bash, ```json, etc.) -- just noise in a digest
        if trimmed.starts_with("```") {
            continue;
        }

        // Skip lines that are pure HTML (start with < and end with > or are self-closing)
        if trimmed.starts_with('<') && (trimmed.ends_with('>') || trimmed.ends_with("/>")) {
            // Keep HTML comments (<!-- -->)
            if trimmed.starts_with("<!--") {
                result.push_str(line);
                result.push('\n');
                prev_blank = false;
            }
            // Skip everything else (badges, images, alignment tags)
            continue;
        }

        // Skip img tags and badge lines
        if trimmed.contains("<img ") && trimmed.contains("src=") {
            continue;
        }
        if trimmed.starts_with("[![") || (trimmed.starts_with("[!") && trimmed.contains("](http")) {
            continue;
        }

        // Collapse consecutive blank lines
        if trimmed.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
            continue;
        }

        prev_blank = false;

        // Clean markdown formatting noise
        let mut cleaned = line.to_string();
        // Strip bold markers: **text** -> text
        while cleaned.contains("**") {
            cleaned = cleaned.replacen("**", "", 2);
        }
        // Strip inline code in doc context: `foo` -> foo
        // (keep backticks that wrap actual code references)
        // Convert bullet markers: *   text -> - text
        if cleaned.trim_start().starts_with("*   ") || cleaned.trim_start().starts_with("*  ") {
            let indent = cleaned.len() - cleaned.trim_start().len();
            cleaned = format!(
                "{}- {}",
                " ".repeat(indent),
                cleaned.trim_start().trim_start_matches('*').trim()
            );
        }

        result.push_str(&cleaned);
        result.push('\n');
    }

    result
}

pub(crate) fn format_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{}B", n)
    } else if n < 1024 * 1024 {
        format!("{:.1}KB", n as f64 / 1024.0)
    } else {
        format!("{:.1}MB", n as f64 / (1024.0 * 1024.0))
    }
}

/// Extract a file's content from the digest by filename.
pub(super) fn extract_section(digest: &str, filename: &str) -> Option<String> {
    let marker = format!("<!-- file: {} -->", filename);
    let start = digest.find(&marker)?;
    // Find the code block after the marker
    let after_marker = &digest[start..];
    let code_start = after_marker.find("```")? + 3;
    let code_content_start = after_marker[code_start..].find('\n')? + code_start + 1;
    let code_end = after_marker[code_content_start..].find("```")? + code_content_start;
    Some(after_marker[code_content_start..code_end].to_string())
}

/// Convert days since Unix epoch to (year, month, day).
pub(super) fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub(crate) fn format_tokens(n: usize) -> String {
    if n < 1000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

pub(super) fn format_modified(entry: &std::fs::DirEntry) -> String {
    let modified = entry
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(modified);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        let (y, mo, day) = days_to_date(modified / 86400);
        format!("{:04}-{:02}-{:02}", y, mo, day)
    }
}
