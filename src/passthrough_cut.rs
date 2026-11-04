//! Tier-3 passthrough cut: keep the head, the tail and the error lines of the
//! middle, so a failure at the end of a long output survives.

use crate::router::handlers::common_markers::is_error_line;

/// `text` unchanged when it fits in `max` bytes, else head + error lines + tail.
/// Returns the cut text and whether anything was dropped.
pub(crate) fn cut(text: &str, max: usize) -> (String, bool) {
    if text.len() <= max {
        return (text.to_string(), false);
    }
    let lines: Vec<&str> = text.lines().collect();
    let (head_budget, tail_budget, signal_budget) = (max / 2, max / 4, max / 4);

    let mut head_end = 0;
    let mut used = 0;
    while head_end < lines.len() && used + lines[head_end].len() < head_budget {
        used += lines[head_end].len() + 1;
        head_end += 1;
    }
    let mut tail_start = lines.len();
    used = 0;
    while tail_start > head_end && used + lines[tail_start - 1].len() < tail_budget {
        tail_start -= 1;
        used += lines[tail_start].len() + 1;
    }

    let mut signals = Vec::new();
    used = 0;
    for line in &lines[head_end..tail_start] {
        if is_error_line(line) && used + line.len() < signal_budget {
            used += line.len() + 1;
            signals.push(*line);
        }
    }

    if head_end == 0 && tail_start == lines.len() {
        return (cut_bytes(text, max), true);
    }
    let omitted = tail_start - head_end - signals.len();
    let mut out = lines[..head_end].join("\n");
    if !signals.is_empty() {
        out.push_str(&format!(
            "\n[trs: {omitted} lines omitted, error lines kept]\n"
        ));
        out.push_str(&signals.join("\n"));
        out.push_str("\n[trs: ...]\n");
    } else {
        out.push_str(&format!("\n[trs: {omitted} lines omitted]\n"));
    }
    out.push_str(&lines[tail_start..].join("\n"));
    (out, true)
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
        assert_eq!(cut("a\nb\n", 100), ("a\nb\n".to_string(), false));
    }

    #[test]
    fn the_tail_survives() {
        let text = format!("{}FAILED at the very end\n", long(500));
        let (out, cut_any) = cut(&text, 1000);
        assert!(cut_any);
        assert!(out.contains("line 0 of"));
        assert!(out.contains("FAILED at the very end"));
        assert!(out.len() < 1300, "{}", out.len());
    }

    #[test]
    fn an_error_in_the_middle_is_kept() {
        let text = format!("{}error: boom in the middle\n{}", long(300), long(300));
        let (out, _) = cut(&text, 1000);
        assert!(out.contains("error: boom in the middle"));
        assert!(out.contains("error lines kept"));
    }

    #[test]
    fn multibyte_text_does_not_panic() {
        let text = "ñandú ".repeat(2000);
        let (out, _) = cut(&text, 500);
        assert!(out.starts_with("ñandú") && out.contains("bytes omitted"));
    }
}
