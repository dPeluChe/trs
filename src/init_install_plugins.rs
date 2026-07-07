//! Custom installers for the plugin-dir agents (OpenClaw, Hermes). Both are
//! global-only: plugin files under the agent's home dir plus a config-file
//! enable entry. Shipped from 2026-06-11 docs research — live validation
//! pending (see docs/support/agents.md per-agent status).

use std::fs;
use std::path::{Path, PathBuf};

use crate::init::{home_dir, InstallOpts};
use crate::init_templates::{
    HERMES_PLUGIN_INIT, HERMES_PLUGIN_YAML, OPENCLAW_PLUGIN_INDEX, OPENCLAW_PLUGIN_MANIFEST,
};

// Validate live: manifest schema (docs verified 2026-06-11; `entry` points
// at the plugin module file).

const HERMES_ITEM: &str = "- trs-rewrite";

/// Hermes home dir — `HERMES_HOME` env override, default `~/.hermes`.
pub(crate) fn hermes_home() -> Result<PathBuf, String> {
    if let Ok(custom) = std::env::var("HERMES_HOME") {
        if !custom.trim().is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    Ok(home_dir()?.join(".hermes"))
}

pub(crate) fn install_openclaw_plugin(opts: InstallOpts) -> Result<String, String> {
    if !opts.global {
        eprintln!("note: OpenClaw plugins are global — installing to ~/.openclaw/plugins/trs/");
    }
    let openclaw_home = home_dir()?.join(".openclaw");
    let plugin_dir = openclaw_home.join("plugins").join("trs");
    let mut changed = false;
    changed |= write_plugin_file(
        &plugin_dir.join("openclaw.plugin.json"),
        OPENCLAW_PLUGIN_MANIFEST,
        opts,
    )?;
    changed |= write_plugin_file(&plugin_dir.join("index.js"), OPENCLAW_PLUGIN_INDEX, opts)?;

    let config_path = openclaw_home.join("openclaw.json");
    let mut root = read_json_or_empty(&config_path)?;
    let plugin_dir_str = plugin_dir.display().to_string();
    let config_changed = merge_openclaw_config(&mut root, &plugin_dir_str)
        .map_err(|e| format!("{}: {}", config_path.display(), e))?;
    if config_changed {
        if opts.dry_run {
            println!(
                "  would enable plugins.entries.trs in {}",
                config_path.display()
            );
        } else {
            ensure_parent(&config_path)?;
            let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
            fs::write(&config_path, format!("{}\n", pretty))
                .map_err(|e| format!("Cannot write {}: {}", config_path.display(), e))?;
            println!("  enabled plugins.entries.trs in {}", config_path.display());
        }
        changed = true;
    } else {
        println!("  {} (already configured)", config_path.display());
    }

    if !opts.dry_run {
        println!("  restart the OpenClaw gateway to load the plugin: openclaw gateway restart");
    }
    if changed {
        Ok(plugin_dir.display().to_string())
    } else {
        Ok(format!("{} (already configured)", plugin_dir.display()))
    }
}

pub(crate) fn install_hermes_plugin(opts: InstallOpts) -> Result<String, String> {
    if !opts.global {
        eprintln!("note: Hermes plugins are global — installing to the Hermes home dir");
    }
    let home = hermes_home()?;
    let plugin_dir = home.join("plugins").join("trs-rewrite");
    let mut changed = false;
    changed |= write_plugin_file(&plugin_dir.join("__init__.py"), HERMES_PLUGIN_INIT, opts)?;
    changed |= write_plugin_file(&plugin_dir.join("plugin.yaml"), HERMES_PLUGIN_YAML, opts)?;

    let config_path = home.join("config.yaml");
    let existing = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|e| format!("Cannot read {}: {}", config_path.display(), e))?
    } else {
        String::new()
    };
    match patch_hermes_config(&existing) {
        HermesConfigPatch::AlreadyPresent => {
            println!("  {} (already configured)", config_path.display());
        }
        HermesConfigPatch::Patched(new_config) => {
            if opts.dry_run {
                println!(
                    "  would add trs-rewrite to plugins.enabled in {}",
                    config_path.display()
                );
            } else {
                ensure_parent(&config_path)?;
                fs::write(&config_path, new_config)
                    .map_err(|e| format!("Cannot write {}: {}", config_path.display(), e))?;
                println!(
                    "  added trs-rewrite to plugins.enabled in {}",
                    config_path.display()
                );
            }
            changed = true;
        }
        HermesConfigPatch::Manual => {
            // Conservative on purpose: no YAML lib, so exotic layouts
            // (inline arrays, plugins without enabled) get a manual note
            // instead of a risky rewrite. Plugin files are still installed.
            eprintln!(
                "  note: {} has a plugins layout this installer doesn't rewrite.\n\
                 \x20 Add `trs-rewrite` to the `plugins.enabled` list manually.",
                config_path.display()
            );
        }
    }

    if !opts.dry_run {
        println!("  restart Hermes to load the plugin");
    }
    if changed {
        Ok(plugin_dir.display().to_string())
    } else {
        Ok(format!("{} (already configured)", plugin_dir.display()))
    }
}

/// Create-or-refresh a file we own by location (a dedicated trs plugin
/// dir). Returns true when the disk would change/changed.
fn write_plugin_file(path: &Path, content: &str, opts: InstallOpts) -> Result<bool, String> {
    if path.exists() {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing == content {
            println!("  {} (already configured)", path.display());
            return Ok(false);
        }
        if opts.dry_run {
            println!("  {} (would refresh trs plugin)", path.display());
            return Ok(true);
        }
        fs::write(path, content).map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
        println!("  {} (refreshed)", path.display());
        return Ok(true);
    }
    if opts.dry_run {
        println!("  {} (would create)", path.display());
        return Ok(true);
    }
    ensure_parent(path)?;
    fs::write(path, content).map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    println!("  {} (created)", path.display());
    Ok(true)
}

fn read_json_or_empty(path: &Path) -> Result<serde_json::Value, String> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&content).map_err(|e| {
        format!(
            "{} is not valid JSON ({}). Fix it manually or back up and re-run.",
            path.display(),
            e
        )
    })
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create {}: {}", parent.display(), e))?;
    }
    Ok(())
}

/// Ensure `plugins.entries.trs.enabled = true` and `plugins.load.paths`
/// contains `plugin_dir`, preserving everything else. Returns true when
/// the tree changed.
pub(crate) fn merge_openclaw_config(
    root: &mut serde_json::Value,
    plugin_dir: &str,
) -> Result<bool, String> {
    let obj = root
        .as_object_mut()
        .ok_or_else(|| "config root is not a JSON object".to_string())?;
    let mut changed = false;

    let plugins = obj
        .entry("plugins".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let plugins = plugins
        .as_object_mut()
        .ok_or_else(|| "`plugins` is not an object".to_string())?;

    {
        let entries = plugins
            .entry("entries".to_string())
            .or_insert_with(|| serde_json::json!({}));
        let entries = entries
            .as_object_mut()
            .ok_or_else(|| "`plugins.entries` is not an object".to_string())?;
        let trs = entries
            .entry("trs".to_string())
            .or_insert_with(|| serde_json::json!({}));
        let trs = trs
            .as_object_mut()
            .ok_or_else(|| "`plugins.entries.trs` is not an object".to_string())?;
        if trs.get("enabled") != Some(&serde_json::json!(true)) {
            trs.insert("enabled".to_string(), serde_json::json!(true));
            changed = true;
        }
    }

    let load = plugins
        .entry("load".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let load = load
        .as_object_mut()
        .ok_or_else(|| "`plugins.load` is not an object".to_string())?;
    let paths = load
        .entry("paths".to_string())
        .or_insert_with(|| serde_json::json!([]));
    let paths = paths
        .as_array_mut()
        .ok_or_else(|| "`plugins.load.paths` is not an array".to_string())?;
    if !paths.iter().any(|p| p.as_str() == Some(plugin_dir)) {
        paths.push(serde_json::json!(plugin_dir));
        changed = true;
    }
    Ok(changed)
}

#[derive(Debug, PartialEq)]
pub(crate) enum HermesConfigPatch {
    AlreadyPresent,
    Patched(String),
    /// Layout we won't rewrite without a YAML parser — tell the user to
    /// add `trs-rewrite` to `plugins.enabled` themselves.
    Manual,
}

/// Conservative line-based patch of Hermes' `config.yaml`: only the
/// `plugins:` / `enabled:` block-list shape is rewritten; anything exotic
/// falls back to `Manual`.
pub(crate) fn patch_hermes_config(existing: &str) -> HermesConfigPatch {
    if existing.trim().is_empty() {
        return HermesConfigPatch::Patched("plugins:\n  enabled:\n    - trs-rewrite\n".to_string());
    }
    let lines: Vec<&str> = existing.lines().collect();
    let Some(plugins_idx) = lines.iter().position(|l| l.trim_end() == "plugins:") else {
        if lines.iter().any(|l| l.trim_start().starts_with("plugins:")) {
            // Inline (`plugins: {…}`) or nested key — don't guess.
            return HermesConfigPatch::Manual;
        }
        // No plugins key at all — appending a fresh top-level block is safe.
        let mut out = existing.trim_end().to_string();
        out.push_str("\nplugins:\n  enabled:\n    - trs-rewrite\n");
        return HermesConfigPatch::Patched(out);
    };

    // The plugins block: indented lines until the next top-level key.
    let mut enabled_idx = None;
    let mut block_end = lines.len();
    for (i, line) in lines.iter().enumerate().skip(plugins_idx + 1) {
        if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            block_end = i;
            break;
        }
        let trimmed = line.trim();
        if trimmed == "enabled:" {
            enabled_idx = Some(i);
        } else if trimmed.starts_with("enabled:") && enabled_idx.is_none() {
            // Inline array form (`enabled: [a, b]`).
            return HermesConfigPatch::Manual;
        }
    }
    let Some(enabled_idx) = enabled_idx else {
        return HermesConfigPatch::Manual;
    };

    let enabled_indent = indent_of(lines[enabled_idx]);
    let mut item_indent = None;
    let mut insert_after = enabled_idx;
    for (i, line) in lines
        .iter()
        .enumerate()
        .take(block_end)
        .skip(enabled_idx + 1)
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let indent = indent_of(line);
        if indent <= enabled_indent || !trimmed.starts_with('-') {
            break;
        }
        if trimmed == HERMES_ITEM || trimmed == "- \"trs-rewrite\"" || trimmed == "- 'trs-rewrite'"
        {
            return HermesConfigPatch::AlreadyPresent;
        }
        item_indent = Some(indent);
        insert_after = i;
    }

    let indent = item_indent.unwrap_or(enabled_indent + 2);
    let new_line = format!("{}{}", " ".repeat(indent), HERMES_ITEM);
    let mut out_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    out_lines.insert(insert_after + 1, new_line);
    let mut out = out_lines.join("\n");
    if existing.ends_with('\n') {
        out.push('\n');
    }
    HermesConfigPatch::Patched(out)
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openclaw_merge_fresh_config() {
        let mut root = serde_json::json!({});
        let changed = merge_openclaw_config(&mut root, "/home/u/.openclaw/plugins/trs").unwrap();
        assert!(changed);
        assert_eq!(root["plugins"]["entries"]["trs"]["enabled"], true);
        assert_eq!(
            root["plugins"]["load"]["paths"][0],
            "/home/u/.openclaw/plugins/trs"
        );
    }

    #[test]
    fn openclaw_merge_preserves_existing_entries() {
        let mut root = serde_json::json!({
            "gateway": { "port": 8080 },
            "plugins": {
                "entries": { "other": { "enabled": false } },
                "load": { "paths": ["/opt/plugins"] }
            }
        });
        let changed = merge_openclaw_config(&mut root, "/h/.openclaw/plugins/trs").unwrap();
        assert!(changed);
        assert_eq!(root["gateway"]["port"], 8080);
        assert_eq!(root["plugins"]["entries"]["other"]["enabled"], false);
        assert_eq!(root["plugins"]["entries"]["trs"]["enabled"], true);
        let paths = root["plugins"]["load"]["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "/opt/plugins");
    }

    #[test]
    fn openclaw_merge_is_idempotent() {
        let mut root = serde_json::json!({});
        assert!(merge_openclaw_config(&mut root, "/h/p/trs").unwrap());
        assert!(!merge_openclaw_config(&mut root, "/h/p/trs").unwrap());
        let paths = root["plugins"]["load"]["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 1, "re-run must not duplicate the load path");
    }

    #[test]
    fn openclaw_merge_rejects_non_object_plugins() {
        let mut root = serde_json::json!({ "plugins": [] });
        assert!(merge_openclaw_config(&mut root, "/h/p/trs").is_err());
    }

    #[test]
    fn hermes_patch_empty_file_writes_full_block() {
        assert_eq!(
            patch_hermes_config(""),
            HermesConfigPatch::Patched("plugins:\n  enabled:\n    - trs-rewrite\n".to_string())
        );
    }

    #[test]
    fn hermes_patch_appends_to_existing_enabled_list() {
        let existing = "model: hermes-4\nplugins:\n  enabled:\n    - web-search\nlogging: true\n";
        match patch_hermes_config(existing) {
            HermesConfigPatch::Patched(out) => {
                assert_eq!(
                    out,
                    "model: hermes-4\nplugins:\n  enabled:\n    - web-search\n    - trs-rewrite\nlogging: true\n"
                );
            }
            other => panic!("expected Patched, got {:?}", other),
        }
    }

    #[test]
    fn hermes_patch_matches_sibling_indentation() {
        let existing = "plugins:\n    enabled:\n        - web-search\n";
        match patch_hermes_config(existing) {
            HermesConfigPatch::Patched(out) => {
                assert!(out.contains("\n        - trs-rewrite\n"), "got: {out}");
            }
            other => panic!("expected Patched, got {:?}", other),
        }
    }

    #[test]
    fn hermes_patch_already_present_is_noop() {
        let existing = "plugins:\n  enabled:\n    - trs-rewrite\n";
        assert_eq!(
            patch_hermes_config(existing),
            HermesConfigPatch::AlreadyPresent
        );
    }

    #[test]
    fn hermes_patch_empty_enabled_list_gets_first_item() {
        let existing = "plugins:\n  enabled:\n";
        match patch_hermes_config(existing) {
            HermesConfigPatch::Patched(out) => {
                assert_eq!(out, "plugins:\n  enabled:\n    - trs-rewrite\n");
            }
            other => panic!("expected Patched, got {:?}", other),
        }
    }

    #[test]
    fn hermes_patch_appends_block_when_plugins_key_absent() {
        let existing = "model: hermes-4\n";
        match patch_hermes_config(existing) {
            HermesConfigPatch::Patched(out) => {
                assert_eq!(
                    out,
                    "model: hermes-4\nplugins:\n  enabled:\n    - trs-rewrite\n"
                );
            }
            other => panic!("expected Patched, got {:?}", other),
        }
    }

    #[test]
    fn hermes_patch_exotic_layouts_fall_back_to_manual() {
        // Inline array.
        assert_eq!(
            patch_hermes_config("plugins:\n  enabled: [web-search]\n"),
            HermesConfigPatch::Manual
        );
        // Inline plugins object.
        assert_eq!(
            patch_hermes_config("plugins: {enabled: [web-search]}\n"),
            HermesConfigPatch::Manual
        );
        // plugins key without an enabled list.
        assert_eq!(
            patch_hermes_config("plugins:\n  dirs:\n    - /opt\n"),
            HermesConfigPatch::Manual
        );
    }
}
