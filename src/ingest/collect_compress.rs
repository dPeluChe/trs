//! File reading and compression for the ingest pipeline.
//!
//! Reads source files and extracts what matters for LLM consumption:
//! signatures from code, schemas from data, summaries from docs.

use std::path::Path;

use crate::router::handlers::read_filters::{detect_language, filter_minimal, Language};

use super::collect_index::{extract_module_doc, extract_symbols, is_module_anchor};
use super::collect_manifests::{
    compress_jupyter_notebook, compress_package_json, compress_toml_manifest, extract_data_schema,
};
use super::IngestLevel;

/// Output of a single-file compression pass.
pub(super) struct CompressResult {
    pub content: String,
    pub raw_imports: Vec<String>,
    /// Module-level doc for directory annotations (mod.rs / lib.rs / __init__.py).
    pub module_doc: Option<String>,
    /// Public / exported symbol names for the symbol index.
    pub symbols: Vec<String>,
}

/// Intelligently extract what matters from a file for LLM consumption.
/// Returns a `CompressResult` with the compressed content plus metadata
/// (raw imports, module-level docstring, exported symbol names).
pub(super) fn read_and_compress(path: &Path, level: IngestLevel) -> Option<CompressResult> {
    let content = std::fs::read_to_string(path).ok()?;

    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    let mut ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let lower_name = name.to_lowercase();

    // Skip: CSS, HTML, SVG, XML, plist, lock files, minified, type declarations
    if matches!(
        ext.as_str(),
        "css" | "scss" | "less" | "svg" | "html" | "htm" | "xml" | "plist" | "xib" | "storyboard"
    ) {
        return None;
    }
    if lower_name.ends_with(".lock")
        || lower_name.ends_with(".min.js")
        || lower_name.ends_with(".min.css")
        || lower_name.ends_with(".d.ts")
        || lower_name == "__init__.py"
        || lower_name == ".gitkeep"
        || lower_name == ".nojekyll"
        || lower_name == ".gitattributes"
        || lower_name == "py.typed"
    {
        return None;
    }
    // Skip empty or near-empty files
    if content.trim().len() < 5 {
        return None;
    }

    // No extension but has a shebang → infer language from the interpreter.
    // This matters for files like `tmux-bridge`, `hooks/pre-commit`, etc.
    if ext.is_empty() {
        if let Some(first) = content.lines().next() {
            if first.starts_with("#!") {
                if first.contains("bash") || first.contains("/sh") || first.contains("zsh") {
                    ext = "sh".to_string();
                } else if first.contains("python") {
                    ext = "py".to_string();
                } else if first.contains("node") {
                    ext = "js".to_string();
                } else if first.contains("ruby") {
                    ext = "rb".to_string();
                }
            }
        }
    }

    let raw_imports = super::deps_extract::extract_raw_imports(&content, &ext);
    let module_doc = if is_module_anchor(&lower_name) {
        extract_module_doc(&content, &ext)
    } else {
        None
    };
    let symbols = extract_symbols(&content, &ext);

    // Wrap the compressed content in a CompressResult carrying the
    // already-extracted metadata. Each early-return branch calls `ok` exactly
    // once, so the metadata is moved (not cloned) into the result via
    // interior mutability on an Option.
    let mut meta = Some((raw_imports, module_doc, symbols));
    let mut ok = |s: String| {
        let (imports, doc, syms) = meta.take().expect("CompressResult built twice");
        Some(CompressResult {
            content: s,
            raw_imports: imports,
            module_doc: doc,
            symbols: syms,
        })
    };

    // Jupyter notebooks: parse JSON, show code + markdown cells.
    if ext == "ipynb" {
        return ok(compress_jupyter_notebook(&content));
    }

    // All markdown files: treat as docs, never extract signatures
    if ext == "md" || ext == "mdx" {
        // Translated READMEs (README_es.md, README_fr.md, etc.): skip content
        // They're variants of README.md -- just mention they exist
        if lower_name.starts_with("readme_") || lower_name.starts_with("readme-") {
            return ok(format!(
                "(translation of README.md, {} lines)",
                content.lines().count()
            ));
        }

        let cleaned = super::format::strip_html_from_markdown(&content);
        let is_key = matches!(
            lower_name.as_str(),
            "readme.md" | "claude.md" | "agents.md" | "contributing.md" | "changelog.md"
        );
        let max_lines = if is_key { 40 } else { 20 };
        let lines: Vec<&str> = cleaned.lines().collect();
        if lines.len() > max_lines + 5 {
            let preview: String = lines[..max_lines].join("\n");
            // Collect headings in the TRUNCATED portion so the reader knows
            // what the truncation hid.
            let tail_headings: Vec<String> = lines[max_lines..]
                .iter()
                .filter_map(|l| {
                    let t = l.trim_start();
                    t.strip_prefix("## ")
                        .or_else(|| t.strip_prefix("# "))
                        .map(|rest| rest.trim().to_string())
                })
                .take(8)
                .collect();
            let suffix = if tail_headings.is_empty() {
                format!("\n... ({} lines)", lines.len())
            } else {
                format!(
                    "\n... ({} lines, hidden sections: {})",
                    lines.len(),
                    tail_headings.join(" · ")
                )
            };
            return ok(format!("{}{}", preview, suffix));
        }
        return ok(cleaned);
    }

    // SKILL files and similar: cap at ~1KB
    if lower_name.contains("skill") || lower_name.ends_with(".prompt") {
        if content.len() > 1024 {
            let mut cut = 1024;
            while cut > 0 && !content.is_char_boundary(cut) {
                cut -= 1;
            }
            if let Some(nl) = content[..cut].rfind('\n') {
                cut = nl;
            }
            return ok(format!(
                "{}\n... ({} lines)",
                &content[..cut],
                content.lines().count()
            ));
        }
        return ok(content);
    }

    // Manifest files: compact + dependency-complete view. Never truncate the
    // dep list — it's the most informative part of the manifest for an LLM.
    if lower_name == "package.json" {
        return ok(compress_package_json(&content));
    }
    if lower_name == "pyproject.toml" || lower_name == "cargo.toml" {
        return ok(compress_toml_manifest(&content));
    }
    if lower_name == "requirements.txt" {
        // requirements are already a list — pass through unchanged, it's small.
        return ok(content);
    }

    // JSON/YAML: extract schema (keys + 1 sample), not all data
    let lang = detect_language(&path.to_path_buf());
    if lang == Language::Data {
        return ok(extract_data_schema(&content, &ext));
    }

    // Source code compression ladder:
    //   Full       — raw content, nothing removed
    //   Minimal    — strip comments + normalize blank lines (shared with `trs read`)
    //   Aggressive — signatures only (imports + defs), no function bodies
    match level {
        IngestLevel::Full => ok(content),
        IngestLevel::Minimal => ok(filter_minimal(&content, lang)),
        IngestLevel::Aggressive => ok(extract_signatures(&content, &ext)),
    }
}

/// Quick check: does this Python source have any multi-line `def`/`class`
/// header that would benefit from the joining pass? If not, we can pipe the
/// original content straight through and skip the allocation.
fn has_multiline_python_sig(content: &str) -> bool {
    let mut in_sig = false;
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with("def ") || t.starts_with("async def ") || t.starts_with("class ") {
            let opens = t.matches('(').count();
            let closes = t.matches(')').count();
            if opens > closes {
                return true;
            }
            in_sig = false;
            // `def foo(` with comma-trailing on same line but no close: handled above.
        } else if in_sig {
            return true;
        }
    }
    false
}

/// Join multi-line Python `def name(...)` signatures onto a single line.
/// Only touches `def`/`async def`/`class` headers; other lines pass through.
fn join_python_multiline_sigs(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        let is_sig = trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("class ");
        if !is_sig {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        // Count parenthesis balance across lines until we close the signature
        // and reach a colon or line end without an open paren.
        let mut accumulated = String::from(line);
        let mut depth: i32 =
            trimmed.matches('(').count() as i32 - trimmed.matches(')').count() as i32;
        let ends_with_colon = |s: &str| {
            let t = s.trim_end();
            t.ends_with(':') || t.ends_with(": ...")
        };
        while depth > 0 || (!ends_with_colon(&accumulated) && accumulated.trim_end().ends_with(','))
        {
            let Some(next) = lines.next() else {
                break;
            };
            depth += next.matches('(').count() as i32 - next.matches(')').count() as i32;
            // Collapse continuation whitespace.
            let cont = next.trim_start();
            accumulated.push(' ');
            accumulated.push_str(cont);
            if depth <= 0 && ends_with_colon(&accumulated) {
                break;
            }
        }
        // Tidy up the joined signature: remove redundant spaces around
        // parens/brackets and trailing commas before closing brackets.
        let tidy = accumulated
            .replace("( ", "(")
            .replace(" )", ")")
            .replace(",)", ")")
            .replace(", )", ")")
            .replace("  ", " ");
        out.push_str(&tidy);
        out.push('\n');
    }
    out
}

/// Extract function/class signatures from source code -- names without bodies.
/// Deduplicates repeated functions, adds spacing before classes.
fn extract_signatures(content: &str, ext: &str) -> String {
    let mut result = String::new();
    let mut seen_sigs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Python signatures commonly span multiple lines when they carry type
    // annotations; fuse those back onto a single line so the extractor loop
    // keeps the hints. Fast path: skip the allocation when the file has no
    // multi-line headers to join.
    let joined_buf: String;
    let source: &str = if matches!(ext, "py" | "pyi") && has_multiline_python_sig(content) {
        joined_buf = join_python_multiline_sigs(content);
        &joined_buf
    } else {
        content
    };

    for line in source.lines() {
        let t = line.trim();

        // Skip imports (agent can see these in package.json/Cargo.toml)
        if t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") {
            continue;
        }

        let is_class = t.starts_with("class ")
            || t.starts_with("interface ")
            || t.starts_with("struct ")
            || t.starts_with("enum ")
            || t.starts_with("trait ")
            || t.starts_with("impl ")
            || (t.starts_with("export ") && (t.contains("class ") || t.contains("interface ")));

        let keep = match ext {
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" | "vue" | "svelte" => {
                t.starts_with("export ")
                    || t.starts_with("function ")
                    || is_class
                    || t.starts_with("type ")
                    || t.starts_with("const ")
                        && (t.contains("= mutation(")
                            || t.contains("= query(")
                            || t.contains("= action(")
                            || t.contains("= internalMutation(")
                            || t.contains("=> {")
                            || t.contains("= defineTable("))
            }
            "rs" => {
                t.starts_with("pub ")
                    || t.starts_with("fn ")
                    || is_class
                    || t.starts_with("mod ")
                    || t.starts_with("type ")
            }
            "py" | "pyi" => {
                t.starts_with("def ") || t.starts_with("class ") || t.starts_with("async def ")
            }
            "go" => {
                t.starts_with("func ")
                    || t.starts_with("type ")
                    || t.starts_with("var ")
                    || t.starts_with("const ")
            }
            "sh" | "bash" | "zsh" | "fish" => {
                // Keep: function definitions (both `foo() {` and `function foo`),
                // top-level constants (UPPER_CASE=...), and the usage() / help
                // conventions. Comments that look like section headers are caught
                // below via the #!/bin/... or ^# blocks.
                t.ends_with("() {")
                    || t.starts_with("function ")
                    || (t.contains('=')
                        && !t.contains(' ')
                        && t.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
            }
            _ => {
                t.starts_with("export ")
                    || t.starts_with("pub ")
                    || t.starts_with("fn ")
                    || t.starts_with("def ")
                    || t.starts_with("class ")
                    || t.starts_with("function ")
            }
        };

        if !keep {
            continue;
        }

        let cleaned = clean_signature(t);
        if cleaned.is_empty() {
            continue;
        }

        // Dedup: skip if we've seen this exact signature before (e.g. multiple to_dict)
        if !is_class && seen_sigs.contains(&cleaned) {
            continue;
        }
        seen_sigs.insert(cleaned.clone());

        // Add blank line before a class only if the previous line was a method
        // (not before consecutive class declarations with no methods)
        if is_class && !result.is_empty() {
            let last_line = result.lines().last().unwrap_or("");
            let prev_is_method = !last_line.is_empty()
                && !last_line.starts_with("class ")
                && !last_line.starts_with("struct ")
                && !last_line.starts_with("interface ")
                && !last_line.starts_with("enum ")
                && !last_line.starts_with("trait ");
            if prev_is_method {
                result.push('\n');
            }
        }

        // Signatures with type hints / generics pack a lot of info into a
        // single line (e.g. `def encode(text: str, prepend: Optional[str] =
        // None, num_threads: int = 8) -> list[int]:`). Prefer to keep the
        // full signature — only truncate when it's truly verbose (>200c).
        if cleaned.len() > 200 {
            let mut end = 197;
            while end > 0 && !cleaned.is_char_boundary(end) {
                end -= 1;
            }
            result.push_str(&cleaned[..end]);
            result.push_str("...\n");
        } else {
            result.push_str(&cleaned);
            result.push('\n');
        }
    }

    if result.is_empty() {
        // No recognizable signatures -- just report size
        let line_count = content.lines().count();
        result.push_str(&format!("({} lines)\n", line_count));
    }
    result
}

/// Strip trailing noise from a signature line.
/// `export function foo(): string {` -> `export function foo(): string`
/// `export const POINTS = {` -> `export const POINTS`
/// `const handleAnswer = useCallback((index: number) => {` -> `const handleAnswer = useCallback((index: number))`
/// `def merge_blocks(prefix, count, output_file):` -> `def merge_blocks(prefix, count, output_file)`
fn clean_signature(line: &str) -> String {
    let mut s = line.to_string();

    // Strip trailing { => { = [ = { : ;
    s = s.trim_end().to_string();
    loop {
        let before = s.len();
        if s.ends_with("=> {") {
            s = s[..s.len() - 4].trim_end().to_string();
            if !s.ends_with(')') {
                s.push(')');
            }
        }
        // Strip trailing block openers/closers but keep `[` / `]` — those are
        // almost always part of type annotations like `list[int]`,
        // `Optional[str]`, `Vec<T>` that we want to preserve.
        while s.ends_with('{') || s.ends_with('}') {
            s.pop();
            s = s.trim_end().to_string();
        }
        for suffix in &["= ", "=", ":", ";"] {
            if s.ends_with(suffix) {
                s = s[..s.len() - suffix.len()].trim_end().to_string();
            }
        }
        if s.len() == before {
            break;
        }
    }

    // Python: strip self from first param
    // def foo(self, x, y) -> def foo(x, y)
    // def foo(self) -> def foo()
    if s.contains("(self, ") {
        s = s.replace("(self, ", "(");
    } else if s.contains("(self)") {
        s = s.replace("(self)", "()");
    }

    // Strip pub(crate) -> pub
    s = s.replace("pub(crate) ", "pub ");

    // Simplify long Result types: Result<Vec<Account>, String> -> Result<Vec<Account>>
    if let Some(result_start) = s.find("Result<") {
        if let Some(comma) = s[result_start..].find(", String>") {
            let end = result_start + comma + ", String>".len();
            let inner = &s[result_start + 7..result_start + comma];
            s = format!("{}Result<{}>{}", &s[..result_start], inner, &s[end..]);
        }
    }

    // Strip struct field declarations (pub id: String, etc.)
    if s.starts_with("pub ")
        && s.contains(": ")
        && !s.contains("fn ")
        && !s.contains("async ")
        && !s.contains("struct ")
    {
        // It's a struct field like "pub id: String," -- skip these
        return String::new();
    }

    s
}
