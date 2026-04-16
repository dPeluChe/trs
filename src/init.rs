//! `trs init` — Install hooks for AI coding tools.
//!
//! Generates configuration files that make the AI tool automatically
//! route commands through trs for token-optimized output.

use std::fs;
use std::path::{Path, PathBuf};

/// Supported AI tools for hook installation.
pub(crate) enum AiTool {
    Claude,
    Gemini,
    Cursor,
    Codex,
    OpenCode,
    Kilo,
    Antigravity,
    Droid,
}

/// Hook installation spec — data-driven to avoid per-tool code duplication.
struct HookSpec {
    local_dir: &'static str,
    global_dir: Option<&'static str>,
    filename: &'static str,
    content: &'static str,
}

impl AiTool {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "gemini" => Some(Self::Gemini),
            "cursor" => Some(Self::Cursor),
            "codex" => Some(Self::Codex),
            "opencode" => Some(Self::OpenCode),
            "kilo" | "kilocode" => Some(Self::Kilo),
            "antigravity" | "gravity" => Some(Self::Antigravity),
            "droid" | "factory" => Some(Self::Droid),
            _ => None,
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Claude => "Claude Code",
            Self::Gemini => "Gemini CLI",
            Self::Cursor => "Cursor",
            Self::Codex => "Codex",
            Self::OpenCode => "OpenCode",
            Self::Kilo => "Kilo Code",
            Self::Antigravity => "Google Antigravity",
            Self::Droid => "Factory Droid",
        }
    }

    pub(crate) fn all_names() -> &'static str {
        "claude, gemini, cursor, codex, opencode, kilo, antigravity, droid"
    }

    pub(crate) fn all_tools() -> [Self; 8] {
        [
            Self::Claude,
            Self::Gemini,
            Self::Cursor,
            Self::Codex,
            Self::OpenCode,
            Self::Kilo,
            Self::Antigravity,
            Self::Droid,
        ]
    }

    fn spec(&self) -> Option<HookSpec> {
        match self {
            Self::Claude => Some(HookSpec {
                local_dir: "hooks",
                global_dir: Some(".claude"),
                filename: "hooks.json",
                content: CLAUDE_HOOKS,
            }),
            Self::Gemini => Some(HookSpec {
                local_dir: ".gemini",
                global_dir: Some(".gemini"),
                filename: "settings.json",
                content: GEMINI_HOOKS,
            }),
            Self::Cursor => Some(HookSpec {
                local_dir: ".cursor",
                global_dir: Some(".cursor"),
                filename: "hooks.json",
                content: CURSOR_HOOKS,
            }),
            Self::OpenCode => Some(HookSpec {
                local_dir: ".opencode/plugins",
                global_dir: None,
                filename: "trs.ts",
                content: OPENCODE_PLUGIN,
            }),
            Self::Kilo => Some(HookSpec {
                local_dir: ".kilo/plugins",
                global_dir: None,
                filename: "trs.ts",
                content: OPENCODE_PLUGIN,
            }),
            Self::Antigravity => Some(HookSpec {
                local_dir: ".antigravity",
                global_dir: Some(".antigravity"),
                filename: "settings.json",
                content: GEMINI_HOOKS, // Same hook format as Gemini CLI
            }),
            Self::Droid => Some(HookSpec {
                local_dir: ".factory",
                global_dir: Some(".factory"),
                filename: "settings.json",
                content: CLAUDE_HOOKS, // Factory Droid uses the same matcher/hooks shape
            }),
            Self::Codex => None, // Codex uses AGENTS.md append, not a spec
        }
    }
}

/// Install hooks for the specified tool.
pub(crate) fn install_hook(tool: &AiTool, global: bool) {
    let result = if let AiTool::Codex = tool {
        install_codex()
    } else if let Some(spec) = tool.spec() {
        install_from_spec(&spec, global)
    } else {
        Err("No hook spec for this tool".to_string())
    };

    match result {
        Ok(path) => {
            println!("trs hook installed for {} at {}", tool.name(), path);
            eprintln!(
                "note: restart any open {} sessions for the hook to take effect",
                tool.name()
            );
            // Warn if trs is not in PATH
            if !is_trs_in_path() {
                eprintln!(
                    "warning: 'trs' not found in PATH. The hook may fail silently.\n\
                     Make sure trs is installed: cargo install --path . or npm install -g tars-cli"
                );
            }
        }
        Err(e) => eprintln!("Failed to install hook for {}: {}", tool.name(), e),
    }
}

/// Install hooks for all supported tools, skipping already-configured ones.
pub(crate) fn install_all(global: bool) {
    let tools = AiTool::all_tools();
    let mut installed = 0;
    let mut skipped = 0;

    for tool in &tools {
        if check_tool(tool) {
            println!("  + {} (already configured)", tool.name());
            skipped += 1;
        } else {
            install_hook(tool, global);
            installed += 1;
        }
    }

    println!(
        "\n{} installed, {} already configured, {} total",
        installed,
        skipped,
        tools.len()
    );
    if installed > 0 {
        eprintln!("note: restart any open AI tool sessions for hooks to take effect");
    }
}

/// Check if trs binary is available in PATH.
fn is_trs_in_path() -> bool {
    std::process::Command::new("trs")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Show current hook installation status.
pub(crate) fn show_status() {
    println!("trs init — hook status\n");

    let tools = AiTool::all_tools();
    let mut count = 0;
    for tool in &tools {
        let installed = check_tool(tool);
        let marker = if installed { "+" } else { "-" };
        println!("  {} {}", marker, tool.name());
        if installed {
            count += 1;
        }
    }
    println!("\n{}/{} tools configured", count, tools.len());
}

/// Check if a tool has trs hooks installed (local or global).
pub(crate) fn check_tool(tool: &AiTool) -> bool {
    if let AiTool::Codex = tool {
        return check_file_contains("AGENTS.md", "trs");
    }
    if let Some(spec) = tool.spec() {
        // Check local
        let local_path = Path::new(spec.local_dir).join(spec.filename);
        if check_file_contains_path(&local_path, "trs") {
            return true;
        }
        // Check global
        if let Some(global) = spec.global_dir {
            if let Ok(home) = home_dir() {
                let global_path = home.join(global).join(spec.filename);
                if check_file_contains_path(&global_path, "trs") {
                    return true;
                }
                // Also check settings.json (hooks can live there too)
                let settings_path = home.join(global).join("settings.json");
                if check_file_contains_path(&settings_path, "trs rewrite") {
                    return true;
                }
            }
        }
    }
    false
}

// ============================================================
// Data-driven installer
// ============================================================

fn install_from_spec(spec: &HookSpec, global: bool) -> Result<String, String> {
    if global {
        if let Some(global_dir) = spec.global_dir {
            let home = home_dir()?;
            let dir = home.join(global_dir);
            let path = dir.join(spec.filename);
            return write_hook(&dir, &path, spec.content);
        }
        // No global config location for this tool — fall back to local install.
        eprintln!("note: --global not supported for this tool, installing locally instead");
    }
    let dir = PathBuf::from(spec.local_dir);
    let path = dir.join(spec.filename);
    write_hook(&dir, &path, spec.content)
}

// ============================================================
// Codex — AGENTS.md append (unique pattern)
// ============================================================

fn install_codex() -> Result<String, String> {
    let path = PathBuf::from("AGENTS.md");
    let marker = "trs (TARS CLI)";

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if content.contains(marker) {
            // Idempotent: already installed is a success, not a failure.
            return Ok(format!("{} (already configured)", path.display()));
        }
        let updated = format!("{}\n{}", content, CODEX_AGENTS_SECTION);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        fs::write(&path, CODEX_AGENTS_SECTION.trim()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

// ============================================================
// Helpers
// ============================================================

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "HOME not set".to_string())
}

fn check_file_contains(path_str: &str, needle: &str) -> bool {
    check_file_contains_path(Path::new(path_str), needle)
}

fn check_file_contains_path(path: &Path, needle: &str) -> bool {
    path.exists()
        && fs::read_to_string(path)
            .map(|c| c.contains(needle))
            .unwrap_or(false)
}

/// Write a hook file. For JSON settings files, merge our `hooks` section into
/// existing content (preserving user's other config). For non-JSON files,
/// refuse to overwrite if the file already has foreign content.
fn write_hook(dir: &Path, path: &Path, content: &str) -> Result<String, String> {
    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));

    if is_json {
        return merge_json_hook(dir, path, content);
    }

    // Non-JSON file (e.g. plugin .ts, hooks.json written by us directly).
    if path.exists() {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing.contains("trs rewrite") || existing.contains("trs (TARS CLI)") {
            return Ok(format!("{} (already configured)", path.display()));
        }
        return Err(format!(
            "{} exists with other config.\n  Back up and re-run `trs init` to replace.",
            path.display()
        ));
    }
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    fs::write(path, content).map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    Ok(path.display().to_string())
}

/// Merge the `hooks` section from a template into an existing JSON settings file.
/// Keeps user's other keys (model, auth, permissions, etc.) intact.
fn merge_json_hook(dir: &Path, path: &Path, template: &str) -> Result<String, String> {
    let template_value: serde_json::Value = serde_json::from_str(template)
        .map_err(|e| format!("internal: template JSON invalid: {}", e))?;
    let template_hooks = template_value
        .get("hooks")
        .ok_or_else(|| "internal: template has no `hooks` key".to_string())?;

    let mut root = if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&content).map_err(|e| {
                format!(
                    "{} is not valid JSON ({}). Fix it manually or back up and re-run.",
                    path.display(),
                    e
                )
            })?
        }
    } else {
        serde_json::json!({})
    };

    let Some(root_obj) = root.as_object_mut() else {
        return Err(format!("{} root is not a JSON object", path.display()));
    };

    let existing_hooks = root_obj
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !existing_hooks.is_object() {
        return Err(format!(
            "{} has a `hooks` key that is not an object",
            path.display()
        ));
    }
    let existing_hooks_obj = existing_hooks.as_object_mut().unwrap();

    let Some(template_hooks_obj) = template_hooks.as_object() else {
        return Err("internal: template `hooks` is not an object".into());
    };

    let mut appended_any = false;
    for (event, tmpl_entries) in template_hooks_obj {
        let event_arr = existing_hooks_obj
            .entry(event.clone())
            .or_insert_with(|| serde_json::json!([]));
        if !event_arr.is_array() {
            return Err(format!(
                "{} has `hooks.{}` that is not an array",
                path.display(),
                event
            ));
        }
        let event_arr_mut = event_arr.as_array_mut().unwrap();

        // Skip if any existing entry already references trs rewrite.
        if event_arr_mut.iter().any(contains_trs_rewrite) {
            continue;
        }
        if let Some(tmpl_arr) = tmpl_entries.as_array() {
            for entry in tmpl_arr {
                event_arr_mut.push(entry.clone());
                appended_any = true;
            }
        }
    }

    if !appended_any && path.exists() {
        return Ok(format!("{} (already configured)", path.display()));
    }

    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    Ok(path.display().to_string())
}

/// Recursively check if any string in a JSON value contains "trs rewrite".
fn contains_trs_rewrite(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::String(s) => s.contains("trs rewrite"),
        serde_json::Value::Object(o) => o.values().any(contains_trs_rewrite),
        serde_json::Value::Array(a) => a.iter().any(contains_trs_rewrite),
        _ => false,
    }
}

// ============================================================
// Hook templates
// ============================================================

const CLAUDE_HOOKS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite"
          }
        ],
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

const GEMINI_HOOKS: &str = r#"{
  "hooks": {
    "BeforeTool": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "trs rewrite",
            "name": "trs-rewrite",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}"#;

const CURSOR_HOOKS: &str = r#"{
  "hooks": {
    "beforeShellExecution": [
      {
        "command": "trs rewrite",
        "event": "beforeShellExecution",
        "description": "Route commands through trs for token-optimized output"
      }
    ]
  }
}"#;

// OpenCode/Kilo plugin: unconditionally prefix trs, let trs decide
// whether to compress or passthrough. No stale rewrite list needed.
const OPENCODE_PLUGIN: &str = r#"// trs plugin — route commands through trs for token-optimized output
import type { Plugin } from "opencode";

export default function trsPlugin(): Plugin {
  return {
    name: "trs",
    hooks: {
      before_tool_call: async (ctx) => {
        if (ctx.tool === "bash" && ctx.input?.command) {
          const cmd = ctx.input.command;
          // Skip if already using trs or if it's a trs command itself
          if (!cmd.startsWith("trs ") && !cmd.startsWith("cd ")) {
            ctx.input.command = `trs ${cmd}`;
          }
        }
        return ctx;
      },
    },
  };
}
"#;

const CODEX_AGENTS_SECTION: &str = r#"
## Terminal Output Optimization

This project uses `trs` (TARS CLI) for token-optimized terminal output.
When running shell commands, prefer prefixing with `trs` for compact output:

```bash
# Instead of:
git status
cargo test
cargo clippy

# Use:
trs git status
trs cargo test
trs cargo clippy
```

This reduces token consumption by 68-99% without losing signal.
See https://github.com/dPeluChe/trs for details.
"#;
