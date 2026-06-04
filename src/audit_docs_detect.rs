//! `audit-docs` detection: paragraph blocks + simhash near-duplicate finder,
//! and dead-reference resolution. Shared types live in `audit_docs.rs`.

use std::path::{Path, PathBuf};

use super::audit_docs::{Block, DeadRef, DocFile, DupPair};

/// Split content into paragraph-ish blocks. Only blank lines are boundaries —
/// headings stay attached to their following paragraph so a bare `## Section`
/// doesn't become its own "duplicate" when the same heading appears twice
/// legitimately in different parts of a doc.
pub(crate) fn split_into_blocks(content: &str) -> Vec<Block> {
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
pub(crate) fn compute_simhash(text: &str) -> u64 {
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
pub(crate) fn find_near_duplicates(blocks: &[Block]) -> Vec<DupPair> {
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

pub(crate) fn find_dead_refs(docs: &[DocFile], root: &Path) -> Vec<DeadRef> {
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
