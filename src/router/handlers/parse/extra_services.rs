use super::super::common::{CommandContext, CommandResult, CommandStats};
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Truncate a string to max_len chars, appending "..." if truncated.
    fn truncate_str(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
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
                if fields.len() >= 2 {
                    let number = fields[0].trim();
                    let title = fields[1].trim();
                    // Extract just author name (strip branch ref like "user:branch-name")
                    let author = fields.get(2)
                        .map(|s| s.split(':').next().unwrap_or("").trim())
                        .unwrap_or("");
                    prs.push(serde_json::json!({
                        "number": number, "title": title, "author": author
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
                        let title = Self::truncate_str(pr["title"].as_str().unwrap_or(""), 60);
                        let author = Self::truncate_str(pr["author"].as_str().unwrap_or(""), 30);
                        out.push_str(&format!("  #{} {} ({})\n",
                            pr["number"].as_str().unwrap_or(""),
                            title, author
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
                        let title = Self::truncate_str(issue["title"].as_str().unwrap_or(""), 60);
                        out.push_str(&format!("  #{} {}\n",
                            issue["number"].as_str().unwrap_or(""),
                            title
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
                // TSV format: status\tconclusion\tname\tdisplay_title\tbranch\tevent\tid\telapsed\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 3 {
                    let status_text = fields[0].trim().to_lowercase();
                    let conclusion = fields[1].trim().to_lowercase();
                    let name = fields[2].trim();
                    let event = fields.get(5).map(|s| s.trim()).unwrap_or("");
                    let status = if conclusion == "success" { "success" }
                        else if conclusion == "failure" { "failure" }
                        else if status_text == "in_progress" { "in_progress" }
                        else if conclusion == "cancelled" { "cancelled" }
                        else { &status_text };
                    runs.push(serde_json::json!({"name": name, "event": event, "status": status}));
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
                        let name = Self::truncate_str(run["name"].as_str().unwrap_or(""), 50);
                        let event = run["event"].as_str().unwrap_or("");
                        if !event.is_empty() {
                            out.push_str(&format!("  {} {} ({})\n",
                                marker, name, event
                            ));
                        } else {
                            out.push_str(&format!("  {} {}\n",
                                marker, name
                            ));
                        }
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
