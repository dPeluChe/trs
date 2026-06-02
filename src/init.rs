//! `trs init` — Install hooks for AI coding tools.
//!
//! Generates configuration files that make the AI tool automatically
//! route commands through trs for token-optimized output.

use std::fs;
use std::path::{Path, PathBuf};

use crate::init_collision;
use crate::init_install::{
    install_antigravity_rules, install_codex_agents, install_from_spec, install_rules,
};
use crate::init_templates::{
    CLAUDE_HOOKS, CURSOR_HOOKS, DROID_HOOKS, GEMINI_HOOKS, KILO_PLUGIN, OPENCODE_PLUGIN,
    WINDSURF_RULES,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AiTool {
    Claude,
    Gemini,
    Cursor,
    Codex,
    OpenCode,
    Kilo,
    /// Google Antigravity desktop IDE (Antigravity 2.0). Currently
    /// rules-only — v0.6.6 reverted the jetski hook integration because
    /// agy v1.0.1 doesn't expose user-configurable PreTool hooks. The
    /// integration is the `@trs.md` import in `~/.gemini/GEMINI.md` plus
    /// a recommendation to manually prefix `trs <cmd>`. Re-enable as a
    /// programmatic hook once Google ships user-configurable
    /// PreToolHooks upstream — see
    /// `docs/development/antigravity-hooks-research.md`.
    Antigravity,
    /// Google Antigravity CLI (binary `agy`). Shares the rules-only
    /// integration with the IDE — same `~/.gemini/GEMINI.md` target.
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

/// Single source of truth for an AI tool's *identity*: the names it
/// answers to, its human display name, and the one-line target label.
///
/// What stays OUT of this table on purpose: the per-consumer *path* logic.
/// `spec()` (hook install), `init_collision::target_paths` (competitor
/// scan over home+project), `uninstall::candidate_paths` (where trs wrote),
/// and `output_saver::resolve_target` (the single canonical rules file)
/// each resolve different path sets for different jobs and in different
/// orders — folding them into one row would silently change scan/`--show`
/// ordering. They stay separate; the `output_saver_agents_match_registry`
/// test pins their display strings to this table so they can't drift.
pub(crate) struct AiToolSpec {
    pub variant: AiTool,
    /// Primary CLI token shown in `all_names()` (e.g. "agy" for the
    /// Antigravity CLI, "antigravity" for the IDE).
    pub cli_name: &'static str,
    /// Every `from_str`-accepted alias (lowercase), including `cli_name`.
    pub aliases: &'static [&'static str],
    /// Human-facing display name.
    pub display: &'static str,
    /// Where the integration lives — shown in `trs init --show`.
    pub target_label: &'static str,
}

/// The tool registry. Order matters: it defines `all_tools()` iteration
/// order (install/uninstall/doctor sweeps) — keep it stable.
pub(crate) const TOOLS: &[AiToolSpec] = &[
    AiToolSpec {
        variant: AiTool::Claude,
        cli_name: "claude",
        aliases: &["claude"],
        display: "Claude Code",
        target_label: "hooks → ~/.claude/settings.json",
    },
    AiToolSpec {
        variant: AiTool::Gemini,
        cli_name: "gemini",
        aliases: &["gemini"],
        display: "Gemini CLI",
        target_label: "hooks → ~/.gemini/settings.json",
    },
    AiToolSpec {
        variant: AiTool::Cursor,
        cli_name: "cursor",
        aliases: &["cursor"],
        display: "Cursor",
        target_label: "hooks → ~/.cursor/hooks.json",
    },
    AiToolSpec {
        variant: AiTool::Codex,
        cli_name: "codex",
        aliases: &["codex"],
        display: "Codex",
        target_label: "rules → AGENTS.md (Codex hooks don't support rewrite)",
    },
    AiToolSpec {
        variant: AiTool::OpenCode,
        cli_name: "opencode",
        aliases: &["opencode"],
        display: "OpenCode",
        target_label: "plugin → .opencode/plugins/trs.ts",
    },
    AiToolSpec {
        variant: AiTool::Kilo,
        cli_name: "kilo",
        aliases: &["kilo", "kilocode"],
        display: "Kilo Code",
        target_label: "plugin → .kilo/plugins/trs.ts",
    },
    // `antigravity` keeps mapping to the IDE for back-compat with
    // pre-v0.6.4 users; the CLI gets its own explicit aliases.
    AiToolSpec {
        variant: AiTool::Antigravity,
        cli_name: "antigravity",
        aliases: &["antigravity", "antigravity-ide", "gravity"],
        display: "Antigravity IDE",
        target_label: "rules → ~/.gemini/GEMINI.md (jetski hooks not yet user-config)",
    },
    AiToolSpec {
        variant: AiTool::AntigravityCLI,
        cli_name: "agy",
        aliases: &["antigravity-cli", "agy"],
        display: "Antigravity CLI",
        target_label: "rules → ~/.gemini/GEMINI.md (jetski hooks not yet user-config)",
    },
    AiToolSpec {
        variant: AiTool::Droid,
        cli_name: "droid",
        aliases: &["droid", "factory"],
        display: "Factory Droid",
        target_label: "hooks → ~/.factory/settings.json",
    },
    AiToolSpec {
        variant: AiTool::Windsurf,
        cli_name: "windsurf",
        aliases: &["windsurf", "cascade"],
        display: "Windsurf",
        target_label: "rules → .windsurfrules",
    },
];

impl AiTool {
    fn identity(&self) -> &'static AiToolSpec {
        // Every variant has exactly one row; construction is compile-time.
        TOOLS
            .iter()
            .find(|t| t.variant == *self)
            .expect("AiTool missing from TOOLS registry")
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        TOOLS
            .iter()
            .find(|t| t.aliases.contains(&lower.as_str()))
            .map(|t| t.variant)
    }

    pub(crate) fn name(&self) -> &str {
        self.identity().display
    }

    pub(crate) fn all_names() -> String {
        TOOLS
            .iter()
            .map(|t| t.cli_name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn all_tools() -> Vec<Self> {
        TOOLS.iter().map(|t| t.variant).collect()
    }

    /// Short label describing where the integration lives, e.g. "~/.claude/settings.json"
    /// or "AGENTS.md". Used in `trs init --show` for transparency.
    pub(crate) fn target_label(&self) -> &'static str {
        self.identity().target_label
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
            // Antigravity 2.0 (IDE + CLI/`agy`) — v0.6.6 reverted the
            // jetski PreToolUse integration shipped in v0.6.5. Empirically
            // verified against agy v1.0.1 binary + cli.log: `hooks.json`
            // loads only as **subagent** specs (the `name`+`description`
            // fields are required with jsonschema_description "Unique
            // name for the subagent. Used to invoke it via invoke_subagent").
            // PreTool hooks for `Step_RunCommand` (Bash) are internal-only
            // — only MCP browser hooks are exposed user-side. See
            // `docs/development/antigravity-hooks-research.md` for the
            // full investigation. Until Google ships user-configurable
            // PreToolUse, Antigravity is rules-only — the output-saver
            // import in `~/.gemini/GEMINI.md` is the entire integration.
            Self::Codex | Self::Antigravity | Self::AntigravityCLI | Self::Windsurf => None,
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
        AiTool::Antigravity | AiTool::AntigravityCLI => install_antigravity_rules(opts),
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
        AiTool::Antigravity | AiTool::AntigravityCLI => {
            // Rules-only: trs marker lives in `~/.gemini/GEMINI.md`
            // (the @trs.md import line or the antigravity rules block).
            if let Ok(home) = home_dir() {
                return has_any_trs_marker_at_path(&home.join(".gemini").join("GEMINI.md"));
            }
            return false;
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
    fn antigravity_variants_are_rules_only() {
        // v0.6.6 revert: agy v1.0.1 doesn't expose user-configurable
        // PreTool hooks. Both Antigravity variants moved to rules-only
        // (like Codex/Windsurf). Regression on this test would mean
        // we accidentally re-installed a non-functional hook.
        assert!(
            AiTool::Antigravity.spec().is_none(),
            "Antigravity IDE must have no HookSpec — rules-only"
        );
        assert!(
            AiTool::AntigravityCLI.spec().is_none(),
            "Antigravity CLI must have no HookSpec — rules-only"
        );
    }

    #[test]
    fn antigravity_target_label_signals_rules_only() {
        // The label drives `trs init --show`; it must read as
        // rules-only so users know auto-rewriting is off, with a hint
        // at the upstream limitation.
        let ide_label = AiTool::Antigravity.target_label();
        let cli_label = AiTool::AntigravityCLI.target_label();
        assert!(ide_label.starts_with("rules → "));
        assert!(cli_label.starts_with("rules → "));
        assert!(ide_label.contains("GEMINI.md"));
        assert!(cli_label.contains("GEMINI.md"));
    }

    #[test]
    fn registry_covers_every_variant_and_has_no_dup_aliases() {
        // identity() panics if a variant is missing from TOOLS.
        for spec in TOOLS {
            assert_eq!(spec.variant.identity().display, spec.display);
        }
        // No alias is claimed by two tools (would make from_str ambiguous).
        let mut seen = std::collections::HashSet::new();
        for spec in TOOLS {
            for a in spec.aliases {
                assert!(seen.insert(*a), "duplicate alias across tools: {a}");
            }
            // cli_name must itself be a valid alias.
            assert!(
                spec.aliases.contains(&spec.cli_name),
                "{} cli_name not in aliases",
                spec.cli_name
            );
        }
        assert_eq!(AiTool::all_tools().len(), TOOLS.len());
    }

    #[test]
    fn all_names_is_the_cli_name_list() {
        // Pins the exact public string `trs uninstall` prints on bad input.
        assert_eq!(
            AiTool::all_names(),
            "claude, gemini, cursor, codex, opencode, kilo, antigravity, agy, droid, windsurf"
        );
    }

    #[test]
    fn output_saver_agents_match_registry() {
        // The output_saver AGENTS list is a separate const (it owns its own
        // display order for `--show`/verify). This guards against its
        // display strings drifting from the identity registry: every agent
        // id must resolve to a known tool whose display name matches.
        for agent in crate::output_saver::AGENTS {
            let tool = AiTool::from_str(agent.id).unwrap_or_else(|| {
                panic!("output_saver agent id `{}` unknown to registry", agent.id)
            });
            assert_eq!(
                tool.name(),
                agent.display,
                "display drift for `{}`: registry={}, output_saver={}",
                agent.id,
                tool.name(),
                agent.display
            );
        }
    }
}
