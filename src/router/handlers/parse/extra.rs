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
                        // Single file from stdin, no filename
                        out.push_str(&format!("{} lines, {} words, {} bytes\n", lines, words, bytes));
                    } else {
                        out.push_str(&format!("{}: {} lines, {} words, {} bytes\n", name, lines, words, bytes));
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

    /// Check if an env var key is internal noise that should be filtered.
    fn is_env_noise(key: &str) -> bool {
        // Internal shell/terminal noise prefixes
        let noise_prefixes = [
            "_P9K_", "P9K_", "LESS", "LS_COLORS", "LSCOLORS",
            "_", "__", "COMP_", "BASH_FUNC_",
            "ZSH_HIGHLIGHT", "ZSH_AUTOSUGGEST",
            "POWERLEVEL", "ITERM", "TERM_SESSION",
            "SECURITYSESSION", "TMPDIR",
            "LaunchInstanceID", "LOGNAME",
            "Apple_PubSub", "DISPLAY",
            "COMMAND_MODE", "COLORTERM",
            "MANPATH", "INFOPATH", "FPATH",
            "SSH_AUTH_SOCK", "SSH_AGENT_PID",
            "TERM_PROGRAM", "TERM_PROGRAM_VERSION",
            "ORIGINAL_XDG", "XPC_",
            "SUPERSET_", "ZDOTDIR",
            "CARGO_PKG_", "CARGO_MANIFEST", "CARGO_BIN",
            "CARGO_CRATE", "CARGO_PRIMARY",
            "NoDefault", "SSL_CERT",
            "rvm_", "GEM_",
        ];
        for prefix in &noise_prefixes {
            if key.starts_with(prefix) && key != "PATH" && key != "LANG" {
                return true;
            }
        }
        // Single underscore var
        if key == "_" { return true; }
        false
    }

    /// Categorize an env var for grouping.
    fn env_category(key: &str) -> &'static str {
        if key == "PATH" || key.ends_with("_PATH") || key.ends_with("PATH") || key == "MANPATH" || key == "INFOPATH" || key == "FPATH" {
            return "path";
        }
        if matches!(key, "LANG" | "LC_ALL" | "LC_CTYPE" | "LC_MESSAGES" | "LANGUAGE" | "TZ" | "TERM" | "SHELL" | "USER" | "HOME" | "HOSTNAME" | "PWD" | "OLDPWD" | "SHLVL" | "EDITOR" | "VISUAL" | "PAGER" | "XDG_CONFIG_HOME" | "XDG_DATA_HOME" | "XDG_CACHE_HOME" | "XDG_RUNTIME_DIR") {
            return "system";
        }
        if matches!(key, "GOPATH" | "GOROOT" | "CARGO_HOME" | "RUSTUP_HOME" | "PYENV_ROOT" | "RBENV_ROOT" | "NVM_DIR" | "JAVA_HOME" | "ANDROID_HOME" | "CONDA_DEFAULT_ENV" | "VIRTUAL_ENV" | "NODE_OPTIONS" | "NODE_ENV" | "PYTHONPATH" | "RUBY_VERSION" | "RUSTC_WRAPPER" | "npm_config_prefix")
            || key.starts_with("PYTHON") || key.starts_with("RUBY") || key.starts_with("GO") || key.starts_with("RUST") || key.starts_with("NODE") || key.starts_with("NVM") || key.starts_with("JAVA") || key.starts_with("CONDA") {
            return "lang";
        }
        "user"
    }

    pub(crate) fn handle_env(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut all_vars: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].to_string();
                let val = line[eq+1..].to_string();
                all_vars.push((key, val));
            }
        }

        // JSON output: include everything (unfiltered, just sorted)
        let output = match ctx.format {
            OutputFormat::Json => {
                let mut sorted = all_vars.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let jv: serde_json::Map<String, serde_json::Value> = sorted.iter().map(|(k,v)| {
                    let display = if v.len() > 80 { format!("{}...", &v[..77]) } else { v.clone() };
                    (k.clone(), serde_json::Value::String(display))
                }).collect();
                serde_json::json!({"variables": jv, "count": sorted.len()}).to_string()
            }
            _ => {
                // Compact: filter noise and empty values, group by category
                let mut path_vars: Vec<(String, String)> = Vec::new();
                let mut system_vars: Vec<(String, String)> = Vec::new();
                let mut lang_vars: Vec<(String, String)> = Vec::new();
                let mut user_vars: Vec<(String, String)> = Vec::new();
                let mut filtered_count = 0usize;

                for (key, val) in &all_vars {
                    // Skip empty values
                    if val.is_empty() { filtered_count += 1; continue; }
                    // Skip noise
                    if Self::is_env_noise(key) { filtered_count += 1; continue; }

                    let category = Self::env_category(key);
                    let display_val = if key == "PATH" || key.ends_with("PATH") || key == "FPATH" {
                        // For PATH-like vars, show entry count
                        let entries: Vec<&str> = val.split(':').filter(|s| !s.is_empty()).collect();
                        format!("({} entries)", entries.len())
                    } else if val.len() > 60 {
                        format!("{}...", &val[..57])
                    } else {
                        val.clone()
                    };

                    match category {
                        "path" => path_vars.push((key.clone(), display_val)),
                        "system" => system_vars.push((key.clone(), display_val)),
                        "lang" => lang_vars.push((key.clone(), display_val)),
                        _ => user_vars.push((key.clone(), display_val)),
                    }
                }

                path_vars.sort_by(|a, b| a.0.cmp(&b.0));
                system_vars.sort_by(|a, b| a.0.cmp(&b.0));
                lang_vars.sort_by(|a, b| a.0.cmp(&b.0));
                user_vars.sort_by(|a, b| a.0.cmp(&b.0));

                let shown = path_vars.len() + system_vars.len() + lang_vars.len() + user_vars.len();
                let mut out = format!("{} vars ({} filtered)\n", shown, filtered_count);

                if !path_vars.is_empty() {
                    // Show PATH vars inline: just PATH=46 entries
                    for (k, v) in &path_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !system_vars.is_empty() {
                    out.push_str("[system]\n");
                    for (k, v) in &system_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !lang_vars.is_empty() {
                    out.push_str("[lang/runtime]\n");
                    for (k, v) in &lang_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !user_vars.is_empty() {
                    out.push_str("[user/other]\n");
                    for (k, v) in &user_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("env").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(all_vars.len()).print(); }
        Ok(())
    }

    /// Parse GitHub CLI `gh pr list` output.
    ///
    /// Supports two formats:
    /// 1. TTY/pipe: `#123 fix: title (author)` (with emoji header)
    /// 2. Non-TTY (subprocess): `123\ttitle\tauthor:branch\tOPEN\tdate` (TSV)
    pub(crate) fn handle_gh_pr(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut prs: Vec<serde_json::Value> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            // Detect format: TSV (non-TTY) has tabs
            if trimmed.contains('\t') {
                // TSV format: number\ttitle\tauthor:branch\tstate\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 3 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    let author = fields[2].split(':').next().unwrap_or("").trim();
                    let state = fields.get(3).map(|s| s.trim()).unwrap_or("OPEN");
                    prs.push(serde_json::json!({
                        "number": number, "title": title,
                        "author": author, "state": state
                    }));
                }
            } else if trimmed.contains('#') {
                // TTY format: #123 title (author)
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let remainder = parts[1].trim();
                        let (title, author) = if let Some(paren_start) = remainder.rfind('(') {
                            (remainder[..paren_start].trim(), remainder[paren_start + 1..].trim_end_matches(')').trim())
                        } else {
                            (remainder, "")
                        };
                        prs.push(serde_json::json!({
                            "number": number, "title": title, "author": author
                        }));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"pull_requests": prs, "count": prs.len()}).to_string(),
            _ => {
                if prs.is_empty() {
                    "no open pull requests\n".to_string()
                } else {
                    let mut out = format!("pull requests: {}\n", prs.len());
                    for pr in &prs {
                        out.push_str(&format!("  #{} {} ({})\n",
                            pr["number"].as_str().unwrap_or(""),
                            pr["title"].as_str().unwrap_or(""),
                            pr["author"].as_str().unwrap_or("")
                        ));
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("gh-pr").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(prs.len()).print(); }
        Ok(())
    }

    /// Parse GitHub CLI `gh issue list` output.
    ///
    /// Supports TTY format (#123 title) and non-TTY TSV (123\ttitle\tlabels\tdate).
    pub(crate) fn handle_gh_issue(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut issues: Vec<serde_json::Value> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            if trimmed.contains('\t') {
                // TSV format: number\ttitle\tlabels\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 2 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    let labels = fields.get(2).map(|s| s.trim()).unwrap_or("");
                    issues.push(serde_json::json!({"number": number, "title": title, "labels": labels}));
                }
            } else if trimmed.contains('#') {
                if let Some(hash_pos) = trimmed.find('#') {
                    let rest = &trimmed[hash_pos + 1..];
                    let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let number = parts[0].trim();
                        let title = parts[1].trim();
                        issues.push(serde_json::json!({"number": number, "title": title}));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"issues": issues, "count": issues.len()}).to_string(),
            _ => {
                if issues.is_empty() {
                    "no open issues\n".to_string()
                } else {
                    let mut out = format!("issues: {}\n", issues.len());
                    for issue in &issues {
                        out.push_str(&format!("  #{} {}\n",
                            issue["number"].as_str().unwrap_or(""),
                            issue["title"].as_str().unwrap_or("")
                        ));
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("gh-issue").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(issues.len()).print(); }
        Ok(())
    }

    /// Parse GitHub CLI `gh run list` output.
    pub(crate) fn handle_gh_run(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read raw input to detect status emoji markers before stripping
        let raw_input = Self::read_input_raw(file)?;
        let input = super::super::common::strip_emojis(&raw_input);
        let input_bytes = raw_input.len();
        let mut runs: Vec<serde_json::Value> = Vec::new();

        let raw_lines: Vec<&str> = raw_input.lines().collect();
        let clean_lines: Vec<&str> = input.lines().collect();

        for (i, line) in clean_lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            let raw_line = raw_lines.get(i).unwrap_or(&"");

            // Detect format: TSV (non-TTY) has tabs
            if trimmed.contains('\t') {
                // TSV format: status\tconclusion\tname\tbranch\tevent\tid\telapsed\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 4 {
                    let status_text = fields[0].trim().to_lowercase();
                    let conclusion = fields[1].trim().to_lowercase();
                    let name = fields[2].trim();
                    let id = fields.get(5).map(|s| s.trim()).unwrap_or("");
                    let status = if conclusion == "success" { "success" }
                        else if conclusion == "failure" { "failure" }
                        else if status_text == "in_progress" { "in_progress" }
                        else if conclusion == "cancelled" { "cancelled" }
                        else { &status_text };
                    runs.push(serde_json::json!({"name": name, "id": id, "status": status}));
                }
            } else {
                // TTY format: skip headers
                if trimmed.starts_with("Workflow") || trimmed.starts_with("Showing") { continue; }

                // Parse: name [id]
                if let Some(bracket_start) = trimmed.rfind('[') {
                    let name = trimmed[..bracket_start].trim();
                    let id = trimmed[bracket_start + 1..].trim_end_matches(']').trim();

                    let status = if raw_line.contains('\u{2705}') || raw_line.contains("success") || raw_line.contains("completed") {
                        "success"
                    } else if raw_line.contains('\u{274C}') || raw_line.contains("failure") || raw_line.contains("failed") {
                        "failure"
                    } else if raw_line.contains("in_progress") || raw_line.contains("queued") || raw_line.contains('\u{1F7E1}') {
                        "in_progress"
                    } else if raw_line.contains('\u{1F534}') || raw_line.contains("cancelled") {
                        "cancelled"
                    } else {
                        "unknown"
                    };

                    if !name.is_empty() {
                        runs.push(serde_json::json!({"name": name, "id": id, "status": status}));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"runs": runs, "count": runs.len()}).to_string(),
            _ => {
                if runs.is_empty() {
                    "no workflow runs\n".to_string()
                } else {
                    let mut out = format!("runs: {}\n", runs.len());
                    for run in &runs {
                        let marker = match run["status"].as_str().unwrap_or("") {
                            "success" => "+",
                            "failure" => "-",
                            "in_progress" => "~",
                            _ => "?",
                        };
                        out.push_str(&format!("  {} {} [{}]\n",
                            marker,
                            run["name"].as_str().unwrap_or(""),
                            run["id"].as_str().unwrap_or("")
                        ));
                    }
                    out
                }
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("gh-run").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(runs.len()).print(); }
        Ok(())
    }

    /// Parse `cargo test` output.
    ///
    /// Format:
    /// ```text
    /// running N tests
    /// test module::name ... ok
    /// test module::name ... FAILED
    /// test module::name ... ignored
    /// test result: ok. X passed; Y failed; Z ignored; 0 measured; W filtered out; finished in Ns
    /// ```
    pub(crate) fn handle_cargo_test(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut ignored = 0usize;
        let mut filtered = 0usize;
        let mut failed_names: Vec<String> = Vec::new();
        let mut duration = String::new();
        let mut compile_errors: Vec<String> = Vec::new();
        let mut suites = 0usize;

        for line in input.lines() {
            let trimmed = line.trim();

            // Summary line: "test result: ok. X passed; Y failed; Z ignored; ..."
            if trimmed.starts_with("test result:") {
                suites += 1;
                // Remove "test result: ok." or "test result: FAILED." prefix
                let summary_part = trimmed
                    .strip_prefix("test result:")
                    .unwrap_or(trimmed)
                    .trim()
                    .trim_start_matches("ok.")
                    .trim_start_matches("FAILED.")
                    .trim();
                for part in summary_part.split(';') {
                    let t = part.trim();
                    if t.contains("passed") {
                        passed += t.split_whitespace().next()
                            .and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    } else if t.contains("failed") {
                        let n = t.split_whitespace().next()
                            .and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                        failed += n;
                    } else if t.contains("ignored") {
                        ignored += t.split_whitespace().next()
                            .and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    } else if t.contains("filtered out") {
                        filtered += t.split_whitespace().next()
                            .and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    } else if t.contains("finished in") {
                        if let Some(pos) = t.find("finished in ") {
                            duration = t[pos + 12..].to_string();
                        }
                    }
                }
                continue;
            }

            // Individual test result: "test name ... FAILED"
            if trimmed.starts_with("test ") && trimmed.contains(" ... ") {
                if trimmed.ends_with("FAILED") {
                    let name = trimmed.strip_prefix("test ").unwrap_or("")
                        .split(" ... ").next().unwrap_or("").to_string();
                    failed_names.push(name);
                }
                continue;
            }

            // Compile errors (before tests run)
            if trimmed.starts_with("error[") || trimmed.starts_with("error:") {
                compile_errors.push(trimmed.to_string());
            }
        }

        let total = passed + failed + ignored;
        let success = failed == 0 && compile_errors.is_empty();

        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({
                    "success": success,
                    "passed": passed,
                    "failed": failed,
                    "ignored": ignored,
                    "filtered": filtered,
                    "total": total,
                    "suites": suites,
                    "duration": duration,
                    "failed_tests": failed_names,
                    "compile_errors": compile_errors,
                }).to_string()
            }
            _ => {
                let mut out = String::new();
                if !compile_errors.is_empty() {
                    out.push_str(&format!("compile errors ({}):\n", compile_errors.len()));
                    for err in compile_errors.iter().take(10) {
                        out.push_str(&format!("  {}\n", err));
                    }
                    if compile_errors.len() > 10 {
                        out.push_str(&format!("  ...+{} more\n", compile_errors.len() - 10));
                    }
                }
                let status = if success { "ok" } else { "FAILED" };
                out.push_str(&format!("cargo test: {} ({} passed", status, passed));
                if failed > 0 { out.push_str(&format!(", {} failed", failed)); }
                if ignored > 0 { out.push_str(&format!(", {} ignored", ignored)); }
                if filtered > 0 { out.push_str(&format!(", {} filtered", filtered)); }
                if suites > 1 { out.push_str(&format!(", {} suites", suites)); }
                if !duration.is_empty() { out.push_str(&format!(", {}", duration)); }
                out.push_str(")\n");

                if !failed_names.is_empty() {
                    out.push_str(&format!("failures ({}):\n", failed_names.len()));
                    for name in &failed_names {
                        out.push_str(&format!("  {}\n", name));
                    }
                }
                out
            }
        };

        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("cargo-test").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(total).print(); }
        Ok(())
    }
}
