//! Permission gate for Gemini CLI: TOML policies and `tools.allowed`; it ignores a hook allow, so the gate only decides Defer (rewrite, no decision) or Skip.
//! Contract: docs/development/agent-integrations.md (permissions).

use crate::rewrite_permissions::Gate;

/// `cmd` is the ORIGINAL command; `cwd` and `permission_mode` come from the hook payload.
pub(crate) fn gate(_cmd: &str, _cwd: Option<&str>, _permission_mode: Option<&str>) -> Gate {
    Gate::Defer
}
