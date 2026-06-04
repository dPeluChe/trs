//! `trs audit-docs` — audit AI-agent instruction files for bloat and drift.
//!
//! AI agents load a set of markdown/rules files into context on every prompt
//! (CLAUDE.md, AGENTS.md, .windsurfrules, .cursor/rules/*, .agent/rules/*).
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
use std::process::Command;

/// Agent instruction files we know about. Paths are relative to the project
/// root; `is_dir: true` entries get walked for `*.md` descendants.
const KNOWN_PATHS: &[(&str, bool)] = &[
    ("CLAUDE.md", false),
    ("AGENTS.md", false),
    ("GEMINI.md", false),
    ("CURSOR.md", false),
    (".windsurfrules", false),
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
            "  (looked for CLAUDE.md, AGENTS.md, .windsurfrules, .cursor/rules/*, .agent/rules/*)"
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

/// Split content into paragraph-ish blocks. Only blank lines are boundaries —
/// headings stay attached to their following paragraph so a bare `## Section`
/// doesn't become its own "duplicate" when the same heading appears twice
/// legitimately in different parts of a doc.
fn split_into_blocks(content: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut start_line: usize = 0;
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            flush_block(&mut blocks, &mut current, start_line, idx);
            continue;
        }
        if current.is_empty() {
            start_line = idx + 1;
        } else {
            current.push('\n');
        }
        current.push_str(line);
    }
    flush_block(&mut blocks, &mut current, start_line, lines.len());
    blocks
}

fn flush_block(blocks: &mut Vec<Block>, buf: &mut String, start: usize, end_exclusive: usize) {
    let text = buf.trim().to_string();
    buf.clear();
    // Noise thresholds — too-short blocks generate false-positive duplicates
    // (same short heading, same "```" code-fence marker, etc.)
    const MIN_CHARS: usize = 60;
    if text.chars().count() < MIN_CHARS {
        return;
    }
    // Skip single-line blocks — almost always heading-only or import-style.
    // Multi-line blocks are where real cross-file duplication hides.
    if text.lines().count() < 2 {
        return;
    }
    let simhash = compute_simhash(&text);
    // end_exclusive is the (1-based) index of the blank/EOF boundary; the
    // last content line is end_exclusive itself when the caller passed a
    // raw 0-based blank-line index, so `.max(start)` guards single-line
    // blocks that somehow slipped past the 2-line filter.
    blocks.push(Block {
        file_idx: 0,
        start_line: start,
        end_line: end_exclusive.max(start),
        text,
        simhash,
    });
}

// ================================================================
// SimHash (64-bit, 3-word shingles)
// ================================================================

/// Charikar SimHash over 3-word shingles. Produces a 64-bit fingerprint;
/// Hamming distance ≤3 on 64 bits ≈ 95% similarity — good enough to flag
/// copy-paste and slight rewording, false-positive rate acceptable for a
/// human review workflow.
fn compute_simhash(text: &str) -> u64 {
    const SHINGLE_SIZE: usize = 3;
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return 0;
    }

    let mut counters = [0i32; 64];
    let shingle_count = words.len().saturating_sub(SHINGLE_SIZE - 1).max(1);
    for i in 0..shingle_count {
        let end = (i + SHINGLE_SIZE).min(words.len());
        let shingle = words[i..end].join(" ");
        let h = fnv1a_64(shingle.as_bytes());
        for (bit, counter) in counters.iter_mut().enumerate() {
            if (h >> bit) & 1 == 1 {
                *counter += 1;
            } else {
                *counter -= 1;
            }
        }
    }
    let mut sim = 0u64;
    for (bit, counter) in counters.iter().enumerate() {
        if *counter > 0 {
            sim |= 1 << bit;
        }
    }
    sim
}

/// FNV-1a 64-bit — small, non-cryptographic, enough for SimHash shingles.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Find pairs of blocks whose fingerprints differ by at most 6 bits (≈90%
/// similarity). The quadratic pairwise compare is fine up to a few thousand
/// blocks — typical agent-doc corpora are under 200 blocks total.
fn find_near_duplicates(blocks: &[Block]) -> Vec<DupPair> {
    const MAX_HAMMING: u32 = 6;
    let mut out: Vec<DupPair> = Vec::new();
    for i in 0..blocks.len() {
        for j in (i + 1)..blocks.len() {
            let dist = (blocks[i].simhash ^ blocks[j].simhash).count_ones();
            if dist <= MAX_HAMMING {
                let similarity_pct = 100u32.saturating_sub(dist * 100 / 64);
                out.push(DupPair {
                    a: i,
                    b: j,
                    similarity_pct,
                });
            }
        }
    }
    out.sort_by_key(|p| std::cmp::Reverse(p.similarity_pct));
    out
}

// ================================================================
// Dead-reference detection
// ================================================================

fn find_dead_refs(docs: &[DocFile], root: &Path) -> Vec<DeadRef> {
    let mut out: Vec<DeadRef> = Vec::new();
    for (idx, doc) in docs.iter().enumerate() {
        let doc_dir = doc.path.parent().unwrap_or(root);
        for (line_num, line) in doc.content.lines().enumerate() {
            for reference in extract_references(line) {
                if !ref_resolves(&reference, doc_dir, root) {
                    out.push(DeadRef {
                        file_idx: idx,
                        line: line_num + 1,
                        reference,
                    });
                }
            }
        }
    }
    out
}

/// Pick out `@imports` and markdown `[text](./path)` links whose targets
/// look like LOCAL files we should be able to resolve (skip URLs, anchors,
/// npm package names, code-block content).
fn extract_references(line: &str) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    // @imports — Claude Code / agent rules file include syntax.
    // Be strict here: the `@foo/bar` form is also npm-package syntax
    // (@heroicons/react, @types/bcryptjs). Only treat as an import if it
    // has an explicit relative prefix or a recognizable doc extension.
    for token in line.split_whitespace() {
        let Some(rest) = token.strip_prefix('@') else {
            continue;
        };
        let cleaned = rest.trim_end_matches(|c: char| matches!(c, ',' | '.' | ';' | ':' | ')'));
        if looks_like_import_path(cleaned) {
            refs.push(cleaned.to_string());
        }
    }

    // Markdown links: [text](path) — only local paths. Same strictness
    // applies, but markdown link targets are less ambiguous than @imports.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            if let Some(end) = line[i + 2..].find(')') {
                let path = &line[i + 2..i + 2 + end];
                if looks_like_local_markdown_link(path) {
                    refs.push(path.to_string());
                }
                i += 2 + end;
                continue;
            }
        }
        i += 1;
    }
    refs
}

/// True for tokens like `@./foo.md`, `@../rules/bar.md`, `@docs/guide.md`
/// but NOT `@heroicons/react`, `@types/bcryptjs` (npm scope packages).
fn looks_like_import_path(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // Explicit relative path — definitely a local import.
    if s.starts_with("./") || s.starts_with("../") {
        return true;
    }
    // Has a doc-looking extension we expect to find on disk.
    const DOC_EXTENSIONS: &[&str] = &[
        ".md", ".mdx", ".txt", ".json", ".yaml", ".yml", ".toml", ".rs", ".ts", ".tsx", ".js",
        ".jsx", ".py", ".go", ".rb", ".sh", ".html", ".xml", ".sql",
    ];
    DOC_EXTENSIONS.iter().any(|ext| s.ends_with(ext))
}

fn looks_like_local_markdown_link(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.starts_with('#') {
        return false;
    }
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("mailto:") {
        return false;
    }
    // Skip image/data URIs and anchors — require at least one path-ish char.
    s.contains('.') || s.contains('/')
}

fn ref_resolves(reference: &str, doc_dir: &Path, root: &Path) -> bool {
    let candidate = if reference.starts_with('/') {
        root.join(reference.trim_start_matches('/'))
    } else {
        doc_dir.join(reference)
    };
    // Strip anchor (`.md#section`) before checking.
    let candidate = match candidate.to_str() {
        Some(s) => s.split('#').next().map(PathBuf::from).unwrap_or(candidate),
        None => candidate,
    };
    candidate.exists()
}

// ================================================================
// Git staleness
// ================================================================

// ================================================================
// Inline bloat detection (code fences + tables)
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
fn extract_fence_symbols(lang: &str, body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for raw in body.lines() {
        let line = raw.trim_start();
        let sym = match lang {
            "ts" | "tsx" | "js" | "jsx" | "javascript" | "typescript" => {
                extract_js_like_symbol(line)
            }
            "py" | "python" => extract_python_symbol(line),
            "rs" | "rust" => extract_rust_symbol(line),
            "go" => extract_go_symbol(line),
            "swift" => extract_swift_symbol(line),
            _ => None,
        };
        if let Some(name) = sym {
            if is_meaningful_symbol(&name) && seen.insert(name.clone()) {
                out.push(name);
            }
        }
    }
    out
}

/// Filter out ultra-generic identifiers that almost always collide with
/// unrelated definitions in a large codebase. A match on `data` or `result`
/// isn't informative; a match on `RouteBuilder` is.
fn is_meaningful_symbol(name: &str) -> bool {
    if name.len() < 4 {
        return false;
    }
    const GENERIC_BLOCKLIST: &[&str] = &[
        // Ultra-common variable / parameter names.
        "data",
        "result",
        "results",
        "value",
        "values",
        "item",
        "items",
        "response",
        "request",
        "req",
        "res",
        "page",
        "pages",
        "search",
        "json",
        "xml",
        "html",
        "text",
        "name",
        "names",
        "id",
        "ids",
        "key",
        "keys",
        "index",
        "count",
        "size",
        "length",
        "total",
        "form",
        "forms",
        "input",
        "output",
        "error",
        "errors",
        "err",
        "props",
        "state",
        "config",
        "options",
        "args",
        "params",
        "user",
        "users",
        "client",
        "clients",
        "list",
        "lists",
        "main",
        "init",
        "setup",
        "start",
        "stop",
        "run",
        "test",
        "tests",
        "util",
        "utils",
        "helper",
        "helpers",
        "message",
        "messages",
        "status",
        "title",
        "body",
        "header",
        "headers",
        "footer",
        "row",
        "rows",
        "column",
        "columns",
        "table",
        "tables",
        "type",
        "types",
        "kind",
        "method",
        "methods",
        "callback",
        "handler",
        "handlers",
        "filter",
        "filters",
        "reducer",
        "action",
        "store",
        "stores",
        "model",
        "models",
        "view",
        "views",
        "render",
        "click",
        "submit",
        "change",
        "select",
        "update",
        "create",
        "delete",
        "isValid",
        "isEditing",
        "isLoading",
        "isOpen",
        "isActive",
    ];
    let lower = name.to_ascii_lowercase();
    !GENERIC_BLOCKLIST.iter().any(|g| *g == lower)
}

fn extract_js_like_symbol(line: &str) -> Option<String> {
    // export default function Foo(
    // export function Foo(
    // export const Foo =
    // export class Foo
    // export interface Foo
    // export type Foo =
    // function Foo(
    // class Foo
    let rest = line
        .strip_prefix("export default ")
        .or_else(|| line.strip_prefix("export "))
        .unwrap_or(line);
    for kw in &[
        "async function ",
        "function ",
        "const ",
        "let ",
        "class ",
        "interface ",
        "type ",
        "enum ",
    ] {
        if let Some(body) = rest.strip_prefix(kw) {
            return first_ident(body);
        }
    }
    None
}

fn extract_python_symbol(line: &str) -> Option<String> {
    if let Some(body) = line
        .strip_prefix("async def ")
        .or_else(|| line.strip_prefix("def "))
        .or_else(|| line.strip_prefix("class "))
    {
        let name = first_ident(body)?;
        if name.starts_with('_') {
            return None;
        }
        return Some(name);
    }
    None
}

fn extract_rust_symbol(line: &str) -> Option<String> {
    let rest = line.strip_prefix("pub ")?;
    let rest = rest
        .strip_prefix("async ")
        .or_else(|| rest.strip_prefix("unsafe "))
        .unwrap_or(rest);
    for kw in &["fn ", "struct ", "enum ", "trait ", "type ", "const "] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn extract_go_symbol(line: &str) -> Option<String> {
    for kw in &["func ", "type ", "var ", "const "] {
        if let Some(body) = line.strip_prefix(*kw) {
            let body = body.trim_start_matches('(');
            let name = first_ident(body)?;
            if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                return Some(name);
            }
            return None;
        }
    }
    None
}

fn extract_swift_symbol(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("public ")
        .or_else(|| line.strip_prefix("open "))?;
    for kw in &["func ", "class ", "struct ", "enum ", "protocol "] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn first_ident(s: &str) -> Option<String> {
    let mut chars = s.char_indices();
    let start = chars.position(|(_, c)| c.is_ascii_alphabetic() || c == '_')?;
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        if c.is_ascii_alphanumeric() || c == '_' {
            end = start + i + c.len_utf8();
        } else {
            break;
        }
    }
    if end > start {
        Some(s[start..end].to_string())
    } else {
        None
    }
}

/// A fence opener is `~~~` or ``` (three backticks), optionally followed by
/// a language hint token. Returns the hint (or empty string) or None if the
/// line is not a fence opener.
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
fn resolve_symbol_matches(bloat: &mut [InlineBloat], root: &Path) {
    // Dedup symbols across all bloat entries so we walk the tree once.
    let mut wanted: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for (idx, b) in bloat.iter().enumerate() {
        for sym in &b.symbols {
            wanted.entry(sym.clone()).or_default().push(idx);
        }
    }
    if wanted.is_empty() {
        return;
    }

    let mut found: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    let walker = ignore::WalkBuilder::new(root)
        .standard_filters(true)
        .max_depth(Some(8))
        .build();

    let mut files_scanned = 0usize;
    for entry in walker.flatten() {
        if files_scanned >= 2000 || found.len() == wanted.len() {
            break;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "py" | "rs" | "go" | "swift" | "java" | "kt"
        ) {
            continue;
        }
        // Skip synthetic/generated areas the ignore crate may not know about.
        let path_str = path.to_string_lossy();
        if path_str.contains("/dist/")
            || path_str.contains("/build/")
            || path_str.contains("/coverage/")
            || path_str.contains("/.next/")
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        files_scanned += 1;
        for sym in wanted.keys() {
            if found.contains_key(sym) {
                continue;
            }
            if contains_symbol_definition(&content, sym, ext) {
                found.insert(sym.clone(), path.to_path_buf());
            }
        }
    }

    for (sym, src) in found {
        if let Some(bloat_indices) = wanted.get(&sym) {
            for idx in bloat_indices {
                bloat[*idx].symbol_matches.push(SymbolMatch {
                    name: sym.clone(),
                    source_path: src.clone(),
                });
            }
        }
    }
}

/// True if `content` contains what looks like a definition of `sym` in a
/// file of extension `ext`. Cheap substring match — not a full parser, but
/// accurate enough for the "does this identifier live somewhere in src?"
/// question.
fn contains_symbol_definition(content: &str, sym: &str, ext: &str) -> bool {
    // Build a few candidate prefixes — the patterns are common definition
    // forms across the supported languages.
    let needles: Vec<String> = match ext {
        "ts" | "tsx" | "js" | "jsx" => vec![
            format!("function {}(", sym),
            format!("function {} (", sym),
            format!("class {}", sym),
            format!("interface {}", sym),
            format!("type {} ", sym),
            format!("type {}=", sym),
            format!("const {} =", sym),
            format!("const {}=", sym),
            format!("let {} =", sym),
            format!("enum {}", sym),
        ],
        "py" => vec![
            format!("def {}(", sym),
            format!("async def {}(", sym),
            format!("class {}(", sym),
            format!("class {}:", sym),
        ],
        "rs" => vec![
            format!("fn {}(", sym),
            format!("fn {}<", sym),
            format!("struct {}", sym),
            format!("enum {}", sym),
            format!("trait {}", sym),
            format!("type {} ", sym),
            format!("const {} ", sym),
        ],
        "go" => vec![
            format!("func {}(", sym),
            format!("func ({} ", sym),
            format!("type {} ", sym),
        ],
        "swift" => vec![
            format!("func {}(", sym),
            format!("class {}", sym),
            format!("struct {}", sym),
            format!("enum {}", sym),
            format!("protocol {}", sym),
        ],
        "java" | "kt" => vec![
            format!("class {}", sym),
            format!("interface {}", sym),
            format!("fun {}(", sym),
            format!("{} {}(", "void", sym),
        ],
        _ => return false,
    };
    needles.iter().any(|n| content.contains(n.as_str()))
}

fn last_commit_days_ago(path: &Path, root: &Path) -> Option<u64> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(path)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ts_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let ts: u64 = ts_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(now.saturating_sub(ts) / 86400)
}

use crate::audit_docs_report::render_report;
