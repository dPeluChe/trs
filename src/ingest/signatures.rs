//! Signature extraction: reducing a source file to its imports and its
//! function/class declarations. Split out of `collect_compress.rs`, which
//! is about reading and budgeting files rather than about parsing them.

/// Deduplicates repeated functions, adds spacing before classes.
pub(super) fn extract_signatures(content: &str, ext: &str) -> String {
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
                // `pub(` catches visibility-qualified items — `pub(crate) fn`,
                // `pub(super) struct`, `pub(in …)` — which a plain `pub ` prefix
                // check misses. This codebase is heavily `pub(crate)`, so without
                // it ~500 real symbols (incl. `execute_and_parse`) never make the
                // digest.
                t.starts_with("pub ")
                    || t.starts_with("pub(")
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
                    || t.starts_with("pub(")
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

    // Strip struct field declarations (pub id: String, pub(crate) id: …, etc.)
    if (s.starts_with("pub ") || s.starts_with("pub("))
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
