//! Parser for `git commit` output.
//!
//! Collapses the per-file `create/delete mode` and `rename` lines into
//! counts — the big win is initial commits with hundreds of `create mode`
//! lines. Keeps the header and summary numbers exactly faithful.

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_git_commit(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let result = parse_git_commit(&input);

        let output = match ctx.format {
            OutputFormat::Json => format_git_commit_json(&result),
            _ => format_git_commit_compact(&result, &input),
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("git-commit")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

#[derive(Default)]
struct GitCommitResult {
    branch: String,
    hash: String,
    message: String,
    root_commit: bool,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
    created: usize,
    deleted: usize,
    renamed: usize,
    has_header: bool,
    has_summary: bool,
    /// Hook output and other unrecognized lines — kept verbatim.
    extras: Vec<String>,
}

fn parse_git_commit(input: &str) -> GitCommitResult {
    let mut r = GitCommitResult::default();

    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        // Header: "[main abc1234] msg", "[main (root-commit) abc1234] msg",
        // "[detached HEAD abc1234] msg"
        if !r.has_header && t.starts_with('[') {
            if let Some(close) = t.find(']') {
                let inside = &t[1..close];
                r.message = t[close + 1..].trim().to_string();
                r.root_commit = inside.contains("(root-commit)");
                let inside = inside.replace("(root-commit)", " ");
                let mut tokens: Vec<&str> = inside.split_whitespace().collect();
                if let Some(hash) = tokens.pop() {
                    r.hash = hash.to_string();
                }
                r.branch = tokens.join(" ");
                r.has_header = true;
                continue;
            }
        }

        // Summary: "3 files changed, 45 insertions(+), 2 deletions(-)"
        if !r.has_summary && (t.contains("file changed") || t.contains("files changed")) {
            for part in t.split(',') {
                let part = part.trim();
                let n: usize = part
                    .split_whitespace()
                    .next()
                    .and_then(|w| w.parse().ok())
                    .unwrap_or(0);
                if part.contains("file") {
                    r.files_changed = n;
                } else if part.contains("insertion") {
                    r.insertions = n;
                } else if part.contains("deletion") {
                    r.deletions = n;
                }
            }
            r.has_summary = true;
            continue;
        }

        if t.starts_with("create mode ") {
            r.created += 1;
            continue;
        }
        if t.starts_with("delete mode ") {
            r.deleted += 1;
            continue;
        }
        if t.starts_with("rename ") {
            r.renamed += 1;
            continue;
        }

        r.extras.push(t.to_string());
    }

    r
}

fn format_git_commit_compact(r: &GitCommitResult, input: &str) -> String {
    // No commit header found — unrecognized shape, pass through untouched.
    if !r.has_header {
        return input.to_string();
    }

    let mut out = String::new();
    let marker = if r.root_commit { " (root-commit)" } else { "" };
    out.push_str(&format!(
        "[{}{} {}] {}\n",
        r.branch, marker, r.hash, r.message
    ));

    if r.has_summary {
        let unit = if r.files_changed == 1 {
            "file"
        } else {
            "files"
        };
        out.push_str(&format!("{} {} changed", r.files_changed, unit));
        if r.insertions > 0 {
            out.push_str(&format!(", +{}", r.insertions));
        }
        if r.deletions > 0 {
            let sep = if r.insertions > 0 { " " } else { ", " };
            out.push_str(&format!("{}-{}", sep, r.deletions));
        }

        let mut modes: Vec<String> = Vec::new();
        if r.created > 0 {
            modes.push(format!("created {}", r.created));
        }
        if r.deleted > 0 {
            modes.push(format!("deleted {}", r.deleted));
        }
        if r.renamed > 0 {
            modes.push(format!("renamed {}", r.renamed));
        }
        if !modes.is_empty() {
            out.push_str(&format!(" · {}", modes.join(", ")));
        }
        out.push('\n');
    }

    for extra in &r.extras {
        out.push_str(extra);
        out.push('\n');
    }

    out
}

fn format_git_commit_json(r: &GitCommitResult) -> String {
    serde_json::json!({
        "branch": r.branch,
        "hash": r.hash,
        "message": r.message,
        "files_changed": r.files_changed,
        "insertions": r.insertions,
        "deletions": r.deletions,
        "created": r.created,
        "deleted": r.deleted,
        "renamed": r.renamed,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_commit_basic() {
        let input = "\
[main abc1234] feat: add the thing
 3 files changed, 45 insertions(+), 2 deletions(-)
 create mode 100644 src/foo.rs
 create mode 100644 src/bar.rs
 delete mode 100644 old.rs
 rename src/{a.rs => b.rs} (90%)";

        let r = parse_git_commit(input);
        assert_eq!(r.branch, "main");
        assert_eq!(r.hash, "abc1234");
        assert_eq!(r.message, "feat: add the thing");
        assert_eq!(r.files_changed, 3);
        assert_eq!(r.insertions, 45);
        assert_eq!(r.deletions, 2);
        assert_eq!(r.created, 2);
        assert_eq!(r.deleted, 1);
        assert_eq!(r.renamed, 1);

        let out = format_git_commit_compact(&r, input);
        assert_eq!(
            out,
            "[main abc1234] feat: add the thing\n\
             3 files changed, +45 -2 · created 2, deleted 1, renamed 1\n"
        );
    }

    #[test]
    fn test_git_commit_root_commit() {
        let input = "\
[main (root-commit) abc1234] initial commit
 200 files changed, 9000 insertions(+)
 create mode 100644 a.rs
 create mode 100644 b.rs";

        let r = parse_git_commit(input);
        assert!(r.root_commit);
        assert_eq!(r.branch, "main");
        assert_eq!(r.hash, "abc1234");
        assert_eq!(r.created, 2);

        let out = format_git_commit_compact(&r, input);
        assert!(out.starts_with("[main (root-commit) abc1234] initial commit\n"));
        assert!(out.contains("200 files changed, +9000 · created 2\n"));
    }

    #[test]
    fn test_git_commit_detached_head_and_slash_branch() {
        let input = "[detached HEAD abc1234] fix: hotfix\n 1 file changed, 1 insertion(+)";
        let r = parse_git_commit(input);
        assert_eq!(r.branch, "detached HEAD");
        assert_eq!(r.hash, "abc1234");

        let input2 = "[feat/some-thing def5678] wip\n 1 file changed, 2 deletions(-)";
        let r2 = parse_git_commit(input2);
        assert_eq!(r2.branch, "feat/some-thing");
        let out = format_git_commit_compact(&r2, input2);
        assert!(out.contains("1 file changed, -2\n"));
    }

    #[test]
    fn test_git_commit_no_mode_lines() {
        let input = "[main abc1234] tweak\n 1 file changed, 2 insertions(+), 1 deletion(-)";
        let r = parse_git_commit(input);
        let out = format_git_commit_compact(&r, input);
        assert_eq!(out, "[main abc1234] tweak\n1 file changed, +2 -1\n");
        assert!(!out.contains("·"));
    }

    #[test]
    fn test_git_commit_unrecognized_passthrough() {
        let input = "On branch main\nnothing to commit, working tree clean\n";
        let r = parse_git_commit(input);
        assert!(!r.has_header);
        assert_eq!(format_git_commit_compact(&r, input), input);
    }

    #[test]
    fn test_git_commit_empty() {
        let r = parse_git_commit("");
        assert!(!r.has_header);
        assert_eq!(format_git_commit_compact(&r, ""), "");
    }

    #[test]
    fn test_git_commit_json() {
        let input = "[main abc1234] msg\n 2 files changed, 5 insertions(+)";
        let r = parse_git_commit(input);
        let json = format_git_commit_json(&r);
        assert!(json.contains("\"branch\":\"main\""));
        assert!(json.contains("\"hash\":\"abc1234\""));
        assert!(json.contains("\"files_changed\":2"));
        assert!(json.contains("\"insertions\":5"));
    }
}
