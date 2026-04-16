//! File reading and compression for the ingest pipeline.
//!
//! Reads source files and extracts what matters for LLM consumption:
//! signatures from code, schemas from data, summaries from docs.

use std::path::Path;

use crate::router::handlers::read_filters::{detect_language, Language};

use super::IngestLevel;

/// Intelligently extract what matters from a file for LLM consumption.
/// Returns (compressed_content, raw_imports) where raw_imports are extracted
/// from the original content before compression.
pub(super) fn read_and_compress(path: &Path, level: IngestLevel) -> Option<(String, Vec<String>)> {
    let content = std::fs::read_to_string(path).ok()?;

    if content.chars().take(512).any(|c| c == '\0') {
        return None;
    }

    let ext = path
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

    // Extract imports from raw content now, before any compression transforms it
    let raw_imports = super::deps_extract::extract_raw_imports(&content, &ext);

    let ok = |s: String| Some((s, raw_imports.clone()));

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
            return ok(format!("{}\n... ({} lines)", preview, lines.len()));
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

    // package.json: compress to name + deps names only
    if lower_name == "package.json" {
        return ok(compress_package_json(&content));
    }

    // JSON/YAML: extract schema (keys + 1 sample), not all data
    let lang = detect_language(&path.to_path_buf());
    if lang == Language::Data {
        return ok(extract_data_schema(&content, &ext));
    }

    // Source code: always extract signatures in minimal/aggressive mode
    // An agent needs to know WHAT exists, not HOW it's implemented
    match level {
        IngestLevel::Full => ok(content),
        _ => ok(extract_signatures(&content, &ext)),
    }
}

/// Extract JSON schema: show structure with 1 sample, count arrays.
fn extract_data_schema(content: &str, ext: &str) -> String {
    if ext == "json" || ext == "jsonl" {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            return summarize_json_value(&val, 0);
        }
    }
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= 20 {
        return content.to_string();
    }
    format!("{}\n... ({} lines)", lines[..20].join("\n"), lines.len())
}

fn summarize_json_value(val: &serde_json::Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    match val {
        serde_json::Value::Array(arr) if arr.is_empty() => "[]".into(),
        serde_json::Value::Array(arr) => {
            let first = summarize_json_value(&arr[0], depth + 1);
            format!("[{}, ...] ({} items)", first, arr.len())
        }
        serde_json::Value::Object(map) if map.is_empty() => "{}".into(),
        serde_json::Value::Object(map) => {
            let mut out = String::from("{\n");
            for (key, val) in map.iter() {
                let v = match val {
                    serde_json::Value::String(s) if s.len() > 40 => {
                        let t: String = s.chars().take(37).collect();
                        format!("\"{}...\"", t)
                    }
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".into(),
                    serde_json::Value::Array(a) if a.is_empty() => "[]".into(),
                    serde_json::Value::Array(a) => {
                        let inner = summarize_json_value(&a[0], depth + 2);
                        format!("[{}, ...] ({} items)", inner, a.len())
                    }
                    serde_json::Value::Object(_) => summarize_json_value(val, depth + 1),
                };
                out.push_str(&format!("{}  \"{}\": {},\n", indent, key, v));
            }
            out.push_str(&format!("{}}}", indent));
            out
        }
        serde_json::Value::String(s) if s.len() > 40 => {
            let t: String = s.chars().take(37).collect();
            format!("\"{}...\"", t)
        }
        other => other.to_string(),
    }
}

/// Extract function/class signatures from source code -- names without bodies.
/// Deduplicates repeated functions, adds spacing before classes.
fn extract_signatures(content: &str, ext: &str) -> String {
    let mut result = String::new();
    let mut seen_sigs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in content.lines() {
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

        if cleaned.len() > 120 {
            let mut end = 117;
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
        while s.ends_with('{') || s.ends_with('}') || s.ends_with('[') || s.ends_with(']') {
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

/// Compress package.json: name, version, scripts names, dep names.
fn compress_package_json(content: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        let mut out = String::new();
        if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
            out.push_str(&format!("name: {}\n", n));
        }
        if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
            out.push_str(&format!("version: {}\n", v));
        }
        if let Some(s) = val.get("scripts").and_then(|v| v.as_object()) {
            out.push_str(&format!(
                "scripts: {}\n",
                s.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        for key in &["dependencies", "devDependencies"] {
            if let Some(deps) = val.get(*key).and_then(|v| v.as_object()) {
                let names: Vec<&str> = deps.keys().map(|k| k.as_str()).collect();
                out.push_str(&format!("{}: {}\n", key, names.join(", ")));
            }
        }
        return out;
    }
    content.to_string()
}
