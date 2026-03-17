use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Handle the git-diff subcommand.
    pub(crate) fn handle_git_diff(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git diff output
        let diff = Self::parse_git_diff(&input)?;

        // Format output based on the requested format
        let output = Self::format_git_diff(&diff, ctx.format);

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

        print!("{}", output);

        Ok(())
    }

    /// Parse git diff output into structured data.
    pub(crate) fn parse_git_diff(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        let mut current_file: Option<GitDiffEntry> = None;
        let mut in_hunk = false;

        // Detect --stat format: lines contain " | " with +/- counts
        let is_stat = input.lines().any(|l| l.contains(" | ") && (l.contains('+') || l.contains('-') || l.contains("Bin ")));
        if is_stat {
            return Self::parse_git_diff_stat(input);
        }

        for line in input.lines() {
            // Detect diff header for a new file
            if line.starts_with("diff --git ") {
                // Save the previous file if any
                if let Some(file) = current_file.take() {
                    diff.files.push(file);
                }

                // Parse the file path from "diff --git a/path b/path"
                let parts: Vec<&str> = line.split_whitespace().collect();
                let (path, new_path) = if parts.len() >= 3 {
                    // Format: "diff --git a/new b/new"
                    let a_path = parts
                        .get(2)
                        .unwrap_or(&"")
                        .strip_prefix("a/")
                        .unwrap_or(parts.get(2).unwrap_or(&""));
                    let b_path = parts
                        .get(3)
                        .unwrap_or(&"")
                        .strip_prefix("b/")
                        .unwrap_or(parts.get(3).unwrap_or(&""));
                    if a_path != b_path {
                        (b_path.to_string(), Some(a_path.to_string()))
                    } else {
                        (b_path.to_string(), None)
                    }
                } else {
                    (String::new(), None)
                };

                current_file = Some(GitDiffEntry {
                    path,
                    new_path,
                    change_type: "M".to_string(), // Default, will be updated
                    additions: 0,
                    deletions: 0,
                    is_binary: false,
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

            // Detect binary file
            if line.contains("Binary files ") && line.contains(" differ") {
                if let Some(ref mut file) = current_file {
                    file.is_binary = true;
                }
                continue;
            }

            // Detect hunk header "@@ -start,count +start,count @@"
            if line.starts_with("@@ ") {
                in_hunk = true;
                continue;
            }

            // Count additions and deletions in hunks
            if in_hunk {
                if let Some(ref mut file) = current_file {
                    if line.starts_with('+') && !line.starts_with("+++") {
                        file.additions += 1;
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        file.deletions += 1;
                    }
                }
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

    /// Parse git diff --stat format.
    /// Lines like: " src/main.rs | 10 +++---"
    /// Summary: " 3 files changed, 5 insertions(+), 3 deletions(-)"
    fn parse_git_diff_stat(input: &str) -> CommandResult<GitDiff> {
        let mut diff = GitDiff::default();
        for line in input.lines() {
            let trimmed = line.trim();
            // Summary line: "N file(s) changed, N insertion(s), N deletion(s)"
            if trimmed.contains("file") && trimmed.contains("changed") {
                for part in trimmed.split(',') {
                    let p = part.trim();
                    if p.contains("insertion") {
                        diff.total_additions = p.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
                    } else if p.contains("deletion") {
                        diff.total_deletions = p.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
                    }
                }
                continue;
            }
            // File line: " path | N +++---" or " path | Bin X -> Y bytes"
            if let Some(pipe_pos) = trimmed.find(" | ") {
                let path = trimmed[..pipe_pos].trim().to_string();
                let rest = trimmed[pipe_pos + 3..].trim();
                let is_binary = rest.starts_with("Bin ");
                let additions = rest.chars().filter(|c| *c == '+').count();
                let deletions = rest.chars().filter(|c| *c == '-').count();
                let change_type = if is_binary { "M" } else if deletions == 0 && additions > 0 { "A" } else if additions == 0 && deletions > 0 { "D" } else { "M" };
                diff.files.push(GitDiffEntry {
                    path, new_path: None, change_type: change_type.to_string(),
                    additions, deletions, is_binary,
                });
            }
        }
        diff.total_files = diff.files.len();
        diff.files_shown = diff.files.len();
        diff.is_empty = diff.files.is_empty();
        Ok(diff)
    }

    /// Default maximum number of files to show in diff output before truncation.
    #[allow(dead_code)]
    const DEFAULT_MAX_DIFF_FILES: usize = 50;

    /// Truncate diff files list if it exceeds the limit.
    #[allow(dead_code)]
    pub(crate) fn truncate_diff(diff: &mut GitDiff, max_files: usize) {
        if diff.files.len() > max_files {
            diff.is_truncated = true;
            diff.files_shown = max_files;
            diff.files.truncate(max_files);
        }
    }

    /// Format git diff for output.
    pub(crate) fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_diff_json(diff),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_diff_compact(diff),
            OutputFormat::Raw => Self::format_git_diff_raw(diff),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_git_diff_compact(diff),
        }
    }

    /// Format git diff as JSON.
    pub(crate) fn format_git_diff_json(diff: &GitDiff) -> String {
        serde_json::json!({
            "is_empty": diff.is_empty,
            "is_truncated": diff.is_truncated,
            "total_files": diff.total_files,
            "files_shown": diff.files_shown,
            "files": diff.files.iter().map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "new_path": file.new_path,
                    "change_type": file.change_type,
                    "additions": file.additions,
                    "deletions": file.deletions,
                    "is_binary": file.is_binary,
                })
            }).collect::<Vec<_>>(),
            "total_additions": diff.total_additions,
            "total_deletions": diff.total_deletions,
            "truncation": if diff.is_truncated {
                Some(serde_json::json!({
                    "hidden_files": diff.total_files.saturating_sub(diff.files_shown),
                    "message": format!("Output truncated: showing {} of {} files", diff.files_shown, diff.total_files),
                }))
            } else {
                None
            },
        })
        .to_string()
    }

    /// Format git diff in compact format.
    pub(crate) fn format_git_diff_compact(diff: &GitDiff) -> String {
        let mut output = String::new();

        if diff.is_empty {
            output.push_str("diff: empty\n");
            return output;
        }

        // Show file count with truncation info if applicable
        if diff.is_truncated {
            output.push_str(&format!(
                "files ({}/{} shown):\n",
                diff.files_shown, diff.total_files
            ));
        } else {
            output.push_str(&format!("files ({}):\n", diff.files.len()));
        }

        for file in &diff.files {
            let change_indicator = match file.change_type.as_str() {
                "A" => "+",
                "D" => "-",
                "R" => "R",
                "C" => "C",
                _ => "M",
            };

            if let Some(ref new_path) = file.new_path {
                output.push_str(&format!(
                    "  {} {} -> {} (+{}/-{})\n",
                    change_indicator, new_path, file.path, file.additions, file.deletions
                ));
            } else {
                output.push_str(&format!(
                    "  {} {} (+{}/-{})\n",
                    change_indicator, file.path, file.additions, file.deletions
                ));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("  ... {} more file(s) not shown\n", hidden));
        }

        output.push_str(&format!(
            "summary: +{} -{}\n",
            diff.total_additions, diff.total_deletions
        ));

        output
    }

    /// Format git diff as raw output (just the files).
    pub(crate) fn format_git_diff_raw(diff: &GitDiff) -> String {
        let mut output = String::new();

        for file in &diff.files {
            if let Some(ref new_path) = file.new_path {
                output.push_str(&format!(
                    "{} {} -> {}\n",
                    file.change_type, new_path, file.path
                ));
            } else {
                output.push_str(&format!("{} {}\n", file.change_type, file.path));
            }
        }

        // Show truncation warning if applicable
        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            output.push_str(&format!("... {} more file(s) truncated\n", hidden));
        }

        output
    }
}
