//! Filter functions for the read command.
//!
//! Contains language detection, comment stripping, and aggressive filtering
//! logic used by the read handler.

use std::path::PathBuf;

/// Detected language for comment stripping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    Rust,
    Python,
    JavaScript, // also TypeScript
    Go,
    Java, // also C, C++, C#, Kotlin, Swift
    Ruby,
    Shell, // also Bash, Zsh
    Data,  // JSON, YAML, TOML, XML, CSV — never filter
    Unknown,
}

/// Detect language from file extension.
pub(crate) fn detect_language(path: &PathBuf) -> Language {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        // Data formats — NEVER filter
        "json" | "jsonl" | "yaml" | "yml" | "toml" | "xml" | "csv" | "tsv" | "lock" | "svg"
        | "html" | "htm" => Language::Data,

        // Rust
        "rs" => Language::Rust,
        // Python
        "py" | "pyi" | "pyw" => Language::Python,
        // JavaScript/TypeScript
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts" | "vue" | "svelte" => {
            Language::JavaScript
        }
        // Go
        "go" => Language::Go,
        // Java-like (C-style comments)
        "java" | "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "cs" | "kt" | "kts" | "swift"
        | "scala" | "groovy" | "dart" | "m" | "mm" => Language::Java,
        // Ruby
        "rb" | "rake" | "gemspec" => Language::Ruby,
        // Shell
        "sh" | "bash" | "zsh" | "fish" | "ksh" => Language::Shell,

        _ => Language::Unknown,
    }
}

/// Returns true if this line is a single-line comment for the given language.
fn is_comment_line(line: &str, lang: Language) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Preserve important comments
    let important = [
        "TODO",
        "FIXME",
        "HACK",
        "XXX",
        "SAFETY",
        "IMPORTANT",
        "type:ignore",
        "noqa",
        "pragma",
        "eslint-disable",
        "ts-ignore",
        "@ts-expect-error",
        "nolint",
        "allow(",
        "deny(",
    ];
    if important.iter().any(|kw| trimmed.contains(kw)) {
        return false;
    }

    match lang {
        Language::Rust | Language::Go | Language::Java | Language::JavaScript => {
            trimmed.starts_with("//")
        }
        Language::Python | Language::Ruby | Language::Shell => {
            trimmed.starts_with('#') && !trimmed.starts_with("#!") // preserve shebangs
        }
        _ => false,
    }
}

/// Minimal filter: strip comments, normalize blank lines (max 1 consecutive).
pub(crate) fn filter_minimal(content: &str, lang: Language) -> String {
    let mut result = Vec::new();
    let mut in_block_comment = false;
    let mut prev_blank = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track block comments (/* ... */)
        if !in_block_comment && (trimmed.starts_with("/*") || trimmed.starts_with("/**")) {
            match lang {
                Language::Rust | Language::Go | Language::Java | Language::JavaScript => {
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                    continue;
                }
                _ => {}
            }
        }
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Python docstrings (triple quotes) — skip multi-line
        if matches!(lang, Language::Python)
            && (trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''"))
        {
            let quote = &trimmed[..3];
            // Single-line docstring: """text""" — keep
            if trimmed.len() > 3 && trimmed[3..].contains(quote) {
                continue;
            }
            // Multi-line docstring start — skip until close
            // We handle this simply by skipping the line
            continue;
        }

        // Skip single-line comments
        if is_comment_line(line, lang) {
            continue;
        }

        // Normalize blank lines (max 1 consecutive)
        let is_blank = trimmed.is_empty();
        if is_blank && prev_blank {
            continue;
        }
        prev_blank = is_blank;

        result.push(line);
    }

    // Trim trailing blank lines
    while result.last().map_or(false, |l| l.trim().is_empty()) {
        result.pop();
    }

    result.join("\n")
}

/// Aggressive filter: keep only imports, definitions, and signatures.
pub(crate) fn filter_aggressive(content: &str, lang: Language) -> String {
    let mut result = Vec::new();
    let mut in_block_comment = false;
    let mut brace_depth: i32 = 0;
    let mut in_body = false;
    let mut body_start_depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip block comments
        if !in_block_comment && (trimmed.starts_with("/*") || trimmed.starts_with("/**")) {
            if matches!(
                lang,
                Language::Rust | Language::Go | Language::Java | Language::JavaScript
            ) {
                if !trimmed.contains("*/") {
                    in_block_comment = true;
                }
                continue;
            }
        }
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Skip comments
        if is_comment_line(line, lang) {
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // Always keep: imports, use, require, include, from...import
        if is_import_line(trimmed, lang) {
            result.push(line);
            continue;
        }

        // Always keep: decorators / attributes
        if is_decorator(trimmed, lang) {
            result.push(line);
            continue;
        }

        // Detect definition lines (functions, structs, classes, traits, enums)
        if is_definition_line(trimmed, lang) {
            result.push(line);
            // If the line opens a brace block, track it to skip the body
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_body = true;
                body_start_depth = brace_depth;
                brace_depth += count_braces(trimmed);
            } else if matches!(lang, Language::Python) && trimmed.ends_with(':') {
                // Python: definition followed by indented body
                in_body = true;
                body_start_depth = 0; // use indentation tracking
            }
            continue;
        }

        // Track brace depth for body skipping
        if in_body
            && matches!(
                lang,
                Language::Rust | Language::Go | Language::Java | Language::JavaScript
            )
        {
            brace_depth += count_braces(trimmed);
            if brace_depth <= body_start_depth {
                in_body = false;
                // Keep closing brace
                if trimmed == "}" || trimmed == "};" {
                    result.push(line);
                }
            }
            continue;
        }

        // Python body skipping: skip indented lines after definition
        if in_body && matches!(lang, Language::Python) {
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                // Back to top level — not part of body anymore
                in_body = false;
                // Fall through to check if this line should be kept
            } else {
                continue; // skip indented body line
            }
        }

        // Keep type definitions, constants, statics
        if is_type_or_const(trimmed, lang) {
            result.push(line);
            continue;
        }

        // Skip everything else when in aggressive mode
        // (variables, expressions, standalone calls, etc.)
    }

    result.join("\n")
}

/// Check if line is an import/use/require statement.
fn is_import_line(trimmed: &str, lang: Language) -> bool {
    match lang {
        Language::Rust => {
            trimmed.starts_with("use ")
                || trimmed.starts_with("pub use ")
                || trimmed.starts_with("extern crate ")
                || trimmed.starts_with("mod ")
                || trimmed.starts_with("pub mod ")
                || trimmed.starts_with("pub(crate) mod ")
        }
        Language::Python => trimmed.starts_with("import ") || trimmed.starts_with("from "),
        Language::JavaScript => {
            trimmed.starts_with("import ")
                || trimmed.starts_with("export ")
                || trimmed.contains("require(")
                || trimmed.contains("require('")
                || trimmed.contains("require(\"")
        }
        Language::Go => trimmed.starts_with("import ") || trimmed == "import (",
        Language::Java => trimmed.starts_with("import ") || trimmed.starts_with("package "),
        Language::Ruby => {
            trimmed.starts_with("require ")
                || trimmed.starts_with("require_relative ")
                || trimmed.starts_with("include ")
                || trimmed.starts_with("extend ")
        }
        Language::Shell => trimmed.starts_with("source ") || trimmed.starts_with(". "),
        _ => false,
    }
}

/// Check if line is a decorator or attribute.
fn is_decorator(trimmed: &str, lang: Language) -> bool {
    match lang {
        Language::Python => trimmed.starts_with('@'),
        Language::Rust => trimmed.starts_with("#[") || trimmed.starts_with("#!["),
        Language::Java => trimmed.starts_with('@'),
        Language::JavaScript => trimmed.starts_with('@'),
        _ => false,
    }
}

/// Check if line is a function/class/struct/trait/enum definition.
fn is_definition_line(trimmed: &str, lang: Language) -> bool {
    match lang {
        Language::Rust => {
            trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("impl<")
                || trimmed.starts_with("pub impl ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("pub type ")
                || trimmed.starts_with("macro_rules!")
        }
        Language::Python => {
            trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("class ")
        }
        Language::JavaScript => {
            trimmed.starts_with("function ")
                || trimmed.starts_with("async function ")
                || trimmed.starts_with("class ")
                || trimmed.contains("=> {")
                || (trimmed.contains("(")
                    && trimmed.contains(") {")
                    && !trimmed.starts_with("if ")
                    && !trimmed.starts_with("for ")
                    && !trimmed.starts_with("while "))
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("export default function ")
                || trimmed.starts_with("export class ")
                || trimmed.starts_with("export default class ")
                || trimmed.starts_with("interface ")
                || trimmed.starts_with("export interface ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("export type ")
        }
        Language::Go => {
            trimmed.starts_with("func ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("interface ")
        }
        Language::Java => {
            // public void method() { ... }
            let has_access = trimmed.starts_with("public ")
                || trimmed.starts_with("private ")
                || trimmed.starts_with("protected ")
                || trimmed.starts_with("static ")
                || trimmed.starts_with("abstract ")
                || trimmed.starts_with("final ");
            let has_def = trimmed.contains("class ")
                || trimmed.contains("interface ")
                || trimmed.contains("enum ")
                || (trimmed.contains("(")
                    && (trimmed.contains(") {") || trimmed.contains(") throws")));
            has_access && has_def
        }
        Language::Ruby => {
            trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
        }
        Language::Shell => {
            // function name() { or name() {
            trimmed.starts_with("function ") || (trimmed.contains("()") && trimmed.contains('{'))
        }
        _ => false,
    }
}

/// Check if line is a type alias, constant, or static declaration.
fn is_type_or_const(trimmed: &str, lang: Language) -> bool {
    match lang {
        Language::Rust => {
            trimmed.starts_with("const ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("static ")
                || trimmed.starts_with("pub static ")
        }
        Language::Python => {
            // ALL_CAPS = ... pattern (constants)
            trimmed.chars().next().map_or(false, |c| c.is_uppercase())
                && trimmed.contains(" = ")
                && trimmed.split('=').next().map_or(false, |name| {
                    name.trim().chars().all(|c| c.is_uppercase() || c == '_')
                })
        }
        Language::JavaScript => {
            trimmed.starts_with("const ")
                || trimmed.starts_with("export const ")
                || trimmed.starts_with("let ")
                || trimmed.starts_with("var ")
        }
        Language::Go => trimmed.starts_with("const ") || trimmed.starts_with("var "),
        Language::Java => {
            (trimmed.contains("final ") || trimmed.contains("static "))
                && trimmed.contains(" = ")
                && !trimmed.contains("(")
        }
        _ => false,
    }
}

/// Count net brace depth change in a line.
pub(crate) fn count_braces(line: &str) -> i32 {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut quote_char = '"';

    for c in line.chars() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if (c == '"' || c == '\'') && !in_string {
            in_string = true;
            quote_char = c;
            continue;
        }
        if c == quote_char && in_string {
            in_string = false;
            continue;
        }
        if !in_string {
            if c == '{' {
                depth += 1;
            }
            if c == '}' {
                depth -= 1;
            }
        }
    }
    depth
}

/// Apply line range limits (head/tail).
pub(crate) fn apply_line_range<'a>(
    lines: &'a [&str],
    max_lines: Option<usize>,
    tail_lines: Option<usize>,
) -> Vec<&'a str> {
    if let Some(tail) = tail_lines {
        let start = lines.len().saturating_sub(tail);
        lines[start..].to_vec()
    } else if let Some(max) = max_lines {
        lines[..lines.len().min(max)].to_vec()
    } else {
        lines.to_vec()
    }
}
