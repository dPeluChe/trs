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
