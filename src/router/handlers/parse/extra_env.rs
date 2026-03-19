use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Check if an env var key is internal noise that should be filtered.
    fn is_env_noise(key: &str) -> bool {
        // Internal shell/terminal noise prefixes
        let noise_prefixes = [
            "_P9K_",
            "P9K_",
            "LESS",
            "LS_COLORS",
            "LSCOLORS",
            "_",
            "__",
            "COMP_",
            "BASH_FUNC_",
            "ZSH_HIGHLIGHT",
            "ZSH_AUTOSUGGEST",
            "POWERLEVEL",
            "ITERM",
            "TERM_SESSION",
            "SECURITYSESSION",
            "TMPDIR",
            "LaunchInstanceID",
            "LOGNAME",
            "Apple_PubSub",
            "DISPLAY",
            "COMMAND_MODE",
            "COLORTERM",
            "MANPATH",
            "INFOPATH",
            "FPATH",
            "SSH_AUTH_SOCK",
            "SSH_AGENT_PID",
            "TERM_PROGRAM",
            "TERM_PROGRAM_VERSION",
            "ORIGINAL_XDG",
            "XPC_",
            "SUPERSET_",
            "ZDOTDIR",
            "CARGO_PKG_",
            "CARGO_MANIFEST",
            "CARGO_BIN",
            "CARGO_CRATE",
            "CARGO_PRIMARY",
            "NoDefault",
            "SSL_CERT",
            "rvm_",
            "GEM_",
        ];
        for prefix in &noise_prefixes {
            if key.starts_with(prefix) && key != "PATH" && key != "LANG" {
                return true;
            }
        }
        // Single underscore var
        if key == "_" {
            return true;
        }
        false
    }

    /// Categorize an env var for grouping.
    fn env_category(key: &str) -> &'static str {
        if key == "PATH"
            || key.ends_with("_PATH")
            || key.ends_with("PATH")
            || key == "MANPATH"
            || key == "INFOPATH"
            || key == "FPATH"
        {
            return "path";
        }
        if matches!(
            key,
            "LANG"
                | "LC_ALL"
                | "LC_CTYPE"
                | "LC_MESSAGES"
                | "LANGUAGE"
                | "TZ"
                | "TERM"
                | "SHELL"
                | "USER"
                | "HOME"
                | "HOSTNAME"
                | "PWD"
                | "OLDPWD"
                | "SHLVL"
                | "EDITOR"
                | "VISUAL"
                | "PAGER"
                | "XDG_CONFIG_HOME"
                | "XDG_DATA_HOME"
                | "XDG_CACHE_HOME"
                | "XDG_RUNTIME_DIR"
        ) {
            return "system";
        }
        if matches!(
            key,
            "GOPATH"
                | "GOROOT"
                | "CARGO_HOME"
                | "RUSTUP_HOME"
                | "PYENV_ROOT"
                | "RBENV_ROOT"
                | "NVM_DIR"
                | "JAVA_HOME"
                | "ANDROID_HOME"
                | "CONDA_DEFAULT_ENV"
                | "VIRTUAL_ENV"
                | "NODE_OPTIONS"
                | "NODE_ENV"
                | "PYTHONPATH"
                | "RUBY_VERSION"
                | "RUSTC_WRAPPER"
                | "npm_config_prefix"
        ) || key.starts_with("PYTHON")
            || key.starts_with("RUBY")
            || key.starts_with("GO")
            || key.starts_with("RUST")
            || key.starts_with("NODE")
            || key.starts_with("NVM")
            || key.starts_with("JAVA")
            || key.starts_with("CONDA")
        {
            return "lang";
        }
        "user"
    }

    pub(crate) fn handle_env(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut all_vars: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].to_string();
                let val = line[eq + 1..].to_string();
                all_vars.push((key, val));
            }
        }

        // JSON output: include everything (unfiltered, just sorted)
        let output = match ctx.format {
            OutputFormat::Json => {
                let mut sorted = all_vars.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let jv: serde_json::Map<String, serde_json::Value> = sorted
                    .iter()
                    .map(|(k, v)| {
                        let display = if v.len() > 80 {
                            format!("{}...", &v[..77])
                        } else {
                            v.clone()
                        };
                        (k.clone(), serde_json::Value::String(display))
                    })
                    .collect();
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
                    if val.is_empty() {
                        filtered_count += 1;
                        continue;
                    }
                    // Skip noise
                    if Self::is_env_noise(key) {
                        filtered_count += 1;
                        continue;
                    }

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
                    for (k, v) in &path_vars {
                        out.push_str(&format!("  {}={}\n", k, v));
                    }
                }
                if !system_vars.is_empty() {
                    out.push_str("[system]\n");
                    for (k, v) in &system_vars {
                        out.push_str(&format!("  {}={}\n", k, v));
                    }
                }
                if !lang_vars.is_empty() {
                    out.push_str("[lang/runtime]\n");
                    for (k, v) in &lang_vars {
                        out.push_str(&format!("  {}={}\n", k, v));
                    }
                }
                if !user_vars.is_empty() {
                    out.push_str("[user/other]\n");
                    for (k, v) in &user_vars {
                        out.push_str(&format!("  {}={}\n", k, v));
                    }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("env")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(all_vars.len())
                .print();
        }
        Ok(())
    }
}
