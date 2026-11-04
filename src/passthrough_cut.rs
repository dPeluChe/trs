//! Tier-3 passthrough cut: keep the head, the tail and the error lines of the
//! middle, so a failure at the end of a long output survives.

use crate::router::handlers::common_markers::is_error_line;

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

    let mut signals = Vec::new();
    let mut used = 0;
    for line in &lines[head_end..tail_start] {
        if used + line.len() >= max / 4 {
            break;
        }
        if is_error_line(line) {
            used += line.len() + 1;
            signals.push(*line);
        }
    }

    let omitted = tail_start - head_end - signals.len();
    let mut out = lines[..head_end].join("\n");
    if signals.is_empty() {
        out.push_str(&format!("\n[trs: {omitted} lines omitted]\n"));
    } else {
        out.push_str(&format!(
            "\n[trs: {omitted} lines omitted, error lines kept]\n"
        ));
        out.push_str(&signals.join("\n"));
        out.push_str("\n[trs: ...]\n");
    }
    out.push_str(&lines[tail_start..].join("\n"));
    Some(out)
}

/// How many leading lines fit in `budget` bytes (newlines counted).
fn fit<'a>(lines: impl Iterator<Item = &'a str>, budget: usize) -> usize {
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
        assert!(out.contains("error lines kept"));
    }

    #[test]
    fn multibyte_text_does_not_panic() {
        let text = "ñandú ".repeat(2000);
        let out = cut(&text, 500).unwrap();
        assert!(out.starts_with("ñandú") && out.contains("bytes omitted"));
    }
}
