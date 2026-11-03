//! Permission gate for Codex: its exec-policy `.rules` match the REWRITTEN command and the hook must answer `allow` for the rewrite to apply, so the gate only decides Allow (rewrite) or Skip (leave the command alone).
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_permissions::Gate;

/// `cmd` is the ORIGINAL command; `cwd` and `permission_mode` come from the hook payload.
pub(crate) fn gate(_cmd: &str, _cwd: Option<&str>, _permission_mode: Option<&str>) -> Gate {
    Gate::Allow
}
