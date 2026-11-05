use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

/// Change lines past which a diff stops printing whole. Its hunks then share
/// `DIFF_BUDGET`, so a big diff costs about what a big-but-whole one does.
const LARGE_DIFF: usize = 500;
const DIFF_BUDGET: usize = 400;

/// Lockfiles and bundles: the first to go, since nobody reviews them by hunk.
fn is_generated(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    crate::ingest::SKIP_FILES.contains(&name)
        || name == "go.sum"
        || [".lock", ".min.js", ".min.css", ".map"]
            .iter()
            .any(|ext| name.ends_with(ext))
}

/// Fair shares of `budget`: a small file keeps everything, and what it leaves
/// goes to the larger ones, so one huge file cannot starve the rest.
fn shares(sizes: &[usize], budget: usize) -> Vec<usize> {
    let mut by_size: Vec<usize> = (0..sizes.len()).collect();
    by_size.sort_by_key(|&i| sizes[i]);
    let mut left = budget;
    let mut out = vec![0; sizes.len()];
    for (done, &i) in by_size.iter().enumerate() {
        out[i] = sizes[i].min(left / (sizes.len() - done));
        left -= out[i];
    }
    out
}

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
                    "mode": file.mode,
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

    fn format_hunk_compressed(hunk: &GitDiffHunk) -> Vec<String> {
        let mut result = Vec::new();
        result.push(hunk.header.clone());

        // Only keep change lines (+/-), drop context lines entirely
        for line in &hunk.lines {
            if line.starts_with('\\') {
                result.push(line.clone());
            } else if line.starts_with('+') || line.starts_with('-') {
                // A changed minified bundle or lockfile blob: keep its ends.
                let (line, cut) = crate::safe_folds::elide_dense(line);
                if cut {
                    crate::parse_out::mark_dropped();
                }
                result.push(line);
            }
        }

        result
    }

    /// `(+0/-0)` alone would read as untouched on a binary or a chmod.
    fn file_label(file: &GitDiffEntry) -> String {
        let indicator = match file.change_type.as_str() {
            "A" => "+",
            "D" => "-",
            "R" => "R",
            "C" => "C",
            _ => "M",
        };
        let name = match file.new_path {
            Some(ref old) => format!("{} -> {}", old, file.path),
            None => file.path.clone(),
        };
        let change = if file.is_binary {
            "binary".to_string()
        } else {
            format!("+{}/-{}", file.additions, file.deletions)
        };
        let mode = match file.mode {
            Some(ref m) => format!(" mode {m}"),
            None => String::new(),
        };
        format!("{indicator} {name} ({change}){mode}")
    }

    fn build_file_summary(diff: &GitDiff) -> String {
        let mut summary = String::from("--- file summary ---\n");
        for file in &diff.files {
            if let Some(n) = file.stat_total {
                summary.push_str(&format!("  {} | {}\n", file.path, n));
                continue;
            }
            summary.push_str(&format!("  {}\n", Self::file_label(file)));
        }
        summary.push_str(&format!(
            "total: +{} -{}\n",
            diff.total_additions, diff.total_deletions
        ));
        summary.push_str("---\n");
        summary
    }

    fn format_git_diff_budgeted(diff: &GitDiff) -> String {
        let sizes: Vec<usize> = diff
            .files
            .iter()
            .map(|f| {
                if is_generated(&f.path) {
                    0
                } else {
                    f.additions + f.deletions + f.hunks.len()
                }
            })
            .collect();
        let budget = shares(&sizes, DIFF_BUDGET);

        let mut output = format!(
            "{} files (+{} -{})\n",
            diff.files.len(),
            diff.total_additions,
            diff.total_deletions,
        );
        for (file, &share) in diff.files.iter().zip(&budget) {
            output.push_str(&format!("\n{}\n", Self::file_label(file)));
            let hunks: Vec<Vec<String>> = file
                .hunks
                .iter()
                .map(Self::format_hunk_compressed)
                .collect();
            let lengths: Vec<usize> = hunks.iter().map(Vec::len).collect();
            let mut cut = 0;
            for (lines, take) in hunks.iter().zip(shares(&lengths, share)) {
                for line in &lines[..take] {
                    output.push_str(&format!("  {}\n", line));
                }
                cut += lines.len() - take;
            }
            if cut > 0 {
                output.push_str(&format!("  [+{} lines cut]\n", cut));
            }
        }
        crate::parse_out::mark_dropped();
        output
    }

    pub(crate) fn format_git_diff_compact(diff: &GitDiff) -> String {
        let mut output = String::new();

        if diff.is_empty {
            output.push_str("diff: empty\n");
            return output;
        }

        // Estimate total change lines (only +/- lines, not context)
        let total_change_lines: usize = diff.files.iter().map(|f| f.additions + f.deletions).sum();

        if total_change_lines > LARGE_DIFF {
            if diff.files.iter().any(|f| !f.hunks.is_empty()) {
                return Self::format_git_diff_budgeted(diff);
            }
            crate::parse_out::mark_dropped();
            return Self::build_file_summary(diff);
        }

        // Show each file with inline stats + compressed hunks
        output.push_str(&format!(
            "{} files (+{} -{})\n",
            diff.files.len(),
            diff.total_additions,
            diff.total_deletions
        ));

        for file in &diff.files {
            if let Some(n) = file.stat_total {
                output.push_str(&format!("{} | {}\n", file.path, n));
                continue;
            }
            output.push_str(&format!("\n{}\n", Self::file_label(file)));

            // Compressed hunks right after the file header
            for hunk in &file.hunks {
                let compressed = Self::format_hunk_compressed(hunk);
                for line in &compressed {
                    output.push_str(&format!("  {}\n", line));
                }
            }
        }

        if diff.is_truncated {
            let hidden = diff.total_files.saturating_sub(diff.files_shown);
            crate::parse_out::mark_dropped();
            output.push_str(&format!("\n... {} more file(s) not shown\n", hidden));
        }

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
            crate::parse_out::mark_dropped();
            output.push_str(&format!("... {} more file(s) truncated\n", hidden));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_give_small_files_everything_and_never_overspend() {
        assert_eq!(shares(&[10, 1000, 30], 400), vec![10, 360, 30]);
        assert_eq!(shares(&[0, 500, 500], 400), vec![0, 200, 200]);
        assert_eq!(shares(&[5, 5], 400), vec![5, 5]);
        assert_eq!(shares(&[], 400), Vec::<usize>::new());
    }

    #[test]
    fn generated_means_the_file_not_a_word_in_its_path() {
        assert!(is_generated("Cargo.lock"));
        assert!(is_generated("web/package-lock.json"));
        assert!(is_generated("dist/app.min.js"));
        assert!(!is_generated("src/lock.rs"));
        assert!(!is_generated("lockfile/parse.rs"));
    }
}
