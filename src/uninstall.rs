//! `trs uninstall` — remove trs hooks, plugins, rules, and output-saver
//! blocks from agent configs. Interactive by default; flags shortcut the
//! prompt for scripts.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::init::{file_has_any_trs_marker, AiTool};
use crate::init_install::contains_trs_rewrite;
use crate::init_templates::{CODEX_AGENTS_SENTINEL_END, CODEX_AGENTS_SENTINEL_START};

#[derive(Debug, Clone, Copy)]
pub(crate) struct UninstallOpts {
    pub all: bool,
    pub output_saver: bool,
    pub dry_run: bool,
    pub yes: bool,
}

const DRY_RUN_NOTE: &str =
    "note: dry-run — nothing was written. Re-run without --dry-run to apply.";

pub(crate) fn run_uninstall(tool: Option<&str>, opts: UninstallOpts) {
    if opts.output_saver {
        run_output_saver_only(opts);
        return;
    }
    if opts.all {
        run_all(opts);
        return;
    }
    if let Some(name) = tool {
        match AiTool::from_str(name) {
            Some(t) => {
                if !opts.yes && !opts.dry_run && !confirm(&format!("Remove trs from {}?", t.name()))
                {
                    println!("aborted");
                    return;
                }
                uninstall_one(&t, opts);
            }
            None => eprintln!(
                "Unknown tool: '{}'. Supported: {}",
                name,
                AiTool::all_names()
            ),
        }
        return;
    }
    run_interactive(opts);
}

fn run_all(opts: UninstallOpts) {
    if !opts.yes && !opts.dry_run && !confirm("Remove trs from EVERY detected agent?") {
        println!("aborted");
        return;
    }
    for tool in &AiTool::all_tools() {
        uninstall_one(tool, opts);
    }
    print_dry_run_note(opts);
}

fn run_output_saver_only(opts: UninstallOpts) {
    if !opts.yes
        && !opts.dry_run
        && !confirm("Remove the trs output-saver block from every agent that has it?")
    {
        println!("aborted");
        return;
    }
    for tool in &AiTool::all_tools() {
        if let Some(agent_id) = output_saver_agent_id(tool) {
            run_output_saver_removal(tool.name(), agent_id, opts.dry_run);
        }
    }
    print_dry_run_note(opts);
}

fn print_dry_run_note(opts: UninstallOpts) {
    if opts.dry_run {
        eprintln!("{}", DRY_RUN_NOTE);
    }
}

fn run_interactive(opts: UninstallOpts) {
    let tools = AiTool::all_tools();
    let installed: Vec<&AiTool> = tools.iter().filter(|t| has_trs_artifacts(t)).collect();
    if installed.is_empty() {
        println!("No trs installation detected. Nothing to remove.");
        return;
    }

    println!("trs uninstall — interactive\n");
    println!("Installed:");
    for (i, t) in installed.iter().enumerate() {
        println!("  [{}] {:<20} {}", i + 1, t.name(), t.target_label());
    }
    println!("  [s] output-saver blocks only (preserve hooks)");
    println!("  [a] all of the above");
    println!("  [q] quit");
    print!("\nPick (e.g. 1,3 or 'a'): ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line).is_err() {
        eprintln!("read error — aborting");
        return;
    }
    let pick = line.trim();
    if pick.is_empty() || pick == "q" {
        println!("aborted");
        return;
    }

    if pick == "s" {
        run_output_saver_only(UninstallOpts { yes: true, ..opts });
        return;
    }
    if pick == "a" {
        for t in &installed {
            uninstall_one(t, opts);
        }
        print_dry_run_note(opts);
        return;
    }

    let selected: Vec<&AiTool> = pick
        .split(',')
        .filter_map(|tok| {
            let n: usize = tok.trim().parse().ok()?;
            if n >= 1 && n <= installed.len() {
                Some(*installed.get(n - 1)?)
            } else {
                None
            }
        })
        .collect();
    if selected.is_empty() {
        eprintln!("no valid selection — aborting");
        return;
    }
    for t in &selected {
        uninstall_one(t, opts);
    }
    print_dry_run_note(opts);
}

/// Per-tool uninstall — dispatches by install surface. Walks every path
/// the install path could have written to. Silent when nothing matches.
fn uninstall_one(tool: &AiTool, opts: UninstallOpts) {
    let mut actions: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for path in candidate_paths(tool) {
        let result = if is_json(&path) {
            scrub_trs_from_json(&path, opts.dry_run)
        } else if path.ends_with("AGENTS.md") {
            remove_between_sentinels(
                &path,
                CODEX_AGENTS_SENTINEL_START,
                CODEX_AGENTS_SENTINEL_END,
                opts.dry_run,
            )
        } else if path.extension().and_then(|e| e.to_str()) == Some("ts") {
            delete_plugin_file(&path, opts.dry_run)
        } else {
            delete_rules_file(&path, opts.dry_run)
        };
        match result {
            Ok(Some(msg)) => actions.push(msg),
            Ok(None) => {}
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // Imported agents — only act when output-saver is actually installed.
    if let Some(agent_id) = output_saver_agent_id(tool) {
        if has_output_saver_installed(agent_id) {
            match remove_output_saver(agent_id, opts.dry_run) {
                Ok(msg) => actions.push(msg),
                Err(e) => errors.push(format!("output-saver: {}", e)),
            }
        }
    }

    if actions.is_empty() && errors.is_empty() {
        return;
    }
    let verb = if opts.dry_run {
        "would remove from"
    } else {
        "removed from"
    };
    println!("trs {} {}:", verb, tool.name());
    for msg in &actions {
        println!("  - {}", msg);
    }
    for e in &errors {
        eprintln!("  ! {}", e);
    }
}

/// Every path this tool's install could have written — both the
/// `~/.tool/...` (global) and `./.tool/...` (project-local) variants so
/// uninstall catches installs run from either flag.
fn candidate_paths(tool: &AiTool) -> Vec<PathBuf> {
    let home = crate::init::home_dir().ok();
    let mut v: Vec<PathBuf> = Vec::new();
    let mut push_home = |rel: &str| {
        if let Some(h) = &home {
            v.push(h.join(rel));
        }
    };
    match tool {
        AiTool::Claude => {
            push_home(".claude/hooks.json");
            push_home(".claude/settings.json");
            v.push(PathBuf::from("hooks/hooks.json"));
        }
        AiTool::Gemini => {
            push_home(".gemini/settings.json");
            v.push(PathBuf::from(".gemini/settings.json"));
        }
        AiTool::Cursor => {
            push_home(".cursor/hooks.json");
            v.push(PathBuf::from(".cursor/hooks.json"));
        }
        AiTool::Droid => {
            push_home(".factory/settings.json");
            v.push(PathBuf::from(".factory/settings.json"));
        }
        AiTool::OpenCode => {
            push_home(".config/opencode/plugins/trs.ts");
            v.push(PathBuf::from(".opencode/plugins/trs.ts"));
        }
        AiTool::Kilo => {
            push_home(".config/kilo/plugins/trs.ts");
            v.push(PathBuf::from(".kilo/plugins/trs.ts"));
        }
        AiTool::Codex => {
            push_home(".codex/AGENTS.md");
            v.push(PathBuf::from("AGENTS.md"));
        }
        AiTool::Antigravity => {
            v.push(PathBuf::from(".agent/rules/antigravity-trs-rules.md"));
        }
        AiTool::Windsurf => {
            v.push(PathBuf::from(".windsurfrules"));
        }
    }
    v.sort();
    v.dedup();
    v
}

/// Filter `hooks.*[]` arrays in a JSON config to drop entries whose
/// `command` contains `trs rewrite`. Empty arrays are kept (they're
/// user-shaped — we don't reorganize). Returns `Some(msg)` if the file
/// changed, `None` if it was a no-op (file missing or already clean).
fn scrub_trs_from_json(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
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
fn remove_between_sentinels(
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

fn delete_plugin_file(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
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
fn delete_rules_file(path: &Path, dry_run: bool) -> Result<Option<String>, String> {
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

fn remove_output_saver(agent_id: &str, dry_run: bool) -> Result<String, String> {
    if dry_run {
        return Ok(format!("would remove output-saver block ({})", agent_id));
    }
    crate::output_saver::remove_agent(agent_id)
}

/// Best-effort: does this agent have any output-saver artifact on disk?
/// Imported agents (claude / gemini) store it as a sidecar `trs.md`;
/// inline agents (codex / cursor / windsurf) wrap it with sentinels in
/// their primary rules file.
fn has_output_saver_installed(agent_id: &str) -> bool {
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
        "windsurf" => vec![PathBuf::from(".windsurfrules")],
        _ => vec![],
    };
    inline_paths.iter().any(|p| {
        p.exists()
            && fs::read_to_string(p)
                .map(|c| c.contains(crate::output_saver::SENTINEL_START))
                .unwrap_or(false)
    })
}

fn run_output_saver_removal(tool_name: &str, agent_id: &str, dry_run: bool) {
    if !has_output_saver_installed(agent_id) {
        return;
    }
    match remove_output_saver(agent_id, dry_run) {
        Ok(msg) => println!("  - {}: {}", tool_name, msg),
        Err(e) => eprintln!("  ! {} output-saver: {}", tool_name, e),
    }
}

fn has_trs_artifacts(tool: &AiTool) -> bool {
    candidate_paths(tool).iter().any(|p| {
        if !p.exists() {
            return false;
        }
        // Plugin files (`.ts`) are 100% ours by name — existence is enough.
        if p.extension().and_then(|e| e.to_str()) == Some("ts") {
            return true;
        }
        fs::read_to_string(p)
            .map(|c| file_has_any_trs_marker(&c))
            .unwrap_or(false)
    })
}

fn output_saver_agent_id(tool: &AiTool) -> Option<&'static str> {
    match tool {
        AiTool::Claude => Some("claude"),
        AiTool::Gemini => Some("gemini"),
        AiTool::Codex => Some("codex"),
        AiTool::Cursor => Some("cursor"),
        AiTool::Windsurf => Some("windsurf"),
        _ => None,
    }
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}
