//! Tier-3 passthrough cut: keep the head, the tail and the error lines of the
//! middle, so a failure at the end of a long output survives. Lines carrying an
//! identifier that appears nowhere else (UUID, hash, URL) come next, within the
//! same budget, so the agent does not rerun a narrower query to find them.

use crate::router::handlers::common_markers::is_error_line;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static NEEDLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b|https?://[^\s"')>]+|\b[0-9a-f]{12,64}\b"#,
    )
    .unwrap()
});

fn needles(line: &str) -> impl Iterator<Item = &str> {
    NEEDLE.find_iter(line).map(|m| m.as_str()).filter(|t| {
        !t.bytes().all(|b| b.is_ascii_hexdigit()) || crate::anchor_guard::looks_like_hash(t)
    })
}

/// `None` when `text` fits in `max` bytes, else head + error lines + tail.
pub(crate) fn cut(text: &str, max: usize) -> Option<String> {
    if text.len() <= max {
        return None;
    }
    let lines: Vec<&str> = text.lines().collect();
    let head_end = fit(lines.iter().copied(), max / 2);
    let tail_len = fit(lines[head_end..].iter().rev().copied(), max / 4);
    let tail_start = lines.len() - tail_len;
    if head_end == 0 && tail_len == 0 {
        return Some(cut_bytes(text, max));
    }

    let middle = &lines[head_end..tail_start];
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for t in lines.iter().flat_map(|l| needles(l)) {
        *seen.entry(t).or_default() += 1;
    }
    let error = |i: &usize| is_error_line(middle[*i]);
    let rare = |i: &usize| needles(middle[*i]).any(|t| seen[t] == 1);
    // Errors first, then needles, both capped by one budget: an id-dense log
    // still shrinks to `max`.
    let candidates = (0..middle.len())
        .filter(error)
        .chain((0..middle.len()).filter(|i| !error(i) && rare(i)));
    let mut used = 0;
    let mut picked: Vec<usize> = candidates
        .take_while(|&i| {
            used += middle[i].len() + 1;
            used < max / 4
        })
        .collect();
    picked.sort_unstable();
    let signals: Vec<&str> = picked.iter().map(|&i| middle[i]).collect();

    let omitted = tail_start - head_end - signals.len();
    let mut out = lines[..head_end].join("\n");
    if signals.is_empty() {
        out.push_str(&format!("\n[trs: {omitted} lines omitted]\n"));
    } else {
        out.push_str(&format!(
            "\n[trs: {omitted} lines omitted, error and unique-id lines kept]\n"
        ));
        out.push_str(&signals.join("\n"));
        out.push_str("\n[trs: ...]\n");
    }
    out.push_str(&lines[tail_start..].join("\n"));
    Some(out)
}

/// How many leading lines fit in `budget` bytes (newlines counted).
pub(crate) fn fit<'a>(lines: impl Iterator<Item = &'a str>, budget: usize) -> usize {
    let mut used = 0;
    lines
        .take_while(|l| {
            used += l.len() + 1;
            used <= budget
        })
        .count()
}

/// One giant line (minified output): keep both ends at char boundaries.
fn cut_bytes(text: &str, max: usize) -> String {
    let mut head = max * 3 / 4;
    while !text.is_char_boundary(head) {
        head -= 1;
    }
    let mut tail = text.len() - max / 4;
    while !text.is_char_boundary(tail) {
        tail += 1;
    }
    let omitted = tail - head;
    format!(
        "{}\n[trs: {omitted} bytes omitted]\n{}",
        &text[..head],
        &text[tail..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long(n: usize) -> String {
        (0..n)
            .map(|i| format!("line {i} of plain output\n"))
            .collect()
    }

    #[test]
    fn short_text_is_untouched() {
        assert_eq!(cut("a\nb\n", 100), None);
    }

    #[test]
    fn the_tail_survives() {
        let text = format!("{}FAILED at the very end\n", long(500));
        let out = cut(&text, 1000).unwrap();
        assert!(out.contains("line 0 of"));
        assert!(out.contains("FAILED at the very end"));
        assert!(out.len() < 1300, "{}", out.len());
    }

    #[test]
    fn an_error_in_the_middle_is_kept() {
        let text = format!("{}error: boom in the middle\n{}", long(300), long(300));
        let out = cut(&text, 1000).unwrap();
        assert!(out.contains("error: boom in the middle"));
        assert!(out.contains("lines kept"));
    }

    #[test]
    fn a_unique_id_in_the_middle_is_kept_and_a_repeated_one_is_not() {
        let uuid = "3f2b8c1e-9d4a-4e6b-8a1f-5c7d2e9b0a14";
        let text = format!(
            "{}saved order {uuid} ok\n{}\
             seen https://example.com/a/very/specific/path\n{}",
            long(300),
            long(100),
            long(300)
        );
        let out = cut(&text, 1000).unwrap();
        assert!(out.contains(uuid));
        assert!(out.contains("https://example.com/a/very/specific/path"));

        let repeated: String = (0..600)
            .map(|i| format!("worker {i} handled 5d41402abc4b2a76b9719d911017c592\n"))
            .collect();
        let out = cut(&repeated, 1000).unwrap();
        assert!(!out.contains("worker 300 "), "a repeated id is no needle");
    }

    #[test]
    fn an_id_dense_log_still_shrinks_to_the_budget() {
        let dense: String = (0..2000)
            .map(|i| format!("req {i:012x}{i:04} done in 3ms\n"))
            .collect();
        let out = cut(&dense, 2000).unwrap();
        assert!(out.len() <= 2300, "{}", out.len());
    }

    #[test]
    fn multibyte_text_does_not_panic() {
        let text = "ñandú ".repeat(2000);
        let out = cut(&text, 500).unwrap();
        assert!(out.starts_with("ñandú") && out.contains("bytes omitted"));
    }
}
