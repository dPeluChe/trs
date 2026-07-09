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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_identifier() {
        assert_eq!(first_ident("  foo_bar(x)").as_deref(), Some("foo_bar"));
        assert_eq!(first_ident("123 abc").as_deref(), Some("abc"));
        assert_eq!(first_ident("_priv = 1").as_deref(), Some("_priv"));
        assert_eq!(first_ident("!!! ---"), None);
        assert_eq!(first_ident(""), None);
    }
}
