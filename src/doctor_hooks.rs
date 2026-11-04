//! Which agents have a trs hook, and whether any of them has ever fired it.
//! A hook file on disk only says it was written; the agent still has to load
//! it (VS Code settings, Copilot CLI's COPILOT_HOME, org policy), and that is
//! invisible from here except through history.

use super::doctor::Check;
use crate::init::{check_tool, check_tool_in_scope, AiTool};

pub(crate) fn check_hooks_installed() -> Check {
    let tools = AiTool::all_tools();
    let configured: Vec<String> = tools
        .iter()
        .filter(|t| check_tool(t))
        .map(|t| t.name().to_string())
        .collect();
    if configured.is_empty() {
        return Check::warn("hooks", "no AI tool hooks installed")
            .with_hint("trs init --all  (or trs init <tool>)");
    }
    let summary = format!(
        "AI tool hooks ({}/{} configured)",
        configured.len(),
        tools.len()
    );
    let names = vec![configured.join(", ")];
    // A project hook in `~` is a file no agent loads: they read project hooks
    // from a repo root. It made Copilot look configured while the global hook
    // it actually reads did not exist.
    let in_home = std::env::current_dir().ok() == crate::tracker::home_dir();
    let home_only: Vec<String> = tools
        .iter()
        .filter(|t| in_home && check_tool(t) && !check_tool_in_scope(t, true))
        .map(|t| t.name().to_string())
        .collect();
    if !home_only.is_empty() {
        return Check::warn(
            "hooks",
            format!(
                "{summary}, but {} only as a project hook in ~",
                home_only.join(", ")
            ),
        )
        .with_sub(names)
        .with_hint("agents load project hooks only from a repo root: trs init --all --global");
    }
    match crate::tracker::trs_dir().map(|d| hook_has_fired(&d)) {
        Some(false) => Check::warn("hooks", format!("{summary}, but none has run trs yet"))
            .with_sub(names)
            .with_hint(
                "ask your agent to run `git status`, then `trs stats --by-agent`; still empty \
                 means the agent is not loading the hook (usetrs.dev/support/agents.md)",
            ),
        _ => Check::pass("hooks", summary).with_sub(names),
    }
}

/// Global script plugins: (check name, tool, accepts only a default export). A copy written before 0.8.9 is not loaded by OpenCode
/// 2.x; one written before 0.8.11 prefixed every command with `trs`, pipes
/// included, so the next stage read compressed text.
const SCRIPT_PLUGINS: [(&str, AiTool, bool); 3] = [
    ("opencode plugin", AiTool::OpenCode, true),
    ("kilo plugin", AiTool::Kilo, false),
    ("pi extension", AiTool::Pi, false),
];

pub(crate) fn check_script_plugins() -> Vec<Check> {
    let Ok(home) = crate::init::home_dir() else {
        return Vec::new();
    };
    SCRIPT_PLUGINS
        .iter()
        .filter_map(|&(name, tool, default_export)| {
            let spec = tool.spec()?;
            let path = home.join(spec.global_dir?).join(spec.filename);
            let content = std::fs::read_to_string(path).ok()?;
            Some(script_plugin_check(name, &content, default_export))
        })
        .collect()
}

fn script_plugin_check(name: &'static str, content: &str, default_export: bool) -> Check {
    const REFRESH: &str = "trs init --all --global --force, then restart the agent";
    if default_export && !content.contains("export default") {
        return Check::warn(
            name,
            format!("{name}: old format, OpenCode 2.x will not load it"),
        )
        .with_hint(REFRESH);
    }
    if !content.contains("\"rewrite\"") {
        return Check::warn(
            name,
            format!("{name}: old version, it rewrites pipes and redirects too"),
        )
        .with_hint(REFRESH);
    }
    Check::pass(name, format!("{name} is current"))
}

/// OpenClaw and Hermes load a plugin by name from their own config, so the
/// file alone is not enough: stale content and a missing config entry both
/// leave trs out of the loop (or, for a copy that predates `trs rewrite`,
/// prefix every command blindly).
pub(crate) fn check_host_plugins() -> Vec<Check> {
    use crate::init_install_plugins::{
        hermes_home, hermes_plugin_dir, openclaw_home, openclaw_plugin_dir,
    };
    let mut out = Vec::new();
    if let Ok(dir) = openclaw_home() {
        if let Ok(content) = std::fs::read_to_string(openclaw_plugin_dir(&dir).join("index.js")) {
            let enabled = openclaw_enabled(&dir);
            out.push(host_plugin_check("openclaw plugin", &content, enabled));
        }
    }
    if let Ok(dir) = hermes_home() {
        if let Ok(content) = std::fs::read_to_string(hermes_plugin_dir(&dir).join("__init__.py")) {
            let enabled = hermes_enabled(&dir);
            out.push(host_plugin_check("hermes plugin", &content, enabled));
        }
    }
    out
}

/// `enabled` is `None` when the config could not be judged (left alone then).
fn host_plugin_check(name: &'static str, content: &str, enabled: Option<bool>) -> Check {
    let check = script_plugin_check(name, content, false);
    if check.status != crate::doctor::CheckStatus::Pass || enabled != Some(false) {
        return check;
    }
    Check::warn(
        name,
        format!("{name}: installed but not enabled in the agent's config"),
    )
    .with_hint("trs init --all --global, then restart the agent")
}

fn openclaw_enabled(dir: &std::path::Path) -> Option<bool> {
    let text = std::fs::read_to_string(dir.join("openclaw.json")).unwrap_or_else(|_| "{}".into());
    let mut root: serde_json::Value = serde_json::from_str(&text).ok()?;
    let plugin_dir = crate::init_install_plugins::openclaw_plugin_dir(dir)
        .display()
        .to_string();
    let changed =
        crate::init_install_plugins::merge_openclaw_config(&mut root, &plugin_dir).ok()?;
    Some(!changed)
}

fn hermes_enabled(dir: &std::path::Path) -> Option<bool> {
    use crate::init_install_plugins::{patch_hermes_config, HermesConfigPatch};
    let text = std::fs::read_to_string(dir.join("config.yaml")).unwrap_or_default();
    match patch_hermes_config(&text) {
        HermesConfigPatch::AlreadyPresent => Some(true),
        HermesConfigPatch::Patched(_) => Some(false),
        HermesConfigPatch::Manual => None,
    }
}

/// Runs that came through a hook carry an `agent` label; direct runs don't.
/// Rotated monthly files count too, so a quiet month is not a false alarm.
pub(crate) fn hook_has_fired(trs_dir: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(trs_dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let name = e.file_name();
        let name = name.to_string_lossy();
        name.starts_with("history")
            && name.ends_with(".jsonl")
            && std::fs::read_to_string(e.path()).is_ok_and(|s| s.contains("\"agent\":\""))
    })
}

#[cfg(test)]
mod tests {
    use super::{hook_has_fired, script_plugin_check};
    use crate::doctor::CheckStatus;

    #[test]
    fn an_outdated_script_plugin_is_flagged() {
        use crate::init_templates::{KILO_PLUGIN, OPENCODE_PLUGIN, PI_EXTENSION};
        let named_export = "export const TrsPlugin = async () => ({});";
        assert_eq!(
            script_plugin_check("opencode plugin", named_export, true).status,
            CheckStatus::Warn
        );
        let blind_prefix = "export default { server: () => `trs ${cmd}` };";
        assert_eq!(
            script_plugin_check("opencode plugin", blind_prefix, true).status,
            CheckStatus::Warn
        );
        for (name, current, default_export) in [
            ("opencode plugin", OPENCODE_PLUGIN, true),
            ("kilo plugin", KILO_PLUGIN, false),
            ("pi extension", PI_EXTENSION, false),
        ] {
            assert_eq!(
                script_plugin_check(name, current, default_export).status,
                CheckStatus::Pass,
                "{name}"
            );
        }
    }

    #[test]
    fn host_plugins_are_checked_for_staleness_and_for_being_enabled() {
        use super::{hermes_enabled, host_plugin_check, openclaw_enabled};
        use crate::init_templates::{HERMES_PLUGIN_INIT, OPENCLAW_PLUGIN_INDEX};
        let ok = host_plugin_check("openclaw plugin", OPENCLAW_PLUGIN_INDEX, Some(true));
        assert_eq!(ok.status, CheckStatus::Pass);
        let off = host_plugin_check("openclaw plugin", OPENCLAW_PLUGIN_INDEX, Some(false));
        assert_eq!(off.status, CheckStatus::Warn);
        let unknown = host_plugin_check("hermes plugin", HERMES_PLUGIN_INIT, None);
        assert_eq!(unknown.status, CheckStatus::Pass);
        let blind = "register(api) { api.on('x', (e) => `trs ${e}`) }";
        let stale = host_plugin_check("openclaw plugin", blind, Some(true));
        assert_eq!(stale.status, CheckStatus::Warn);

        let dir = tempfile::tempdir().unwrap();
        assert_eq!(openclaw_enabled(dir.path()), Some(false));
        assert_eq!(hermes_enabled(dir.path()), Some(false));
        let plugin = dir.path().join("plugins/trs").display().to_string();
        let cfg = format!(
            r#"{{"plugins":{{"entries":{{"trs":{{"enabled":true}}}},"load":{{"paths":["{plugin}"]}}}}}}"#
        );
        std::fs::write(dir.path().join("openclaw.json"), cfg).unwrap();
        assert_eq!(openclaw_enabled(dir.path()), Some(true));
        std::fs::write(
            dir.path().join("config.yaml"),
            "plugins:\n  enabled:\n    - trs-rewrite\n",
        )
        .unwrap();
        assert_eq!(hermes_enabled(dir.path()), Some(true));
        std::fs::write(dir.path().join("openclaw.json"), "{ nope").unwrap();
        assert_eq!(openclaw_enabled(dir.path()), None);
    }

    #[test]
    fn only_a_labelled_run_counts_as_a_hook_firing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!hook_has_fired(dir.path()), "no history at all");
        std::fs::write(
            dir.path().join("history.jsonl"),
            "{\"cmd\":\"git status\"}\n",
        )
        .unwrap();
        assert!(
            !hook_has_fired(dir.path()),
            "a direct run is not a hook firing"
        );
        std::fs::write(
            dir.path().join("history.2026-08.jsonl"),
            "{\"cmd\":\"git status\",\"agent\":\"copilot-cli\"}\n",
        )
        .unwrap();
        assert!(
            hook_has_fired(dir.path()),
            "a labelled run in a rotated file counts"
        );
    }
}
