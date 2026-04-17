//! Manifest and notebook compression helpers used by read_and_compress.
//!
//! Extracted from collect_compress.rs to keep that file under the 500-LOC
//! guideline. All functions here return the compact markdown-ready string
//! a digest should embed.

/// Extract JSON schema: show structure with 1 sample, count arrays.
/// Extract JSON schema: show structure with 1 sample, count arrays.
pub(super) fn extract_data_schema(content: &str, ext: &str) -> String {
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

pub(super) fn summarize_json_value(val: &serde_json::Value, depth: usize) -> String {
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

/// Compress pyproject.toml / Cargo.toml — keep dep list intact (crucial for LLMs),
/// but drop heavy metadata (long descriptions, build profiles, workspace members).
pub(super) fn compress_toml_manifest(content: &str) -> String {
    let Ok(doc) = content.parse::<toml::Value>() else {
        // Invalid TOML — fall back to raw content, truncated lightly.
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 50 {
            return format!("{}\n... ({} lines)", lines[..50].join("\n"), lines.len());
        }
        return content.to_string();
    };

    let mut out = String::new();

    // Top-level [package]
    if let Some(pkg) = doc.get("package").and_then(|v| v.as_table()) {
        if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
            out.push_str(&format!("name: {}\n", name));
        }
        if let Some(v) = pkg.get("version").and_then(|v| v.as_str()) {
            out.push_str(&format!("version: {}\n", v));
        }
        if let Some(e) = pkg.get("edition").and_then(|v| v.as_str()) {
            out.push_str(&format!("edition: {}\n", e));
        }
        if let Some(desc) = pkg.get("description").and_then(|v| v.as_str()) {
            let short: String = desc.chars().take(120).collect();
            out.push_str(&format!("description: {}\n", short));
        }
    }

    // [project] (pyproject PEP 621)
    if let Some(proj) = doc.get("project").and_then(|v| v.as_table()) {
        if let Some(name) = proj.get("name").and_then(|v| v.as_str()) {
            out.push_str(&format!("name: {}\n", name));
        }
        if let Some(v) = proj.get("version").and_then(|v| v.as_str()) {
            out.push_str(&format!("version: {}\n", v));
        }
        if let Some(py) = proj.get("requires-python").and_then(|v| v.as_str()) {
            out.push_str(&format!("requires-python: {}\n", py));
        }
        if let Some(deps) = proj.get("dependencies").and_then(|v| v.as_array()) {
            out.push_str("dependencies:\n");
            for dep in deps {
                if let Some(s) = dep.as_str() {
                    out.push_str(&format!("  - {}\n", s));
                }
            }
        }
        if let Some(opt) = proj.get("optional-dependencies").and_then(|v| v.as_table()) {
            for (group, arr) in opt {
                out.push_str(&format!("optional-dependencies.{}:\n", group));
                if let Some(list) = arr.as_array() {
                    for dep in list {
                        if let Some(s) = dep.as_str() {
                            out.push_str(&format!("  - {}\n", s));
                        }
                    }
                }
            }
        }
    }

    // Cargo [dependencies], [dev-dependencies], [build-dependencies]
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(t) = doc.get(*section).and_then(|v| v.as_table()) {
            out.push_str(&format!("[{}]\n", section));
            for (k, v) in t {
                let ver = match v {
                    toml::Value::String(s) => s.clone(),
                    toml::Value::Table(t) => t
                        .get("version")
                        .and_then(|x| x.as_str())
                        .unwrap_or("*")
                        .to_string(),
                    _ => "*".to_string(),
                };
                out.push_str(&format!("  {} = {}\n", k, ver));
            }
        }
    }

    // Cargo [features]
    if let Some(features) = doc.get("features").and_then(|v| v.as_table()) {
        out.push_str("[features]\n");
        for (k, _) in features {
            out.push_str(&format!("  {}\n", k));
        }
    }

    // Cargo [[bin]] / [lib] — just record presence
    if let Some(bins) = doc.get("bin").and_then(|v| v.as_array()) {
        for b in bins {
            if let Some(name) = b.get("name").and_then(|v| v.as_str()) {
                out.push_str(&format!("bin: {}\n", name));
            }
        }
    }

    if out.is_empty() {
        // Unknown TOML shape — keep raw but truncate to avoid noise.
        return content.lines().take(60).collect::<Vec<_>>().join("\n");
    }
    out
}

/// Jupyter notebook: parse JSON, emit code + markdown cell contents.
/// A .ipynb is JSON, but LLMs want the actual cell contents, not the metadata.
pub(super) fn compress_jupyter_notebook(content: &str) -> String {
    let Ok(nb) = serde_json::from_str::<serde_json::Value>(content) else {
        return format!("(invalid .ipynb, {} bytes)", content.len());
    };
    let Some(cells) = nb.get("cells").and_then(|v| v.as_array()) else {
        return "(.ipynb has no cells)".to_string();
    };

    let mut out = String::new();
    let lang = nb
        .get("metadata")
        .and_then(|m| m.get("kernelspec"))
        .and_then(|k| k.get("language"))
        .and_then(|v| v.as_str())
        .unwrap_or("python");
    out.push_str(&format!(
        "(jupyter notebook, {} cells, {})\n\n",
        cells.len(),
        lang
    ));

    let mut code_total = 0usize;
    for (i, cell) in cells.iter().enumerate() {
        let cell_type = cell.get("cell_type").and_then(|v| v.as_str()).unwrap_or("");
        let source = cell.get("source");
        let text = match source {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(""),
            _ => continue,
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        match cell_type {
            "markdown" => {
                out.push_str(&format!("# cell {} (markdown)\n", i + 1));
                // Keep first 20 lines of markdown to preserve context
                let lines: Vec<&str> = trimmed.lines().take(20).collect();
                out.push_str(&lines.join("\n"));
                out.push_str("\n\n");
            }
            "code" => {
                code_total += 1;
                out.push_str(&format!("# cell {} (code)\n", i + 1));
                // Keep imports + function/class defs + first few executable lines
                let kept: Vec<&str> = trimmed
                    .lines()
                    .enumerate()
                    .filter(|(idx, l)| {
                        let t = l.trim();
                        *idx < 25
                            || t.starts_with("import ")
                            || t.starts_with("from ")
                            || t.starts_with("def ")
                            || t.starts_with("class ")
                            || t.starts_with("async def ")
                    })
                    .map(|(_, l)| l)
                    .collect();
                out.push_str(&kept.join("\n"));
                out.push_str("\n\n");
                if code_total >= 8 {
                    out.push_str(&format!(
                        "... ({} more cells, use --full to include)\n",
                        cells.len() - i - 1
                    ));
                    break;
                }
            }
            _ => {}
        }
    }
    out
}

/// Compress package.json: name, version, scripts names, dep names.
pub(super) fn compress_package_json(content: &str) -> String {
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
