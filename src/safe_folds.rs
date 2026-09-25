//! Transforms that are safe on output of any shape, because they either lose
//! nothing (checked by undoing them) or keep what they cut in the tee file.
//!
//! - `fold_repeats`: consecutive identical lines become `line (xN)`, and a
//!   block of 2-4 lines repeated back to back is kept once plus
//!   `(last K lines xN)`. Undone and compared before it is trusted.
//! - `fold_timestamped`: log lines equal but for their timestamps keep the
//!   first line plus `(xN, until <last time>)`; the raw is kept.
//! - `elide_dense`: a 300+ char line with almost no spaces (minified code,
//!   base64, a data URI) keeps its head and tail around a label saying what
//!   was cut. The caller marks the output as dropped so the raw is saved.

/// `line (xN)` and `(last K lines xN)` are the only markers; input already
/// containing either is left alone, so a marker is never ambiguous.
fn has_marker(line: &str) -> bool {
    let Some(inner) = line.trim_end().strip_suffix(')') else {
        return false;
    };
    let Some(open) = inner.rfind('(') else {
        return false;
    };
    let body = &inner[open + 1..];
    let count = body.strip_prefix('x').or_else(|| {
        body.strip_prefix("last ")
            .and_then(|b| b.split_once(" lines x"))
            .map(|(k, n)| {
                if k.chars().all(|c| c.is_ascii_digit()) {
                    n
                } else {
                    ""
                }
            })
    });
    count.is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Fold repeats, or return the input unchanged when folding would be
/// ambiguous, would not shrink it, or does not undo to the exact original.
pub(crate) fn fold_repeats(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() < 2 || lines.iter().any(|l| has_marker(l)) {
        return text.to_string();
    }
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    'outer: while i < lines.len() {
        let line = lines[i];
        // A block of k lines repeated right after itself.
        for k in (2..=4).rev() {
            if i + 2 * k > lines.len() {
                continue;
            }
            let block = &lines[i..i + k];
            if block.iter().all(|l| l.trim().is_empty()) || block.iter().all(|l| *l == block[0]) {
                continue;
            }
            let mut n = 1;
            while i + (n + 1) * k <= lines.len() && lines[i + n * k..i + (n + 1) * k] == *block {
                n += 1;
            }
            if n > 1 {
                out.extend(block.iter().map(|l| l.to_string()));
                out.push(format!("(last {} lines x{})", k, n));
                i += n * k;
                continue 'outer;
            }
        }
        let mut n = 1;
        while i + n < lines.len() && lines[i + n] == line {
            n += 1;
        }
        // `line (x2)` only pays when the line outweighs the marker.
        if n > 1 && !line.trim().is_empty() && (n > 2 || line.len() > 8) {
            out.push(format!("{} (x{})", line, n));
        } else {
            out.extend(std::iter::repeat_n(line.to_string(), n));
        }
        i += n;
    }
    let folded = out.join("\n");
    if folded.len() < text.len() && unfold(&folded) == text {
        folded
    } else {
        text.to_string()
    }
}

/// Exact inverse of `fold_repeats`, used to check it before trusting it.
fn unfold(folded: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in folded.split('\n') {
        if let Some(rest) = line
            .strip_prefix("(last ")
            .and_then(|r| r.strip_suffix(')'))
        {
            if let Some((k, n)) = rest.split_once(" lines x") {
                if let (Ok(k), Ok(n)) = (k.parse::<usize>(), n.parse::<usize>()) {
                    let block: Vec<&str> = out[out.len().saturating_sub(k)..].to_vec();
                    for _ in 1..n {
                        out.extend(block.iter().copied());
                    }
                    continue;
                }
            }
        }
        if let Some((l, n)) = line.rsplit_once(" (x").and_then(|(l, r)| {
            r.strip_suffix(')')
                .and_then(|n| n.parse::<usize>().ok())
                .map(|n| (l, n))
        }) {
            out.extend(std::iter::repeat_n(l, n));
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn timestamp_re() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(
            r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:Z|[+-]\d{2}:?\d{2})?|\b\d{2}:\d{2}:\d{2}(?:[.,]\d+)?\b",
        )
        .expect("static timestamp pattern")
    })
}

/// Consecutive log lines equal except for their timestamps become the first
/// line plus `(xN, until <last timestamp>)`. Only times are compared as
/// equal: spacing, symbols and numbers still have to match, since in code
/// indentation and line numbers are content. The times in between are
/// dropped, so the caller keeps the raw output.
pub(crate) fn fold_timestamped(text: &str) -> (String, bool) {
    if text.contains(", until ") {
        return (text.to_string(), false);
    }
    let re = timestamp_re();
    let lines: Vec<&str> = text.split('\n').collect();
    let keys: Vec<Option<String>> = lines
        .iter()
        .map(|l| {
            re.is_match(l)
                .then(|| re.replace_all(l, "\u{0}").into_owned())
        })
        .collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut cut = false;
    let mut i = 0;
    while i < lines.len() {
        let mut j = i;
        while keys[i].is_some() && j + 1 < lines.len() && keys[j + 1] == keys[i] {
            j += 1;
        }
        // Identical lines are fold_repeats' job, and stay lossless there.
        if j > i && lines[i..=j].iter().any(|l| *l != lines[i]) {
            let until = re.find(lines[j]).map_or("", |m| m.as_str());
            out.push(format!("{} (x{}, until {})", lines[i], j - i + 1, until));
            cut = true;
        } else {
            out.extend(lines[i..=j].iter().map(|l| l.to_string()));
        }
        i = j + 1;
    }
    (out.join("\n"), cut)
}

const DENSE_MIN: usize = 300;
const HEAD: usize = 120;
const TAIL: usize = 60;

/// Kind label for a dense line, or `None` if the line is not dense: under 300
/// chars, 6%+ spaces, or tab-separated (a table row, not a blob).
fn dense_kind(line: &str) -> Option<String> {
    let len = line.len();
    if len < DENSE_MIN || line.contains('\t') {
        return None;
    }
    if line.bytes().filter(|b| *b == b' ').count() * 100 >= len * 6 {
        return None;
    }
    if let Some(start) = line.find("data:") {
        if let Some(end) = line[start..].find(";base64,") {
            return Some(format!("data URI {}", &line[start + 5..start + end]));
        }
    }
    let t = line.trim();
    if t.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"+/=_-".contains(&b))
    {
        return Some("base64".into());
    }
    Some("minified".into())
}

/// Head and tail of each dense line around `…[kind, N chars]…`. Returns the
/// text and whether anything was cut (the caller then keeps the raw output).
pub(crate) fn elide_dense(text: &str) -> (String, bool) {
    let mut cut = false;
    let out: Vec<String> = text
        .split('\n')
        .map(|line| match dense_kind(line) {
            Some(kind) => {
                let head_end = (0..=HEAD)
                    .rev()
                    .find(|i| line.is_char_boundary(*i))
                    .unwrap_or(0);
                let tail_start = (line.len() - TAIL..line.len())
                    .find(|i| line.is_char_boundary(*i))
                    .unwrap_or(line.len());
                cut = true;
                format!(
                    "{}…[{}, {} chars]…{}",
                    &line[..head_end],
                    kind,
                    line.len(),
                    &line[tail_start..]
                )
            }
            None => line.to_string(),
        })
        .collect();
    (out.join("\n"), cut)
}

#[cfg(test)]
#[path = "safe_folds_tests.rs"]
mod tests;
