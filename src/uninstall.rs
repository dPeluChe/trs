//! `trs uninstall`: remove trs hooks, plugins, rules, and output-saver
//! blocks from agent configs. Interactive by default; flags shortcut the
//! prompt for scripts.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::init::{file_has_any_trs_marker, AiTool};
use crate::init_templates::{CODEX_AGENTS_SENTINEL_END, CODEX_AGENTS_SENTINEL_START};
use crate::uninstall_scrub::{
    confirm, delete_plugin_file, delete_rules_file, has_output_saver_installed, is_json,
    is_trs_plugin_dir_file, output_saver_agent_id, remove_between_sentinels, remove_output_saver,
    run_output_saver_removal, scrub_trs_from_json,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct UninstallOpts {
    pub all: bool,
    pub output_saver: bool,
    pub dry_run: bool,
    pub yes: bool,
}

const DRY_RUN_NOTE: &str = "note: dry-run: nothing was written. Re-run without --dry-run to apply.";

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

    println!("trs uninstall: interactive\n");
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
        eprintln!("read error, aborting");
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
        eprintln!("no valid selection, aborting");
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
        let result = if is_trs_plugin_dir_file(&path) {
            // Files under a dir named trs/trs-rewrite are ours by
            // location — delete, never scrub (the OpenClaw manifest is
            // .json but has no `hooks` key to scrub).
            delete_plugin_file(&path, opts.dry_run)
        } else if is_json(&path) {
            scrub_trs_from_json(&path, opts.dry_run)
        } else if path.ends_with("AGENTS.md") {
            remove_between_sentinels(
                &path,
                CODEX_AGENTS_SENTINEL_START,
                CODEX_AGENTS_SENTINEL_END,
                opts.dry_run,
            )
        } else if path.ends_with("GEMINI.md") {
            // v0.6.6+ Antigravity rules block. The sentinels keep it from
            // being mistaken for the Gemini CLI output-saver `@trs.md`
            // import line (which lives in the same file but is managed
            // separately by `output_saver::remove_agent`).
            remove_between_sentinels(
                &path,
                crate::init_templates::ANTIGRAVITY_RULES_SENTINEL_START,
                crate::init_templates::ANTIGRAVITY_RULES_SENTINEL_END,
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
    // v1 leaves the OpenClaw/Hermes config enable entries alone — they
    // point at removed files harmlessly. Tell the user how to finish.
    match tool {
        AiTool::OpenClaw => actions.push(
            "note: remove `plugins.entries.trs` from ~/.openclaw/openclaw.json manually if desired"
                .to_string(),
        ),
        AiTool::Hermes => actions.push(
            "note: remove `trs-rewrite` from plugins.enabled in ~/.hermes/config.yaml manually if desired"
                .to_string(),
        ),
        _ => {}
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
        AiTool::Pi => {
            push_home(".pi/agent/extensions/trs.ts");
            v.push(PathBuf::from(".pi/extensions/trs.ts"));
        }
        AiTool::Codex => {
            push_home(".codex/AGENTS.md");
            // Legacy installs (pre-v0.6.x) wrote `trs rewrite` into
            // `~/.codex/hooks.json`. Codex versions vary in `updatedInput`
            // support — orphans cause "unsupported updatedInput" errors
            // on every PreToolUse. Sweep them here so uninstall removes
            // the trs entries even though current installs no longer write
            // there. Ordering matters: all push_home calls run before any
            // direct v.push to keep the closure's mutable borrow contiguous.
            push_home(".codex/hooks.json");
            v.push(PathBuf::from("AGENTS.md"));
        }
        AiTool::Antigravity | AiTool::AntigravityCLI => {
            // v0.6.6 reclassified Antigravity to rules-only. Sweep covers
            // every install variant that previous trs versions wrote, so
            // upgrades produce a clean state:
            //   - v0.6.6+ rules block in `~/.gemini/GEMINI.md` (current)
            //   - v0.6.5  `~/.gemini/antigravity-{cli,ide}/hooks.json`
            //     (jetski PreToolUse — never actually fired)
            //   - v0.6.4  BeforeTool entry in `~/.gemini/settings.json`
            //     (aliased to Gemini's harness — also never fired)
            //   - pre-v0.6.4 `.agent/rules/antigravity-trs-rules.md`
            //     per-project rules file
            push_home(".gemini/GEMINI.md");
            push_home(".gemini/antigravity-ide/hooks.json");
            push_home(".gemini/antigravity-cli/hooks.json");
            push_home(".gemini/settings.json");
            v.push(PathBuf::from(".gemini/antigravity-ide/hooks.json"));
            v.push(PathBuf::from(".gemini/antigravity-cli/hooks.json"));
            v.push(PathBuf::from(".gemini/settings.json"));
            v.push(PathBuf::from(".agent/rules/antigravity-trs-rules.md"));
        }
        AiTool::Devin => {
            v.push(PathBuf::from(".devin/rules/trs.md"));
            v.push(PathBuf::from(".windsurfrules"));
        }
        AiTool::VsCode => {
            push_home(".copilot/hooks/trs.json");
            v.push(PathBuf::from(".github/hooks/trs.json"));
        }
        AiTool::OpenClaw => {
            push_home(".openclaw/plugins/trs/openclaw.plugin.json");
            push_home(".openclaw/plugins/trs/index.js");
        }
        AiTool::Hermes => {
            // Honors HERMES_HOME the same way the installer does.
            if let Ok(h) = crate::init_install_plugins::hermes_home() {
                v.push(h.join("plugins/trs-rewrite/__init__.py"));
                v.push(h.join("plugins/trs-rewrite/plugin.yaml"));
            }
        }
        // Same project AGENTS.md as Codex — sentinel scrub is shared.
        AiTool::Zed => v.push(PathBuf::from("AGENTS.md")),
        AiTool::DevinCLI => {
            // JSON merge target — scrub_trs_from_json drops the `trs rewrite`
            // entry and preserves the user's model/org_id/theme config.
            push_home(".config/devin/config.json");
            v.push(PathBuf::from(".devin/config.json"));
        }
    }
    v.sort();
    v.dedup();
    v
}

fn has_trs_artifacts(tool: &AiTool) -> bool {
    candidate_paths(tool).iter().any(|p| {
        if !p.exists() {
            return false;
        }
        // Plugin files (`.ts`, or anything in a trs/trs-rewrite plugin
        // dir) are 100% ours by name — existence is enough.
        if p.extension().and_then(|e| e.to_str()) == Some("ts") || is_trs_plugin_dir_file(p) {
            return true;
        }
        fs::read_to_string(p)
            .map(|c| file_has_any_trs_marker(&c))
            .unwrap_or(false)
    })
}
