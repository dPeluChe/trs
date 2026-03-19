use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_tree(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
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
                    if t.ends_with("directories") || t.ends_with("directory") {
                        total_dirs = t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(0);
                    } else if t.ends_with("files") || t.ends_with("file") {
                        total_files = t
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(0);
                    }
                }
                continue;
            }
            let name: String = line
                .chars()
                .filter(|c| !matches!(c, '│' | '├' | '└' | '─'))
                .collect::<String>()
                .trim()
                .to_string();
            if name.is_empty() || name == "." {
                continue;
            }
            if name.ends_with('/') {
                dirs.push(name.trim_end_matches('/').to_string());
            } else {
                files.push(name);
            }
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
        if ctx.stats {
            CommandStats::new()
                .with_reducer("tree")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_docker_ps(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
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
                    let status = line
                        .find("Up ")
                        .or_else(|| line.find("Exited "))
                        .map(|s| line[s..].split("  ").next().unwrap_or("").trim())
                        .unwrap_or("unknown");
                    let name = parts.last().unwrap_or(&"");
                    containers.push(serde_json::json!({"id": id, "image": image, "status": status, "name": name}));
                }
            }
        }
        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({"containers": containers, "count": containers.len()}).to_string()
            }
            _ => {
                let mut out = format!("containers: {}\n", containers.len());
                for c in &containers {
                    let st = c["status"].as_str().unwrap_or("");
                    let mk = if st.starts_with("Up") { "+" } else { "-" };
                    out.push_str(&format!(
                        "  {} {} {} ({})\n",
                        mk,
                        c["name"].as_str().unwrap_or(""),
                        c["image"].as_str().unwrap_or(""),
                        st
                    ));
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("docker-ps")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_docker_logs(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        Self::handle_logs(file, ctx)
    }

    pub(crate) fn handle_deps(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut deps: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            let clean = line
                .replace(['│', '├', '└', '─'], "")
                .replace("deduped", "")
                .trim()
                .to_string();
            if clean.is_empty() {
                continue;
            }
            // npm: name@version
            if let Some(at) = clean.rfind('@') {
                if at > 0 {
                    let n = clean[..at].trim().to_string();
                    let v = clean[at + 1..].trim().to_string();
                    if !n.is_empty() {
                        deps.push((n, v));
                        continue;
                    }
                }
            }
            // pip/cargo: name version
            let parts: Vec<&str> = clean.split_whitespace().collect();
            if parts.len() >= 2 {
                let n = parts[0].to_string();
                if n == "Package" || n == "---" || n.starts_with("==") {
                    continue;
                }
                deps.push((n, parts[1].trim_start_matches('v').to_string()));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jd: Vec<serde_json::Value> = deps
                    .iter()
                    .map(|(n, v)| serde_json::json!({"name":n,"version":v}))
                    .collect();
                serde_json::json!({"dependencies": jd, "count": deps.len()}).to_string()
            }
            _ => {
                let mut out = format!("dependencies: {}\n", deps.len());
                for (n, v) in &deps {
                    if v.is_empty() {
                        out.push_str(&format!("  {}\n", n));
                    } else {
                        out.push_str(&format!("  {}@{}\n", n, v));
                    }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("deps")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(deps.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_install(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut added: Vec<String> = Vec::new();
        let mut warnings = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut summary = String::new();

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let lower = t.to_lowercase();
            if (lower.contains("added") || lower.contains("removed")) && lower.contains("package") {
                summary = t.to_string();
            } else if lower.starts_with("successfully installed") {
                for pkg in t.split_whitespace().skip(2) {
                    added.push(pkg.to_string());
                }
            } else if lower.starts_with("npm warn") || lower.starts_with("warn ") {
                warnings += 1;
            } else if lower.starts_with("npm error")
                || lower.starts_with("error")
                || lower.starts_with("err!")
            {
                errors.push(t.to_string());
            }
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
        if ctx.stats {
            CommandStats::new()
                .with_reducer("install")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_build(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut info_last = String::new();
        let mut success = true;

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let lower = t.to_lowercase();
            if lower.contains("error[")
                || lower.starts_with("error:")
                || lower.contains("): error ")
                || lower.contains(": error:")
            {
                errors.push(t.to_string());
                success = false;
            } else if lower.contains("warning[")
                || lower.starts_with("warning:")
                || lower.contains(": warning:")
            {
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
        if ctx.stats {
            CommandStats::new()
                .with_reducer("build")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }

    pub(crate) fn handle_wc(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut entries: Vec<(String, u64, u64, u64)> = Vec::new(); // (name, lines, words, bytes)

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

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
                    let first = parts[0].parse::<u64>().unwrap_or(0);
                    let second = parts[1].parse::<u64>().unwrap_or(0);
                    let third = parts[2].parse::<u64>().unwrap_or(0);
                    // Could be "lines words bytes" (stdin) or "count count filename"
                    if parts[2].parse::<u64>().is_ok() {
                        entries.push((String::new(), first, second, third));
                    } else {
                        // "count filename" with extra column — treat as partial
                        entries.push((parts[2].to_string(), first, second, 0));
                    }
                }
                // wc -l/-w/-c: single count + filename (e.g. "22 file.ts")
                2 => {
                    let count = parts[0].parse::<u64>().unwrap_or(0);
                    let name = parts[1].to_string();
                    // We don't know which flag was used, show as lines
                    entries.push((name, count, 0, 0));
                }
                // wc -l from stdin: just a number
                1 => {
                    if let Ok(count) = parts[0].parse::<u64>() {
                        entries.push((String::new(), count, 0, 0));
                    }
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
                let has_full_stats = entries.iter().any(|(_, _, w, b)| *w > 0 || *b > 0);
                for (name, lines, words, bytes) in &entries {
                    let stats = if has_full_stats {
                        format!("{}L {}W {}B", lines, words, bytes)
                    } else {
                        format!("{}L", lines)
                    };
                    if name.is_empty() {
                        out.push_str(&format!("{}\n", stats));
                    } else if name == "total" {
                        out.push_str(&format!("total: {}\n", stats));
                    } else {
                        out.push_str(&format!("{} {}\n", name, stats));
                    }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("wc")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(entries.len())
                .print();
        }
        Ok(())
    }
}
