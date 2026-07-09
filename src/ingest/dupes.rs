//! Near-duplicate function detection — a lean port of codebase-memory-mcp's
//! MinHash+LSH clone finder. Instead of AST trigrams (which need a grammar) we
//! scan function bodies with a brace/indent boundary detector and fingerprint a
//! token stream: identifiers/keywords kept verbatim, only string literals and
//! numbers masked. Keeping names trades rename-invariance for **precision** —
//! it flags actionable copy-paste (shared names) and never cries wolf on
//! coincidental control-flow shapes. MinHash+LSH make the pairwise search cheap.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::DigestFile;

const K: usize = 32; // MinHash signature length
const BANDS: usize = 16; // LSH bands (rows = K/BANDS = 2)
const SHINGLE: usize = 3; // token-trigram shingles
const MIN_TOKENS: usize = 70; // skip small functions (tiny bodies over-match)
const JACCARD: f64 = 0.85; // similarity threshold

/// A near-duplicate pair: two function labels + their estimated similarity.
pub(super) struct Dupe {
    pub a: String,
    pub b: String,
    pub sim: f64,
}

/// 64 fixed multipliers → K independent hash functions (no RNG; deterministic).
fn seed(k: usize) -> u64 {
    // odd constants derived from the golden ratio, distinct per k.
    0x9e3779b97f4a7c15u64
        .wrapping_mul(2 * k as u64 + 1)
        .wrapping_add(0xD1B54A32D192ED03)
}

fn hash64(x: u64) -> u64 {
    // splitmix64 finalizer
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Tokenize a body, UTF-8 safe. We KEEP identifiers/keywords verbatim (so two
/// structurally-similar-but-unrelated functions don't collide) and only mask
/// string literals → `"S"` and numbers → `"N"` (tolerates literal tweaks).
/// This trades rename-invariance for **precision** — it flags real copy-paste
/// that shares variable/function names, which is the actionable case, and
/// never cries wolf on coincidental control-flow shapes.
fn tokenize(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' || c == '\'' || c == '`' {
            let q = c;
            chars.next();
            while let Some(d) = chars.next() {
                if d == '\\' {
                    chars.next();
                } else if d == q {
                    break;
                }
            }
            out.push("\"S".to_string());
        } else if c.is_alphanumeric() || c == '_' {
            let mut word = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_alphanumeric() || d == '_' {
                    word.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if word
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                out.push("#N".to_string());
            } else {
                out.push(word);
            }
        } else {
            chars.next();
            out.push(c.to_string());
        }
    }
    out
}

/// Set of shingle hashes (token trigrams) for a token stream.
fn shingles(tokens: &[String]) -> HashSet<u64> {
    let mut set = HashSet::new();
    if tokens.len() < SHINGLE {
        return set;
    }
    for w in tokens.windows(SHINGLE) {
        let mut h = 0xcbf29ce484222325u64;
        for t in w {
            for b in t.bytes() {
                h = (h ^ b as u64).wrapping_mul(0x100000001b3);
            }
            h = h.wrapping_mul(31).wrapping_add(0x9e37);
        }
        set.insert(h);
    }
    set
}

/// K-length MinHash signature of a shingle set.
fn minhash(set: &HashSet<u64>) -> [u64; K] {
    let mut sig = [u64::MAX; K];
    for &s in set {
        for (k, slot) in sig.iter_mut().enumerate() {
            let h = hash64(s ^ seed(k));
            if h < *slot {
                *slot = h;
            }
        }
    }
    sig
}

fn jaccard(a: &HashSet<u64>, b: &HashSet<u64>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.iter().filter(|x| b.contains(x)).count();
    let union = a.len() + b.len() - inter;
    inter as f64 / union as f64
}

struct Func {
    label: String,
    shingles: HashSet<u64>,
    sig: [u64; K],
}

/// Extract functions from a source file's (already comment-stripped) content.
/// Brace languages: capture `fn`/`function`/method bodies by brace balance.
/// Python: capture `def` bodies by indentation.
fn functions(rel: &str, content: &str) -> Vec<Func> {
    let is_py = rel.ends_with(".py");
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let short = Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(rel);
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        let name = fn_name(l, is_py);
        if let Some(name) = name {
            if name.starts_with("test") || name.starts_with("bench") {
                i += 1;
                continue;
            }
            let (body, next) = if is_py {
                capture_py(&lines, i)
            } else {
                capture_braces(&lines, i)
            };
            let toks = tokenize(&body);
            if toks.len() >= MIN_TOKENS {
                let sh = shingles(&toks);
                if !sh.is_empty() {
                    let sig = minhash(&sh);
                    out.push(Func {
                        label: format!("{}:{}", short, name),
                        shingles: sh,
                        sig,
                    });
                }
            }
            i = next.max(i + 1);
            continue;
        }
        i += 1;
    }
    out
}

/// Detect a function signature line and return the function name.
fn fn_name(line: &str, is_py: bool) -> Option<String> {
    let t = line.trim_start();
    if is_py {
        let rest = t
            .strip_prefix("def ")
            .or_else(|| t.strip_prefix("async def "))?;
        return Some(ident(rest));
    }
    // Rust
    if let Some(idx) = t.find("fn ") {
        // avoid matching inside strings/comments crudely: fn must be a word
        let before_ok = idx == 0 || !t.as_bytes()[idx - 1].is_ascii_alphanumeric();
        if before_ok {
            let rest = &t[idx + 3..];
            let name = ident(rest);
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    // JS/TS `function name(`
    if let Some(rest) = t.strip_prefix("function ") {
        let name = ident(rest);
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn ident(s: &str) -> String {
    s.trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Capture a brace-delimited body starting at `start`; returns (body, next_line).
fn capture_braces(lines: &[&str], start: usize) -> (String, usize) {
    let mut depth = 0i32;
    let mut started = false;
    let mut body = String::new();
    let mut j = start;
    while j < lines.len() {
        for c in lines[j].chars() {
            if c == '{' {
                depth += 1;
                started = true;
            } else if c == '}' {
                depth -= 1;
            }
        }
        body.push_str(lines[j]);
        body.push('\n');
        j += 1;
        if started && depth <= 0 {
            break;
        }
        if body.len() > 60_000 {
            break; // safety
        }
    }
    (body, j)
}

/// Capture an indentation-delimited Python body.
fn capture_py(lines: &[&str], start: usize) -> (String, usize) {
    let indent = |s: &str| s.len() - s.trim_start().len();
    let base = indent(lines[start]);
    let mut body = String::new();
    body.push_str(lines[start]);
    body.push('\n');
    let mut j = start + 1;
    while j < lines.len() {
        let l = lines[j];
        if l.trim().is_empty() {
            body.push('\n');
            j += 1;
            continue;
        }
        if indent(l) <= base {
            break;
        }
        body.push_str(l);
        body.push('\n');
        j += 1;
    }
    (body, j)
}

/// Find near-duplicate function pairs across all code files. LSH buckets by
/// band, then confirms with exact Jaccard on the shingle sets.
pub(super) fn find_dupes(files: &[&DigestFile]) -> Vec<Dupe> {
    let mut funcs: Vec<Func> = Vec::new();
    for f in files {
        // Skip test files — test functions are deliberately similar in shape
        // (assert patterns) and would swamp the real copy-paste signal.
        if is_code(&f.rel_path) && !f.rel_path.to_lowercase().contains("test") {
            funcs.extend(functions(&f.rel_path, &f.content));
        }
    }
    if funcs.len() < 2 {
        return Vec::new();
    }
    // LSH: bucket band-signatures → candidate pairs.
    let rows = K / BANDS;
    let mut buckets: HashMap<(usize, u64), Vec<usize>> = HashMap::new();
    for (idx, f) in funcs.iter().enumerate() {
        for b in 0..BANDS {
            let mut h = 0xcbf29ce484222325u64;
            for r in 0..rows {
                h ^= f.sig[b * rows + r];
                h = h.wrapping_mul(0x100000001b3);
            }
            buckets.entry((b, h)).or_default().push(idx);
        }
    }
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut out: Vec<Dupe> = Vec::new();
    for group in buckets.values() {
        if group.len() < 2 {
            continue;
        }
        for a in 0..group.len() {
            for b in a + 1..group.len() {
                let (i, j) = (group[a].min(group[b]), group[a].max(group[b]));
                if !seen.insert((i, j)) {
                    continue;
                }
                let sim = jaccard(&funcs[i].shingles, &funcs[j].shingles);
                if sim >= JACCARD && funcs[i].label != funcs[j].label {
                    out.push(Dupe {
                        a: funcs[i].label.clone(),
                        b: funcs[j].label.clone(),
                        sim,
                    });
                }
            }
        }
    }
    out.sort_by(|x, y| {
        y.sim
            .partial_cmp(&x.sim)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

fn is_code(rel: &str) -> bool {
    let ext = Path::new(rel)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "cs"
            | "php"
    )
}
