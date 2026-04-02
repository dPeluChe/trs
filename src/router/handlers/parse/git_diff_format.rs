use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn format_git_diff(diff: &GitDiff, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_diff_json(diff),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_diff_compact(diff),
            OutputFormat::Raw => Self::format_git_diff_raw(diff),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_git_diff_compact(diff),
        }
    }

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

    const CONTEXT_COMPRESS_THRESHOLD: usize = 4;
    const LARGE_DIFF_LINE_THRESHOLD: usize = 200;

    /// Compress context block: first 1 + "[...N unchanged...]" + last 1.
    fn compress_context_block(block: &[String]) -> Vec<String> {
        if block.len() <= Self::CONTEXT_COMPRESS_THRESHOLD {
            return block.to_vec();
        }
        let hidden = block.len() - 2;
        vec![
            block[0].clone(),
            format!("  [...{} unchanged lines...]", hidden),
            block[block.len() - 1].clone(),
        ]
    }

    fn format_hunk_compressed(hunk: &GitDiffHunk) -> Vec<String> {
        let mut result = Vec::new();
        result.push(hunk.header.clone());

        let mut context_block: Vec<String> = Vec::new();

        for line in &hunk.lines {
            let is_context =
                line.starts_with(' ') || (!line.starts_with('+') && !line.starts_with('-'));
            if is_context {
                context_block.push(line.clone());
            } else {
                // Flush context block with compression
                if !context_block.is_empty() {
                    result.extend(Self::compress_context_block(&context_block));
                    context_block.clear();
                }
                result.push(line.clone());
            }
        }

        // Flush trailing context block
        if !context_block.is_empty() {
            result.extend(Self::compress_context_block(&context_block));
        }

        result
    }

    fn build_file_summary(diff: &GitDiff) -> String {
        let mut summary = String::from("--- file summary ---\n");
        for file in &diff.files {
            let indicator = match file.change_type.as_str() {
                "A" => "+",
                "D" => "-",
                "R" => "R",
                "C" => "C",
                _ => "M",
            };
            if let Some(ref old_path) = file.new_path {
                summary.push_str(&format!(
                    "  {} {} -> {} (+{}/-{})\n",
                    indicator, old_path, file.path, file.additions, file.deletions
                ));
            } else {
                summary.push_str(&format!(
                    "  {} {} (+{}/-{})\n",
                    indicator, file.path, file.additions, file.deletions
                ));
            }
        }
        summary.push_str(&format!(
            "total: +{} -{}\n",
            diff.total_additions, diff.total_deletions
        ));
        summary.push_str("---\n");
        summary
    }

    pub(crate) fn format_git_diff_compact(diff: &GitDiff) -> String {
        let mut output = String::new();

        if diff.is_empty {
            output.push_str("diff: empty\n");
            return output;
        }

        // Estimate total hunk lines to decide if we should show hunks or just summary
        let total_hunk_lines: usize = diff
            .files
            .iter()
            .flat_map(|f| &f.hunks)
            .map(|h| h.lines.len())
            .sum();
        let show_hunks = total_hunk_lines <= Self::LARGE_DIFF_LINE_THRESHOLD;

        // For large diffs, show only the file summary (stat-style)
        if !show_hunks {
            return Self::build_file_summary(diff);
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

            // Output compressed hunks (only for small diffs)
            for hunk in &file.hunks {
                let compressed = Self::format_hunk_compressed(hunk);
                for line in &compressed {
                    output.push_str(&format!("    {}\n", line));
                }
            }
        }

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
