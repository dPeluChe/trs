//! Small, dependency-free text helpers shared across subsystems.

/// Pick the first identifier-like token from a string: the first run of
/// `[A-Za-z_][A-Za-z0-9_]*`. UTF-8 safe. Used by the ingest symbol indexer and
/// the audit-docs symbol resolver.
pub(crate) fn first_ident(s: &str) -> Option<String> {
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

/// Program name without its directory: `./gradlew` and `/usr/bin/mvn` route
/// like `gradlew` and `mvn`.
pub(crate) fn cmd_basename(cmd: &str) -> &str {
    cmd.rsplit(['/', '\\']).next().unwrap_or(cmd)
}

/// `s` cut to `max_len` bytes with `...`, at a char boundary.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len.saturating_sub(3);
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

/// `s` in at most `max` chars, ending in `…` when cut. Counts chars, not
/// bytes, so it never splits one and fits a padded column.
pub(crate) fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_never_splits_a_char() {
        assert_eq!(truncate_chars("short", 14), "short");
        assert_eq!(truncate_chars("herramientañññ", 14), "herramientañññ");
        assert_eq!(truncate_chars("herramientaññññ", 14), "herramientaññ…");
        assert_eq!(truncate_chars("abc", 0), "…");
    }

    #[test]
    fn truncate_cuts_with_an_ellipsis_at_a_char_boundary() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 3), "hi");
        assert_eq!(truncate("😀😀😀", 8), "😀...");
        assert!(truncate("ñañañañaña", 8).ends_with("..."));
    }

    #[test]
    fn extracts_first_identifier() {
        assert_eq!(first_ident("  foo_bar(x)").as_deref(), Some("foo_bar"));
        assert_eq!(first_ident("123 abc").as_deref(), Some("abc"));
        assert_eq!(first_ident("_priv = 1").as_deref(), Some("_priv"));
        assert_eq!(first_ident("!!! ---"), None);
        assert_eq!(first_ident(""), None);
    }
}
