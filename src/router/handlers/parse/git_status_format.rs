use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Format git status for output.
    pub(crate) fn format_git_status(status: &GitStatus, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_git_status_json(status),
            OutputFormat::Csv => Self::format_git_status_csv(status),
            OutputFormat::Tsv => Self::format_git_status_tsv(status),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_git_status_compact(status),
            OutputFormat::Raw => Self::format_git_status_raw(status),
        }
    }

    /// Format git status as CSV.
    pub(crate) fn format_git_status_csv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status,path,new_path,section\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{},{},{},staged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{},{},{},unstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{},{},{},untracked\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{},{},{},unmerged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as TSV.
    pub(crate) fn format_git_status_tsv(status: &GitStatus) -> String {
        let mut result = String::new();
        result.push_str("status\tpath\tnew_path\tsection\n");

        for entry in &status.staged {
            result.push_str(&format!(
                "{}\t{}\t{}\tstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unstaged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunstaged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.untracked {
            result.push_str(&format!(
                "{}\t{}\t{}\tuntracked\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        for entry in &status.unmerged {
            result.push_str(&format!(
                "{}\t{}\t{}\tunmerged\n",
                entry.status,
                entry.path,
                entry.new_path.as_deref().unwrap_or(&String::new())
            ));
        }
        result
    }

    /// Format git status as JSON.
    pub(crate) fn format_git_status_json(status: &GitStatus) -> String {
        serde_json::json!({
            "branch": status.branch,
            "is_clean": status.is_clean,
            "ahead": status.ahead,
            "behind": status.behind,
            "staged_count": status.staged_count,
            "unstaged_count": status.unstaged_count,
            "untracked_count": status.untracked_count,
            "unmerged_count": status.unmerged_count,
            "staged": status.staged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "unstaged": status.unstaged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "untracked": status.untracked.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
            "unmerged": status.unmerged.iter().map(|e| serde_json::json!({
                "status": e.status,
                "path": e.path,
                "new_path": e.new_path,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format git status in compact format.
    pub(crate) fn format_git_status_compact(status: &GitStatus) -> String {
        let limits = &crate::config::config().limits;
        let max_files = limits.status_max_files;
        let max_untracked = limits.status_max_untracked;
        let mut output = String::new();

        // Branch info with ahead/behind inline
        if !status.branch.is_empty() {
            let mut branch_line = status.branch.clone();
            let mut markers = Vec::new();
            if let Some(ahead) = status.ahead {
                markers.push(format!("ahead {}", ahead));
            }
            if let Some(behind) = status.behind {
                markers.push(format!("behind {}", behind));
            }
            if !markers.is_empty() {
                branch_line.push_str(&format!(" [{}]", markers.join(", ")));
            }
            output.push_str(&branch_line);
            output.push('\n');
        }

        // Clean state
        if status.is_clean {
            output.push_str("clean\n");
            return output;
        }

        // Staged changes (capped by config)
        if !status.staged.is_empty() {
            output.push_str(&format!("staged ({}):\n", status.staged.len()));
            Self::format_entries_capped(&status.staged, max_files, &mut output);
        }

        // Unstaged changes (capped by config)
        if !status.unstaged.is_empty() {
            output.push_str(&format!("unstaged ({}):\n", status.unstaged.len()));
            Self::format_entries_capped(&status.unstaged, max_files, &mut output);
        }

        // Untracked files (capped by config — separate, usually more aggressive)
        if !status.untracked.is_empty() {
            output.push_str(&format!("untracked ({}):\n", status.untracked.len()));
            Self::format_entries_capped(&status.untracked, max_untracked, &mut output);
        }

        // Unmerged files (always show all — critical info)
        if !status.unmerged.is_empty() {
            output.push_str(&format!("unmerged ({}):\n", status.unmerged.len()));
            for entry in &status.unmerged {
                if let Some(ref new_path) = entry.new_path {
                    output.push_str(&format!(
                        "  {} {} -> {}\n",
                        entry.status, new_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.status, entry.path));
                }
            }
        }

        output
    }

    /// Format a list of entries with a cap, showing truncation hint.
    fn format_entries_capped(entries: &[GitStatusEntry], max: usize, output: &mut String) {
        let show = entries.len().min(max);
        for entry in entries.iter().take(show) {
            if let Some(ref new_path) = entry.new_path {
                output.push_str(&format!(
                    "  {} {} -> {}\n",
                    entry.status, new_path, entry.path
                ));
            } else {
                output.push_str(&format!("  {} {}\n", entry.status, entry.path));
            }
        }
        if entries.len() > max {
            output.push_str(&format!("  ...+{} more\n", entries.len() - max));
        }
    }

    /// Format git status as raw output (just the files).
    pub(crate) fn format_git_status_raw(status: &GitStatus) -> String {
        let mut output = String::new();

        for entry in &status.staged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unstaged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.untracked {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }
        for entry in &status.unmerged {
            output.push_str(&format!("{} {}\n", entry.status, entry.path));
        }

        output
    }
}
