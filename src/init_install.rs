//! `trs init` write-paths. Each entry point returns the destination path
//! on success (suffix `(already configured)` when the file was a no-op).
//! All paths honor `opts.dry_run` — they preview instead of writing.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init::{file_has_any_trs_marker, has_trs_marker, home_dir, HookSpec, InstallOpts};
use crate::init_collision;
use crate::init_templates::{ANTIGRAVITY_RULES_SECTION, CODEX_AGENTS_SECTION, CODEX_HOOKS};

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
///
/// Also defensively scrubs legacy `trs rewrite` entries from
/// `~/.codex/hooks.json` — pre-v0.6.x installs wrote a PreToolUse hook
/// there, but Codex versions vary in `updatedInput` support so orphans
/// cause "unsupported updatedInput" errors on every tool call. Scrub
/// runs only on `--global` since that's where the orphan lives.
pub(crate) fn install_codex_agents(opts: InstallOpts) -> Result<String, String> {
    if opts.global {
        if let Ok(home) = home_dir() {
            let codex_dir = home.join(".codex");
            let hooks_path = codex_dir.join("hooks.json");
            if crate::codex::rewrite_hook_available() {
                // codex >= 0.134 honors updatedInput — install the real
                // PreToolUse rewrite hook (merge preserves the user's other
                // hooks; idempotent).
                match merge_json_hook(&codex_dir, &hooks_path, CODEX_HOOKS, opts) {
                    Ok(msg) => println!("  codex hook: {}", msg),
                    Err(e) => eprintln!("  warning: could not install codex hook: {}", e),
                }
            } else if let Err(e) = scrub_legacy_codex_hook(&hooks_path, opts.dry_run) {
                // Older/undetectable codex rejects updatedInput — scrub any
                // orphan trs entry so it doesn't error on every tool call.
                eprintln!("  warning: could not scrub {}: {}", hooks_path.display(), e);
            }
        }
    }

    let path = if opts.global {
        home_dir()?.join(".codex").join("AGENTS.md")
    } else {
        PathBuf::from("AGENTS.md")
    };
    let rules_msg = write_agents_md_block(&path, opts)?;

    // Compose the output-saver reply-brevity rules as their own
    // sentinel-managed block instead of embedding a (sentinel-less) copy in
    // CODEX_AGENTS_SECTION. The installer is idempotent — re-runs replace the
    // block in place — so `trs init codex` + `trs output-saver --install`
    // can run in any order without duplicating it. Global only: the
    // output-saver installer targets ~/.codex/AGENTS.md.
    if opts.global && !opts.dry_run {
        if let Err(e) = crate::output_saver::install_agent("codex") {
            eprintln!("  note: output-saver block install failed: {}", e);
        }
    }
    Ok(rules_msg)
}

/// Append the trs sentinel block to an `AGENTS.md`. Shared write path for
/// Codex (project + global) and Zed (project only) — one template, no copies.
fn write_agents_md_block(path: &Path, opts: InstallOpts) -> Result<String, String> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        if file_has_any_trs_marker(&content) {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!("{} (would append trs rules block)", path.display()));
        }
        ensure_parent(path)?;
        let updated = format!("{}\n{}", content, CODEX_AGENTS_SECTION);
        fs::write(path, updated).map_err(|e| e.to_string())?;
        return Ok(path.display().to_string());
    }
    if opts.dry_run {
        return Ok(format!("{} (would create with trs rules)", path.display()));
    }
    ensure_parent(path)?;
    fs::write(path, CODEX_AGENTS_SECTION.trim()).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// Zed Agent Panel — rules-only. The native agent reads the project
/// `AGENTS.md` as always-on instructions (no tool hooks: zed#52688), so the
/// sentinel block IS the integration. ACP external agents (Claude Code,
/// Codex, Gemini CLI, OpenCode) are covered by their own trs hooks.
pub(crate) fn install_zed_agents(opts: InstallOpts) -> Result<String, String> {
    write_agents_md_block(Path::new("AGENTS.md"), opts)
}

/// Append the Antigravity rules block to `~/.gemini/GEMINI.md`. Shared by
/// both Antigravity IDE and Antigravity CLI (`agy`) — same target file.
///
/// Idempotent against the antigravity rules sentinel + general trs marker.
/// Also defensively scrubs orphaned v0.6.5 jetski hook entries from
/// `~/.gemini/antigravity-{cli,ide}/hooks.json` and the v0.6.4 BeforeTool
/// entry from `~/.gemini/settings.json` — both wrote installs that never
/// actually fired (see docs/development/antigravity-hooks-research.md).
pub(crate) fn install_antigravity_rules(opts: InstallOpts) -> Result<String, String> {
    if let Ok(home) = home_dir() {
        // Best-effort: don't fail the rules install if a scrub hits an FS
        // edge case; log and continue. These files might not exist at all
        // (user is on a fresh machine), which is also a no-op.
        let _ = remove_hookspec_at(
            &home.join(".gemini/antigravity-cli/hooks.json"),
            opts.dry_run,
        );
        let _ = remove_hookspec_at(
            &home.join(".gemini/antigravity-ide/hooks.json"),
            opts.dry_run,
        );
        let _ = scrub_legacy_codex_hook(&home.join(".gemini/settings.json"), opts.dry_run);
    }

    let path = home_dir()?.join(".gemini").join("GEMINI.md");

    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if file_has_any_trs_marker(&content)
            || content.contains(crate::init_templates::ANTIGRAVITY_RULES_SENTINEL_START)
        {
            return Ok(format!("{} (already configured)", path.display()));
        }
        if opts.dry_run {
            return Ok(format!(
                "{} (would append Antigravity trs rules block)",
                path.display()
            ));
        }
        ensure_parent(&path)?;
        let updated = format!("{}\n{}", content, ANTIGRAVITY_RULES_SECTION);
        fs::write(&path, updated).map_err(|e| e.to_string())?;
    } else {
        if opts.dry_run {
            return Ok(format!(
                "{} (would create with Antigravity trs rules)",
                path.display()
            ));
        }
        ensure_parent(&path)?;
        fs::write(&path, ANTIGRAVITY_RULES_SECTION.trim()).map_err(|e| e.to_string())?;
    }
    Ok(path.display().to_string())
}

/// Remove the trs JSON hook file outright if it exists AND only contains
/// our trs-rewrite entry. Used by the Antigravity v0.6.5 → v0.6.6
/// migration: the file we shipped was never fired, so it's safe to drop.
fn remove_hookspec_at(path: &Path, dry_run: bool) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !content.contains("trs rewrite") {
        return Ok(());
    }
    if dry_run {
        println!("  would remove orphaned {}", path.display());
        return Ok(());
    }
    fs::remove_file(path).map_err(|e| e.to_string())?;
    println!("  removed orphaned {}", path.display());
    Ok(())
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

/// Stable marker identifying a trs-authored plugin file (OpenCode/Kilo).
/// Present in every version of the template, so we can recognize our own
/// file and refresh it when the bundled template changes (e.g. the #53
/// Windows fix) instead of leaving a stale copy in place.
const TRS_PLUGIN_MARKER: &str = "trs plugin — route commands through trs";

/// Route hook installs by file type. JSON → merge with existing user hooks;
/// non-JSON (plugin .ts) → create, or refresh our own file on drift; never
/// clobber foreign content.
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
        let is_ours = has_trs_marker(&existing) || existing.contains(TRS_PLUGIN_MARKER);
        if is_ours {
            if existing == content {
                return Ok(format!("{} (already configured)", path.display()));
            }
            // Our file but stale (older template) — refresh it so fixes like
            // the #53 Windows plugin land on re-init without --force.
            if opts.dry_run {
                return Ok(format!("{} (would refresh trs plugin)", path.display()));
            }
            fs::write(path, content)
                .map_err(|e| format!("Cannot write {}: {}", path.display(), e))?;
            return Ok(format!("{} (refreshed)", path.display()));
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

/// Remove any `trs rewrite` entries from `~/.codex/hooks.json` while
/// preserving user-added hooks. Returns `Some(msg)` if anything changed.
/// No-op when the file is missing or already clean.
pub(crate) fn scrub_legacy_codex_hook(
    path: &Path,
    dry_run: bool,
) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !content.contains("trs rewrite") {
        return Ok(None);
    }
    let mut val: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("not valid JSON: {}", e))?;
    let mut removed = 0usize;
    let mut empty_events: Vec<String> = Vec::new();
    if let Some(hooks) = val.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for (event, event_val) in hooks.iter_mut() {
            if let Some(arr) = event_val.as_array_mut() {
                let before = arr.len();
                arr.retain(|e| !contains_trs_rewrite(e));
                removed += before - arr.len();
                if arr.is_empty() && before > 0 {
                    empty_events.push(event.clone());
                }
            }
        }
        // Drop event keys that we emptied — leaves a tidier file for
        // users who only had the trs entry (and matches what they'd
        // expect after "uninstall").
        for k in &empty_events {
            hooks.remove(k);
        }
    }
    if removed == 0 {
        return Ok(None);
    }
    if dry_run {
        return Ok(Some(format!(
            "would scrub {} legacy trs entry/entries from {}",
            removed,
            path.display()
        )));
    }
    let pretty = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    fs::write(path, format!("{}\n", pretty)).map_err(|e| e.to_string())?;
    println!(
        "    scrubbed {} legacy trs entry/entries from {}",
        removed,
        crate::path_display::tilde(&path.display().to_string())
    );
    Ok(Some(format!(
        "scrubbed {} from {}",
        removed,
        path.display()
    )))
}

#[cfg(test)]
#[path = "init_install_tests.rs"]
mod tests;
