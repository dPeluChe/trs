//! `audit-docs` report rendering: the human-readable summary of duplicates,
//! dead refs, and inline bloat. Detection lives in the sibling modules.

use super::audit_docs::{
    estimate_tokens, BloatKind, Block, DeadRef, DocFile, DupPair, InlineBloat,
};

// ================================================================
// Report rendering
// ================================================================

pub(crate) fn render_report(
    docs: &[DocFile],
    all_blocks: &[Block],
    duplicates: &[DupPair],
    dead_refs: &[DeadRef],
    inline_bloat: &[InlineBloat],
) {
    let total_tokens: usize = docs.iter().map(|d| d.tokens).sum();
    println!(
        "trs audit-docs: {} file{}, {} tokens total loaded per agent session",
        docs.len(),
        if docs.len() == 1 { "" } else { "s" },
        human_tokens(total_tokens),
    );
    println!();

    // Per-file table
    let max_path = docs
        .iter()
        .map(|d| d.path.to_string_lossy().len())
        .max()
        .unwrap_or(0)
        .max(4);
    println!(
        "  {:<width$}  {:>8}  {:>8}  staleness",
        "file",
        "tokens",
        "blocks",
        width = max_path
    );
    for doc in docs {
        let stale = match doc.last_touch_days {
            Some(0) => "today".to_string(),
            Some(1) => "1 day ago".to_string(),
            Some(d) if d < 30 => format!("{} days ago", d),
            Some(d) if d < 365 => format!("{} months ago", d / 30),
            Some(d) => format!("{} years ago", d / 365),
            None => "—".to_string(),
        };
        let bloat = if doc.tokens > 5000 { " ⚠" } else { "" };
        println!(
            "  {:<width$}  {:>8}  {:>8}  {}{}",
            doc.path.display(),
            human_tokens(doc.tokens),
            doc.blocks.len(),
            stale,
            bloat,
            width = max_path
        );
    }

    // Bloat warning
    let big: Vec<_> = docs.iter().filter(|d| d.tokens > 5000).collect();
    if !big.is_empty() {
        println!();
        println!("⚠ Large files (>5k tokens):");
        for d in big {
            println!(
                "  - {} ({} tokens), consider splitting heavy sections into linked docs",
                d.path.display(),
                human_tokens(d.tokens)
            );
        }
    }

    // Duplicates
    if !duplicates.is_empty() {
        println!();
        println!(
            "⚠ Near-duplicate blocks ({} pair{}):",
            duplicates.len(),
            if duplicates.len() == 1 { "" } else { "s" }
        );
        for pair in duplicates.iter().take(10) {
            let a = &all_blocks[pair.a];
            let b = &all_blocks[pair.b];
            let a_path = docs[a.file_idx].path.display();
            let b_path = docs[b.file_idx].path.display();
            let preview: String = a.text.chars().take(60).collect();
            let marker = if pair.similarity_pct == 100 {
                "≡"
            } else {
                "≈"
            };
            println!(
                "  {} {}:{}-{}  {}  {}:{}-{}  ({}%)",
                marker,
                a_path,
                a.start_line,
                a.end_line,
                marker,
                b_path,
                b.start_line,
                b.end_line,
                pair.similarity_pct
            );
            println!("      \"{}…\"", preview);
        }
        if duplicates.len() > 10 {
            println!("  ... +{} more pairs", duplicates.len() - 10);
        }
    }

    // Dead refs
    if !dead_refs.is_empty() {
        println!();
        println!("⚠ Dead references ({}):", dead_refs.len());
        for dr in dead_refs.iter().take(10) {
            let path = docs[dr.file_idx].path.display();
            println!("  {}:{} → {} (not found)", path, dr.line, dr.reference);
        }
        if dead_refs.len() > 10 {
            println!("  ... +{} more", dead_refs.len() - 10);
        }
    }

    // Inline bloat: large code fences, reference-data dumps, big tables
    if !inline_bloat.is_empty() {
        println!();
        println!(
            "⚠ Embedded reference content ({} block{}):",
            inline_bloat.len(),
            if inline_bloat.len() == 1 { "" } else { "s" }
        );
        for b in inline_bloat.iter().take(10) {
            let path = docs[b.file_idx].path.display();
            let (label, hint) = match &b.kind {
                BloatKind::ReferenceCodeFence { lang, lines } => (
                    format!("`{}` code fence ({} lines)", lang, lines),
                    match lang.to_ascii_lowercase().as_str() {
                        "sql" | "postgres" | "postgresql" | "mysql" | "sqlite" => {
                            "move queries to docs/queries.sql or a dedicated doc"
                        }
                        "json" | "yaml" | "yml" | "toml" => {
                            "move config samples to a standalone example file"
                        }
                        "graphql" | "gql" => "move schema/queries to docs/schema.graphql",
                        "xml" | "html" => "extract fixture to a standalone file, link it",
                        _ => "move to a standalone file, link it from here",
                    },
                ),
                BloatKind::LargeCodeFence { lang, lines } => {
                    let lang_str = if lang.is_empty() {
                        "code".to_string()
                    } else {
                        format!("`{}`", lang)
                    };
                    (
                        format!("{} block ({} lines)", lang_str, lines),
                        "extract to a source file the doc links to",
                    )
                }
                BloatKind::LargeTable { rows } => (
                    format!("markdown table ({} rows)", rows),
                    "move reference tables (API fields, env vars) to their own doc",
                ),
            };
            // Header: path + kind + (if any) compact list of declared symbols.
            let sym_suffix = if b.symbols.is_empty() {
                String::new()
            } else {
                let joined = b
                    .symbols
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let tail = if b.symbols.len() > 4 {
                    format!(", +{} more", b.symbols.len() - 4)
                } else {
                    String::new()
                };
                format!(", declares: {}{}", joined, tail)
            };
            println!(
                "  {}:{}-{}  {}{}",
                path, b.start_line, b.end_line, label, sym_suffix
            );
            if !b.preview.is_empty() {
                println!("      \"{}…\"", b.preview);
            }

            // Per-symbol verdict: which ones already live in source vs which
            // need extraction. Gives the user a concrete remove-vs-extract call.
            if !b.symbols.is_empty() {
                let found_count = b.symbol_matches.len();
                if found_count > 0 {
                    println!(
                        "      ▸ REMOVE from doc: {} of {} symbols already in source:",
                        found_count,
                        b.symbols.len()
                    );
                    for m in b.symbol_matches.iter().take(5) {
                        println!("          {} → {}", m.name, m.source_path.display());
                    }
                    if b.symbol_matches.len() > 5 {
                        println!("          ...");
                    }
                }
                let missing: Vec<&String> = b
                    .symbols
                    .iter()
                    .filter(|s| !b.symbol_matches.iter().any(|m| &m.name == *s))
                    .collect();
                if !missing.is_empty() {
                    let shown: Vec<String> = missing.iter().take(5).map(|s| (*s).clone()).collect();
                    let tail = if missing.len() > shown.len() {
                        format!(", +{} more", missing.len() - shown.len())
                    } else {
                        String::new()
                    };
                    println!(
                        "      ▸ EXTRACT: {} symbol(s) not found in source: {}{}",
                        missing.len(),
                        shown.join(", "),
                        tail
                    );
                }
            }

            println!("      → {}", hint);
        }
        if inline_bloat.len() > 10 {
            println!("  ... +{} more", inline_bloat.len() - 10);
        }
    }

    // Recommendations
    let has_bloat_file = docs.iter().any(|d| d.tokens > 5000);
    let any_finding = !duplicates.is_empty()
        || !dead_refs.is_empty()
        || has_bloat_file
        || !inline_bloat.is_empty();
    println!();
    if !any_finding {
        println!("✓ No bloat, duplicates, or dead references detected.");
    } else {
        println!("Recommendations:");
        if !duplicates.is_empty() {
            let potential: usize = duplicates
                .iter()
                .take(10)
                .map(|p| estimate_tokens(&all_blocks[p.b].text))
                .sum();
            println!(
                "  - Dedup the flagged block pairs, ~{} tokens saved per session",
                human_tokens(potential)
            );
        }
        if !inline_bloat.is_empty() {
            println!(
                "  - Extract the {} embedded reference block{} to standalone files",
                inline_bloat.len(),
                if inline_bloat.len() == 1 { "" } else { "s" }
            );
        }
        if !dead_refs.is_empty() {
            println!(
                "  - Fix or remove the {} dead reference{}",
                dead_refs.len(),
                if dead_refs.len() == 1 { "" } else { "s" }
            );
        }
        if has_bloat_file {
            println!("  - Split large files; link heavy sections from a small index");
        }
    }
}

fn human_tokens(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}
