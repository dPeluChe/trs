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
}
