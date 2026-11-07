//! The edit `replace` makes to one file's text. Line endings are kept as
//! written: a CRLF file stays CRLF.

use regex::Regex;

use super::common::needs_newline;

#[derive(Debug, Clone)]
pub(crate) struct Replacement {
    pub(crate) line_number: usize,
    pub(crate) original: String,
    pub(crate) replaced: String,
    /// Number of individual matches replaced in this line.
    pub(crate) match_count: usize,
}

/// The search pattern, compiled once. A pattern that needs a newline
/// (`\n`, `\x0A`) matches across lines; any other matches within one line,
/// so `[^\n]` and `\s` never reach a line ending.
pub(crate) struct Pattern {
    re: Regex,
    bytes: regex::bytes::Regex,
    multiline: bool,
}

impl Pattern {
    pub(crate) fn new(pattern: &str) -> Result<Self, regex::Error> {
        let multiline = needs_newline(pattern);
        let re = regex::RegexBuilder::new(pattern)
            .multi_line(multiline)
            .crlf(multiline)
            .build()?;
        let bytes = regex::bytes::RegexBuilder::new(pattern)
            .multi_line(true)
            .crlf(true)
            .build()?;
        Ok(Self {
            re,
            bytes,
            multiline,
        })
    }

    /// Whether bytes that are not text hold a match.
    pub(crate) fn matches_bytes(&self, haystack: &[u8]) -> bool {
        self.bytes.is_match(haystack)
    }

    /// `content` with every match replaced, and what changed. None when
    /// nothing matched.
    pub(crate) fn apply(&self, content: &str, with: &str) -> Option<(String, Vec<Replacement>)> {
        let (new, reps) = if self.multiline {
            self.across_lines(content, with)
        } else {
            self.per_line(content, with)
        };
        (!reps.is_empty()).then_some((new, reps))
    }

    fn per_line(&self, content: &str, with: &str) -> (String, Vec<Replacement>) {
        let mut out = String::with_capacity(content.len());
        let mut reps = Vec::new();
        for (i, piece) in content.split_inclusive('\n').enumerate() {
            let body = without_ending(piece);
            let mut match_count = 0;
            let replaced = self.re.replace_all(body, |caps: &regex::Captures| {
                match_count += 1;
                let mut expanded = String::new();
                caps.expand(with, &mut expanded);
                expanded
            });
            if match_count == 0 {
                out.push_str(piece);
                continue;
            }
            out.push_str(&replaced);
            out.push_str(&piece[body.len()..]);
            reps.push(Replacement {
                line_number: i + 1,
                original: body.to_string(),
                replaced: replaced.into_owned(),
                match_count,
            });
        }
        (out, reps)
    }

    fn across_lines(&self, content: &str, with: &str) -> (String, Vec<Replacement>) {
        let mut out = String::with_capacity(content.len());
        let mut reps = Vec::new();
        let (mut line, mut copied) = (1, 0);
        for caps in self.re.captures_iter(content) {
            let m = caps.get(0).unwrap();
            line += content[copied..m.start()].matches('\n').count();
            let start = content[..m.start()].rfind('\n').map_or(0, |i| i + 1);
            let end = if m.as_str().ends_with('\n') {
                m.end()
            } else {
                content[m.end()..]
                    .find('\n')
                    .map_or(content.len(), |i| m.end() + i + 1)
            };
            let mut expanded = String::new();
            caps.expand(with, &mut expanded);
            out.push_str(&content[copied..m.start()]);
            out.push_str(&expanded);
            let replaced = format!(
                "{}{expanded}{}",
                &content[start..m.start()],
                &content[m.end()..end]
            );
            reps.push(Replacement {
                line_number: line,
                original: without_ending(&content[start..end]).to_string(),
                replaced: without_ending(&replaced).to_string(),
                match_count: 1,
            });
            line += m.as_str().matches('\n').count();
            copied = m.end();
        }
        out.push_str(&content[copied..]);
        (out, reps)
    }
}

fn without_ending(line: &str) -> &str {
    line.strip_suffix('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(pattern: &str, content: &str, with: &str) -> Option<(String, Vec<Replacement>)> {
        Pattern::new(pattern).unwrap().apply(content, with)
    }

    #[test]
    fn crlf_and_a_missing_final_newline_are_kept() {
        let (new, reps) = apply("foo", "foo a\r\nbar\r\nfoo", "baz").unwrap();
        assert_eq!(new, "baz a\r\nbar\r\nbaz");
        assert_eq!(reps.len(), 2);
        assert_eq!(reps[1].line_number, 3);
    }

    #[test]
    fn counts_every_match_on_a_line() {
        let (new, reps) = apply(r"o", "foo\n", "0").unwrap();
        assert_eq!(new, "f00\n");
        assert_eq!(reps[0].match_count, 2);
        assert_eq!(reps[0].original, "foo");
    }

    #[test]
    fn a_pattern_with_a_newline_matches_across_lines() {
        let (new, reps) = apply(r"a\nb", "x\na\nb\ny\n", "AB").unwrap();
        assert_eq!(new, "x\nAB\ny\n");
        assert_eq!(reps[0].line_number, 2);
        assert_eq!(reps[0].original, "a\nb");
        assert_eq!(reps[0].replaced, "AB");
    }

    #[test]
    fn anchors_still_mean_line_start_and_end_across_lines() {
        assert_eq!(apply(r"^a$\n^b$", "a\nb\n", "c").unwrap().0, "c\n");
        assert_eq!(apply(r"a\r\nb", "a\r\nb\r\n", "c").unwrap().0, "c\r\n");
    }

    #[test]
    fn groups_expand_in_both_modes() {
        assert_eq!(apply(r"(\w+)=1", "k=1\n", "$1=2").unwrap().0, "k=2\n");
        assert_eq!(apply(r"(\w+)\n=", "k\n=1\n", "$1 =").unwrap().0, "k =1\n");
    }

    #[test]
    fn only_a_pattern_that_needs_a_newline_crosses_lines() {
        assert!(needs_newline(r"a\nb"));
        assert!(needs_newline(r"a\x0Ab"));
        assert!(!needs_newline(r"a\\n"));
        assert!(!needs_newline(r"foo[^\n]*"));
        assert!(!needs_newline(r"a\s+b"));
        assert!(apply(r"\\n", "a\nb\n", "x").is_none());
    }

    #[test]
    fn rest_of_line_idiom_keeps_crlf() {
        let (new, _) = apply(r"foo[^\n]*", "foo a\r\nbar\r\n", "x").unwrap();
        assert_eq!(new, "x\r\nbar\r\n");
    }

    #[test]
    fn several_matches_across_lines_keep_their_line_numbers() {
        let (new, reps) = apply(r"a\nb", "a\nb\nc\na\nb\n", "X").unwrap();
        assert_eq!(new, "X\nc\nX\n");
        assert_eq!(reps[0].line_number, 1);
        assert_eq!(reps[1].line_number, 4);
    }
}
