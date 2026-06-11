//! Custom installers for the plugin-dir agents (OpenClaw). Global-only:
//! plugin files under the agent's home dir plus a config-file enable
//! entry. Shipped from 2026-06-11 docs research — live validation
//! pending (see docs/support/agents.md per-agent status).

use std::fs;
use std::path::Path;

use crate::init::{home_dir, InstallOpts};

// Validate live: manifest schema (docs verified 2026-06-11; `entry` points
// at the plugin module file).
const OPENCLAW_PLUGIN_MANIFEST: &str = r#"{
  "id": "trs",
  "name": "trs",
  "description": "Route exec commands through trs for token-optimized output",
  "entry": "index.js"
}
"#;

const OPENCLAW_PLUGIN_INDEX: &str = r#"// trs plugin — route exec commands through trs for token-optimized output
// Validate live: hook payload field names + manifest schema (docs verified
// 2026-06-11: before_tool_call rewrites params; resolve_exec_env merges env).
export default {
  id: "trs",
  register(api) {
    api.on("before_tool_call", (event) => {
      if (event.toolName !== "exec") return;
      const cmd = event.params?.command;
      if (typeof cmd !== "string") return;
      // Idempotent: skip anything already routed through trs (or a cd).
      if (cmd.startsWith("trs ") || cmd.startsWith("cd ") || cmd.startsWith("TRS_AGENT=")) return;
      return { params: { ...event.params, command: `trs ${cmd}` } };
    });
    // Cross-platform attribution: env var, never a shell prefix.
    api.on("resolve_exec_env", () => ({ TRS_AGENT: "openclaw" }));
  },
};
"#;

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
}
