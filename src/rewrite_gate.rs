//! One row per agent: its label and the permission gate that applies to it.
//! Every agent matches its own rules against the REWRITTEN command, so the
//! hook has to apply them to the original one. Verified per agent:
//! docs/development/agent-integrations.md.

use serde_json::Value;

/// What the hook does with a command, given the user's rules.
#[derive(Debug, PartialEq)]
pub(crate) enum Gate {
    /// A deny/ask rule matches or the rules cannot be read: leave the command alone.
    Skip,
    /// Every part is allowed: rewrite and approve.
    Allow,
    /// Rewrite, but let the agent's own permission flow decide.
    Defer,
}

type GateFn = fn(cmd: &str, cwd: Option<&str>, mode: Option<&str>) -> Gate;

pub(crate) struct AgentPolicy {
    pub label: &'static str,
    /// The hook command sets `TRS_AGENT=<label>` (shares Claude's envelope).
    pub from_env: bool,
    gate: GateFn,
}

/// Rules this hook cannot read (flags, IDE settings) or an agent not verified
/// yet: rewrite, but never send an approval of our own.
fn defer(_: &str, _: Option<&str>, _: Option<&str>) -> Gate {
    Gate::Defer
}

const fn row(label: &'static str, from_env: bool, gate: GateFn) -> AgentPolicy {
    AgentPolicy {
        label,
        from_env,
        gate,
    }
}

/// A new `--caller` agent starts as one row here (plus its template in
/// init_templates.rs); `defer` until its rules are verified.
static AGENTS: &[AgentPolicy] = &[
    row("claude", false, |c, d, _| {
        crate::rewrite_permissions::gate(c, d)
    }),
    row("gemini", false, |c, d, _| {
        crate::rewrite_perm_gemini::gate(c, d)
    }),
    row("cursor", false, |c, d, _| {
        crate::rewrite_perm_cursor::gate(c, d)
    }),
    row("codex", true, |c, d, _| {
        crate::rewrite_perm_codex::gate(c, d)
    }),
    row("vscode", true, defer),
    row("copilot-cli", true, defer),
    row("droid", false, crate::rewrite_perm_droid::gate),
    row("antigravity", false, defer),
    row("opencode", false, defer),
    row("kilo", false, defer),
    row("pi", false, defer),
    row("openclaw", false, defer),
    row("hermes", false, defer),
    row("devin-cli", false, |c, d, _| {
        crate::rewrite_perm_devin::gate(c, d)
    }),
];

pub(crate) fn policy(label: &str) -> Option<&'static AgentPolicy> {
    AGENTS.iter().find(|p| p.label == label)
}

pub(crate) fn gate_for(label: &str, cmd: &str, json: &Value) -> Gate {
    let cwd = json
        .get("cwd")
        .or_else(|| json.pointer("/workspace_roots/0"))
        .and_then(|v| v.as_str());
    let mode = json.get("permission_mode").and_then(|v| v.as_str());
    policy(label).map_or(Gate::Defer, |p| (p.gate)(cmd, cwd, mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_unique() {
        for (i, p) in AGENTS.iter().enumerate() {
            assert!(AGENTS[i + 1..].iter().all(|q| q.label != p.label));
        }
    }
}
