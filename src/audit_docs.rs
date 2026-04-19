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
    ("docs/agent-integrations.md", false),
];

/// One audited file with its computed metrics.
struct DocFile {
    path: PathBuf,
    content: String,
    tokens: usize,
    blocks: Vec<Block>,
    last_touch_days: Option<u64>,
}

/// A paragraph-sized unit inside a file. Line numbers are 1-based and
/// inclusive on both ends.
#[derive(Clone)]
struct Block {
    file_idx: usize,
    start_line: usize,
    end_line: usize,
    text: String,
    simhash: u64,
}

/// Pair of near-duplicate blocks across two files (or the same file).
struct DupPair {
    a: usize, // index into all_blocks
    b: usize,
    similarity_pct: u32,
}

/// A reference (link, @import) that points at a path that doesn't exist.
struct DeadRef {
    file_idx: usize,
    line: usize,
    reference: String,
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

    render_report(&docs, &all_blocks, &duplicates, &dead_refs);
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

/// Split content into paragraph-ish blocks:
///   - blank lines or heading lines are separators
///   - we track start/end line for reporting
///   - each block's simhash is computed upfront
fn split_into_blocks(content: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut start_line: usize = 0;
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_boundary = trimmed.is_empty() || trimmed.starts_with('#');
        if is_boundary {
            flush_block(&mut blocks, &mut current, start_line, idx);
            if trimmed.starts_with('#') {
                // Headings themselves become 1-line blocks so cross-file
                // identical heading text still gets flagged.
                let text = trimmed.to_string();
                let simhash = compute_simhash(&text);
                blocks.push(Block {
                    file_idx: 0,
                    start_line: idx + 1,
                    end_line: idx + 1,
                    text,
                    simhash,
                });
            }
            start_line = idx + 2;
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
    if text.chars().count() < 20 {
        // Blocks shorter than ~20 chars generate noisy near-duplicates
        // (headings-only, bullet markers). Skip.
        return;
    }
    let simhash = compute_simhash(&text);
    blocks.push(Block {
        file_idx: 0,
        start_line: start,
        end_line: end_exclusive.saturating_sub(1).max(start),
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

/// Pick out `@path` imports and markdown `[text](./path)` links whose targets
/// look like local files (skip URLs, anchors, empty paths).
fn extract_references(line: &str) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    // @imports — Claude Code / agent rules file include syntax.
    for token in line.split_whitespace() {
        if let Some(rest) = token.strip_prefix('@') {
            let cleaned = rest.trim_end_matches(|c: char| matches!(c, ',' | '.' | ';' | ':'));
            if looks_like_local_path(cleaned) {
                refs.push(cleaned.to_string());
            }
        }
    }

    // Markdown links: [text](path) — only local paths
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            if let Some(end) = line[i + 2..].find(')') {
                let path = &line[i + 2..i + 2 + end];
                if looks_like_local_path(path) {
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

fn looks_like_local_path(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.starts_with('#') {
        return false;
    }
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("mailto:") {
        return false;
    }
    // Anchor-only (`#section`) filtered above. Allow `./x.md`, `../x`, `path/x`, `x.md`.
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

// ================================================================
// Report rendering
// ================================================================

fn render_report(
    docs: &[DocFile],
    all_blocks: &[Block],
    duplicates: &[DupPair],
    dead_refs: &[DeadRef],
) {
    let total_tokens: usize = docs.iter().map(|d| d.tokens).sum();
    println!(
        "trs audit-docs — {} file{}, {} tokens total loaded per agent session",
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
                "  - {} ({} tokens) — consider splitting heavy sections into linked docs",
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

    // Recommendations
    println!();
    if duplicates.is_empty() && dead_refs.is_empty() && !docs.iter().any(|d| d.tokens > 5000) {
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
                "  - Dedup the flagged block pairs — ~{} tokens saved per session",
                human_tokens(potential)
            );
        }
        if !dead_refs.is_empty() {
            println!(
                "  - Fix or remove the {} dead reference{}",
                dead_refs.len(),
                if dead_refs.len() == 1 { "" } else { "s" }
            );
        }
        if docs.iter().any(|d| d.tokens > 5000) {
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
