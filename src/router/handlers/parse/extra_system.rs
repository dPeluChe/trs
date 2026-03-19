use super::super::common::{CommandContext, CommandResult, CommandStats};
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    pub(crate) fn handle_tree(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();
        let mut total_dirs = 0usize;
        let mut total_files = 0usize;

        for line in input.lines() {
            if line.contains("director") && line.contains("file") {
                for part in line.split(',') {
                    let t = part.trim();
                    if t.ends_with("directories") || t.ends_with("directory") { total_dirs = t.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0); }
                    else if t.ends_with("files") || t.ends_with("file") { total_files = t.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0); }
                }
                continue;
            }
            let name: String = line.chars().filter(|c| !matches!(c, '│' | '├' | '└' | '─')).collect::<String>().trim().to_string();
            if name.is_empty() || name == "." { continue; }
            if name.ends_with('/') { dirs.push(name.trim_end_matches('/').to_string()); }
            else { files.push(name); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"directories": dirs, "files": files, "total_directories": total_dirs, "total_files": total_files}).to_string(),
            _ => {
                let mut out = format!("{} directories, {} files\n", total_dirs, total_files);
                if !dirs.is_empty() { out.push_str(&format!("dirs:")); for d in dirs.iter().take(20) { out.push_str(&format!(" {}", d)); } if dirs.len() > 20 { out.push_str(&format!(" ...+{}", dirs.len()-20)); } out.push('\n'); }
                if !files.is_empty() { out.push_str(&format!("files:")); for f in files.iter().take(30) { out.push_str(&format!(" {}", f)); } if files.len() > 30 { out.push_str(&format!(" ...+{}", files.len()-30)); } out.push('\n'); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("tree").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_docker_ps(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut containers: Vec<serde_json::Value> = Vec::new();
        let lines: Vec<&str> = input.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let id = parts[0];
                    let image = parts[1];
                    let status = line.find("Up ").or_else(|| line.find("Exited ")).map(|s| line[s..].split("  ").next().unwrap_or("").trim()).unwrap_or("unknown");
                    let name = parts.last().unwrap_or(&"");
                    containers.push(serde_json::json!({"id": id, "image": image, "status": status, "name": name}));
                }
            }
        }
        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"containers": containers, "count": containers.len()}).to_string(),
            _ => {
                let mut out = format!("containers: {}\n", containers.len());
                for c in &containers {
                    let st = c["status"].as_str().unwrap_or("");
                    let mk = if st.starts_with("Up") { "+" } else { "-" };
                    out.push_str(&format!("  {} {} {} ({})\n", mk, c["name"].as_str().unwrap_or(""), c["image"].as_str().unwrap_or(""), st));
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("docker-ps").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_docker_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        Self::handle_logs(file, ctx)
    }

    pub(crate) fn handle_deps(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut deps: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            let clean = line.replace('│', "").replace('├', "").replace('└', "").replace('─', "").replace("deduped", "").trim().to_string();
            if clean.is_empty() { continue; }
            // npm: name@version
            if let Some(at) = clean.rfind('@') {
                if at > 0 { let n = clean[..at].trim().to_string(); let v = clean[at+1..].trim().to_string(); if !n.is_empty() { deps.push((n, v)); continue; } }
            }
            // pip/cargo: name version
            let parts: Vec<&str> = clean.split_whitespace().collect();
            if parts.len() >= 2 {
                let n = parts[0].to_string();
                if n == "Package" || n == "---" || n.starts_with("==") { continue; }
                deps.push((n, parts[1].trim_start_matches('v').to_string()));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => { let jd: Vec<serde_json::Value> = deps.iter().map(|(n,v)| serde_json::json!({"name":n,"version":v})).collect(); serde_json::json!({"dependencies": jd, "count": deps.len()}).to_string() }
            _ => { let mut out = format!("dependencies: {}\n", deps.len()); for (n,v) in &deps { if v.is_empty() { out.push_str(&format!("  {}\n", n)); } else { out.push_str(&format!("  {}@{}\n", n, v)); } } out }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("deps").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(deps.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_install(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut added: Vec<String> = Vec::new();
        let mut warnings = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut summary = String::new();

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() { continue; }
            let lower = t.to_lowercase();
            if (lower.contains("added") || lower.contains("removed")) && lower.contains("package") { summary = t.to_string(); }
            else if lower.starts_with("successfully installed") { for pkg in t.split_whitespace().skip(2) { added.push(pkg.to_string()); } }
            else if lower.starts_with("npm warn") || lower.starts_with("warn ") { warnings += 1; }
            else if lower.starts_with("npm error") || lower.starts_with("error") || lower.starts_with("err!") { errors.push(t.to_string()); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"summary": summary, "added": added, "added_count": added.len(), "warnings": warnings, "errors": errors.len()}).to_string(),
            _ => {
                let mut out = String::new();
                if !summary.is_empty() { out.push_str(&format!("{}\n", summary)); }
                if !added.is_empty() { out.push_str(&format!("added ({}): {}\n", added.len(), added.join(", "))); }
                if warnings > 0 { out.push_str(&format!("warnings: {}\n", warnings)); }
                if !errors.is_empty() { out.push_str(&format!("errors ({}):\n", errors.len())); for e in &errors { out.push_str(&format!("  {}\n", e)); } }
                if out.is_empty() { out = "install: ok\n".to_string(); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("install").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_build(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut info_last = String::new();
        let mut success = true;

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() { continue; }
            let lower = t.to_lowercase();
            if lower.contains("error[") || lower.starts_with("error:") || lower.contains("): error ") || lower.contains(": error:") {
                errors.push(t.to_string()); success = false;
            } else if lower.contains("warning[") || lower.starts_with("warning:") || lower.contains(": warning:") {
                warnings.push(t.to_string());
            } else if lower.starts_with("compiling ") || lower.starts_with("finished ") {
                info_last = t.to_string();
            }
        }
        warnings.dedup();

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"success": success, "errors": errors, "error_count": errors.len(), "warnings": warnings, "warning_count": warnings.len()}).to_string(),
            _ => {
                let mut out = format!("build: {} ({} errors, {} warnings)\n", if success {"ok"} else {"FAILED"}, errors.len(), warnings.len());
                if !errors.is_empty() { out.push_str(&format!("errors ({}):\n", errors.len())); for e in errors.iter().take(20) { out.push_str(&format!("  {}\n", e)); } if errors.len() > 20 { out.push_str(&format!("  ...+{} more\n", errors.len()-20)); } }
                if !warnings.is_empty() { out.push_str(&format!("warnings ({}):\n", warnings.len())); for w in warnings.iter().take(10) { out.push_str(&format!("  {}\n", w)); } if warnings.len() > 10 { out.push_str(&format!("  ...+{} more\n", warnings.len()-10)); } }
                if !info_last.is_empty() { out.push_str(&format!("{}\n", info_last)); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("build").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_wc(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut entries: Vec<(String, u64, u64, u64)> = Vec::new(); // (name, lines, words, bytes)

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            match parts.len() {
                // wc with file: lines words bytes filename
                4 => {
                    let lines = parts[0].parse::<u64>().unwrap_or(0);
                    let words = parts[1].parse::<u64>().unwrap_or(0);
                    let bytes = parts[2].parse::<u64>().unwrap_or(0);
                    let name = parts[3].to_string();
                    entries.push((name, lines, words, bytes));
                }
                // wc from stdin (no filename): lines words bytes
                3 => {
                    let lines = parts[0].parse::<u64>().unwrap_or(0);
                    let words = parts[1].parse::<u64>().unwrap_or(0);
                    let bytes = parts[2].parse::<u64>().unwrap_or(0);
                    entries.push((String::new(), lines, words, bytes));
                }
                _ => continue,
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jv: Vec<serde_json::Value> = entries.iter().map(|(name, lines, words, bytes)| {
                    serde_json::json!({"file": name, "lines": lines, "words": words, "bytes": bytes})
                }).collect();
                serde_json::json!({"entries": jv, "count": entries.len()}).to_string()
            }
            _ => {
                let mut out = String::new();
                for (name, lines, words, bytes) in &entries {
                    if name.is_empty() {
                        out.push_str(&format!("{}L {}W {}B\n", lines, words, bytes));
                    } else if name == "total" {
                        out.push_str(&format!("total: {}L {}W {}B\n", lines, words, bytes));
                    } else {
                        out.push_str(&format!("{} {}L {}W {}B\n", name, lines, words, bytes));
                    }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("wc").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(entries.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_download(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
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
            if trimmed.is_empty() { continue; }
            if trimmed.contains("###") || trimmed.contains("ETA") { continue; }
            if trimmed.contains('%') && (trimmed.contains("Dload") || trimmed.contains("Upload") || trimmed.contains("Total") || trimmed.contains("Received") || trimmed.contains("Average")) { continue; }
            // Skip curl progress lines (e.g., "  0  1234    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0")
            if trimmed.starts_with("0 ") || trimmed.starts_with("100 ") {
                if trimmed.contains("--:--:--") || trimmed.contains("0:00:") { continue; }
            }

            // Skip repeated connection info
            if trimmed.starts_with("* ") {
                // Keep URL-related lines
                if trimmed.contains("Connected to") || trimmed.contains("Trying") {
                    continue;
                }
                if trimmed.contains("TLS") || trimmed.contains("SSL") || trimmed.contains("ALPN") || trimmed.contains("CAfile") || trimmed.contains("CApath") {
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
                        status_text = if parts.len() >= 3 { parts[2].to_string() } else { String::new() };
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
                    status_text = if parts.len() >= 3 { parts[2].to_string() } else { String::new() };
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
                        if parts.len() >= 2 { status_text = parts[1].to_string(); }
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
                if !status_code.is_empty() { map.insert("status_code".to_string(), serde_json::Value::String(status_code.clone())); }
                if !status_text.is_empty() { map.insert("status_text".to_string(), serde_json::Value::String(status_text.clone())); }
                if !url.is_empty() { map.insert("url".to_string(), serde_json::Value::String(url.clone())); }
                if !content_type.is_empty() { map.insert("content_type".to_string(), serde_json::Value::String(content_type.clone())); }
                if !content_length.is_empty() { map.insert("content_length".to_string(), serde_json::Value::String(content_length.clone())); }
                if !redirect_url.is_empty() { map.insert("redirect_url".to_string(), serde_json::Value::String(redirect_url.clone())); }
                serde_json::Value::Object(map).to_string()
            }
            _ => {
                if is_head_request || (!status_code.is_empty() && content_type.is_empty() && content_length.is_empty()) {
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
                            let ct = content_type.split(';').next().unwrap_or(&content_type).trim();
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
        if ctx.stats { CommandStats::new().with_reducer("download").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }
}
