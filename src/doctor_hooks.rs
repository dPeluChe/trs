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
    match crate::tracker::home_dir().map(|h| hook_has_fired(&h.join(".trs"))) {
        Some(false) => Check::warn("hooks", format!("{summary}, but none has run trs yet"))
            .with_sub(names)
            .with_hint(
                "ask your agent to run `git status`, then `trs stats --by-agent`; still empty \
                 means the agent is not loading the hook (usetrs.dev/support/agents.md)",
            ),
        _ => Check::pass("hooks", summary).with_sub(names),
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
    use super::hook_has_fired;

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
