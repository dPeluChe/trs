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

/// Global script plugins: (check name, path under $HOME, accepts only a
/// default export). A copy written before 0.8.9 is not loaded by OpenCode
/// 2.x; one written before 0.8.11 prefixed every command with `trs`, pipes
/// included, so the next stage read compressed text.
const SCRIPT_PLUGINS: [(&str, &str, bool); 3] = [
    ("opencode plugin", ".config/opencode/plugins/trs.ts", true),
    ("kilo plugin", ".config/kilo/plugins/trs.ts", false),
    ("pi extension", ".pi/agent/extensions/trs.ts", false),
];

pub(crate) fn check_script_plugins() -> Vec<Check> {
    let Ok(home) = crate::init::home_dir() else {
        return Vec::new();
    };
    SCRIPT_PLUGINS
        .iter()
        .filter_map(|&(name, rel, default_export)| {
            let content = std::fs::read_to_string(home.join(rel)).ok()?;
            Some(script_plugin_check(name, &content, default_export))
        })
        .collect()
}

fn script_plugin_check(name: &'static str, content: &str, default_export: bool) -> Check {
    const REFRESH: &str = "trs init --all --global --force, then restart the agent";
    if default_export && !content.contains("export default") {
        return Check::warn(name, "old format, OpenCode 2.x will not load it").with_hint(REFRESH);
    }
    if !content.contains("\"rewrite\"") {
        return Check::warn(name, "old version, it rewrites pipes and redirects too")
            .with_hint(REFRESH);
    }
    Check::pass(name, "current")
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
