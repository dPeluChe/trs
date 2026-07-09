//! AI tool registry — the `AiTool` type plus the single per-tool identity
//! table (`TOOLS`): names/aliases, display name, target label, detection, and
//! hook spec. `init.rs` re-exports `AiTool` and owns installation.

use std::path::{Path, PathBuf};

use crate::init_templates::{
    CLAUDE_HOOKS, CURSOR_HOOKS, DEVIN_CLI_HOOKS, DROID_HOOKS, GEMINI_HOOKS, KILO_PLUGIN,
    OPENCODE_PLUGIN, PI_EXTENSION, VSCODE_HOOKS,
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
    /// OpenClaw gateway. JS plugin: `before_tool_call` rewrites exec params,
    /// `resolve_exec_env` injects TRS_AGENT. Validated 2026-06-11 docs
    /// research; live validation pending.
    OpenClaw,
    /// NousResearch hermes-agent. Python plugin (`pre_tool_call` hook) under
    /// `~/.hermes/plugins/` + a `config.yaml` enable entry. Validated
    /// 2026-06-11 docs research; live validation pending.
    Hermes,
    /// Zed Agent Panel. Rules-only — the native agent has no tool hooks
    /// (zed-industries/zed#52688); it reads the project `AGENTS.md`, so the
    /// existing sentinel block is the integration. External agents run via
    /// ACP (Claude Code, Codex, Gemini CLI, OpenCode) are the real CLIs —
    /// their own trs hooks apply transitively.
    Zed,
    /// Devin CLI ("Devin for Terminal", binary `devin`) — a real hook
    /// integration, distinct from the rules-only `Devin` (Desktop /
    /// ex-Windsurf). Speaks Claude's PreToolUse envelope; shell tool is
    /// `exec`, config target is `config.json` under `hooks` (global
    /// `~/.config/devin/`, project `.devin/`). Validated live 2026-07-07:
    /// Devin honors `hookSpecificOutput.updatedInput` (commands run as
    /// `trs …`). Attribution needs `devin-cli` in `known_agent_label`
    /// (rewrite.rs) or `--caller devin-cli` silently falls back to `claude`.
    /// Note: Devin reads `.claude` hooks by default (`read_config_from.claude`)
    /// — set it false so this hook wins instead of the transitive Claude one.
    DevinCLI,
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
    AiToolSpec {
        variant: AiTool::OpenClaw,
        cli_name: "openclaw",
        aliases: &["openclaw", "claw"],
        display: "OpenClaw",
        target_label: "plugin → ~/.openclaw/plugins/trs/ (+ config enable)",
    },
    AiToolSpec {
        variant: AiTool::Hermes,
        cli_name: "hermes",
        aliases: &["hermes", "hermes-agent"],
        display: "Hermes",
        target_label: "plugin → ~/.hermes/plugins/trs-rewrite/ (+ config enable)",
    },
    AiToolSpec {
        variant: AiTool::Zed,
        cli_name: "zed",
        aliases: &["zed", "zed-ide"],
        display: "Zed (Agent Panel)",
        target_label: "rules → AGENTS.md (native agent; ACP external agents use their own hooks)",
    },
    // Devin CLI ("Devin for Terminal") — real PreToolUse hook, `exec` tool.
    // Desktop keeps `devin`; the CLI takes explicit `devin-cli` aliases,
    // mirroring the Antigravity IDE/CLI split.
    AiToolSpec {
        variant: AiTool::DevinCLI,
        cli_name: "devin-cli",
        aliases: &["devin-cli", "devin-terminal", "dcli"],
        display: "Devin CLI",
        target_label: "hooks → ~/.config/devin/config.json (project: .devin/config.json)",
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
            Self::OpenClaw => in_path("openclaw") || home_has(".openclaw"),
            Self::Hermes => in_path("hermes") || home_has(".hermes"),
            Self::Zed => in_path("zed") || app_exists("Zed") || home_has(".config/zed"),
            // Devin CLI writes `~/.config/devin/` (config.json + cli/). The
            // `devin` binary is shared with Devin Desktop, so both variants
            // may report installed — the user disambiguates via `devin-cli`.
            Self::DevinCLI => in_path("devin") || home_has(".config/devin"),
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
            // Devin CLI merges into `config.json` (project `.devin/`, global
            // `~/.config/devin/`) under the `hooks` key — the merge path
            // preserves the user's other config (model, org_id, theme).
            Self::DevinCLI => Some(HookSpec {
                local_dir: ".devin",
                global_dir: Some(".config/devin"),
                filename: "config.json",
                content: DEVIN_CLI_HOOKS,
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
            // OpenClaw/Hermes use custom installers (plugin dir + config
            // enable) — too stateful for the data-driven HookSpec.
            Self::Codex
            | Self::Antigravity
            | Self::AntigravityCLI
            | Self::Devin
            | Self::OpenClaw
            | Self::Hermes
            | Self::Zed => None,
        }
    }
}

#[cfg(test)]
#[path = "ai_tool_tests.rs"]
mod tests;
