//! File-level surgery for `trs uninstall`: the helpers that actually touch
//! disk, split out of `uninstall.rs` so that file stays about the flow
//! (which tool, which prompt, what to report) rather than about JSON
//! scrubbing and sentinel removal.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::init::{file_has_any_trs_marker, AiTool};
use crate::init_install::contains_trs_rewrite;

/// Filter `hooks.*[]` arrays in a JSON config to drop entries whose
/// `command` contains `trs rewrite`. Empty arrays are kept (they're
/// user-shaped — we don't reorganize). Returns `Some(msg)` if the file
/// changed, `None` if it was a no-op (file missing or already clean).
pub(crate) fn scrub_trs_from_json(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !content.contains("trs rewrite") {
        return Ok(None);
    }
    let mut val: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let mut removed = 0;
    if let Some(hooks) = val.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for (_event, event_val) in hooks.iter_mut() {
            if let Some(arr) = event_val.as_array_mut() {
                let before = arr.len();
                arr.retain(|e| !contains_trs_rewrite(e));
                removed += before - arr.len();
            }
        }
    }
    if removed == 0 {
        return Ok(None);
    }
    if dry_run {
        return Ok(Some(format!(
            "would scrub {} trs entry/entries from {}",
            removed,
            path.display()
        )));
    }
    let pretty = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    fs::write(path, format!("{}\n", pretty)).map_err(|e| e.to_string())?;
    Ok(Some(format!(
        "scrubbed {} trs entry/entries from {}",
        removed,
        path.display()
    )))
}

/// Remove the sentinel-delimited block from a text file. When the whole
/// file was just our block, delete the file instead of leaving an empty one.
pub(crate) fn remove_between_sentinels(
    path: &Path,
    start: &str,
    end: &str,
    dry_run: bool,
) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !content.contains(start) {
        return Ok(None);
    }
    if dry_run {
        return Ok(Some(format!(
            "would remove trs block from {}",
            path.display()
        )));
    }
    let stripped = crate::output_saver::replace_between(&content, start, end, "");
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
        return Ok(Some(format!("removed empty {}", path.display())));
    }
    fs::write(path, format!("{}\n", trimmed)).map_err(|e| e.to_string())?;
    Ok(Some(format!("removed trs block from {}", path.display())))
}

pub(crate) fn delete_plugin_file(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    if dry_run {
        return Ok(Some(format!("would delete {}", path.display())));
    }
    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(Some(format!("deleted {}", path.display())))
}

/// Delete a rules file (Antigravity / Windsurf). Whole file is ours; the
/// install path wrote it end-to-end, so there's no surgical-remove option.
pub(crate) fn delete_rules_file(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    // Match all marker variants — legacy `trs (TARS CLI)` (≤ v0.5.8),
    // modern `trs (Token-Reducing Shell)`, sentinels, etc.
    if !file_has_any_trs_marker(&content) {
        return Ok(None);
    }
    if dry_run {
        return Ok(Some(format!("would delete {}", path.display())));
    }
    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(Some(format!("deleted {}", path.display())))
}

pub(crate) fn remove_output_saver(agent_id: &str, dry_run: bool) -> Result<String, String> {
    if dry_run {
        return Ok(format!("would remove output-saver block ({})", agent_id));
    }
    crate::output_saver::remove_agent(agent_id)
}

/// Best-effort: does this agent have any output-saver artifact on disk?
/// Imported agents (claude / gemini) store it as a sidecar `trs.md`;
/// inline agents (codex / cursor / devin) wrap it with sentinels in
/// their primary rules file.
pub(crate) fn has_output_saver_installed(agent_id: &str) -> bool {
    let Ok(home) = crate::init::home_dir() else {
        return false;
    };
    let sidecar_path = match agent_id {
        "claude" => Some(home.join(".claude/trs.md")),
        "gemini" => Some(home.join(".gemini/trs.md")),
        _ => None,
    };
    if let Some(p) = sidecar_path {
        if p.exists() {
            return true;
        }
    }
    let inline_paths: Vec<PathBuf> = match agent_id {
        "codex" => vec![home.join(".codex/AGENTS.md"), PathBuf::from("AGENTS.md")],
        "cursor" => vec![home.join(".cursor/.cursorrules")],
        "devin" => vec![
            PathBuf::from(".devin/rules/trs.md"),
            PathBuf::from(".windsurfrules"),
        ],
        _ => vec![],
    };
    inline_paths.iter().any(|p| {
        p.exists()
            && fs::read_to_string(p)
                .map(|c| c.contains(crate::output_saver::SENTINEL_START))
                .unwrap_or(false)
    })
}

pub(crate) fn run_output_saver_removal(tool_name: &str, agent_id: &str, dry_run: bool) {
    if !has_output_saver_installed(agent_id) {
        return;
    }
    match remove_output_saver(agent_id, dry_run) {
        Ok(msg) => println!("  - {}: {}", tool_name, msg),
        Err(e) => eprintln!("  ! {} output-saver: {}", tool_name, e),
    }
}

pub(crate) fn output_saver_agent_id(tool: &AiTool) -> Option<&'static str> {
    match tool {
        AiTool::Claude => Some("claude"),
        AiTool::Gemini => Some("gemini"),
        AiTool::Codex => Some("codex"),
        AiTool::Cursor => Some("cursor"),
        AiTool::Devin => Some("devin"),
        // Both Antigravity variants share Gemini's trs.md — uninstalling
        // either touches the same file. We return their respective
        // agent_ids so the per-tool report names them clearly; the
        // underlying `output_saver::remove_agent` is idempotent.
        AiTool::Antigravity => Some("antigravity"),
        AiTool::AntigravityCLI => Some("antigravity-cli"),
        _ => None,
    }
}

/// True when the file lives in one of our dedicated plugin dirs
/// (`…/plugins/trs/` for OpenClaw, `…/plugins/trs-rewrite/` for Hermes).
pub(crate) fn is_trs_plugin_dir_file(path: &Path) -> bool {
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());
    matches!(parent_name, Some("trs") | Some("trs-rewrite"))
        && path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some("plugins")
}

pub(crate) fn is_json(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
}

pub(crate) fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}
