//! Droid's `permissionRules` (version 1): ordered argument-token prefixes with
//! a decision. `ask` and `block` stop the rewrite (the user's rule must see the
//! original command); `allow` can approve it. Format:
//! https://docs.factory.com/autonomy-and-safety/permission-rules

use super::{candidates, unquote, words};
use serde_json::Value;

pub(in crate::rewrite_perm_droid) struct Rule {
    allow: bool,
    /// One entry per token position; each lists the accepted alternatives.
    prefix: Vec<Vec<String>>,
}

/// `None` when the container or any rule is not the shape Droid documents, so
/// the caller fails closed instead of guessing what Droid would skip.
/// A project's `allow` rules are dropped: Droid ignores them in untrusted folders.
pub(in crate::rewrite_perm_droid) fn parse(container: &Value, project: bool) -> Option<Vec<Rule>> {
    if container.get("version")?.as_u64()? != 1 {
        return None;
    }
    let mut out = Vec::new();
    for rule in container.get("rules")?.as_array()? {
        if rule.get("id")?.as_str()?.is_empty() {
            return None;
        }
        let allow = match rule.get("decision")?.as_str()? {
            "allow" => true,
            "ask" | "block" => false,
            _ => return None,
        };
        let mut prefix = Vec::new();
        for tok in rule.pointer("/match/prefix")?.as_array()? {
            let alts = match tok {
                Value::String(s) => vec![s.clone()],
                Value::Array(a) => a
                    .iter()
                    .map(|x| x.as_str().map(String::from))
                    .collect::<Option<_>>()?,
                _ => return None,
            };
            if alts.is_empty() {
                return None;
            }
            prefix.push(alts);
        }
        if prefix.is_empty() {
            return None;
        }
        // A disabled rule decides nothing; an unparsable `enabled` fails closed.
        match rule.get("enabled") {
            None | Some(Value::Bool(true)) => {}
            Some(Value::Bool(false)) => continue,
            Some(_) => return None,
        }
        if !(allow && project) {
            out.push(Rule { allow, prefix });
        }
    }
    Some(out)
}

fn matches(rule: &Rule, tokens: &[&str]) -> bool {
    tokens.len() >= rule.prefix.len()
        && rule
            .prefix
            .iter()
            .zip(tokens)
            .all(|(alts, t)| alts.iter().any(|a| a == t))
}

fn tokens(text: &str) -> Vec<&str> {
    words(text).into_iter().map(unquote).collect()
}

/// An `ask` or `block` rule hits the command, a wrapper-stripped part of it,
/// or a nested `$(..)` / `bash -c` body. Generous on purpose: a miss lets the
/// hook's allow override the rule.
pub(in crate::rewrite_perm_droid) fn restricts(cmd: &str, rules: &[Rule]) -> bool {
    if rules.iter().all(|r| r.allow) {
        return false;
    }
    let mut cands = Vec::new();
    candidates(cmd, 0, &mut cands);
    cands.iter().any(|c| {
        let toks = tokens(c);
        rules.iter().any(|r| !r.allow && matches(r, &toks))
    })
}

/// One simple command is covered by an `allow` rule. Strict on purpose: the
/// literal words, no env or wrapper stripped, and no redirect or expansion.
pub(super) fn allows(seg: &str, rules: &[Rule]) -> bool {
    if seg.contains(['>', '<', '$', '`']) {
        return false;
    }
    let toks = tokens(seg);
    rules.iter().any(|r| r.allow && matches(r, &toks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rules(list: Value, project: bool) -> Option<Vec<Rule>> {
        parse(&json!({"version": 1, "rules": list}), project)
    }
    fn rule(decision: &str, prefix: Value) -> Value {
        json!({"id": "t", "decision": decision, "match": {"prefix": prefix}})
    }

    #[test]
    fn ask_and_block_match_by_ordered_tokens_and_alternatives() {
        let r = rules(
            json!([
                rule("ask", json!(["git", "push"])),
                rule("block", json!(["npm", ["publish", "unpublish"]]))
            ]),
            false,
        )
        .unwrap();
        for hit in [
            "git push origin",
            "FOO=1 git push",
            "sudo git push",
            "bash -c \"git push\"",
            "npm unpublish x",
            "echo $(git push)",
        ] {
            assert!(restricts(hit, &r), "{hit}");
        }
        for miss in ["git status", "git pushx", "npm pack", "git -C /x push"] {
            assert!(!restricts(miss, &r), "{miss}");
        }
    }

    #[test]
    fn allow_is_strict_and_a_projects_allow_is_dropped() {
        let list = json!([rule("allow", json!(["git", ["status", "diff"]]))]);
        let r = rules(list.clone(), false).unwrap();
        assert!(allows("git status --short", &r));
        assert!(!allows("FOO=1 git status", &r));
        assert!(!allows("git status > out", &r));
        assert!(!allows("git push", &r));
        assert!(rules(list, true).unwrap().is_empty());
    }

    #[test]
    fn malformed_rules_are_unreadable() {
        for bad in [
            json!([{"id": "", "decision": "ask", "match": {"prefix": ["x"]}}]),
            json!([rule("deny", json!(["x"]))]),
            json!([rule("ask", json!([]))]),
            json!([rule("ask", json!([[]]))]),
            json!([rule("ask", json!([1]))]),
            json!([{"id": "t", "decision": "ask", "match": {"prefix": ["x"]}, "enabled": "no"}]),
        ] {
            assert!(rules(bad.clone(), false).is_none(), "{bad}");
        }
        assert!(parse(&json!({"version": 2, "rules": []}), false).is_none());
        let off =
            json!([{"id": "t", "decision": "block", "match": {"prefix": ["x"]}, "enabled": false}]);
        assert!(rules(off, false).unwrap().is_empty());
    }
}
