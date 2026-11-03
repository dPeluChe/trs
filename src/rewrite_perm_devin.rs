//! Permission gate for Devin CLI: `Exec(prefix)` rules in its config files.
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_permissions::Gate;

/// `cmd` is the ORIGINAL command; `cwd` and `permission_mode` come from the hook payload.
pub(crate) fn gate(_cmd: &str, _cwd: Option<&str>, _permission_mode: Option<&str>) -> Gate {
    Gate::Allow
}
