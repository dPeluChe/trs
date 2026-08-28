use std::collections::BTreeMap;
use std::path::Path;

use super::{DigestFile, IngestConfig};

/// Build the final markdown digest -- directory-driven, no hardcoded categories.
pub(super) fn build_digest(
    files: &[DigestFile],
    config: &IngestConfig,
    _changed_set: &Option<Vec<String>>,
    dep_summary: &str,
    elapsed_ms: u64,
) -> String {
    let mut out = String::new();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_files = files.iter().filter(|f| !f.rel_path.is_empty()).count();

    let project_name = config
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // Detect project type: if >60% of files are .md, it's a docs/skills project
    let md_count = files
        .iter()
        .filter(|f| Path::new(&f.rel_path).extension().and_then(|e| e.to_str()) == Some("md"))
        .count();
    // Also count .txt files as docs
    let txt_count = files
        .iter()
        .filter(|f| Path::new(&f.rel_path).extension().and_then(|e| e.to_str()) == Some("txt"))
        .count();
    let docs_count = md_count + txt_count;
    let is_docs_project = total_files > 0 && docs_count * 100 / total_files.max(1) > 50;

    let project_type: String = if is_docs_project {
        "docs/skills".to_string()
    } else {
        detect_primary_language(files)
    };
    out.push_str(&format!(
        "# {} ({} files, {} tokens, {})\n\n",
        project_name,
        total_files,
        format_tokens(total_tokens),
        project_type
    ));

    // Budget warning, in-band. The stderr warning is invisible to agents (they
    // run `2>/dev/null`), so when an agent pulls a large digest with no budget
    // set, put the nudge inside the digest itself — the one thing it always
    // reads, surviving any redirection.
    if config.agent_mode && config.budget_tokens.is_none() {
        if let Some(threshold) = config.warn_at_tokens {
            if threshold > 0 && total_tokens > threshold {
                out.push_str(&format!(
                    "> ⚠ **Large digest**: {} tokens, no `--budget` set (full dump). \
                     For a tighter, higher-signal context, re-run with `--budget {}`.\n\n",
                    format_tokens(total_tokens),
                    super::resolve::suggest_budget(total_tokens),
                ));
            }
        }
    }

    // About: the project's own one-line purpose (manifest description / README).
    // Gives the reading agent *intent*, not just structure — mirrors the HTML
    // report's subtitle. See `purpose::about`.
    if let Some(about) = super::purpose::about(files) {
        out.push_str(&format!("> {}\n\n", about));
    }

    // Structure
    out.push_str("## Structure\n\n");
    out.push_str(&build_tree(files));
    out.push('\n');

    // Key Dependencies (injected from dep graph, code projects only)
    if !is_docs_project && !dep_summary.is_empty() {
        out.push_str(dep_summary);
    }

    // Architecture: module roles by pure fan-in/fan-out topology (entry / core
    // / leaf / internal). The HTML report shows this as a colored graph; here
    // it's a compact list so an agent gets the same "what routes through what"
    // map without the visual.
    if !is_docs_project {
        let roles = build_roles_section(files);
        if !roles.is_empty() {
            out.push_str(&roles);
        }
    }

    // Symbol index (opt-in via --symbols). Flat `name → path` list so an
    // agent can resolve "where is X?" in one scan without reading any file.
    if !is_docs_project && config.symbols_index {
        let symbols_section = build_symbol_index(files);
        if !symbols_section.is_empty() {
            out.push_str(&symbols_section);
        }
    }

    // Group files by parent directory
    let mut groups: BTreeMap<String, Vec<&DigestFile>> = BTreeMap::new();
    for file in files {
        if file.rel_path.is_empty() {
            continue;
        }
        let dir = Path::new(&file.rel_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        groups.entry(dir).or_default().push(file);
    }

    // Root docs first (README, CLAUDE, AGENTS etc.) -- full content
    if let Some(root_files) = groups.remove("") {
        for file in &root_files {
            let ext = Path::new(&file.rel_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let content = file.content.trim();
            if content.is_empty() {
                continue;
            }

            if ext == "md" {
                let line_count = content.lines().count();
                let is_readme = file.rel_path.to_lowercase().contains("readme");
                // Docs projects: README is the index, show more
                // Code projects: README truncated (setup/install not needed)
                let max_lines = if is_docs_project && is_readme {
                    80
                } else if is_readme {
                    40
                } else if is_docs_project {
                    40
                } else {
                    60
                };

                out.push_str(&format!("## {}\n\n", file.rel_path));
                if line_count > max_lines + 10 {
                    let preview: String = content
                        .lines()
                        .take(max_lines)
                        .collect::<Vec<_>>()
                        .join("\n");
                    out.push_str(&preview);
                    out.push_str(&format!("\n... ({} lines total)\n\n", line_count));
                } else {
                    out.push_str(content);
                    out.push_str("\n\n");
                }
            } else {
                // Config/other root files: compact
                format_file_entry(&mut out, &file.rel_path, content);
            }
        }
    }

    // Each directory as its own section
    for (dir, dir_files) in &groups {
        let dir_display = if dir.is_empty() { "/" } else { dir.as_str() };
        out.push_str(&format!("## {}/\n\n", dir_display));

        for file in dir_files {
            let content = file.content.trim();
            if content.is_empty() {
                continue;
            }

            let ext = Path::new(&file.rel_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let name = Path::new(&file.rel_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file.rel_path);

            if ext == "md" {
                let line_count = content.lines().count();
                // Docs projects: skills/guides are the product, show more
                let sub_max = if is_docs_project { 40 } else { 20 };
                if line_count > sub_max + 10 {
                    let preview: String =
                        content.lines().take(sub_max).collect::<Vec<_>>().join("\n");
                    out.push_str(&format!(
                        "**{}**\n{}\n... ({} lines)\n\n",
                        name, preview, line_count
                    ));
                } else {
                    out.push_str(&format!("**{}**\n{}\n\n", name, content));
                }
            } else {
                // Code/data/config: inline summary
                format_file_entry(&mut out, name, content);
            }
        }
    }

    out.push_str(&format!(
        "---\n*trs ingest v{} | {}ms | {}*\n",
        env!("CARGO_PKG_VERSION"),
        elapsed_ms,
        format_bytes(out.len())
    ));

    out
}

/// Build the "Architecture (module roles)" markdown block: the top graphed
/// modules grouped by fan-in/fan-out role. Empty string if the import graph has
/// no edges (nothing to classify — keeps the digest clean for trivial repos).
fn build_roles_section(files: &[DigestFile]) -> String {
    use std::collections::HashMap;
    let roles = super::purpose::roles(files, 24);
    if roles.is_empty() {
        return String::new();
    }
    let mut by_role: HashMap<&str, Vec<String>> = HashMap::new();
    for r in &roles {
        by_role
            .entry(r.role)
            .or_default()
            .push(format!("`{}` ({}↓/{}↑)", r.module, r.in_deg, r.out_deg));
    }
    let mut out = String::from(
        "## Architecture (module roles)\n\nFrom the import graph (fan-in↓ / fan-out↑), no AST:\n\n",
    );
    for (role, desc) in super::purpose::ROLE_DESC {
        if let Some(mods) = by_role.get(role) {
            // Cap each bucket — `internal` can be long and is the least
            // informative role; entry/core/leaf are naturally short.
            let shown = mods.iter().take(8).cloned().collect::<Vec<_>>().join(", ");
            let more = if mods.len() > 8 {
                format!(" +{} more", mods.len() - 8)
            } else {
                String::new()
            };
            out.push_str(&format!("- **{}**: {} → {}{}\n", role, desc, shown, more));
        }
    }
    out.push('\n');
    out
}

/// Detect the dominant programming language by extension count (code projects only).
/// Returns labels like "rust", "typescript", "python", "go" — or "code" as fallback.
pub(crate) fn detect_primary_language(files: &[DigestFile]) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for f in files {
        let ext = Path::new(&f.rel_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let lang = match ext.as_str() {
            "rs" => "rust",
            "ts" | "tsx" | "mts" | "cts" => "typescript",
            "js" | "jsx" | "mjs" | "cjs" => "javascript",
            "py" | "pyi" => "python",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "swift" => "swift",
            "rb" => "ruby",
            "php" => "php",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "c++",
            "cs" => "c#",
            "sh" | "bash" | "zsh" | "fish" => "shell",
            "lua" => "lua",
            "r" => "r",
            "scala" => "scala",
            "ex" | "exs" => "elixir",
            "erl" => "erlang",
            "dart" => "dart",
            "zig" => "zig",
            _ => continue,
        };
        *counts.entry(lang).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "code".to_string();
    }
    // Pick the language with the most files.
    let total: usize = counts.values().sum();
    let (&primary, &primary_count) = counts.iter().max_by_key(|(_, c)| *c).unwrap();
    // If the winner has <50%, also mention the second one.
    if primary_count * 100 / total < 60 {
        let mut ranked: Vec<(&&str, &usize)> = counts.iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(a.1));
        if ranked.len() >= 2 {
            return format!("{} / {}", ranked[0].0, ranked[1].0);
        }
    }
    primary.to_string()
}

/// Format a single file entry -- code as signatures, data as schema, config as summary.
fn format_file_entry(out: &mut String, name: &str, content: &str) {
    let lines: Vec<&str> = content.lines().collect();

    // Very short content: one-liner
    if lines.len() <= 2 {
        let oneliner = lines.join(" | ");
        out.push_str(&format!("**{}**: {}\n", name, oneliner));
        return;
    }

    // Multi-line signatures (imports + exports)
    let has_signatures = lines.iter().any(|l| {
        l.starts_with("export ")
            || l.starts_with("import ")
            || l.starts_with("pub ")
            || l.starts_with("fn ")
            || l.starts_with("def ")
            || l.starts_with("class ")
            || l.starts_with("use ")
    });

    if has_signatures {
        let sigs: Vec<&str> = lines
            .iter()
            .filter(|l| {
                !l.starts_with("import ") && !l.starts_with("use ") && !l.starts_with("from ")
            })
            .copied()
            .collect();

        out.push_str(&format!("**{}**\n", name));
        for sig in &sigs {
            if sig.is_empty() {
                out.push('\n'); // preserve class spacing
            } else {
                out.push_str(&format!("  {}\n", sig));
            }
        }
    } else {
        // Manifest files have already been compressed to a compact dep-complete
        // form — never re-truncate them at this stage.
        let lower = name.to_lowercase();
        let is_manifest = lower.ends_with("package.json")
            || lower.ends_with("cargo.toml")
            || lower.ends_with("pyproject.toml")
            || lower.ends_with("requirements.txt");
        if is_manifest {
            out.push_str(&format!("**{}**\n{}\n", name, content));
        } else if lines.len() > 10 {
            // Data or config: show as-is but compact
            out.push_str(&format!("**{}**\n", name));
            for line in lines.iter().take(8) {
                out.push_str(line);
                out.push('\n');
            }
            out.push_str(&format!("... ({} lines)\n", lines.len()));
        } else {
            out.push_str(&format!("**{}**\n{}\n", name, content));
        }
    }
    out.push('\n');
}

pub(crate) use super::format_tree::{build_symbol_index, build_tree};

pub(crate) use super::format_util::*;
