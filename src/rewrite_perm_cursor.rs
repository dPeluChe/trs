//! Cursor: `~/.cursor/cli-config.json` holds `permissions.deny` entries like
//! `Shell(git push)`. Unverified whether Cursor matches them against the
//! rewritten command, so a deny on the original stops the rewrite.

use crate::rewrite_gate::Gate;
use crate::rewrite_permissions::split_segments;
use crate::rewrite_rules::{read_json, string_list, word_prefix, Rules};

pub(crate) fn gate(cmd: &str) -> Gate {
    match deny_rules() {
        Some(rules) => gate_with(&rules, cmd),
        None => Gate::Skip,
    }
}

fn gate_with(deny: &[String], cmd: &str) -> Gate {
    let rules = Rules {
        deny: deny.to_vec(),
        ..Rules::default()
    };
    rules.decide(split_segments(cmd), shell_rule_matches, |_| true)
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
    prefix.is_empty() || word_prefix(seg, prefix)
}

/// `None` when the file exists but cannot be read or parsed.
fn deny_rules() -> Option<Vec<String>> {
    if cfg!(test) {
        return Some(Vec::new());
    }
    let path = crate::init::home_dir()
        .ok()?
        .join(".cursor/cli-config.json");
    Some(match read_json(&path)? {
        Some(json) => string_list(&json, "/permissions/deny"),
        None => Vec::new(),
    })
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
