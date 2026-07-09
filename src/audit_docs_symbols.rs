//! `audit-docs` symbol analysis: extract declared symbols from code fences
//! (JS/TS/Python/Rust/Go/Swift) and resolve whether they already exist in the
//! project source (→ a fenced snippet that could be a link, not inline bloat).

use std::path::{Path, PathBuf};
use std::process::Command;

use super::audit_docs::{InlineBloat, SymbolMatch};
use crate::text_util::first_ident;

pub(crate) fn extract_fence_symbols(lang: &str, body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for raw in body.lines() {
        let line = raw.trim_start();
        let sym = match lang {
            "ts" | "tsx" | "js" | "jsx" | "javascript" | "typescript" => {
                extract_js_like_symbol(line)
            }
            "py" | "python" => extract_python_symbol(line),
            "rs" | "rust" => extract_rust_symbol(line),
            "go" => extract_go_symbol(line),
            "swift" => extract_swift_symbol(line),
            _ => None,
        };
        if let Some(name) = sym {
            if is_meaningful_symbol(&name) && seen.insert(name.clone()) {
                out.push(name);
            }
        }
    }
    out
}

/// Filter out ultra-generic identifiers that almost always collide with
/// unrelated definitions in a large codebase. A match on `data` or `result`
/// isn't informative; a match on `RouteBuilder` is.
fn is_meaningful_symbol(name: &str) -> bool {
    if name.len() < 4 {
        return false;
    }
    const GENERIC_BLOCKLIST: &[&str] = &[
        // Ultra-common variable / parameter names.
        "data",
        "result",
        "results",
        "value",
        "values",
        "item",
        "items",
        "response",
        "request",
        "req",
        "res",
        "page",
        "pages",
        "search",
        "json",
        "xml",
        "html",
        "text",
        "name",
        "names",
        "id",
        "ids",
        "key",
        "keys",
        "index",
        "count",
        "size",
        "length",
        "total",
        "form",
        "forms",
        "input",
        "output",
        "error",
        "errors",
        "err",
        "props",
        "state",
        "config",
        "options",
        "args",
        "params",
        "user",
        "users",
        "client",
        "clients",
        "list",
        "lists",
        "main",
        "init",
        "setup",
        "start",
        "stop",
        "run",
        "test",
        "tests",
        "util",
        "utils",
        "helper",
        "helpers",
        "message",
        "messages",
        "status",
        "title",
        "body",
        "header",
        "headers",
        "footer",
        "row",
        "rows",
        "column",
        "columns",
        "table",
        "tables",
        "type",
        "types",
        "kind",
        "method",
        "methods",
        "callback",
        "handler",
        "handlers",
        "filter",
        "filters",
        "reducer",
        "action",
        "store",
        "stores",
        "model",
        "models",
        "view",
        "views",
        "render",
        "click",
        "submit",
        "change",
        "select",
        "update",
        "create",
        "delete",
        "isValid",
        "isEditing",
        "isLoading",
        "isOpen",
        "isActive",
    ];
    let lower = name.to_ascii_lowercase();
    !GENERIC_BLOCKLIST.iter().any(|g| *g == lower)
}

fn extract_js_like_symbol(line: &str) -> Option<String> {
    // export default function Foo(
    // export function Foo(
    // export const Foo =
    // export class Foo
    // export interface Foo
    // export type Foo =
    // function Foo(
    // class Foo
    let rest = line
        .strip_prefix("export default ")
        .or_else(|| line.strip_prefix("export "))
        .unwrap_or(line);
    for kw in &[
        "async function ",
        "function ",
        "const ",
        "let ",
        "class ",
        "interface ",
        "type ",
        "enum ",
    ] {
        if let Some(body) = rest.strip_prefix(kw) {
            return first_ident(body);
        }
    }
    None
}

fn extract_python_symbol(line: &str) -> Option<String> {
    if let Some(body) = line
        .strip_prefix("async def ")
        .or_else(|| line.strip_prefix("def "))
        .or_else(|| line.strip_prefix("class "))
    {
        let name = first_ident(body)?;
        if name.starts_with('_') {
            return None;
        }
        return Some(name);
    }
    None
}

fn extract_rust_symbol(line: &str) -> Option<String> {
    let rest = line.strip_prefix("pub ")?;
    let rest = rest
        .strip_prefix("async ")
        .or_else(|| rest.strip_prefix("unsafe "))
        .unwrap_or(rest);
    for kw in &["fn ", "struct ", "enum ", "trait ", "type ", "const "] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

fn extract_go_symbol(line: &str) -> Option<String> {
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

fn extract_swift_symbol(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("public ")
        .or_else(|| line.strip_prefix("open "))?;
    for kw in &["func ", "class ", "struct ", "enum ", "protocol "] {
        if let Some(body) = rest.strip_prefix(*kw) {
            return first_ident(body);
        }
    }
    None
}

pub(crate) fn resolve_symbol_matches(bloat: &mut [InlineBloat], root: &Path) {
    // Dedup symbols across all bloat entries so we walk the tree once.
    let mut wanted: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for (idx, b) in bloat.iter().enumerate() {
        for sym in &b.symbols {
            wanted.entry(sym.clone()).or_default().push(idx);
        }
    }
    if wanted.is_empty() {
        return;
    }

    let mut found: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    let walker = ignore::WalkBuilder::new(root)
        .standard_filters(true)
        .max_depth(Some(8))
        .build();

    let mut files_scanned = 0usize;
    for entry in walker.flatten() {
        if files_scanned >= 2000 || found.len() == wanted.len() {
            break;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "py" | "rs" | "go" | "swift" | "java" | "kt"
        ) {
            continue;
        }
        // Skip synthetic/generated areas the ignore crate may not know about.
        let path_str = path.to_string_lossy();
        if path_str.contains("/dist/")
            || path_str.contains("/build/")
            || path_str.contains("/coverage/")
            || path_str.contains("/.next/")
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        files_scanned += 1;
        for sym in wanted.keys() {
            if found.contains_key(sym) {
                continue;
            }
            if contains_symbol_definition(&content, sym, ext) {
                found.insert(sym.clone(), path.to_path_buf());
            }
        }
    }

    for (sym, src) in found {
        if let Some(bloat_indices) = wanted.get(&sym) {
            for idx in bloat_indices {
                bloat[*idx].symbol_matches.push(SymbolMatch {
                    name: sym.clone(),
                    source_path: src.clone(),
                });
            }
        }
    }
}

/// True if `content` contains what looks like a definition of `sym` in a
/// file of extension `ext`. Cheap substring match — not a full parser, but
/// accurate enough for the "does this identifier live somewhere in src?"
/// question.
fn contains_symbol_definition(content: &str, sym: &str, ext: &str) -> bool {
    // Build a few candidate prefixes — the patterns are common definition
    // forms across the supported languages.
    let needles: Vec<String> = match ext {
        "ts" | "tsx" | "js" | "jsx" => vec![
            format!("function {}(", sym),
            format!("function {} (", sym),
            format!("class {}", sym),
            format!("interface {}", sym),
            format!("type {} ", sym),
            format!("type {}=", sym),
            format!("const {} =", sym),
            format!("const {}=", sym),
            format!("let {} =", sym),
            format!("enum {}", sym),
        ],
        "py" => vec![
            format!("def {}(", sym),
            format!("async def {}(", sym),
            format!("class {}(", sym),
            format!("class {}:", sym),
        ],
        "rs" => vec![
            format!("fn {}(", sym),
            format!("fn {}<", sym),
            format!("struct {}", sym),
            format!("enum {}", sym),
            format!("trait {}", sym),
            format!("type {} ", sym),
            format!("const {} ", sym),
        ],
        "go" => vec![
            format!("func {}(", sym),
            format!("func ({} ", sym),
            format!("type {} ", sym),
        ],
        "swift" => vec![
            format!("func {}(", sym),
            format!("class {}", sym),
            format!("struct {}", sym),
            format!("enum {}", sym),
            format!("protocol {}", sym),
        ],
        "java" | "kt" => vec![
            format!("class {}", sym),
            format!("interface {}", sym),
            format!("fun {}(", sym),
            format!("{} {}(", "void", sym),
        ],
        _ => return false,
    };
    needles.iter().any(|n| content.contains(n.as_str()))
}

pub(crate) fn last_commit_days_ago(path: &Path, root: &Path) -> Option<u64> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(path)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ts_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let ts: u64 = ts_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(now.saturating_sub(ts) / 86400)
}
