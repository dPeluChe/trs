//! `trs audit-docs` — audit AI-agent instruction files for bloat and drift.
//!
//! AI agents load a set of markdown/rules files into context on every prompt
//! (CLAUDE.md, AGENTS.md, .windsurfrules, .devin/rules/*, .cursor/rules/*,
//! .agent/rules/*).
//! Over time these files accumulate:
//!   - content duplicated across files (same "Testing" section in 3 places)
//!   - sections that grew past their useful size (1000-line "Architecture")
//!   - dead references (@old-file.md that was renamed months ago)
//!   - staleness (instructions the code no longer matches)
//!
//! All of this is invisible to the user but charges token-per-prompt
//! indefinitely. This module surfaces it with a quick scan — no LLM calls,
//! no external dependencies.

use std::path::{Path, PathBuf};

/// Agent instruction files we know about. Paths are relative to the project
/// root; `is_dir: true` entries get walked for `*.md` descendants.
const KNOWN_PATHS: &[(&str, bool)] = &[
    ("CLAUDE.md", false),
    ("AGENTS.md", false),
    ("GEMINI.md", false),
    ("CURSOR.md", false),
    (".windsurfrules", false),
    (".devin/rules", true),
    (".windsurf/rules", true),
    (".cursor/rules", true),
    (".agent/rules", true),
    (".agents/rules", true),
    (".codex/rules", true),
    ("docs/development/agent-integrations.md", false),
];

/// One audited file with its computed metrics.
pub(crate) struct DocFile {
    pub(crate) path: PathBuf,
    pub(crate) content: String,
    pub(crate) tokens: usize,
    pub(crate) blocks: Vec<Block>,
    pub(crate) last_touch_days: Option<u64>,
}

/// A paragraph-sized unit inside a file. Line numbers are 1-based and
/// inclusive on both ends.
#[derive(Clone)]
pub(crate) struct Block {
    pub(crate) file_idx: usize,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) text: String,
    pub(crate) simhash: u64,
}

/// Pair of near-duplicate blocks across two files (or the same file).
pub(crate) struct DupPair {
    pub(crate) a: usize, // index into all_blocks
    pub(crate) b: usize,
    pub(crate) similarity_pct: u32,
}

/// A reference (link, @import) that points at a path that doesn't exist.
pub(crate) struct DeadRef {
    pub(crate) file_idx: usize,
    pub(crate) line: usize,
    pub(crate) reference: String,
}

/// An embedded block that likely belongs in its own file instead of inline
/// in a rules/instructions doc.
pub(crate) struct InlineBloat {
    pub(crate) file_idx: usize,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) kind: BloatKind,
    pub(crate) preview: String,
    /// Symbols declared inside the fence (fn / class / export const / def).
    /// Populated only for code fences with a recognizable language.
    pub(crate) symbols: Vec<String>,
    /// For each symbol that was found defined in the project source,
    /// the path of the first source file where it lives.
    /// Empty if the symbol isn't in the codebase (→ needs extraction).
    pub(crate) symbol_matches: Vec<SymbolMatch>,
}

pub(crate) struct SymbolMatch {
    pub(crate) name: String,
    pub(crate) source_path: PathBuf,
}

pub(crate) enum BloatKind {
    /// Fenced code block with a language tag we consider "reference" content
    /// (SQL, JSON, YAML, XML, HTML, GraphQL) above the tight-threshold.
    ReferenceCodeFence { lang: String, lines: usize },
    /// Any fenced code block larger than the generic threshold, regardless
    /// of language. Catches long TypeScript/Python/etc. snippets.
    LargeCodeFence { lang: String, lines: usize },
    /// Markdown table with more than TABLE_ROWS_THRESHOLD data rows — almost
    /// always reference data (API fields, env-var lists) that inflates context.
    LargeTable { rows: usize },
}

// ================================================================
// Entry point
// ================================================================

pub fn run_audit_docs(root: &Path) {
    let docs = discover(root);
    if docs.is_empty() {
        println!(
            "trs audit-docs: no agent instruction files found under {}",
            root.display()
        );
        println!(
            "  (looked for CLAUDE.md, AGENTS.md, .windsurfrules, .devin/rules/*, .cursor/rules/*, .agent/rules/*)"
        );
        return;
    }

    // Flatten into a single block list so we can do pairwise dedup across files.
    let mut all_blocks: Vec<Block> = Vec::new();
    for (i, doc) in docs.iter().enumerate() {
        for mut b in doc.blocks.clone() {
            b.file_idx = i;
            all_blocks.push(b);
        }
    }

    let duplicates = find_near_duplicates(&all_blocks);
    let dead_refs = find_dead_refs(&docs, root);
    let mut inline_bloat = find_inline_bloat(&docs);
    // Resolve symbol references against the project source — tells the user
    // "this function already exists in src/X, drop the duplicate from your
    // doc and link to the source".
    resolve_symbol_matches(&mut inline_bloat, root);

    render_report(&docs, &all_blocks, &duplicates, &dead_refs, &inline_bloat);
}

// ================================================================
// Discovery
// ================================================================

fn discover(root: &Path) -> Vec<DocFile> {
    let mut out: Vec<DocFile> = Vec::new();
    for (rel, is_dir) in KNOWN_PATHS {
        let full = root.join(rel);
        if *is_dir {
            if full.is_dir() {
                collect_markdown_dir(&full, &mut out, root);
            }
        } else if full.is_file() {
            if let Some(doc) = load_doc(&full, root) {
                out.push(doc);
            }
        }
    }
    out
}

fn collect_markdown_dir(dir: &Path, out: &mut Vec<DocFile>, root: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_dir(&path, out, root);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(doc) = load_doc(&path, root) {
                out.push(doc);
            }
        }
    }
}

fn load_doc(path: &Path, root: &Path) -> Option<DocFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let tokens = estimate_tokens(&content);
    let blocks = split_into_blocks(&content);
    let last_touch_days = last_commit_days_ago(path, root);
    Some(DocFile {
        path: path.to_path_buf(),
        content,
        tokens,
        blocks,
        last_touch_days,
    })
}

// ================================================================
// Token estimation (weighted-char heuristic)
// ================================================================

/// Estimate tokens without shipping a BPE tokenizer. Per-char weights come
/// from the OpenAI cookbook rule-of-thumb (4 chars ≈ 1 token for English)
/// tuned per script so CJK doesn't get massively under-counted.
pub(crate) fn estimate_tokens(text: &str) -> usize {
    let mut sum: f64 = 0.0;
    for c in text.chars() {
        sum += if c.is_ascii_alphanumeric() {
            0.22 // ASCII letters/digits: ~4.5 chars per token
        } else if c == ' ' || c == '\t' {
            0.08 // whitespace is cheap
        } else if c == '\n' {
            0.15
        } else if c.is_ascii_punctuation() {
            0.30
        } else if c as u32 > 0x3000 {
            0.66 // CJK range — each char is ~1.5 tokens
        } else {
            0.50
        };
    }
    sum.round() as usize
}

// ================================================================
// Block splitting
// ================================================================

/// Generic code-fence threshold — anything this big is probably better off
/// in its own file that the doc links to.
const GENERIC_CODE_FENCE_MIN: usize = 20;

/// Tight threshold for reference-only languages. SQL/JSON/YAML/XML/GraphQL
/// blocks above this size are almost always dumped reference material, not
/// instructional content.
const REFERENCE_CODE_FENCE_MIN: usize = 10;

/// Any markdown table with more rows than this gets flagged as reference data.
const TABLE_ROWS_THRESHOLD: usize = 8;

const REFERENCE_LANGS: &[&str] = &[
    "sql",
    "postgres",
    "postgresql",
    "mysql",
    "sqlite",
    "json",
    "yaml",
    "yml",
    "toml",
    "xml",
    "html",
    "graphql",
    "gql",
    "csv",
    "tsv",
];

fn find_inline_bloat(docs: &[DocFile]) -> Vec<InlineBloat> {
    let mut out: Vec<InlineBloat> = Vec::new();
    for (idx, doc) in docs.iter().enumerate() {
        collect_code_fences(idx, &doc.content, &mut out);
        collect_large_tables(idx, &doc.content, &mut out);
    }
    out
}

/// Walk fenced code blocks (lines starting with `~~~` or triple-backtick).
/// Flag when they exceed the generic threshold, or a tighter threshold for
/// reference-style languages (SQL, JSON, YAML, etc.).
fn collect_code_fences(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if let Some(lang) = fence_open_lang(trimmed) {
            let start = i + 1; // 1-based: the opening fence line
            let mut j = i + 1;
            while j < lines.len() && !is_fence_close(lines[j].trim_start()) {
                j += 1;
            }
            let inner_lines = j.saturating_sub(i + 1);
            let end = j + 1; // 1-based closing fence line (or EOF)
            let lang_lower = lang.to_ascii_lowercase();
            let is_reference = REFERENCE_LANGS.iter().any(|l| lang_lower == *l);

            let kind = if is_reference && inner_lines >= REFERENCE_CODE_FENCE_MIN {
                Some(BloatKind::ReferenceCodeFence {
                    lang: lang.clone(),
                    lines: inner_lines,
                })
            } else if inner_lines >= GENERIC_CODE_FENCE_MIN {
                Some(BloatKind::LargeCodeFence {
                    lang: lang.clone(),
                    lines: inner_lines,
                })
            } else {
                None
            };

            if let Some(kind) = kind {
                let preview_src = lines.get(i + 1).copied().unwrap_or("");
                let preview: String = preview_src.trim().chars().take(60).collect();
                let body = &lines[(i + 1)..j].join("\n");
                let symbols = extract_fence_symbols(&lang_lower, body);
                out.push(InlineBloat {
                    file_idx,
                    start_line: start,
                    end_line: end.min(lines.len()),
                    kind,
                    preview,
                    symbols,
                    symbol_matches: Vec::new(),
                });
            }
            i = j + 1;
            continue;
        }
        i += 1;
    }
}

/// Pull function / class / export names out of a code fence body so we can
/// cross-reference them against the project source. Only covers the languages
/// most common in agent instruction docs — adding more is a one-line regex.
fn fence_open_lang(line: &str) -> Option<String> {
    let stripped = line
        .strip_prefix("```")
        .or_else(|| line.strip_prefix("~~~"))?;
    // Guard against `````` inline (multi-backtick) — treat as not-a-fence.
    if stripped.starts_with('`') {
        return None;
    }
    Some(stripped.split_whitespace().next().unwrap_or("").to_string())
}

fn is_fence_close(line: &str) -> bool {
    line == "```" || line == "~~~" || line.starts_with("```") || line.starts_with("~~~")
}

/// Detect long markdown tables (`|---|---|` separator + many data rows).
fn collect_large_tables(file_idx: usize, content: &str, out: &mut Vec<InlineBloat>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        // Pattern: a header row like `| a | b |` followed by `|---|---|`.
        if is_table_row(lines[i]) && i + 1 < lines.len() && is_table_separator(lines[i + 1]) {
            let start = i + 1; // header line in 1-based
            let mut rows = 0usize;
            let mut j = i + 2; // first data row
            while j < lines.len() && is_table_row(lines[j]) {
                rows += 1;
                j += 1;
            }
            if rows >= TABLE_ROWS_THRESHOLD {
                let preview: String = lines[i].trim().chars().take(60).collect();
                out.push(InlineBloat {
                    file_idx,
                    start_line: start,
                    end_line: j,
                    kind: BloatKind::LargeTable { rows },
                    preview,
                    symbols: Vec::new(),
                    symbol_matches: Vec::new(),
                });
            }
            i = j;
            continue;
        }
        i += 1;
    }
}

fn is_table_row(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.ends_with('|') && t.matches('|').count() >= 2
}

fn is_table_separator(line: &str) -> bool {
    let t = line.trim();
    if !t.starts_with('|') {
        return false;
    }
    // Separator row cells look like `---`, `:---`, `---:`, `:---:`.
    t.trim_matches('|').split('|').all(|cell| {
        let c = cell.trim();
        !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')
    })
}

// ================================================================
// Symbol resolution against project source
// ================================================================

/// For each symbol referenced in a flagged code fence, walk the project and
/// find the source file where it's actually defined. Lets us tell the user
/// "this class is already in src/foo.ts — just link to it".
///
/// Uses the `ignore` crate so .gitignore is honored; capped at 2000 files
/// traversed to keep audits snappy on large monorepos.
use crate::audit_docs_report::render_report;

use crate::audit_docs_detect::{find_dead_refs, find_near_duplicates, split_into_blocks};

use crate::audit_docs_symbols::{
    extract_fence_symbols, last_commit_days_ago, resolve_symbol_matches,
};
