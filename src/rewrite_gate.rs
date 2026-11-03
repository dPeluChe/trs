//! Which permission gate applies to each agent. Every agent matches its own
//! rules against the REWRITTEN command, so the hook has to apply them to the
//! original one. Verified per agent: docs/development/agent-integrations.md.

use crate::rewrite_permissions::Gate;
use serde_json::Value;

pub(crate) fn gate_for(label: &str, cmd: &str, json: &Value) -> Gate {
    let cwd = json.get("cwd").and_then(|v| v.as_str());
    let mode = json.get("permission_mode").and_then(|v| v.as_str());
    match label {
        "claude" => crate::rewrite_permissions::gate(cmd, cwd),
        "codex" => crate::rewrite_perm_codex::gate(cmd, cwd, mode),
        "droid" => crate::rewrite_perm_droid::gate(cmd, cwd, mode),
        "devin-cli" => crate::rewrite_perm_devin::gate(cmd, cwd, mode),
        "cursor" => crate::rewrite_perm_cursor::gate(cmd, cwd, mode),
        "gemini" => crate::rewrite_perm_gemini::gate(cmd, cwd, mode),
        // Rules this hook cannot read (flags, IDE settings) or an agent not
        // verified yet: rewrite, but never send an approval of our own.
        _ => Gate::Defer,
    }
}
