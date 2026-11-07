use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

/// Both paths of `diff --git a/path b/path`. A path may hold spaces, so the
/// split is the ` b/` where both sides agree; a rename takes the last one. Git
/// quotes non-ASCII paths, and the quotes are not part of the name.
fn diff_header_paths(line: &str) -> (String, Option<String>) {
    let rest = line
        .strip_prefix("diff --git ")
        .unwrap_or(line)
        .replace('"', "");
    let sides = |i: usize| {
        let (a, b) = (&rest[..i], &rest[i + 1..]);
        (
            a.strip_prefix("a/").unwrap_or(a),
            b.strip_prefix("b/").unwrap_or(b),
        )
    };
    let same = rest
        .match_indices(" b/")
        .map(|(i, _)| sides(i))
        .find(|(a, b)| a == b);
    let (old, new) = same
        .or_else(|| rest.rfind(" b/").map(sides))
        .unwrap_or_else(|| {
            let mut parts = rest.split_whitespace();
            (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
        });
    (new.to_string(), (old != new).then(|| old.to_string()))
}

impl ParseHandler {
    pub(crate) fn handle_git_diff(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git diff output
        let diff = Self::parse_git_diff(&input)?;

        // No hunks in non-empty input is a layout this parser can't read, never
        // an empty diff: `git -C x diff --numstat` gets past the flag table.
        if diff.files.is_empty() && !input.trim().is_empty() {
            return Self::emit_unparsed(&input, "git-diff", ctx);
        }

        // Format output based on the requested format. `git show` opens
        // with the commit itself, which the diff grammar has no place for.
        let mut output = Self::format_git_diff(&diff, ctx.format);
        if !matches!(
            ctx.format,
            OutputFormat::Json | OutputFormat::Csv | OutputFormat::Tsv
        ) {
            if let Some(header) = commit_header(&input) {
                output = format!("{}\n{}", header, output);
            }
        }

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("git-diff")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(diff.files.len())
                .with_extra("Files changed", diff.files.len().to_string())
                .with_extra("Insertions", diff.total_additions.to_string())
                .with_extra("Deletions", diff.total_deletions.to_string());
            stats.print();
        }

        crate::parse_out::emit(&output);

        Ok(())
    }

    pub(crate) fn parse_git_diff(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        let mut current_file: Option<GitDiffEntry> = None;
        let mut in_hunk = false;
        let mut current_hunk: Option<GitDiffHunk> = None;

        // Detect --stat format: lines like " path/file.rs | 10 +++---"
        // Must NOT match diff content lines (which start with +/- and may contain " | ")
        // A patch is never a stat table, even if a context line looks like one.
        let is_patch = input.lines().any(|l| l.starts_with("diff --git "));
        let is_stat = !is_patch
            && input.lines().any(|l| {
                let trimmed = l.trim();
                // Stat lines: " file.rs | 10 +++---" or " file.rs | Bin 0 -> 123"
                // They don't start with diff/---/+++/@@/+/- (diff content markers)
                trimmed.contains(" | ")
                    && !trimmed.starts_with("diff ")
                    && !trimmed.starts_with("---")
                    && !trimmed.starts_with("+++")
                    && !trimmed.starts_with("@@")
                    && !trimmed.starts_with('+')
                    && !trimmed.starts_with('-')
                    && (trimmed.ends_with('+')
                        || trimmed.ends_with('-')
                        || trimmed.contains("Bin ")
                        || trimmed.contains("insertions")
                        || trimmed.contains("deletions"))
            });
        if is_stat {
            return Self::parse_git_diff_stat(input);
        }

        for line in input.lines() {
            // Detect diff header for a new file
            if line.starts_with("diff --git ") {
                // Flush current hunk into current file
                if let (Some(ref mut file), Some(hunk)) = (&mut current_file, current_hunk.take()) {
                    file.hunks.push(hunk);
                }
                // Save the previous file if any
                if let Some(file) = current_file.take() {
                    diff.files.push(file);
                }

                let (path, new_path) = diff_header_paths(line);

                current_file = Some(GitDiffEntry {
                    path,
                    new_path,
                    change_type: "M".to_string(), // Default, will be updated
                    additions: 0,
                    deletions: 0,
                    is_binary: false,
                    mode: None,
                    stat_total: None,
                    hunks: Vec::new(),
                });
                in_hunk = false;
                continue;
            }

            // Detect new file mode (addition)
            if line.starts_with("new file mode ") || line.starts_with("new file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "A".to_string();
                }
                continue;
            }

            // Detect deleted file mode
            if line.starts_with("deleted file mode ") || line.starts_with("deleted file ") {
                if let Some(ref mut file) = current_file {
                    file.change_type = "D".to_string();
                }
                continue;
            }

            // Detect rename from
            if line.starts_with("rename from ") {
                if let Some(ref mut file) = current_file {
                    file.new_path =
                        Some(line.strip_prefix("rename from ").unwrap_or("").to_string());
                    file.change_type = "R".to_string();
                }
                continue;
            }

            // Detect rename to
            if line.starts_with("rename to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("rename to ").unwrap_or("").to_string();
                }
                continue;
            }

            // Detect copy from
            if line.starts_with("copy from ") {
                if let Some(ref mut file) = current_file {
                    file.new_path = Some(line.strip_prefix("copy from ").unwrap_or("").to_string());
                    file.change_type = "C".to_string();
                }
                continue;
            }

            // Detect copy to
            if line.starts_with("copy to ") {
                if let Some(ref mut file) = current_file {
                    file.path = line.strip_prefix("copy to ").unwrap_or("").to_string();
                }
                continue;
            }

            if let Some(mode) = line.strip_prefix("old mode ") {
                if let Some(ref mut file) = current_file {
                    file.mode = Some(mode.to_string());
                }
                continue;
            }
            if let Some(mode) = line.strip_prefix("new mode ") {
                if let Some(ref mut file) = current_file {
                    file.mode = file.mode.take().map(|old| format!("{old} -> {mode}"));
                }
                continue;
            }

            // Detect binary file
            if line.starts_with("Binary files ") || line == "GIT binary patch" {
                if let Some(ref mut file) = current_file {
                    file.is_binary = true;
                }
                continue;
            }

            // Detect hunk header "@@ -start,count +start,count @@"
            if line.starts_with("@@ ") {
                // Flush previous hunk into current file
                if let (Some(ref mut file), Some(hunk)) = (&mut current_file, current_hunk.take()) {
                    file.hunks.push(hunk);
                }
                current_hunk = Some(GitDiffHunk {
                    header: line.to_string(),
                    lines: Vec::new(),
                });
                in_hunk = true;
                continue;
            }

            // Headers end at the first @@: `-- comment` removed reads `--- comment`.
            if in_hunk {
                if let Some(ref mut file) = current_file {
                    if line.starts_with('+') {
                        file.additions += 1;
                    } else if line.starts_with('-') {
                        file.deletions += 1;
                    }
                }
                // Context never reaches the output.
                if !line.starts_with(' ') {
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(line.to_string());
                    }
                }
                continue;
            }

            // Handle "--- a/path" and "+++ b/path" to confirm paths
            if line.starts_with("--- ") {
                // Could be "--- a/path" or "--- /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "A".to_string();
                    }
                }
            }
            if line.starts_with("+++ ") {
                // Could be "+++ b/path" or "+++ /dev/null"
                if line.contains("/dev/null") {
                    if let Some(ref mut file) = current_file {
                        file.change_type = "D".to_string();
                    }
                }
            }
        }

        // Flush the last hunk into the last file
        if let (Some(ref mut file), Some(hunk)) = (&mut current_file, current_hunk.take()) {
            file.hunks.push(hunk);
        }

        // Don't forget the last file
        if let Some(file) = current_file {
            diff.files.push(file);
        }

        // Set total files count before any truncation
        diff.total_files = diff.files.len();
        diff.files_shown = diff.files.len();

        // Calculate totals
        for file in &diff.files {
            diff.total_additions += file.additions;
            diff.total_deletions += file.deletions;
        }

        // Check if empty
        diff.is_empty = diff.files.is_empty();

        Ok(diff)
    }

    /// Parse git diff --stat format (e.g. "src/main.rs | 10 +++---").
    fn parse_git_diff_stat(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        for line in input.lines() {
            let trimmed = line.trim();
            // Summary line: "N file(s) changed, N insertion(s), N deletion(s)"
            if trimmed.contains("file") && trimmed.contains("changed") {
                for part in trimmed.split(',') {
                    let p = part.trim();
                    if p.contains("insertion") {
                        diff.total_additions = p
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(0);
                    } else if p.contains("deletion") {
                        diff.total_deletions = p
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(0);
                    }
                }
                continue;
            }
            // File line: " path | N +++---" or " path | Bin X -> Y bytes"
            if let Some(pipe_pos) = trimmed.find(" | ") {
                let path = trimmed[..pipe_pos].trim().to_string();
                let rest = trimmed[pipe_pos + 3..].trim();
                let is_binary = rest.starts_with("Bin ");
                let total = rest.split_whitespace().next().and_then(|n| n.parse().ok());
                diff.files.push(GitDiffEntry {
                    path,
                    new_path: None,
                    change_type: String::new(),
                    is_binary,
                    stat_total: if is_binary { None } else { total },
                    ..Default::default()
                });
            }
        }
        diff.total_files = diff.files.len();
        diff.files_shown = diff.files.len();
        diff.is_empty = diff.files.is_empty();
        Ok(diff)
    }
}

/// `commit <hash>` / Author / Date / indented message, up to the first diff or
/// stat line, as `<short hash> · <author> · <date>` plus the message.
fn commit_header(input: &str) -> Option<String> {
    let mut lines = input.lines();
    let hash = lines
        .next()?
        .strip_prefix("commit ")?
        .split_whitespace()
        .next()?;
    let (mut author, mut date, mut message) = ("", "", Vec::new());
    for line in lines {
        if line.starts_with("diff --git") || (line.starts_with(' ') && line.contains(" | ")) {
            break;
        }
        if let Some(a) = line.strip_prefix("Author:") {
            author = a.split(" <").next().unwrap_or(a).trim();
        } else if let Some(d) = line.strip_prefix("Date:") {
            date = d.trim();
        } else if let Some(m) = line.strip_prefix("    ") {
            message.push(m);
        } else if line.trim().is_empty() && !message.is_empty() {
            message.push("");
        }
    }
    while message.last() == Some(&"") {
        message.pop();
    }
    let short = &hash[..hash.len().min(7)];
    Some(format!(
        "{} · {} · {}\n{}\n",
        short,
        author,
        date,
        message.join("\n")
    ))
}
