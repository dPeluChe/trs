//! Pieces shared by the per-agent permission gates (`rewrite_perm_*`).

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::strip_wrappers;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(crate) struct Rules {
    pub(crate) allow: Vec<String>,
    pub(crate) ask: Vec<String>,
    pub(crate) deny: Vec<String>,
}

impl Rules {
    /// Deny/ask on any part (with or without a `VAR=` prefix) stops the
    /// rewrite; every part allowed approves it; anything else defers.
    pub(crate) fn decide<'a>(
        &self,
        segs: impl IntoIterator<Item = &'a str>,
        matches: fn(&str, &str) -> bool,
        auto_ok: fn(&str) -> bool,
    ) -> Gate {
        let (mut any, mut all_allowed) = (false, true);
        for seg in segs {
            any = true;
            let bare = strip_wrappers(seg);
            if self
                .deny
                .iter()
                .chain(&self.ask)
                .any(|r| matches(r, seg) || matches(r, bare))
            {
                return Gate::Skip;
            }
            if !(self.allow.iter().any(|r| matches(r, seg)) || auto_ok(seg)) {
                all_allowed = false;
            }
        }
        if any && all_allowed {
            Gate::Allow
        } else {
            Gate::Defer
        }
    }

    /// Adds `permissions.{allow,ask,deny}` of one settings file; `allow` only
    /// when `allow_counts` (untrusted project files may not grant).
    pub(crate) fn extend_from(&mut self, json: &Value, allow_counts: bool) {
        if allow_counts {
            self.allow.extend(string_list(json, "/permissions/allow"));
        }
        self.ask.extend(string_list(json, "/permissions/ask"));
        self.deny.extend(string_list(json, "/permissions/deny"));
    }
}

/// `Some(None)` when the file is missing, `None` when it exists but cannot be
/// read or parsed: gates fail closed on that.
pub(crate) fn read_json(path: &Path) -> Option<Option<Value>> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).ok().map(Some),
        Err(e) if is_missing(&e) => Some(None),
        Err(_) => None,
    }
}

pub(crate) fn string_list(json: &Value, pointer: &str) -> Vec<String> {
    json.pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(String::from)
        .collect()
}

/// Missing file, or a parent that is a file: nothing to read either way.
pub(crate) fn is_missing(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    )
}

/// `prefix` is the whole command or its leading words.
pub(crate) fn word_prefix(cmd: &str, prefix: &str) -> bool {
    cmd == prefix
        || cmd
            .strip_prefix(prefix)
            .is_some_and(|r| r.starts_with(char::is_whitespace))
}

/// `dir` or one of its parents is a trusted folder.
pub(crate) fn is_trusted(dir: &Path, trusted: &[PathBuf]) -> bool {
    dir.ancestors().any(|a| trusted.iter().any(|t| t == a))
}

/// Whether a gate reads this machine's own agent config. Unit tests in this
/// crate never do (they would depend on the developer's settings); the
/// integration tests run the real binary under a temp HOME instead.
pub(crate) const fn reads_host_config() -> bool {
    !cfg!(test)
}

/// Every dir from cwd up to the git root (or cwd), minus `skip` (the user's home).
pub(crate) fn project_dirs(cwd: &Path, skip: Option<&Path>) -> Vec<PathBuf> {
    let root = cwd
        .ancestors()
        .find(|d| d.join(".git").exists())
        .unwrap_or(cwd);
    let mut dirs = Vec::new();
    for d in cwd.ancestors() {
        if Some(d) != skip {
            dirs.push(d.to_path_buf());
        }
        if d == root {
            break;
        }
    }
    dirs
}
