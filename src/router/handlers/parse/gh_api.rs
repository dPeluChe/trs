//! `gh api` response compression.
//!
//! GitHub's REST responses are dominated by link boilerplate. A single pull
//! request carries roughly eighty `*_url` template fields across its `head`,
//! `base` and `user` objects, and `gh api` already emits minified JSON, so
//! the generic whitespace reducer had nothing to take: measured 0% on a
//! 19 KB response before this handler existed.
//!
//! What goes: every `*_url` key except `html_url` (the one an agent or a
//! human actually follows), the API self-link `url`, the GraphQL `node_id`,
//! the always-empty `gravatar_id`, the `_links` block that only restates the
//! URLs, and a commit's `verification.payload` / `verification.signature`,
//! which are a PGP blob plus a raw copy of fields already present as
//! structured keys. `verification.verified` and `.reason` stay, since that
//! is the part anyone reads.
//!
//! What stays: everything else, byte for byte, and the output is still valid
//! JSON so `jq` and friends keep working on it. `trs diff gh api <path>`
//! shows exactly which keys went.

use super::super::common::{CommandContext, CommandResult};
use super::ParseHandler;
use serde_json::Value;

/// Keys dropped wherever they appear.
const DROP_ANYWHERE: &[&str] = &["node_id", "gravatar_id", "_links", "url"];
/// The one link worth keeping.
const KEEP: &str = "html_url";
/// Keys dropped only inside a `verification` object.
const DROP_IN_VERIFICATION: &[&str] = &["payload", "signature"];

fn drops(key: &str, parent: &str) -> bool {
    if key == KEEP {
        return false;
    }
    key.ends_with("_url")
        || DROP_ANYWHERE.contains(&key)
        || (parent == "verification" && DROP_IN_VERIFICATION.contains(&key))
}

fn prune(value: &mut Value, parent: &str) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| !drops(k, parent));
            // `iter_mut` yields the key and value as independent borrows,
            // so the key needs no clone.
            for (k, v) in map.iter_mut() {
                prune(v, k);
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                // An array does not rename its parent: `parents[]` entries are
                // still under `parents`, and `verification` nesting must carry.
                prune(v, parent);
            }
        }
        _ => {}
    }
}

/// Prune API boilerplate and re-emit compact JSON. None when the body is not
/// JSON (an error page, raw file content, a `--method DELETE` empty body), or
/// when pruning did not actually help, so the caller falls back to raw.
pub(super) fn compress_gh_api(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut value: Value = serde_json::from_str(trimmed).ok()?;
    prune(&mut value, "");
    let mut out = serde_json::to_string(&value).ok()?;
    // `emit_compressed` guards this too, as the floor for every parser. Kept
    // here as well so the compressor's own contract stays "None means I have
    // nothing to offer", which is what its tests pin.
    if out.len() >= trimmed.len() {
        return None;
    }
    out.push('\n');
    Some(out)
}

impl ParseHandler {
    pub(crate) fn handle_gh_api(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        Self::emit_compressed(&input, compress_gh_api(&input), "gh-api", ctx)
    }
}

#[cfg(test)]
#[path = "gh_api_tests.rs"]
mod tests;
