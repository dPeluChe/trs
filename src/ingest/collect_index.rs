//! Directory annotations and symbol-index extraction.
//!
//! These helpers run during read_and_compress and attach metadata to
//! each DigestFile. The ingest formatter consumes the metadata later to
//! render the Structure tree and the optional Symbols section.

/// Files whose module-level doc we promote to a directory annotation.
pub(super) const MODULE_ANCHOR_NAMES: &[&str] = &[
    "mod.rs",
    "lib.rs",
    "__init__.py",
    "index.ts",
    "index.js",
    "index.tsx",
    "index.jsx",
];

/// True for filenames whose module-level doc we promote to a directory
/// annotation. Lets the caller skip `extract_module_doc` for the ~99% of
/// files that will never contribute one.
pub(super) fn is_module_anchor(lower_name: &str) -> bool {
    MODULE_ANCHOR_NAMES.contains(&lower_name)
}

/// Extract the module-level docstring from an anchor file (mod.rs / lib.rs /
/// __init__.py / index.*). Caller should gate this with `is_module_anchor`.
pub(super) fn extract_module_doc(content: &str, ext: &str) -> Option<String> {
    let first = match ext {
        "rs" => first_rust_module_doc(content),
        "py" => first_python_docstring(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" => first_jsdoc_summary(content),
        _ => None,
    }?;

    let mut line = first.replace('\n', " ");
    line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        return None;
    }
    let line = line.trim_end_matches('.').trim().to_string();

    // Reject config-file leaks like `description: ...` or `key = value` that
    // sometimes appear at the top of a hand-rolled lib.rs.
    if looks_like_config_line(&line) || contains_manifest_field(&line) {
        return None;
    }

    Some(crate::formatter::helpers::truncate(&line, 120))
}

/// Cargo.toml / package.json field names we treat as leakage if they appear
/// anywhere in an accumulated module docstring. Module docs don't legitimately
/// contain ` description: ` prose; when they do, it's almost always because
/// someone embedded manifest metadata in a lib.rs top comment.
const MANIFEST_FIELD_MARKERS: &[&str] = &[
    " description:",
    " version:",
    " authors:",
    " license:",
    " edition:",
    " repository:",
    " homepage:",
    " keywords:",
    " categories:",
];

fn contains_manifest_field(s: &str) -> bool {
    // Prefix with space so "description:" at the start also matches.
    let haystack = format!(" {}", s.to_ascii_lowercase());
    MANIFEST_FIELD_MARKERS.iter().any(|m| haystack.contains(m))
}

/// Detect `key: value` / `key = value` patterns where `key` is a short
/// identifier. These leak into annotations when a lib.rs embeds a metadata
/// line from Cargo.toml or similar.
fn looks_like_config_line(s: &str) -> bool {
    let first_word_end = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
        .map(|(i, _)| i);
    let Some(end) = first_word_end else {
        return false;
    };
    if end == 0 || end > 24 {
        return false;
    }
    let rest = s[end..].trim_start();
    // Accept only if the separator is `:` or `=` right after the token.
    rest.starts_with(": ") || rest.starts_with("= ") || rest.starts_with(":= ")
}

fn first_rust_module_doc(content: &str) -> Option<String> {
    let mut out = String::new();
    for raw in content.lines() {
        let t = raw.trim_start();
        if let Some(rest) = t.strip_prefix("//!") {
            let body = rest.trim();
            // Skip markdown heading lines (# Title, ## Section) — they're a
            // project name, not the prose we want for the annotation.
            if body.starts_with('#') {
                continue;
            }
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(body);
            continue;
        }
        // Ignore blank lines between doc comments, but stop on code.
        if t.is_empty() || t.starts_with("//") {
            if !out.is_empty() {
                break;
            }
            continue;
        }
        break;
    }
    if out.is_empty() {
        None
    } else {
        // Keep only the first sentence for the directory annotation.
        Some(out.split('.').next().unwrap_or("").trim().to_string())
    }
}

fn first_python_docstring(content: &str) -> Option<String> {
    let mut lines = content.lines().peekable();
    // Skip shebang and blanks at the top.
    while let Some(&next) = lines.peek() {
        let t = next.trim();
        if t.is_empty() || t.starts_with("#!") {
            lines.next();
        } else {
            break;
        }
    }
    let first = lines.next()?.trim();
    let (quote, rest) = if let Some(r) = first.strip_prefix("\"\"\"") {
        ("\"\"\"", r)
    } else if let Some(r) = first.strip_prefix("'''") {
        ("'''", r)
    } else {
        return None;
    };
    // Single-line docstring.
    if let Some(end) = rest.find(quote) {
        return Some(rest[..end].trim().to_string());
    }
    // Multi-line: accumulate until the closing triple-quote.
    let mut buf = rest.to_string();
    for line in lines {
        if let Some(end) = line.find(quote) {
            if !buf.is_empty() {
                buf.push(' ');
            }
            buf.push_str(line[..end].trim());
            break;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(line.trim());
    }
    if buf.trim().is_empty() {
        None
    } else {
        Some(buf.split('.').next().unwrap_or("").trim().to_string())
    }
}

fn first_jsdoc_summary(content: &str) -> Option<String> {
    // Match the first /** ... */ block at the top of the file.
    let trimmed = content.trim_start();
    if !trimmed.starts_with("/**") {
        return None;
    }
    let end_idx = trimmed.find("*/")?;
    let inner = &trimmed[3..end_idx];
    let summary: String = inner
        .lines()
        .map(|l| l.trim().trim_start_matches('*').trim())
        .filter(|l| !l.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    if summary.is_empty() {
        None
    } else {
        Some(summary.split('.').next().unwrap_or("").trim().to_string())
    }
}

/// Generic impl-method names that add no information to a symbol index.
const SKIP_TRIVIAL_NAMES: &[&str] = &[
    "new", "default", "from", "into", "try_from", "clone", "drop",
];

/// Extract public / exported symbol names from a source file.
/// Uses the language's own "public" convention — no heuristics on naming.
pub(super) fn extract_symbols(content: &str, ext: &str) -> Vec<String> {
    let extractor: fn(&str) -> Option<String> = match ext {
        "rs" => symbol_from_rust,
        "py" | "pyi" => symbol_from_python,
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" => symbol_from_ts,
        "go" => symbol_from_go,
        "swift" => symbol_from_swift,
        "java" | "kt" | "kts" => symbol_from_java,
        _ => return Vec::new(),
    };

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut syms: Vec<String> = Vec::new();
    for raw in content.lines() {
        let t = raw.trim_start();
        // Cheap filter — blank, comment, or docstring lines can never be a sig.
        if t.is_empty()
            || t.starts_with("//")
            || t.starts_with('#')
            || t.starts_with('*')
            || t.starts_with("/*")
        {
            continue;
        }
        if let Some(name) = extractor(t) {
            if SKIP_TRIVIAL_NAMES.contains(&name.as_str()) {
                continue;
            }
            if seen.insert(name.clone()) {
                syms.push(name);
            }
        }
    }
    syms
}

fn symbol_from_rust(line: &str) -> Option<String> {
    // pub fn foo / pub struct Foo / pub enum / pub trait / pub type / pub const / pub static,
    // plus visibility-qualified forms `pub(crate)` / `pub(super)` / `pub(in …)`.
    // A binary crate's real API surface is `pub(crate)`, so restricting to a
    // bare `pub ` prefix drops nearly everything (e.g. `execute_and_parse`).
    let rest = line.strip_prefix("pub ").or_else(|| {
        let after = line.strip_prefix("pub(")?;
        let close = after.find(')')?;
        after[close + 1..].strip_prefix(' ')
    })?;
    let rest = rest
        .strip_prefix("async ")
        .or_else(|| rest.strip_prefix("unsafe "))
        .unwrap_or(rest);
    for kw in &[
        "fn ", "struct ", "enum ", "trait ", "type ", "const ", "static ", "mod ",
    ] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn symbol_from_python(line: &str) -> Option<String> {
    // Top-level def / class only; underscore-prefix names are private by convention.
    if !line.starts_with("def ") && !line.starts_with("async def ") && !line.starts_with("class ") {
        return None;
    }
    let body = line
        .strip_prefix("async def ")
        .or_else(|| line.strip_prefix("def "))
        .or_else(|| line.strip_prefix("class "))?;
    let name = first_ident(body)?;
    if name.starts_with('_') {
        return None;
    }
    Some(name)
}

fn symbol_from_ts(line: &str) -> Option<String> {
    // export class / function / const / let / type / interface / enum / default <name>
    let rest = line.strip_prefix("export ")?;
    let rest = rest.strip_prefix("default ").unwrap_or(rest);
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    for kw in &[
        "class ",
        "function ",
        "const ",
        "let ",
        "type ",
        "interface ",
        "enum ",
    ] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn symbol_from_go(line: &str) -> Option<String> {
    // func, type, var, const with an uppercase-first identifier (exported).
    for kw in &["func ", "type ", "var ", "const "] {
        if let Some(body) = line.strip_prefix(*kw) {
            let body = body.trim_start_matches('(');
            let name = first_ident(body)?;
            if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                return Some(name);
            }
            return None;
        }
    }
    None
}

fn symbol_from_swift(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("public ")
        .or_else(|| line.strip_prefix("open "))?;
    for kw in &[
        "func ",
        "class ",
        "struct ",
        "enum ",
        "protocol ",
        "extension ",
    ] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn symbol_from_java(line: &str) -> Option<String> {
    let rest = line.strip_prefix("public ")?;
    let rest = rest.strip_prefix("static ").unwrap_or(rest);
    let rest = rest.strip_prefix("final ").unwrap_or(rest);
    for kw in &["class ", "interface ", "enum ", "fun "] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

/// Pick the first identifier-like token from a string.
fn first_ident(s: &str) -> Option<String> {
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
    use super::symbol_from_rust;

    #[test]
    fn captures_visibility_qualified_items() {
        // Regression: the digest dropped every `pub(crate)` symbol because the
        // extractor only stripped a bare `pub ` prefix — ~500 real symbols
        // (incl. `execute_and_parse`) never made the index.
        assert_eq!(
            symbol_from_rust("pub(crate) fn execute_and_parse(cmd: &str) {").as_deref(),
            Some("execute_and_parse")
        );
        assert_eq!(
            symbol_from_rust("pub(super) struct Foo {").as_deref(),
            Some("Foo")
        );
        assert_eq!(
            symbol_from_rust("pub(in crate::ingest) fn helper() {").as_deref(),
            Some("helper")
        );
        // Plain `pub` still works.
        assert_eq!(symbol_from_rust("pub fn bar() {").as_deref(), Some("bar"));
        // Private items and non-defs are still skipped (index = API surface).
        assert_eq!(symbol_from_rust("fn private_helper() {"), None);
        assert_eq!(symbol_from_rust("let x = 5;"), None);
    }
}
