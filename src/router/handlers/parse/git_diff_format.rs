use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

/// Bytes of hunks a diff may print: about 500 change lines. Fewer print whole.
const DIFF_BUDGET: usize = 24 * 1024;

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

/// Code first; generated files get only what the code leaves.
fn file_budgets(files: &[GitDiffEntry], sizes: &[usize], budget: usize) -> Vec<usize> {
    let of_kind = |generated: bool| -> Vec<usize> {
        files
            .iter()
            .zip(sizes)
            .map(|(f, &size)| {
                let name = f.path.rsplit('/').next().unwrap_or(&f.path);
                size * usize::from(crate::ingest::is_generated(name) == generated)
            })
            .collect()
    };
    let code = shares(&of_kind(false), budget);
    let left = budget - code.iter().sum::<usize>();
    let rest = shares(&of_kind(true), left);
    code.iter().zip(rest).map(|(a, b)| a + b).collect()
}

fn hunk_bytes(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
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

    pub(crate) fn format_git_diff_compact(diff: &GitDiff) -> String {
        if diff.is_empty {
            return String::from("diff: empty\n");
        }

        let rendered: Vec<Vec<Vec<String>>> = diff
            .files
            .iter()
            .map(|f| f.hunks.iter().map(Self::format_hunk_compressed).collect())
            .collect();
        let sizes: Vec<usize> = rendered
            .iter()
            .map(|hunks| hunks.iter().map(|h| hunk_bytes(h)).sum())
            .collect();
        let budgets = file_budgets(&diff.files, &sizes, DIFF_BUDGET);

        let mut output = format!(
            "{} files (+{} -{})\n",
            diff.files.len(),
            diff.total_additions,
            diff.total_deletions
        );

        for ((file, hunks), &budget) in diff.files.iter().zip(&rendered).zip(&budgets) {
            if let Some(n) = file.stat_total {
                output.push_str(&format!("{} | {}\n", file.path, n));
                continue;
            }
            output.push_str(&format!("\n{}\n", Self::file_label(file)));

            let lengths: Vec<usize> = hunks.iter().map(|h| hunk_bytes(h)).collect();
            let mut omitted = 0;
            for (lines, share) in hunks.iter().zip(shares(&lengths, budget)) {
                let kept = crate::passthrough_cut::fit(lines.iter().map(String::as_str), share);
                for line in &lines[..kept] {
                    output.push_str(&format!("  {}\n", line));
                }
                omitted += lines.len() - kept;
            }
            if omitted > 0 {
                crate::parse_out::mark_dropped();
                output.push_str(&format!("  [trs: {omitted} lines omitted]\n"));
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

    fn entry(path: &str) -> GitDiffEntry {
        GitDiffEntry {
            path: path.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn generated_files_only_get_what_code_leaves() {
        let files = [entry("Cargo.lock"), entry("src/a.rs"), entry("src/lock.rs")];
        assert_eq!(file_budgets(&files, &[900, 10, 20], 400), vec![370, 10, 20]);
        assert_eq!(
            file_budgets(&files, &[900, 300, 300], 400),
            vec![0, 200, 200]
        );
    }
}
