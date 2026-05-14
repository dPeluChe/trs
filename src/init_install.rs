//! `trs init` write-paths. Each entry point returns the destination path
//! on success (suffix `(already configured)` when the file was a no-op).
//! All paths honor `opts.dry_run` — they preview instead of writing.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init::{file_has_any_trs_marker, has_trs_marker, home_dir, HookSpec, InstallOpts};
use crate::init_collision;
use crate::init_templates::CODEX_AGENTS_SECTION;

/// Install via the data-driven `HookSpec`. JSON targets merge; non-JSON
/// targets create-only (refuse to overwrite foreign content).
pub(crate) fn install_from_spec(spec: &HookSpec, opts: InstallOpts) -> Result<String, String> {
    if opts.global {
        if let Some(global_dir) = spec.global_dir {
            let home = home_dir()?;
            let dir = home.join(global_dir);
            let path = dir.join(spec.filename);
            return write_hook(&dir, &path, spec.content, opts);
        }
        eprintln!("note: --global not supported for this tool, installing locally instead");
    }
    let dir = PathBuf::from(spec.local_dir);
    let path = dir.join(spec.filename);
    write_hook(&dir, &path, spec.content, opts)
}

/// Append the trs rules block to `AGENTS.md`. `--global` targets
/// `~/.codex/AGENTS.md` (Codex reads it globally); otherwise project root.
/// Idempotent against the trs sentinel + markers.
pub(crate) fn install_codex_agents(opts: InstallOpts) -> Result<String, String> {
    let path = if opts.global {
        home_dir()?.join(".codex").join("AGENTS.md")
    } else {
        PathBuf::from("AGENTS.md")
    };

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if file_has_any_trs_marker(&content) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!("{} (would append trs rules block)", path.display()));
        }
        ensure_parent(&path)?;
        let updated = format!("{}\n{}", content, CODEX_AGENTS_SECTION);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if opts.dry_run {
            return Ok(format!("{} (would create with trs rules)", path.display()));
        }
        ensure_parent(&path)?;
        fs::write(&path, CODEX_AGENTS_SECTION.trim()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

/// Install a rules/instructions file (Antigravity, Windsurf). Project-local
/// only — rules tools lack a global equivalent. Appends if the file exists,
/// preserving any user-written prose.
pub(crate) fn install_rules(
    path_rel: &str,
    content: &str,
    opts: InstallOpts,
) -> Result<String, String> {
    let path = PathBuf::from(path_rel);
    if path.exists() {
        let existing = fs::read_to_string(&path).unwrap_or_default();
        if has_trs_marker(&existing) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!("{} (would append trs rules)", path.display()));
        }
        let updated = format!("{}\n\n{}", existing.trim_end(), content);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if opts.dry_run {
            return Ok(format!("{} (would create with trs rules)", path.display()));
        }
        ensure_parent(&path)?;
        fs::write(&path, content.trim_start()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create {}: {}", parent.display(), e))?;
    }
    Ok(())
}

/// Route hook installs by file type. JSON → merge with existing user hooks;
/// non-JSON (plugin .ts) → create-only, refuse to clobber foreign content.
fn write_hook(dir: &Path, path: &Path, content: &str, opts: InstallOpts) -> Result<String, String> {
    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));

    if is_json {
        return merge_json_hook(dir, path, content, opts);
    }

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
    if opts.dry_run {
        return Ok(format!("{} (would create)", path.display()));
    }
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    fs::write(path, content).map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    Ok(path.display().to_string())
}

/// Merge the `hooks` section from `template` into `path`, preserving other
/// keys (model, auth, permissions) and any user-added hooks that don't
/// reference `trs rewrite`. Idempotent: re-running with the same template
/// is a no-op.
fn merge_json_hook(
    dir: &Path,
    path: &Path,
    template: &str,
    opts: InstallOpts,
) -> Result<String, String> {
    let replace = opts.replace;
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

    // Snapshot before mutable borrows so we can detect true no-ops at the
    // end (matters for template changes that map to the same JSON shape).
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

    // Sweep across every event before re-inserting from the template:
    // templates can migrate event between releases (Cursor moved
    // beforeShellExecution → preToolUse) and we don't want orphans.
    // User entries that don't reference `trs rewrite` are preserved.
    for (_event, event_val) in existing_hooks_obj.iter_mut() {
        if let Some(arr) = event_val.as_array_mut() {
            arr.retain(|e| {
                if contains_trs_rewrite(e) {
                    return false;
                }
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

    if opts.dry_run {
        let verb = if path.exists() {
            "would merge trs hook entries into"
        } else {
            "would create with trs hook"
        };
        return Ok(format!("{} ({})", path.display(), verb));
    }
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
    Ok(path.display().to_string())
}

/// Shared with `uninstall::scrub_trs_from_json` so both sides of the
/// install/uninstall loop key on the same hook signature.
pub(crate) fn contains_trs_rewrite(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::String(s) => s.contains("trs rewrite"),
        serde_json::Value::Object(o) => o.values().any(contains_trs_rewrite),
        serde_json::Value::Array(a) => a.iter().any(contains_trs_rewrite),
        _ => false,
    }
}
