use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_download(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        // Bare `curl URL` (no -v / -I) gives us the response body
        // without the HTTP preamble. The rest of this handler expects
        // verbose output, so when there's no protocol marker, route
        // to the body-content compressor and return early.
        if !looks_like_http_trace(&input) {
            let output = compress_http_body(&input);
            print!("{}", output);
            if ctx.stats {
                let reducer = if output.len() == input.len() {
                    "curl-passthrough"
                } else {
                    "curl-body"
                };
                CommandStats::new()
                    .with_reducer(reducer)
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(output.len())
                    .print();
            }
            return Ok(());
        }

        let mut status_code = String::new();
        let mut status_text = String::new();
        let mut url = String::new();
        let mut content_type = String::new();
        let mut content_length = String::new();
        let mut redirect_url = String::new();
        let mut headers: Vec<(String, String)> = Vec::new();
        let mut is_head_request = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip progress bars and connection noise
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.contains("###") || trimmed.contains("ETA") {
                continue;
            }
            if trimmed.contains('%')
                && (trimmed.contains("Dload")
                    || trimmed.contains("Upload")
                    || trimmed.contains("Total")
                    || trimmed.contains("Received")
                    || trimmed.contains("Average"))
            {
                continue;
            }
            // Skip curl progress lines (e.g., "  0  1234    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0")
            if trimmed.starts_with("0 ") || trimmed.starts_with("100 ") {
                if trimmed.contains("--:--:--") || trimmed.contains("0:00:") {
                    continue;
                }
            }

            // Skip repeated connection info
            if trimmed.starts_with("* ") {
                // Keep URL-related lines
                if trimmed.contains("Connected to") || trimmed.contains("Trying") {
                    continue;
                }
                if trimmed.contains("TLS")
                    || trimmed.contains("SSL")
                    || trimmed.contains("ALPN")
                    || trimmed.contains("CAfile")
                    || trimmed.contains("CApath")
                {
                    continue;
                }
                continue;
            }

            // curl verbose: "> GET /path" or "> HEAD /path"
            if trimmed.starts_with("> ") {
                let req_line = &trimmed[2..];
                if req_line.starts_with("HEAD ") {
                    is_head_request = true;
                }
                continue;
            }

            // curl verbose response headers: "< HTTP/1.1 200 OK"
            if trimmed.starts_with("< ") {
                let header_line = trimmed[2..].trim();
                if header_line.starts_with("HTTP/") {
                    // Parse status line: "HTTP/1.1 200 OK"
                    let parts: Vec<&str> = header_line.splitn(3, ' ').collect();
                    if parts.len() >= 2 {
                        status_code = parts[1].to_string();
                        status_text = if parts.len() >= 3 {
                            parts[2].to_string()
                        } else {
                            String::new()
                        };
                    }
                } else if let Some(colon) = header_line.find(':') {
                    let key = header_line[..colon].trim().to_lowercase();
                    let val = header_line[colon + 1..].trim().to_string();
                    match key.as_str() {
                        "content-type" => content_type = val.clone(),
                        "content-length" => content_length = val.clone(),
                        "location" => redirect_url = val.clone(),
                        _ => {}
                    }
                    headers.push((key, val));
                }
                continue;
            }

            // Raw HTTP headers (curl -I output without "< " prefix)
            // e.g., "HTTP/2 200" or "content-type: text/html"
            if trimmed.starts_with("HTTP/") {
                let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
                if parts.len() >= 2 {
                    status_code = parts[1].to_string();
                    status_text = if parts.len() >= 3 {
                        parts[2].to_string()
                    } else {
                        String::new()
                    };
                }
                is_head_request = true;
                continue;
            }
            if trimmed.contains(':') && !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                if let Some(colon) = trimmed.find(':') {
                    let key = trimmed[..colon].trim().to_lowercase();
                    // Only treat as header if key looks like a header name (no spaces, short)
                    if !key.contains(' ') && key.len() < 30 {
                        let val = trimmed[colon + 1..].trim().to_string();
                        match key.as_str() {
                            "content-type" => content_type = val.clone(),
                            "content-length" => content_length = val.clone(),
                            "location" => redirect_url = val.clone(),
                            _ => {}
                        }
                        headers.push((key, val));
                        continue;
                    }
                }
            }

            // wget style: "HTTP request sent, awaiting response... 200 OK"
            if trimmed.contains("awaiting response...") {
                if let Some(pos) = trimmed.find("... ") {
                    let rest = &trimmed[pos + 4..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if !parts.is_empty() {
                        status_code = parts[0].to_string();
                        if parts.len() >= 2 {
                            status_text = parts[1].to_string();
                        }
                    }
                }
                continue;
            }

            // wget: "Length: 12345 (12K) [text/html]"
            if trimmed.starts_with("Length:") {
                let rest = &trimmed[7..].trim();
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if !parts.is_empty() {
                    content_length = parts[0].to_string();
                }
                if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.find(']')) {
                    content_type = trimmed[start + 1..end].to_string();
                }
                continue;
            }

            // wget: URL from "Saving to:" or "--<date>-- <url>"
            if trimmed.starts_with("--") && trimmed.contains("http") {
                if let Some(http_pos) = trimmed.find("http") {
                    url = trimmed[http_pos..].to_string();
                }
                continue;
            }

            // wget: "Location: <url>"
            if trimmed.starts_with("Location:") {
                redirect_url = trimmed[9..].trim().to_string();
                continue;
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let mut map = serde_json::Map::new();
                if !status_code.is_empty() {
                    map.insert(
                        "status_code".to_string(),
                        serde_json::Value::String(status_code.clone()),
                    );
                }
                if !status_text.is_empty() {
                    map.insert(
                        "status_text".to_string(),
                        serde_json::Value::String(status_text.clone()),
                    );
                }
                if !url.is_empty() {
                    map.insert("url".to_string(), serde_json::Value::String(url.clone()));
                }
                if !content_type.is_empty() {
                    map.insert(
                        "content_type".to_string(),
                        serde_json::Value::String(content_type.clone()),
                    );
                }
                if !content_length.is_empty() {
                    map.insert(
                        "content_length".to_string(),
                        serde_json::Value::String(content_length.clone()),
                    );
                }
                if !redirect_url.is_empty() {
                    map.insert(
                        "redirect_url".to_string(),
                        serde_json::Value::String(redirect_url.clone()),
                    );
                }
                serde_json::Value::Object(map).to_string()
            }
            _ => {
                if is_head_request
                    || (!status_code.is_empty()
                        && content_type.is_empty()
                        && content_length.is_empty())
                {
                    // curl -I style: show status + important headers
                    let mut out = String::new();
                    if !status_code.is_empty() {
                        out.push_str(&format!("{} {}\n", status_code, status_text));
                    }
                    for (key, val) in &headers {
                        match key.as_str() {
                            "content-type" | "content-length" | "location" | "server"
                            | "cache-control" | "etag" | "last-modified" | "date" => {
                                out.push_str(&format!("{}: {}\n", key, val));
                            }
                            _ => {}
                        }
                    }
                    out
                } else {
                    // Compact single-line summary
                    let mut out = String::new();
                    if !status_code.is_empty() {
                        out.push_str(&format!("{} {}", status_code, status_text));
                    }
                    if !url.is_empty() {
                        out.push_str(&format!(" {}", url));
                    }
                    if !content_type.is_empty() || !content_length.is_empty() {
                        out.push_str(" (");
                        let mut parts: Vec<String> = Vec::new();
                        if !content_type.is_empty() {
                            // Simplify content type (remove charset etc.)
                            let ct = content_type
                                .split(';')
                                .next()
                                .unwrap_or(&content_type)
                                .trim();
                            parts.push(ct.to_string());
                        }
                        if !content_length.is_empty() {
                            if let Ok(bytes) = content_length.parse::<u64>() {
                                parts.push(Self::format_human_size(bytes));
                            } else {
                                parts.push(content_length.clone());
                            }
                        }
                        out.push_str(&parts.join(", "));
                        out.push(')');
                    }
                    if !redirect_url.is_empty() {
                        out.push_str(&format!(" -> {}", redirect_url));
                    }
                    out.push('\n');
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("download")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

/// True when the input looks like `curl -v` / `curl -I` output — i.e.
/// contains the HTTP protocol markers (`> GET`, `< HTTP/`, or raw
/// header status lines). Plain `curl URL` bodies don't, so we use
/// this check to decide whether to run the body-oriented compressor
/// or the protocol-oriented one.
fn looks_like_http_trace(input: &str) -> bool {
    input.lines().take(20).any(|l| {
        let t = l.trim_start();
        t.starts_with("> ")
            || t.starts_with("< HTTP/")
            || t.starts_with("HTTP/1.")
            || t.starts_with("HTTP/2")
            || t.starts_with("* Connected to")
            || t.starts_with("* Trying")
    })
}

/// Compress a raw HTTP body (no protocol preamble). Detects common
/// content shapes and reduces accordingly:
///
/// - **JSON** (starts with `{` or `[`) → re-emit as compact JSON.
///   GitHub API responses go from pretty-printed ~12KB down to ~3-4KB.
/// - **GitHub API base64-encoded content** (`{"content": "...", "encoding": "base64"}`)
///   → decode and return the decoded text. `gh api /repos/X/contents/README.md`
///   is the common offender here.
/// - **HTML** (starts with `<!DOCTYPE` or `<html`) → passthrough for
///   now; the html2md handler is the dedicated tool.
/// - **Anything else** → passthrough. Don't touch markdown, plain
///   text, or binary payloads — agents may need them exact.
fn compress_http_body(input: &str) -> String {
    let trimmed = input.trim_start();
    let first_byte = trimmed.as_bytes().first().copied();

    if matches!(first_byte, Some(b'{') | Some(b'[')) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            // GitHub API contents endpoint returns base64-encoded
            // file content. Decode it so the agent sees the actual
            // text rather than a multi-kilobyte base64 blob.
            if let Some(decoded) = decode_github_content(&val) {
                return decoded;
            }
            // Otherwise: compact JSON. Preserves all fields but drops
            // pretty-printing whitespace, which is where most of the
            // body bytes sit.
            if let Ok(compact) = serde_json::to_string(&val) {
                return format!("{}\n", compact);
            }
        }
    }

    // Fall-through: body is text we don't specifically handle. Keep
    // as-is — arbitrary compression here would risk corrupting
    // responses the agent needs byte-exact.
    input.to_string()
}

/// Detect and decode a `gh api` / GitHub API `contents` response
/// that carries a `"content": "<base64>"` + `"encoding": "base64"`
/// field pair. Returns the decoded text when it's successfully
/// decoded AND valid UTF-8; otherwise `None` and the caller falls
/// back to compact-JSON rendering.
fn decode_github_content(val: &serde_json::Value) -> Option<String> {
    let obj = val.as_object()?;
    let encoding = obj.get("encoding")?.as_str()?;
    if encoding != "base64" {
        return None;
    }
    let content = obj.get("content")?.as_str()?;
    // GitHub wraps base64 at 60 columns with newlines; strip those.
    let cleaned: String = content.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64_decode(&cleaned)?;
    let text = String::from_utf8(bytes).ok()?;
    Some(text)
}

/// Minimal base64 decoder. Standard alphabet, optional padding.
/// Returns `None` on any invalid character — safer to fall through
/// to compact-JSON than to emit garbage to the agent.
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [255u8; 256];
    for (i, &b) in ALPHABET.iter().enumerate() {
        table[b as usize] = i as u8;
    }
    // '=' is padding; treated as zero here but stripped at the end.
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in s.as_bytes() {
        let v = table[b as usize];
        if v == 255 {
            return None;
        }
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_trace() {
        let verbose = "* Connected to api.github.com\n> GET /\n< HTTP/1.1 200 OK\n";
        assert!(looks_like_http_trace(verbose));
        let body_only = "# Hello\nSome markdown content\n";
        assert!(!looks_like_http_trace(body_only));
    }

    #[test]
    fn compact_json_body_reduces_size() {
        let pretty = r#"{
    "name": "trs",
    "version": "0.5.8",
    "tags": ["cli", "rust", "ai"]
}"#;
        let out = compress_http_body(pretty);
        assert!(out.len() < pretty.len());
        assert!(out.contains("\"version\":\"0.5.8\""));
    }

    #[test]
    fn github_contents_base64_is_decoded() {
        // "hello world" base64 = aGVsbG8gd29ybGQ=
        let input = r#"{"name":"README.md","encoding":"base64","content":"aGVsbG8gd29ybGQ="}"#;
        let out = compress_http_body(input);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn unknown_body_is_passthrough() {
        let markdown = "# Header\n\nParagraph text.\n";
        assert_eq!(compress_http_body(markdown), markdown);
    }

    #[test]
    fn base64_decode_handles_padding() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
    }

    #[test]
    fn base64_decode_rejects_invalid() {
        assert!(base64_decode("not valid!").is_none());
    }
}
