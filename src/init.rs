//! `trs init` — Install hooks for AI coding tools.
//!
//! Generates configuration files that make the AI tool automatically
//! route commands through trs for token-optimized output.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init_collision;
use crate::init_install::{
    install_antigravity_rules, install_codex_agents, install_from_spec, install_rules,
    install_zed_agents,
};
use crate::init_install_plugins::{install_hermes_plugin, install_openclaw_plugin};
use crate::init_templates::{DEVIN_RULE, WINDSURF_RULES};

/// Options for an install run. `global` picks home-dir vs project-local;
/// `replace` scrubs competing compressor hooks before installing trs;
/// `force` installs anyway when a collision is present; `dry_run` prints
/// what would change without touching the filesystem.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InstallOpts {
    pub global: bool,
    pub replace: bool,
    pub force: bool,
    pub dry_run: bool,
}

pub(crate) use crate::ai_tool::{AiTool, HookSpec};

/// Column width for the agent-name field in aligned `--all` / refresh output.
const AGENT_COL: usize = 20;

/// Print one aligned status row: `  <sym> <name>        <detail>`. Keeps the
/// batch install/refresh output scannable — one line per agent, columns lined
/// up, no repeated boilerplate.
fn agent_row(sym: char, name: &str, detail: &str) {
    println!("  {} {:<width$}  {}", sym, name, detail, width = AGENT_COL);
}

/// Install hooks for the specified tool. `batch` = true when called from
/// `install_all`: emits a single aligned row and suppresses the per-agent
/// "restart" note (printed once at the end) so `trs init --all` stays
/// readable. Returns whether the hook was installed/refreshed successfully.
pub(crate) fn install_hook(tool: &AiTool, opts: InstallOpts, batch: bool) -> bool {
    // Pre-install: detect competing compressor hooks (rtk, token-optimizer).
    // Default is to abort — --replace scrubs known competitors before the
    // install proceeds, --force installs anyway and eats the risk.
    let collisions = init_collision::detect(tool, opts.global);
    if !collisions.is_empty() && !opts.force && !opts.replace {
        eprintln!("{}", init_collision::format_report(tool, &collisions));
        return false;
    }
    if !collisions.is_empty() && opts.replace && !init_collision::any_hook_collisions(&collisions) {
        // --replace has no automatic cleanup for text-file rules collisions;
        // surface that clearly rather than silently doing nothing.
        eprintln!("{}", init_collision::format_report(tool, &collisions));
        eprintln!(
            "note: --replace only scrubs JSON hook entries automatically.\n\
             The rules-file collisions above need manual edits."
        );
        return false;
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
            if opts.dry_run {
                println!(
                    "  would scrub competitor hook from {}",
                    c.location.display()
                );
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
        AiTool::Codex => install_codex_agents(opts),
        AiTool::Zed => {
            if opts.global {
                // Standalone `trs init zed --global` lands here; the batch path
                // (install_all) short-circuits Zed+global to a compact row.
                eprintln!(
                    "Zed reads the project AGENTS.md, global personal-instructions location \
                     not yet verified; installing would be a no-op. Run without --global in \
                     each project."
                );
                return false;
            }
            install_zed_agents(opts)
        }
        AiTool::Antigravity | AiTool::AntigravityCLI => install_antigravity_rules(opts),
        AiTool::OpenClaw => install_openclaw_plugin(opts),
        AiTool::Hermes => install_hermes_plugin(opts),
        AiTool::Devin => {
            // Forward target for Devin Desktop; legacy `.windsurfrules` for
            // pre-rebrand Windsurf. Devin reads both — write only one to
            // avoid loading the rule twice into context.
            if devin_desktop_present() {
                install_rules(".devin/rules/trs.md", DEVIN_RULE, opts)
            } else {
                install_rules(".windsurfrules", WINDSURF_RULES, opts)
            }
        }
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
            let shown = crate::path_display::tilde(&path);
            if batch {
                // One aligned row; the summary line + restart note are printed
                // once by install_all, which also prints the write root — so
                // drop the shared `~/` prefix here to avoid repeating it.
                let row = if opts.global {
                    shown.strip_prefix("~/").unwrap_or(&shown)
                } else {
                    &shown
                };
                agent_row('+', tool.name(), row);
                if !opts.dry_run {
                    install_trs_md_for(tool, true);
                }
            } else {
                let verb = if opts.dry_run {
                    "would install"
                } else {
                    "installed"
                };
                println!("trs hook {} for {} at {}", verb, tool.name(), shown);
                if !opts.dry_run {
                    eprintln!(
                        "note: restart any open {} sessions for the hook to take effect",
                        tool.name()
                    );
                    // Warn if trs is not in PATH
                    if !is_trs_in_path() {
                        eprintln!(
                            "warning: 'trs' not found in PATH. The hook may fail silently.\n\
                             Make sure trs is installed: npm install -g @dpeluche/trs\n\
                             (or cargo install trs-cli, or curl-sh script, see README)"
                        );
                    }
                    // For Imported agents (Claude, Gemini): also write trs.md so the
                    // agent config gets both the hook and the output-saver/input-rewrite rules.
                    install_trs_md_for(tool, false);
                }
            }
            true
        }
        Err(e) => {
            if batch {
                agent_row('!', tool.name(), &format!("error: {}", e));
            } else {
                eprintln!("Failed to install hook for {}: {}", tool.name(), e);
            }
            false
        }
    }
}

/// Write `trs.md` (output-saver + input-rewrite rules) for agents that load
/// it via an `@import` line (Claude Code, Gemini CLI). No-op for other agents.
fn install_trs_md_for(tool: &AiTool, quiet: bool) {
    let agent_id = match tool {
        AiTool::Claude => "claude",
        AiTool::Gemini => "gemini",
        _ => return,
    };
    match crate::output_saver::install_agent(agent_id) {
        // In batch mode the agent already has its own aligned row; the trs.md
        // write is an implementation detail, so stay silent unless it fails.
        Ok(msg) => {
            if !quiet {
                println!("  trs.md: {}", msg);
            }
        }
        Err(e) => eprintln!("  note: trs.md install failed for {}: {}", tool.name(), e),
    }
}

/// Install hooks for all detected tools, skipping already-configured ones.
/// Tools not detected on the system are reported but not installed.
pub(crate) fn install_all(opts: InstallOpts) {
    let tools = AiTool::all_tools();
    let mut installed = 0;
    let mut configured = 0;
    let mut skipped = 0;

    // Announce the write root once so per-agent rows don't repeat the
    // shared `~/` (global) or `./` (project) prefix on every line.
    println!("trs init: {} agents", tools.len());
    if opts.global {
        println!("  writing to ~/  (global)\n");
    } else {
        println!("  writing to ./  (this project)\n");
    }

    for tool in &tools {
        if check_tool(tool) {
            agent_row('+', tool.name(), "already configured");
            configured += 1;
            // Ensure trs.md is present even when hooks are already wired up.
            install_trs_md_for(tool, true);
        } else if !tool.detect_installed() {
            agent_row('-', tool.name(), "not detected");
            skipped += 1;
        } else if matches!(tool, AiTool::Zed) && opts.global {
            // Zed has no verified global target — a compact skip instead of
            // the verbose per-tool message install_hook would otherwise print.
            agent_row('~', tool.name(), "per-project only (run without --global)");
            skipped += 1;
        } else if install_hook(tool, opts, true) {
            installed += 1;
        }
    }

    let verb = if opts.dry_run {
        "would install"
    } else {
        "installed"
    };
    println!(
        "\n  {} {} · {} already configured · {} skipped · {} total",
        installed,
        verb,
        configured,
        skipped,
        tools.len()
    );
    if installed > 0 && !opts.dry_run {
        println!("  ↻ restart open agent sessions to load the new hooks");
    }
    if opts.dry_run {
        println!("  note: dry-run: nothing written. Re-run without --dry-run to apply.");
    }
    // When everything is already wired up, remind the user how to force
    // a refresh — template content can change between releases even
    // when the install marker ("trs rewrite") is already present.
    if installed == 0 && configured > 0 {
        println!();
        println!("All detected agents are already configured. If a new trs release");
        println!("ships hook template improvements, re-run with --force to overwrite");
        println!("with the current template (user-added hooks are preserved).");
        println!();
        println!("  trs init --all --global --force");
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

/// Check if a tool has trs hooks installed (local or global).
///
/// Checks for either the legacy `trs (TARS CLI)` marker (installs from
/// v0.5.8 and earlier) or the current `trs (Token-Reducing Shell)`
/// marker. Keeping the legacy string here means users with an existing
/// install don't get flagged as "not configured" until they re-run
/// `trs init --force` to update the template.
pub(crate) fn check_tool(tool: &AiTool) -> bool {
    match tool {
        AiTool::Codex => {
            // Codex reads both ./AGENTS.md (project) and ~/.codex/AGENTS.md
            // (global). Either being configured counts.
            if has_any_trs_marker_at("AGENTS.md") {
                return true;
            }
            if let Ok(home) = home_dir() {
                if has_any_trs_marker_at_path(&home.join(".codex").join("AGENTS.md")) {
                    return true;
                }
            }
            return false;
        }
        AiTool::Devin => {
            return has_any_trs_marker_at(".devin/rules/trs.md")
                || has_any_trs_marker_at(".windsurfrules");
        }
        // Zed's native agent reads the project AGENTS.md only.
        AiTool::Zed => return has_any_trs_marker_at("AGENTS.md"),
        AiTool::Antigravity | AiTool::AntigravityCLI => {
            // Rules-only: trs marker lives in `~/.gemini/GEMINI.md`
            // (the @trs.md import line or the antigravity rules block).
            if let Ok(home) = home_dir() {
                return has_any_trs_marker_at_path(&home.join(".gemini").join("GEMINI.md"));
            }
            return false;
        }
        // Plugin-dir agents: the dir is 100% ours by name — the entry
        // file existing means installed.
        AiTool::OpenClaw => {
            return home_dir()
                .map(|h| h.join(".openclaw/plugins/trs/index.js").exists())
                .unwrap_or(false);
        }
        AiTool::Hermes => {
            return crate::init_install_plugins::hermes_home()
                .map(|h| h.join("plugins/trs-rewrite/__init__.py").exists())
                .unwrap_or(false);
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

// Marker / path helpers shared with init_install.rs.

/// `has_trs_marker(s) == file_has_any_trs_marker(s)`. Kept as a thin alias so
/// install_install's call sites read naturally ("does this rules file have
/// our marker?" vs. "scan generic content").
pub(crate) fn has_trs_marker(content: &str) -> bool {
    file_has_any_trs_marker(content)
}

/// Modern marker, legacy `TARS CLI` marker (v0.5.8 and earlier),
/// `trs rewrite` hook-command, and the Codex sentinel (needed because the
/// codex rules block prose uses `` `trs` `` with backticks).
pub(crate) fn file_has_any_trs_marker(content: &str) -> bool {
    content.contains("trs (Token-Reducing Shell)")
        || content.contains("trs (TARS CLI)")
        || content.contains("trs rewrite")
        || content.contains(crate::init_templates::CODEX_AGENTS_SENTINEL_START)
}

fn has_any_trs_marker_at(path_str: &str) -> bool {
    has_any_trs_marker_at_path(Path::new(path_str))
}

/// True when this looks like a Devin Desktop install (vs pre-rebrand
/// Windsurf) — picks the `.devin/rules/` target over legacy `.windsurfrules`.
/// Checks the Devin.app bundle, the per-OS app-data dir, and a project
/// `.devin/` directory.
fn devin_desktop_present() -> bool {
    if Path::new(".devin").exists() {
        return true;
    }
    let app = |name: &str| {
        Path::new(&format!("/Applications/{name}.app")).exists()
            || home_dir()
                .map(|h| h.join(format!("Applications/{name}.app")).exists())
                .unwrap_or(false)
    };
    if app("Devin") {
        return true;
    }
    home_dir()
        .map(|h| {
            h.join("Library/Application Support/Devin").exists() || h.join(".config/Devin").exists()
        })
        .unwrap_or(false)
}

fn has_any_trs_marker_at_path(path: &Path) -> bool {
    path.exists()
        && fs::read_to_string(path)
            .map(|c| file_has_any_trs_marker(&c))
            .unwrap_or(false)
}

pub(crate) fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "HOME not set".to_string())
}

fn check_file_contains_path(path: &Path, needle: &str) -> bool {
    path.exists()
        && fs::read_to_string(path)
            .map(|c| c.contains(needle))
            .unwrap_or(false)
}
