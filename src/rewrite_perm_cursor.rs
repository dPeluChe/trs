//! Cursor: `~/.cursor/cli-config.json` holds `permissions.deny` entries like
//! `Shell(git push)`. Unverified whether Cursor matches them against the
//! rewritten command, so a deny on the original stops the rewrite.

use crate::rewrite_permissions::{split_segments, strip_env, Gate};
use std::path::PathBuf;

pub(crate) fn gate(cmd: &str, _cwd: Option<&str>, _permission_mode: Option<&str>) -> Gate {
    match deny_rules() {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(deny: &[String], cmd: &str) -> Gate {
    let hit = split_segments(cmd).into_iter().any(|seg| {
        let bare = strip_env(seg);
        deny.iter()
            .any(|r| shell_rule_matches(r, seg) || shell_rule_matches(r, bare))
    });
    if hit {
        Gate::Skip
    } else {
        Gate::Allow
    }
}

/// `Shell`, `Shell(*)`, `Shell(git push)`: whole-word prefix of the segment.
fn shell_rule_matches(rule: &str, seg: &str) -> bool {
    let inner = if rule == "Shell" {
        ""
    } else if let Some(i) = rule
        .strip_prefix("Shell(")
        .and_then(|r| r.strip_suffix(')'))
    {
        i
    } else {
        return false;
    };
    let prefix = inner
        .strip_suffix(":*")
        .unwrap_or(inner)
        .trim_end_matches('*')
        .trim_end();
    prefix.is_empty()
        || seg == prefix
        || seg.strip_prefix(prefix).is_some_and(|r| r.starts_with(' '))
}

/// `None` when the file exists but cannot be read or parsed.
fn deny_rules() -> Option<Vec<String>> {
    if cfg!(test) {
        return Some(Vec::new());
    }
    let path: PathBuf = crate::init::home_dir()
        .ok()?
        .join(".cursor/cli-config.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return if path.exists() {
            None
        } else {
            Some(Vec::new())
        };
    };
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let list = json.pointer("/permissions/deny").and_then(|v| v.as_array());
    Some(
        list.into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deny_rule_on_the_original_stops_the_rewrite() {
        let deny = vec!["Shell(git push)".to_string(), "Shell(rm:*)".to_string()];
        assert_eq!(gate_with(&deny, "git push origin main"), Gate::Skip);
        assert_eq!(gate_with(&deny, "cd x && rm -rf y"), Gate::Skip);
        assert_eq!(gate_with(&deny, "FOO=1 git push"), Gate::Skip);
        assert_eq!(gate_with(&deny, "git status"), Gate::Allow);
        assert_eq!(gate_with(&["Shell".to_string()], "ls"), Gate::Skip);
    }
}
