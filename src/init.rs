//! `trs init` — Install hooks for AI coding tools.
//!
//! Generates configuration files that make the AI tool automatically
//! route commands through trs for token-optimized output.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init_collision;
use crate::init_install::{install_codex_agents, install_from_spec, install_rules};
use crate::init_templates::{
    ANTIGRAVITY_HOOKS, CLAUDE_HOOKS, CURSOR_HOOKS, DROID_HOOKS, GEMINI_HOOKS, KILO_PLUGIN,
    OPENCODE_PLUGIN, WINDSURF_RULES,
};

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

/// Supported AI tools for hook installation.
pub(crate) enum AiTool {
    Claude,
    Gemini,
    Cursor,
    Codex,
    OpenCode,
    Kilo,
    /// Google Antigravity desktop IDE (Antigravity 2.0). Built on Google's
    /// jetski framework — hooks live in `~/.gemini/antigravity-ide/hooks.json`
    /// as `PreToolUse` events (Claude/Codex envelope, not Gemini's BeforeTool).
    Antigravity,
    /// Google Antigravity CLI (binary `agy`, launched 2026-05-19). Same
    /// jetski framework as the IDE but writes to
    /// `~/.gemini/antigravity-cli/hooks.json` so the two variants can
    /// be configured independently.
    AntigravityCLI,
    Droid,
    Windsurf,
}

/// Hook installation spec — data-driven to avoid per-tool code duplication.
pub(crate) struct HookSpec {
    pub local_dir: &'static str,
    pub global_dir: Option<&'static str>,
    pub filename: &'static str,
    pub content: &'static str,
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
            // `antigravity` keeps mapping to the IDE for back-compat with
            // pre-v0.6.4 users; the CLI gets its own explicit aliases.
            "antigravity" | "antigravity-ide" | "gravity" => Some(Self::Antigravity),
            "antigravity-cli" | "agy" => Some(Self::AntigravityCLI),
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
            Self::Antigravity => "Antigravity IDE",
            Self::AntigravityCLI => "Antigravity CLI",
            Self::Droid => "Factory Droid",
            Self::Windsurf => "Windsurf",
        }
    }

    pub(crate) fn all_names() -> &'static str {
        "claude, gemini, cursor, codex, opencode, kilo, antigravity, agy, droid, windsurf"
    }

    pub(crate) fn all_tools() -> [Self; 10] {
        [
            Self::Claude,
            Self::Gemini,
            Self::Cursor,
            Self::Codex,
            Self::OpenCode,
            Self::Kilo,
            Self::Antigravity,
            Self::AntigravityCLI,
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
            Self::Codex => "rules → AGENTS.md (Codex hooks don't support rewrite)",
            Self::OpenCode => "plugin → .opencode/plugins/trs.ts",
            Self::Kilo => "plugin → .kilo/plugins/trs.ts",
            Self::Antigravity => "hooks → ~/.gemini/antigravity-ide/hooks.json (jetski PreToolUse)",
            Self::AntigravityCLI => {
                "hooks → ~/.gemini/antigravity-cli/hooks.json (jetski PreToolUse)"
            }
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
            // IDE (desktop) — installs the app + writes binary state into
            // `~/.gemini/antigravity-ide/`. Keep the legacy `.antigravity`
            // check as a soft signal for pre-Antigravity-2.0 users.
            Self::Antigravity => {
                app_exists("Antigravity")
                    || home_has(".gemini/antigravity-ide")
                    || home_has(".gemini/antigravity")
                    || home_has(".antigravity")
            }
            // CLI — installed as the `agy` binary, writes data into
            // `~/.gemini/antigravity-cli/`. Its hooks config is the
            // sibling `~/.gemini/antigravity-cli/hooks.json` (jetski
            // PreToolUse), written by trs init.
            Self::AntigravityCLI => in_path("agy") || home_has(".gemini/antigravity-cli"),
            Self::Droid => in_path("droid") || home_has(".factory"),
            Self::Windsurf => {
                in_path("windsurf") || app_exists("Windsurf") || home_has(".windsurfrules")
            }
        }
    }

    pub(crate) fn spec(&self) -> Option<HookSpec> {
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
            // Uses its own plugin template so the TRS_AGENT env-var
            // prefix distinguishes Kilo invocations from OpenCode in
            // history.jsonl attribution.
            Self::Kilo => Some(HookSpec {
                local_dir: ".kilo/plugins",
                global_dir: Some(".config/kilo/plugins"),
                filename: "trs.ts",
                content: KILO_PLUGIN,
            }),
            Self::Droid => Some(HookSpec {
                local_dir: ".factory",
                global_dir: Some(".factory"),
                filename: "settings.json",
                content: DROID_HOOKS,
            }),
            // Antigravity 2.0 (IDE + CLI/`agy`) is built on Google's
            // "jetski" framework — same hook system as Codex/Claude:
            // `PreToolUse` event in a `hooks.json` file (not settings.json,
            // and NOT BeforeTool — that's Gemini CLI only). Each variant
            // has its own data dir so the IDE and CLI can have separate
            // hooks without stepping on each other.
            //
            // Discovery confirmed empirically against agy v1.0.1: the
            // binary reads `~/.gemini/antigravity-{cli,ide}/hooks.json`
            // (and `~/.gemini/hooks.json`); BeforeTool entries in
            // settings.json are silently ignored.
            Self::Antigravity => Some(HookSpec {
                local_dir: ".gemini/antigravity-ide",
                global_dir: Some(".gemini/antigravity-ide"),
                filename: "hooks.json",
                content: ANTIGRAVITY_HOOKS,
            }),
            Self::AntigravityCLI => Some(HookSpec {
                local_dir: ".gemini/antigravity-cli",
                global_dir: Some(".gemini/antigravity-cli"),
                filename: "hooks.json",
                content: ANTIGRAVITY_HOOKS,
            }),
            // Codex has PreToolUse hooks but its docs explicitly state
            // `updatedInput` is "parsed but not supported yet" — the hook
            // fails open with "unsupported updatedInput" if we try to rewrite
            // commands. Until OpenAI ships input rewriting we ride on the
            // AGENTS.md rules path only (handled in install_hook via
            // install_codex_agents). Windsurf has no hook mechanism either.
            Self::Codex | Self::Windsurf => None,
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
        AiTool::Windsurf => install_rules(".windsurfrules", WINDSURF_RULES, opts),
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
            let verb = if opts.dry_run {
                "would install"
            } else {
                "installed"
            };
            println!("trs hook {} for {} at {}", verb, tool.name(), path);
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
                         (or cargo install trs-cli, or curl-sh script — see README)"
                    );
                }
                // For Imported agents (Claude, Gemini): also write trs.md so the
                // agent config gets both the hook and the output-saver/input-rewrite rules.
                install_trs_md_for(tool);
            }
        }
        Err(e) => eprintln!("Failed to install hook for {}: {}", tool.name(), e),
    }
}

/// Write `trs.md` (output-saver + input-rewrite rules) for agents that load
/// it via an `@import` line (Claude Code, Gemini CLI). No-op for other agents.
fn install_trs_md_for(tool: &AiTool) {
    let agent_id = match tool {
        AiTool::Claude => "claude",
        AiTool::Gemini => "gemini",
        _ => return,
    };
    match crate::output_saver::install_agent(agent_id) {
        Ok(msg) => println!("  trs.md: {}", msg),
        Err(e) => eprintln!("  note: trs.md install failed: {}", e),
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
            // Ensure trs.md is present even when hooks are already wired up.
            install_trs_md_for(tool);
        } else if !tool.detect_installed() {
            println!("  - {} (not detected on system, skipping)", tool.name());
            undetected += 1;
        } else {
            install_hook(tool, opts);
            installed += 1;
        }
    }

    let installed_label = if opts.dry_run {
        "would install"
    } else {
        "installed"
    };
    println!(
        "\n{} {}, {} already configured, {} skipped (not detected), {} total",
        installed,
        installed_label,
        skipped,
        undetected,
        tools.len()
    );
    if installed > 0 && !opts.dry_run {
        eprintln!("note: restart any open AI tool sessions for hooks to take effect");
    }
    if opts.dry_run {
        eprintln!("note: dry-run — nothing was written. Re-run without --dry-run to apply.");
    }
    // When everything is already wired up, remind the user how to force
    // a refresh — template content can change between releases even
    // when the install marker ("trs rewrite") is already present.
    if installed == 0 && skipped > 0 {
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
        AiTool::Windsurf => {
            return has_any_trs_marker_at(".windsurfrules");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antigravity_aliases_resolve_to_ide() {
        // Back-compat: pre-v0.6.4 users typing `trs init antigravity`
        // land on the IDE variant. Explicit aliases stay explicit.
        assert!(matches!(
            AiTool::from_str("antigravity"),
            Some(AiTool::Antigravity)
        ));
        assert!(matches!(
            AiTool::from_str("antigravity-ide"),
            Some(AiTool::Antigravity)
        ));
        assert!(matches!(
            AiTool::from_str("gravity"),
            Some(AiTool::Antigravity)
        ));
    }

    #[test]
    fn antigravity_cli_aliases() {
        assert!(matches!(
            AiTool::from_str("antigravity-cli"),
            Some(AiTool::AntigravityCLI)
        ));
        // `agy` is the binary name — most likely thing a user will type.
        assert!(matches!(
            AiTool::from_str("agy"),
            Some(AiTool::AntigravityCLI)
        ));
    }

    #[test]
    fn antigravity_variants_use_jetski_hooks_json() {
        // v0.6.5 fix: Antigravity 2.0 (IDE + CLI) is jetski-based, NOT
        // Gemini-CLI-based. Each variant writes its own `hooks.json` with
        // the PreToolUse event (Claude/Codex-style), not BeforeTool in
        // settings.json. A regression here would silently break agy:
        // jetski ignores BeforeTool entries in settings.json.
        let ide = AiTool::Antigravity.spec().expect("IDE spec exists");
        let cli = AiTool::AntigravityCLI.spec().expect("CLI spec exists");
        assert_eq!(ide.filename, "hooks.json");
        assert_eq!(cli.filename, "hooks.json");
        assert_eq!(ide.global_dir, Some(".gemini/antigravity-ide"));
        assert_eq!(cli.global_dir, Some(".gemini/antigravity-cli"));
        // Both variants share the same template content (jetski PreToolUse).
        assert_eq!(ide.content, cli.content);
        // Sanity: the template is PreToolUse-shaped, not BeforeTool-shaped.
        assert!(ide.content.contains("\"PreToolUse\""));
        assert!(!ide.content.contains("\"BeforeTool\""));
        assert!(ide.content.contains("trs rewrite"));
    }

    #[test]
    fn antigravity_does_not_share_gemini_spec() {
        // Defensive: confirm the variants DIVERGE from Gemini's spec —
        // the old code wrongly aliased them, which is what broke agy.
        let cli = AiTool::AntigravityCLI.spec().expect("CLI spec exists");
        let gemini = AiTool::Gemini.spec().expect("Gemini spec exists");
        assert_ne!(cli.filename, gemini.filename);
        assert_ne!(cli.global_dir, gemini.global_dir);
        assert_ne!(cli.content, gemini.content);
    }
}
