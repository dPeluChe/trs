//! AI tool registry — the `AiTool` type plus the single per-tool identity
//! table (`TOOLS`): names/aliases, display name, target label, detection, and
//! hook spec. `init.rs` re-exports `AiTool` and owns installation.

use std::path::{Path, PathBuf};

use crate::init_templates::{
    CLAUDE_HOOKS, CURSOR_HOOKS, DROID_HOOKS, GEMINI_HOOKS, KILO_PLUGIN, OPENCODE_PLUGIN,
    PI_EXTENSION, VSCODE_HOOKS,
};

/// Supported AI tools for hook installation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AiTool {
    Claude,
    Gemini,
    Cursor,
    Codex,
    OpenCode,
    Kilo,
    /// Pi coding agent (pi.dev). Extension-based: a bash `spawnHook` routes
    /// commands through trs and tags them via `TRS_AGENT=pi`.
    Pi,
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
    /// Devin Desktop (formerly Windsurf; Cognition rebrand, 2026-06-02).
    /// Rules-only — Devin Local exposes no shell hook/plugin API. Forward
    /// target is `.devin/rules/trs.md`; `.windsurfrules` kept as the legacy
    /// fallback (still read by Devin and by pre-rebrand Windsurf).
    Devin,
    /// VS Code Copilot agent mode. Agent hooks (preview) speak Claude's
    /// PreToolUse envelope incl. `updatedInput` rewrite — validated live
    /// 2026-06-09. Native hook dir `~/.copilot/hooks/` + `.github/hooks/`.
    VsCode,
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
        variant: AiTool::Devin,
        cli_name: "devin",
        aliases: &["devin", "devin-desktop", "windsurf", "cascade"],
        display: "Devin Desktop",
        target_label: "rules → .devin/rules/trs.md (legacy: .windsurfrules)",
    },
    // Pi (pi.dev) — extension with a bash `spawnHook` that rewrites the
    // command + sets TRS_AGENT via env (cross-platform). Same plugin-file
    // shape as OpenCode/Kilo, different discovery dir.
    AiToolSpec {
        variant: AiTool::Pi,
        cli_name: "pi",
        aliases: &["pi", "pi.dev", "pidev"],
        display: "Pi Coding Agent",
        target_label: "extension → .pi/agent/extensions/trs.ts",
    },
    AiToolSpec {
        variant: AiTool::VsCode,
        cli_name: "vscode",
        aliases: &["vscode", "vs-code", "copilot", "vscode-copilot", "code"],
        display: "VS Code Copilot",
        target_label: "hooks → ~/.copilot/hooks/trs.json",
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
            Self::Pi => in_path("pi") || home_has(".pi"),
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
            // Devin Desktop (ex-Windsurf). New: Devin.app + per-OS app-data
            // dir + project `.devin/`. Legacy: Windsurf.app + `windsurf` bin
            // + `.windsurfrules` + the `~/.codeium`/`~/.windsurf` dirs Devin
            // still reads.
            Self::Devin => {
                app_exists("Devin")
                    || home_has("Library/Application Support/Devin")
                    || home_has(".config/Devin")
                    || Path::new(".devin").exists()
                    || in_path("devin")
                    || in_path("windsurf")
                    || app_exists("Windsurf")
                    || home_has(".windsurfrules")
                    || home_has(".codeium")
                    || home_has(".windsurf")
            }
            Self::VsCode => {
                in_path("code") || app_exists("Visual Studio Code") || home_has(".copilot")
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
            // Pi auto-discovers extensions from `~/.pi/agent/extensions/`
            // (global) and `.pi/extensions/` (project). The extension overrides
            // the bash tool with a spawnHook (see PI_EXTENSION).
            Self::Pi => Some(HookSpec {
                local_dir: ".pi/extensions",
                global_dir: Some(".pi/agent/extensions"),
                filename: "trs.ts",
                content: PI_EXTENSION,
            }),
            Self::Droid => Some(HookSpec {
                local_dir: ".factory",
                global_dir: Some(".factory"),
                filename: "settings.json",
                content: DROID_HOOKS,
            }),
            // VS Code Copilot agent hooks: any *.json under the hooks dir is
            // loaded; we own trs.json entirely.
            Self::VsCode => Some(HookSpec {
                local_dir: ".github/hooks",
                global_dir: Some(".copilot/hooks"),
                filename: "trs.json",
                content: VSCODE_HOOKS,
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
            Self::Codex | Self::Antigravity | Self::AntigravityCLI | Self::Devin => None,
        }
    }
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
            "claude, gemini, cursor, codex, opencode, kilo, antigravity, agy, droid, devin, pi, vscode"
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
