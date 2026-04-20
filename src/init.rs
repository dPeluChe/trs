//! `trs init` — Install hooks for AI coding tools.
//!
//! Generates configuration files that make the AI tool automatically
//! route commands through trs for token-optimized output.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init_collision;
use crate::init_templates::{
    ANTIGRAVITY_RULES, CLAUDE_HOOKS, CODEX_AGENTS_SECTION, CURSOR_HOOKS, DROID_HOOKS, GEMINI_HOOKS,
    OPENCODE_PLUGIN, WINDSURF_RULES,
};

/// Options for an install run. `global` picks home-dir vs project-local;
/// `replace` scrubs competing compressor hooks before installing trs;
/// `force` installs anyway when a collision is present.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InstallOpts {
    pub global: bool,
    pub replace: bool,
    pub force: bool,
}

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
    Windsurf,
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
            "windsurf" | "cascade" => Some(Self::Windsurf),
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
            Self::Windsurf => "Windsurf",
        }
    }

    pub(crate) fn all_names() -> &'static str {
        "claude, gemini, cursor, codex, opencode, kilo, antigravity, droid, windsurf"
    }

    pub(crate) fn all_tools() -> [Self; 9] {
        [
            Self::Claude,
            Self::Gemini,
            Self::Cursor,
            Self::Codex,
            Self::OpenCode,
            Self::Kilo,
            Self::Antigravity,
            Self::Droid,
            Self::Windsurf,
        ]
    }

    /// Short label describing where the integration lives, e.g. "~/.claude/settings.json"
    /// or "AGENTS.md". Used in `trs init --show` for transparency.
    pub(crate) fn target_label(&self) -> &'static str {
        match self {
            Self::Claude => "hooks → ~/.claude/settings.json",
            Self::Gemini => "hooks → ~/.gemini/settings.json",
            Self::Cursor => "hooks → ~/.cursor/hooks.json",
            Self::Codex => "rules → AGENTS.md (hooks.json support is experimental)",
            Self::OpenCode => "plugin → .opencode/plugins/trs.ts",
            Self::Kilo => "plugin → .kilo/plugins/trs.ts",
            Self::Antigravity => "rules → .agent/rules/antigravity-trs-rules.md",
            Self::Droid => "hooks → ~/.factory/settings.json",
            Self::Windsurf => "rules → .windsurfrules",
        }
    }

    /// Best-effort detection: is the tool installed on this system?
    /// Checks for the CLI binary and/or the common config directory.
    pub(crate) fn detect_installed(&self) -> bool {
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let home_has = |rel: &str| home.as_ref().map(|h| h.join(rel).exists()).unwrap_or(false);
        let in_path = |bin: &str| {
            std::process::Command::new("sh")
                .args(["-c", &format!("command -v {}", bin)])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };
        let app_exists = |name: &str| {
            Path::new(&format!("/Applications/{}.app", name)).exists()
                || Path::new(&format!(
                    "{}/Applications/{}.app",
                    std::env::var("HOME").unwrap_or_default(),
                    name
                ))
                .exists()
        };
        match self {
            Self::Claude => in_path("claude") || home_has(".claude"),
            Self::Gemini => in_path("gemini") || home_has(".gemini"),
            Self::Cursor => in_path("cursor") || app_exists("Cursor") || home_has(".cursor"),
            Self::Codex => {
                in_path("codex") || home_has(".codex") || Path::new("AGENTS.md").exists()
            }
            Self::OpenCode => in_path("opencode") || home_has(".opencode"),
            Self::Kilo => in_path("kilo") || home_has(".kilo"),
            Self::Antigravity => app_exists("Antigravity") || home_has(".antigravity"),
            Self::Droid => in_path("droid") || home_has(".factory"),
            Self::Windsurf => {
                in_path("windsurf") || app_exists("Windsurf") || home_has(".windsurfrules")
            }
        }
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
            // OpenCode auto-discovers plugins at startup from both the
            // project-level `.opencode/plugins/` and the global
            // `~/.config/opencode/plugins/`. No opencode.json registration is
            // needed for file-based plugins.
            Self::OpenCode => Some(HookSpec {
                local_dir: ".opencode/plugins",
                global_dir: Some(".config/opencode/plugins"),
                filename: "trs.ts",
                content: OPENCODE_PLUGIN,
            }),
            // Kilo mirrors OpenCode's plugin system: auto-discovery from
            // `~/.config/kilo/plugins/` (global) and `.kilo/plugins/` (project).
            // Shares OPENCODE_PLUGIN — the `tool.execute.before` hook API is
            // identical.
            Self::Kilo => Some(HookSpec {
                local_dir: ".kilo/plugins",
                global_dir: Some(".config/kilo/plugins"),
                filename: "trs.ts",
                content: OPENCODE_PLUGIN,
            }),
            Self::Droid => Some(HookSpec {
                local_dir: ".factory",
                global_dir: Some(".factory"),
                filename: "settings.json",
                content: DROID_HOOKS,
            }),
            // Rules-based tools (no programmatic hooks) — handled via
            // install_codex / install_rules instead.
            Self::Codex | Self::Antigravity | Self::Windsurf => None,
        }
    }
}

/// Install hooks for the specified tool.
pub(crate) fn install_hook(tool: &AiTool, opts: InstallOpts) {
    // Pre-install: detect competing compressor hooks (rtk, token-optimizer).
    // Default is to abort — --replace scrubs known competitors before the
    // install proceeds, --force installs anyway and eats the risk.
    let collisions = init_collision::detect(tool, opts.global);
    if !collisions.is_empty() && !opts.force && !opts.replace {
        eprintln!("{}", init_collision::format_report(tool, &collisions));
        return;
    }
    if !collisions.is_empty() && opts.replace && !init_collision::any_hook_collisions(&collisions) {
        // --replace has no automatic cleanup for text-file rules collisions;
        // surface that clearly rather than silently doing nothing.
        eprintln!("{}", init_collision::format_report(tool, &collisions));
        eprintln!(
            "note: --replace only scrubs JSON hook entries automatically.\n\
             The rules-file collisions above need manual edits."
        );
        return;
    }

    // With --replace, scrub every distinct JSON location we flagged. Our
    // install writes only to the tool's canonical target (e.g. hooks.json),
    // but the competitor may live in a sibling file (settings.json). If we
    // only scrub the file we write to, the competitor survives — and
    // double-compression is back.
    if opts.replace {
        let mut seen = std::collections::HashSet::new();
        for c in &collisions {
            if !init_collision::is_json_location(c) {
                continue;
            }
            if !seen.insert(c.location.clone()) {
                continue;
            }
            match init_collision::scrub_file(&c.location) {
                Ok(true) => println!("  scrubbed competitor hook from {}", c.location.display()),
                Ok(false) => {}
                Err(e) => eprintln!("  warning: could not scrub {}: {}", c.location.display(), e),
            }
        }
    }

    let result = match tool {
        AiTool::Codex => install_codex(),
        AiTool::Antigravity => {
            install_rules(".agent/rules/antigravity-trs-rules.md", ANTIGRAVITY_RULES)
        }
        AiTool::Windsurf => install_rules(".windsurfrules", WINDSURF_RULES),
        _ => {
            if let Some(spec) = tool.spec() {
                install_from_spec(&spec, opts)
            } else {
                Err("No hook spec for this tool".to_string())
            }
        }
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

/// Install hooks for all detected tools, skipping already-configured ones.
/// Tools not detected on the system are reported but not installed.
pub(crate) fn install_all(opts: InstallOpts) {
    let tools = AiTool::all_tools();
    let mut installed = 0;
    let mut skipped = 0;
    let mut undetected = 0;

    for tool in &tools {
        if check_tool(tool) {
            println!("  + {} (already configured)", tool.name());
            skipped += 1;
        } else if !tool.detect_installed() {
            println!("  - {} (not detected on system, skipping)", tool.name());
            undetected += 1;
        } else {
            install_hook(tool, opts);
            installed += 1;
        }
    }

    println!(
        "\n{} installed, {} already configured, {} skipped (not detected), {} total",
        installed,
        skipped,
        undetected,
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
///
/// Markers:
/// - `+` configured with trs
/// - `•` installed on system, not configured
/// - `-` not detected (config still possible for future installs)
pub(crate) fn show_status() {
    println!("trs init — hook status\n");

    let tools = AiTool::all_tools();
    // Compute column width so the target labels align regardless of tool name length.
    let name_width = tools.iter().map(|t| t.name().len()).max().unwrap_or(0);

    let mut configured = 0;
    let mut detected_total = 0;
    for tool in &tools {
        let is_configured = check_tool(tool);
        let is_detected = tool.detect_installed();
        let marker = if is_configured {
            "+"
        } else if is_detected {
            "•"
        } else {
            "-"
        };
        let status = if is_configured {
            tool.target_label()
        } else if !is_detected {
            "not detected on this system"
        } else {
            tool.target_label()
        };
        println!(
            "  {} {:<width$}  {}",
            marker,
            tool.name(),
            status,
            width = name_width
        );
        if is_configured {
            configured += 1;
        }
        if is_detected {
            detected_total += 1;
        }
    }
    println!(
        "\n{}/{} configured  ({} detected on system)",
        configured,
        tools.len(),
        detected_total
    );
}

/// Print `trs init` usage help — combined with status by default.
pub(crate) fn show_status_and_usage() {
    show_status();
    println!();
    println!("Usage:");
    println!("  trs init <tool> [--global]      install for a specific tool");
    println!("  trs init --all [--global]       install for all detected tools");
    println!("  trs init --show                 show this status");
    println!();
    println!("Collision handling:");
    println!("  --replace    remove competing compressor hooks (rtk, etc.)");
    println!("  --force      install alongside anyway (risk: double-compression)");
}

/// Check if a tool has trs hooks installed (local or global).
pub(crate) fn check_tool(tool: &AiTool) -> bool {
    match tool {
        AiTool::Codex => return check_file_contains("AGENTS.md", "trs (TARS CLI)"),
        AiTool::Antigravity => {
            return check_file_contains_path(
                Path::new(".agent/rules/antigravity-trs-rules.md"),
                "trs (TARS CLI)",
            );
        }
        AiTool::Windsurf => {
            return check_file_contains(".windsurfrules", "trs (TARS CLI)");
        }
        _ => {}
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

fn install_from_spec(spec: &HookSpec, opts: InstallOpts) -> Result<String, String> {
    if opts.global {
        if let Some(global_dir) = spec.global_dir {
            let home = home_dir()?;
            let dir = home.join(global_dir);
            let path = dir.join(spec.filename);
            return write_hook(&dir, &path, spec.content, opts.replace);
        }
        // No global config location for this tool — fall back to local install.
        eprintln!("note: --global not supported for this tool, installing locally instead");
    }
    let dir = PathBuf::from(spec.local_dir);
    let path = dir.join(spec.filename);
    write_hook(&dir, &path, spec.content, opts.replace)
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
// Rules-based tools (Antigravity, Windsurf)
// ============================================================

/// Install a rules/instructions file. Project-local only (rules tools lack a
/// global equivalent today). Idempotent: re-running is a no-op.
fn install_rules(path_rel: &str, content: &str) -> Result<String, String> {
    let path = PathBuf::from(path_rel);
    if path.exists() {
        let existing = fs::read_to_string(&path).unwrap_or_default();
        if has_trs_marker(&existing) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        // Append instead of overwriting — the user may have their own rules.
        let updated = format!("{}\n\n{}", existing.trim_end(), content);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create {}: {}", parent.display(), e))?;
        }
        fs::write(&path, content.trim_start()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

/// True if the file content already carries one of our sentinel strings.
fn has_trs_marker(content: &str) -> bool {
    content.contains("trs (TARS CLI)") || content.contains("trs rewrite")
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
fn write_hook(dir: &Path, path: &Path, content: &str, replace: bool) -> Result<String, String> {
    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));

    if is_json {
        return merge_json_hook(dir, path, content, replace);
    }

    // Non-JSON file (e.g. plugin .ts, hooks.json written by us directly).
    if path.exists() {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if has_trs_marker(&existing) {
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
fn merge_json_hook(
    dir: &Path,
    path: &Path,
    template: &str,
    replace: bool,
) -> Result<String, String> {
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

    // Snapshot the full root BEFORE taking mutable borrows for comparison.
    // Lets us detect a true no-op at the end even when template changes
    // (e.g. a widened matcher) would otherwise be silently skipped.
    let before_snapshot = serde_json::to_string(&root).unwrap_or_default();

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

    // Clean ALL existing trs entries across every event before re-inserting
    // from the template. Templates can migrate between events over time
    // (e.g. Cursor moving from beforeShellExecution → preToolUse to gain
    // rewrite support). Scoping the cleanup to just the template's events
    // would leave an orphaned entry on the old event, so we sweep broadly.
    // User-added entries (notify scripts, analytics) that don't reference
    // `trs rewrite` are preserved untouched.
    for (_event, event_val) in existing_hooks_obj.iter_mut() {
        if let Some(arr) = event_val.as_array_mut() {
            arr.retain(|e| {
                // Always drop our own prior entries (idempotent reinstall).
                if contains_trs_rewrite(e) {
                    return false;
                }
                // With --replace, also drop known competitors so trs isn't
                // stacked on top of rtk / token-optimizer. Without --replace
                // we leave foreign entries alone — install_hook already
                // aborted earlier if a collision was detected.
                if replace && init_collision::is_competitor_hook(e) {
                    return false;
                }
                true
            });
        }
    }

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

        if let Some(tmpl_arr) = tmpl_entries.as_array() {
            for entry in tmpl_arr {
                event_arr_mut.push(entry.clone());
            }
        }
    }

    let after_snapshot = serde_json::to_string(&root).unwrap_or_default();
    if path.exists() && before_snapshot == after_snapshot {
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
