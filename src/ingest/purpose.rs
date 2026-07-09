//! Shared "purpose layer": project intent + module-role topology derived from
//! the import graph. Used by BOTH the markdown digest (`format.rs`, agent-
//! facing) and the HTML report (`format_html.rs`, human-facing) so the logic
//! lives once.

use std::collections::HashMap;
use std::path::Path;

use super::deps::build_dep_graph;
use super::DigestFile;

/// Group a file into a display "module" = its directory (files in the same
/// folder share a node), stripping a single leading source-root wrapper
/// (`src`/`lib`/`app`). A top-level file maps to its stem (`src/main.rs` →
/// `main`). Grouping by directory (not first component) keeps monorepo /
/// multi-root layouts from collapsing every edge into a self-loop.
pub(super) fn module_of(rel: &str) -> String {
    let mut parts: Vec<&str> = rel.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return "(root)".to_string();
    }
    if parts.len() > 1 && matches!(parts[0], "src" | "lib" | "app") {
        parts.remove(0);
    }
    if parts.len() == 1 {
        Path::new(parts[0])
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(parts[0])
            .to_string()
    } else {
        parts[..parts.len() - 1].join("/")
    }
}

/// The project's own one-line purpose. Preference, whole-repo signal first:
/// root manifest `description` → root README paragraph → any manifest
/// `description` → any README paragraph. So a workspace with no root
/// `description` falls to its root README rather than an arbitrary sub-crate's
/// (`crates/wire/Cargo.toml` ≠ the whole repo).
pub(super) fn about(files: &[DigestFile]) -> Option<String> {
    let named = |f: &&DigestFile, want: &str| {
        Path::new(&f.rel_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case(want))
            .unwrap_or(false)
    };
    let manifest_desc = |f: &&DigestFile| -> Option<String> {
        if named(f, "Cargo.toml") || named(f, "pyproject.toml") {
            kv_value(&f.content, "description")
        } else if named(f, "package.json") {
            json_value(&f.content, "description")
        } else {
            None
        }
    };
    // Two tiers: root-level files (no `/` in path) first, then everything.
    for root_only in [true, false] {
        let pick = |f: &&DigestFile| !root_only || !f.rel_path.contains('/');
        if let Some(d) = files.iter().filter(pick).find_map(|f| manifest_desc(&f)) {
            return Some(d);
        }
        if let Some(p) = files
            .iter()
            .filter(pick)
            .filter(|f| named(f, "README.md"))
            .find_map(|f| readme_first_para(&f.content))
        {
            return Some(p);
        }
    }
    None
}

fn kv_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix(key) {
            if let Some(rest) = rest.trim_start().strip_prefix('=') {
                let v = rest.trim().trim_matches(|c| c == '"' || c == '\'').trim();
                if !v.is_empty() {
                    return Some(truncate(v, 240));
                }
            }
        }
    }
    None
}

fn json_value(content: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = content.find(&needle)?;
    let after = &content[idx + needle.len()..];
    let colon = after.find(':')?;
    let after = after[colon + 1..].trim_start().strip_prefix('"')?;
    let end = after.find('"')?;
    let v = after[..end].trim();
    if v.is_empty() {
        None
    } else {
        Some(truncate(v, 240))
    }
}

fn readme_first_para(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty()
            || t.starts_with('#')
            || t.starts_with('!')
            || t.starts_with('[')
            || t.starts_with('<')
            || t.starts_with('>')
            || t.starts_with("---")
            || t.starts_with("```")
            || t.starts_with("- ") // list items aren't the project's thesis
            || t.starts_with("* ")
            || t.starts_with("+ ")
            || is_ordered_item(t)
            || t.contains('|')
            || t.contains("://") && t.split_whitespace().count() <= 2
        {
            continue;
        }
        let clean = t.replace(['*', '`'], "");
        if clean.len() >= 20 {
            return Some(truncate(clean.trim(), 240));
        }
    }
    None
}

/// A `1. ` / `2) ` style ordered-list marker.
fn is_ordered_item(t: &str) -> bool {
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    !digits.is_empty()
        && matches!(t[digits.len()..].chars().next(), Some('.') | Some(')'))
        && t[digits.len() + 1..].starts_with(' ')
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}

/// Deduped module-level import edges `(src_module, dst_module)` → weight.
pub(super) fn module_edges(files: &[DigestFile]) -> HashMap<(String, String), usize> {
    let graph = build_dep_graph(files);
    let mut medges: HashMap<(String, String), usize> = HashMap::new();
    for (src, dsts) in &graph.edges {
        let ms = module_of(src);
        for d in dsts {
            let md = module_of(d);
            if ms != md {
                *medges.entry((ms.clone(), md.clone())).or_default() += 1;
            }
        }
    }
    medges
}

/// Fan-in / fan-out per module from a set of module edges.
pub(super) fn degrees(
    medges: &HashMap<(String, String), usize>,
) -> (HashMap<String, usize>, HashMap<String, usize>) {
    let mut in_deg = HashMap::new();
    let mut out_deg = HashMap::new();
    for (s, t) in medges.keys() {
        *out_deg.entry(s.clone()).or_default() += 1;
        *in_deg.entry(t.clone()).or_default() += 1;
    }
    (in_deg, out_deg)
}

/// Threshold above which fan-in counts as "core".
pub(super) fn core_floor(in_deg: &HashMap<String, usize>) -> usize {
    (in_deg.values().copied().max().unwrap_or(0).max(6) / 2).max(3)
}

/// Classify a module by pure fan-in/fan-out topology (no AST):
/// entry (root) · leaf (utility) · core (high fan-in) · internal.
pub(super) fn role_of(in_deg: usize, out_deg: usize, core_floor: usize) -> &'static str {
    if in_deg == 0 && out_deg > 0 {
        "entry"
    } else if out_deg == 0 && in_deg > 0 {
        "leaf"
    } else if in_deg >= core_floor {
        "core"
    } else {
        "internal"
    }
}

/// Human descriptions for each role (order = display order).
pub(super) const ROLE_DESC: &[(&str, &str)] = &[
    ("entry", "roots — nothing imports them (main, CLI)"),
    ("core", "high fan-in — everything routes through"),
    ("leaf", "used by many, import nothing (utils, types)"),
    ("internal", "mid-graph plumbing"),
];

/// One graphed module with its role + topology, ranked by degree.
pub(super) struct RoleInfo {
    pub module: String,
    pub role: &'static str,
    pub in_deg: usize,
    pub out_deg: usize,
}

/// Classify the top-N most-connected modules by role. Used by the markdown
/// digest's architecture section.
pub(super) fn roles(files: &[DigestFile], top: usize) -> Vec<RoleInfo> {
    let medges = module_edges(files);
    let (in_deg, out_deg) = degrees(&medges);
    let floor = core_floor(&in_deg);
    let mut mods: Vec<String> = in_deg.keys().chain(out_deg.keys()).cloned().collect();
    mods.sort_unstable();
    mods.dedup();
    mods.sort_by(|a, b| {
        let da = in_deg.get(a).copied().unwrap_or(0) + out_deg.get(a).copied().unwrap_or(0);
        let db = in_deg.get(b).copied().unwrap_or(0) + out_deg.get(b).copied().unwrap_or(0);
        db.cmp(&da).then(a.cmp(b))
    });
    mods.into_iter()
        .take(top)
        .map(|m| {
            let i = in_deg.get(&m).copied().unwrap_or(0);
            let o = out_deg.get(&m).copied().unwrap_or(0);
            RoleInfo {
                role: role_of(i, o, floor),
                in_deg: i,
                out_deg: o,
                module: m,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn df(rel: &str, content: &str) -> DigestFile {
        DigestFile {
            rel_path: rel.into(),
            content: content.into(),
            tokens: 0,
            loc: 0,
            is_changed: false,
            raw_imports: vec![],
            module_doc: None,
            symbols: vec![],
        }
    }

    #[test]
    fn readme_para_skips_lists_and_blockquotes() {
        // A workspace-container README: only headings, blockquotes and lists.
        assert_eq!(
            readme_first_para("# X\n\n> NOTE\n\n- item one\n1. step one\n* bullet\n"),
            None
        );
        // First real prose wins, list items above it are skipped.
        assert_eq!(
            readme_first_para("# X\n\n- skip me\n\nReal prose describing the thing.\n").as_deref(),
            Some("Real prose describing the thing.")
        );
    }

    #[test]
    fn about_prefers_root_readme_over_nested_manifest() {
        // A workspace root with no description, a real root README, and a
        // sub-package manifest — the root README should win over the sub-crate.
        let files = vec![
            df("Cargo.toml", "[workspace]\nmembers = [\"crates/wire\"]\n"),
            df(
                "README.md",
                "# Repo\n\nThe whole project, in one sentence.\n",
            ),
            df(
                "crates/wire/Cargo.toml",
                "[package]\ndescription = \"just the wire types\"\n",
            ),
        ];
        assert_eq!(
            about(&files).as_deref(),
            Some("The whole project, in one sentence.")
        );
    }

    #[test]
    fn module_of_strips_source_root_and_uses_stem() {
        assert_eq!(module_of("src/main.rs"), "main");
        assert_eq!(module_of("src/ingest/format.rs"), "ingest");
        assert_eq!(module_of("lib/a/b/c.ts"), "a/b");
        assert_eq!(module_of("docu_frontend/src/App.tsx"), "docu_frontend/src");
        assert_eq!(module_of("README.md"), "README");
    }

    #[test]
    fn role_of_classifies_by_topology() {
        // nothing imports it, imports others → entry
        assert_eq!(role_of(0, 5, 3), "entry");
        // imported, imports nothing → leaf
        assert_eq!(role_of(4, 0, 3), "leaf");
        // high fan-in → core
        assert_eq!(role_of(9, 2, 3), "core");
        // mid-graph → internal
        assert_eq!(role_of(2, 2, 3), "internal");
    }

    #[test]
    fn about_prefers_manifest_description() {
        let files = vec![
            df(
                "Cargo.toml",
                "[package]\nname = \"x\"\ndescription = \"a lean tool\"\n",
            ),
            df(
                "README.md",
                "# X\n\nThis readme paragraph should lose to the manifest.\n",
            ),
        ];
        assert_eq!(about(&files).as_deref(), Some("a lean tool"));
    }

    #[test]
    fn about_falls_back_to_readme_paragraph() {
        let files = vec![df(
            "README.md",
            "# Title\n\n![badge](x)\n\nThe first real prose paragraph here.\n",
        )];
        assert_eq!(
            about(&files).as_deref(),
            Some("The first real prose paragraph here.")
        );
    }
}
