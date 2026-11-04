//! Update stamps in always-loaded instruction files ("Updated: ...", "As of
//! 2026-11", "Today is ..."). The agent's prompt cache is keyed on the exact
//! prefix, so rewriting a stamp invalidates the cached prefix from that line
//! on, every time the file is touched. Git already records when a file changed.
//! A date that is never rewritten (a decision's date, a verification date) is
//! not flagged: it costs nothing.

use crate::audit_docs::{fence_open_lang, is_fence_close, DocFile};
use regex::Regex;
use std::sync::LazyLock;

pub(crate) struct VolatileLine {
    pub(crate) path: String,
    pub(crate) line: usize,
    pub(crate) text: String,
}

static STAMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        \b(last\s+updated|updated|as\s+of|actualizad[oa]|última\s+actualización)\b
          \s*:?\s*(\d{4}-\d{2}|[a-z]+\s+\d{4}|\d{1,2}\s+[a-z]+\s+\d{4})
        | \btoday(\x27s\s+date)?\s+is\b",
    )
    .unwrap()
});

pub(crate) fn find_volatile(docs: &[DocFile]) -> Vec<VolatileLine> {
    let mut out = Vec::new();
    for doc in docs {
        let mut in_fence = false;
        for (i, line) in doc.content.lines().enumerate() {
            if in_fence {
                in_fence = !is_fence_close(line);
            } else if fence_open_lang(line).is_some() {
                in_fence = true;
            } else if STAMP.is_match(line) {
                out.push(VolatileLine {
                    path: doc.path.display().to_string(),
                    line: i + 1,
                    text: line.trim().chars().take(80).collect(),
                });
            }
        }
    }
    out
}

pub(crate) fn print_volatile(found: &[VolatileLine]) {
    if found.is_empty() {
        return;
    }
    println!();
    println!(
        "⚠ Update stamps ({}): rewriting one invalidates the cached prompt prefix from that line on; git has the dates",
        found.len()
    );
    for v in found.iter().take(10) {
        println!("  {}:{}  {}", v.path, v.line, v.text);
    }
    if found.len() > 10 {
        println!("  ... +{} more", found.len() - 10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(content: &str) -> DocFile {
        DocFile {
            path: "CLAUDE.md".into(),
            content: content.into(),
            tokens: 0,
            blocks: Vec::new(),
            last_touch_days: None,
        }
    }

    #[test]
    fn stamps_are_found_outside_code_fences_only() {
        let docs = [doc(
            "# Project\n\nUpdated: 2026-11-04\nAs of 2026-11 we use X.\nLast updated November 2026\nToday is Monday.\n\n```\nUpdated: 2026-01-01\n```\n\n~~~\nAs of 2026-02\n~~~\n\nDecided on 2026-12-01, never rewritten.\nNo stamps here, v2026 and 12-34 stay quiet.\n",
        )];
        let lines: Vec<usize> = find_volatile(&docs).iter().map(|v| v.line).collect();
        assert_eq!(lines, [3, 4, 5, 6]);
    }
}
