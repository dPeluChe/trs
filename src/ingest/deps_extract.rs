//! Language-specific import extractors for the dependency graph.
//!
//! Each function scans the top of a source file and returns raw import tokens
//! (module names or relative paths) — not yet resolved to project files.

/// Extract raw import tokens from file content (before compression).
/// Returns module names or relative paths — not yet resolved to project files.
pub(crate) fn extract_raw_imports(content: &str, ext: &str) -> Vec<String> {
    let mut imports = match ext {
        "rs" => extract_rust(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" => extract_ts(content),
        "py" => extract_python(content),
        "go" => extract_go(content),
        _ => return vec![],
    };
    imports.sort();
    imports.dedup();
    imports
}

fn extract_rust(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        // mod foo; — submodule declaration
        if let Some(rest) = t.strip_prefix("mod ") {
            let name = rest.trim_end_matches(';').trim();
            if !name.is_empty()
                && !name.starts_with('{')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                out.push(name.to_string());
            }
            continue;
        }
        // use crate::X or use crate::X::Y
        if let Some(rest) = t.strip_prefix("use crate::") {
            if let Some(first) = rest.split("::").next() {
                let name = first
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(';');
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
            continue;
        }
        // use super::X
        if let Some(rest) = t.strip_prefix("use super::") {
            if let Some(first) = rest.split("::").next() {
                let name = first
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(';');
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
        // Stop scanning after the import block (first non-import line that isn't a comment/attr)
        // This keeps it fast on large files
        if !t.is_empty()
            && !t.starts_with("use ")
            && !t.starts_with("mod ")
            && !t.starts_with("//")
            && !t.starts_with("#[")
            && !t.starts_with("pub mod")
            && !t.starts_with("extern crate")
        {
            // Give up early — imports are at the top in Rust
            if out.len() > 0 {
                break;
            }
        }
    }
    out
}

fn extract_ts(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if !t.starts_with("import ") && !t.starts_with("export ") {
            continue;
        }

        // Find `from '...'` or `from "..."`
        let path = extract_from_path(t);
        if let Some(p) = path {
            // Relative imports: ./foo or ../bar
            // Alias imports: @/lib/foo (Next.js, Vite, etc.)
            if p.starts_with("./") || p.starts_with("../") || p.starts_with("@/") {
                out.push(p);
            }
        }
    }
    out
}

fn extract_from_path(line: &str) -> Option<String> {
    // Try single quotes first, then double quotes
    for (open, close) in [("from '", "'"), ("from \"", "\"")] {
        if let Some(pos) = line.rfind(open) {
            let after = &line[pos + open.len()..];
            if let Some(end) = after.find(close) {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}

fn extract_python(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();

        // from .foo import ...  (relative imports)
        if let Some(rest) = t.strip_prefix("from .") {
            let module = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('.');
            if !module.is_empty() && module != "import" {
                out.push(module.to_string());
            }
            continue;
        }

        // from package.submodule import ...  (absolute internal imports)
        // e.g. `from nanobot.agent.hook import AgentHook`
        // Convert dots to slashes for path-suffix resolution
        if let Some(rest) = t.strip_prefix("from ") {
            let module = rest.split_whitespace().next().unwrap_or("");
            // Skip stdlib/builtins: no dot means single-word (os, sys, typing, etc.)
            // Multi-component paths (has dot) are likely project-internal
            if module.contains('.') && !module.starts_with('_') {
                let path = module.replace('.', "/");
                out.push(path);
            }
            continue;
        }

        // Stop after class/def definitions start — imports are at top
        if t.starts_with("def ") || t.starts_with("class ") || t.starts_with("if __name__") {
            if !out.is_empty() {
                break;
            }
        }
    }
    out
}

fn extract_go(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_import_block = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("import (") || t == "import(" {
            in_import_block = true;
            continue;
        }
        if in_import_block && t == ")" {
            break;
        }
        // Single import: import "path"
        if let Some(rest) = t.strip_prefix("import \"") {
            let path = rest.trim_end_matches('"').trim_end_matches('"');
            if is_go_project_import(path) {
                out.push(path.to_string());
            }
            continue;
        }
        if in_import_block {
            let path = t.trim_matches('"').trim_matches('`');
            if is_go_project_import(path) {
                out.push(path.to_string());
            }
        }
    }
    out
}

/// True for imports that might be project-internal (has a package path, not stdlib).
/// Go stdlib packages have no `.` in the first component (e.g. `fmt`, `os/exec`).
/// Module imports have a host (e.g. `github.com/owner/repo/pkg`).
fn is_go_project_import(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let first = path.split('/').next().unwrap_or("");
    // Relative (rare in Go but handle it)
    if first == "." || first == ".." {
        return true;
    }
    // Module path: first component contains a dot (github.com, golang.org, etc.)
    first.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_imports() {
        let src = r#"
use crate::router;
use crate::config::Config;
use crate::tracker;
use std::collections::HashMap;

fn main() {}
"#;
        let imports = extract_raw_imports(src, "rs");
        assert!(imports.contains(&"router".to_string()));
        assert!(imports.contains(&"config".to_string()));
        assert!(imports.contains(&"tracker".to_string()));
        // std:: should not appear
        assert!(!imports.iter().any(|i| i == "std"));
    }

    #[test]
    fn test_extract_rust_mod() {
        let src = "mod collect;\nmod format;\nmod store;\n\npub fn run() {}\n";
        let imports = extract_raw_imports(src, "rs");
        assert!(imports.contains(&"collect".to_string()));
        assert!(imports.contains(&"format".to_string()));
        assert!(imports.contains(&"store".to_string()));
    }

    #[test]
    fn test_extract_ts_imports() {
        let src = r#"
import { foo } from './utils';
import Bar from '../components/Bar';
import React from 'react';
"#;
        let imports = extract_raw_imports(src, "ts");
        assert!(imports.contains(&"./utils".to_string()));
        assert!(imports.contains(&"../components/Bar".to_string()));
        // External package — should not appear
        assert!(!imports.iter().any(|i| i == "react"));
    }

    #[test]
    fn test_extract_python_imports() {
        let src = "from .models import User\nfrom .utils import helper\nimport os\n";
        let imports = extract_raw_imports(src, "py");
        assert!(imports.contains(&"models".to_string()));
        assert!(imports.contains(&"utils".to_string()));
        assert!(!imports.contains(&"os".to_string()));
    }
}
