//! Parser for `cargo fmt --check` output.
//!
//! Collapses the per-file diff blocks (`Diff in <path>:N:` + context lines)
//! into a file list with diff counts. `cargo fmt` without `--check` prints
//! nothing on success — empty input renders "fmt: clean".

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Bound the file list on repo-wide formatting drift.
const MAX_FILES_SHOWN: usize = 20;

impl ParseHandler {
    pub(crate) fn handle_fmt(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let files = parse_fmt(&input);

        let output = match ctx.format {
            OutputFormat::Json => format_fmt_json(&files),
            _ => format_fmt_compact(&files, &input),
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("fmt")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(files.len())
                .print();
        }
        Ok(())
    }
}

struct FmtFile {
    path: String,
    diffs: usize,
}

fn parse_fmt(input: &str) -> Vec<FmtFile> {
    let cwd_prefix = std::env::current_dir()
        .map(|d| format!("{}/", crate::path_display::display_path(&d)))
        .unwrap_or_default();

    let mut files: Vec<FmtFile> = Vec::new();
    for line in input.lines() {
        // "Diff in /abs/path/src/main.rs:5:" (also "… at line 5:" on newer rustfmt)
        let Some(rest) = line.strip_prefix("Diff in ") else {
            continue;
        };
        let path_part = rest
            .split_once(" at line ")
            .map(|(p, _)| p)
            .unwrap_or_else(|| rest.rsplitn(3, ':').last().unwrap_or(rest));
        let mut path = crate::path_display::normalize(path_part.trim());
        if !cwd_prefix.is_empty() {
            if let Some(stripped) = path.strip_prefix(&cwd_prefix) {
                path = stripped.to_string();
            }
        }
        match files.iter_mut().find(|f| f.path == path) {
            Some(f) => f.diffs += 1,
            None => files.push(FmtFile { path, diffs: 1 }),
        }
    }
    files
}

fn format_fmt_compact(files: &[FmtFile], input: &str) -> String {
    if files.is_empty() {
        // No diff blocks: clean run, or unrecognized output (errors) — passthrough.
        if input.trim().is_empty() {
            return "fmt: clean\n".to_string();
        }
        return input.to_string();
    }

    let unit = if files.len() == 1 {
        "file needs"
    } else {
        "files need"
    };
    let mut out = format!("fmt: {} {} formatting\n", files.len(), unit);
    for f in files.iter().take(MAX_FILES_SHOWN) {
        let diffs = if f.diffs == 1 { "diff" } else { "diffs" };
        out.push_str(&format!("  {} ({} {})\n", f.path, f.diffs, diffs));
    }
    if files.len() > MAX_FILES_SHOWN {
        out.push_str(&format!("  ...+{} more\n", files.len() - MAX_FILES_SHOWN));
    }
    out.push_str("fix: cargo fmt\n");
    out
}

fn format_fmt_json(files: &[FmtFile]) -> String {
    let files_json: Vec<serde_json::Value> = files
        .iter()
        .map(|f| serde_json::json!({ "path": f.path, "diffs": f.diffs }))
        .collect();
    serde_json::json!({
        "files": files_json,
        "total_files": files.len(),
        "fix": "cargo fmt",
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_basic() {
        let input = "\
Diff in /abs/path/src/main.rs:5:
     fn main() {
-    println!(\"x\");
+    println!( \"x\" );
     }
Diff in /abs/path/src/lib.rs:10:
-fn  f() {}
+fn f() {}";

        let files = parse_fmt(input);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "/abs/path/src/main.rs");
        assert_eq!(files[0].diffs, 1);

        let out = format_fmt_compact(&files, input);
        assert!(out.starts_with("fmt: 2 files need formatting\n"));
        assert!(out.contains("  /abs/path/src/main.rs (1 diff)\n"));
        assert!(out.ends_with("fix: cargo fmt\n"));
    }

    #[test]
    fn test_fmt_multiple_diffs_same_file() {
        let input = "\
Diff in /x/src/main.rs:5:
-a
+b
Diff in /x/src/main.rs:42:
-c
+d";
        let files = parse_fmt(input);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].diffs, 2);
        let out = format_fmt_compact(&files, input);
        assert!(out.contains("fmt: 1 file needs formatting\n"));
        assert!(out.contains("(2 diffs)"));
    }

    #[test]
    fn test_fmt_strips_cwd_prefix() {
        let cwd = std::env::current_dir().unwrap();
        let input = format!("Diff in {}/src/main.rs:5:\n-a\n+b", cwd.display());
        let files = parse_fmt(&input);
        assert_eq!(files[0].path, "src/main.rs");
    }

    #[test]
    fn test_fmt_at_line_variant() {
        let input = "Diff in /x/src/lib.rs at line 10:\n-a\n+b";
        let files = parse_fmt(input);
        assert_eq!(files[0].path, "/x/src/lib.rs");
        assert_eq!(files[0].diffs, 1);
    }

    #[test]
    fn test_fmt_empty_is_clean() {
        let files = parse_fmt("");
        assert!(files.is_empty());
        assert_eq!(format_fmt_compact(&files, ""), "fmt: clean\n");
    }

    #[test]
    fn test_fmt_caps_file_list() {
        let input: String = (0..25)
            .map(|i| format!("Diff in /x/src/f{}.rs:1:\n-a\n+b\n", i))
            .collect();
        let files = parse_fmt(&input);
        assert_eq!(files.len(), 25);
        let out = format_fmt_compact(&files, &input);
        assert!(out.contains("fmt: 25 files need formatting"));
        assert!(out.contains("...+5 more"));
    }

    #[test]
    fn test_fmt_unrecognized_passthrough() {
        let input = "error: could not find `Cargo.toml`\n";
        let files = parse_fmt(input);
        assert!(files.is_empty());
        assert_eq!(format_fmt_compact(&files, input), input);
    }

    #[test]
    fn test_fmt_json() {
        let input = "Diff in /x/a.rs:1:\n-a\n+b\nDiff in /x/a.rs:9:\n-c\n+d";
        let files = parse_fmt(input);
        let json = format_fmt_json(&files);
        assert!(json.contains("\"total_files\":1"));
        assert!(json.contains("\"diffs\":2"));
        assert!(json.contains("\"fix\":\"cargo fmt\""));
    }
}
