use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

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

    /// Threshold for switching to grouped-by-directory mode.
    const DIR_GROUP_THRESHOLD: usize = 20;

    /// Format a list of entries with a cap, showing truncation hint.
    /// When entries exceed DIR_GROUP_THRESHOLD, groups files by directory.
    fn format_entries_capped(entries: &[GitStatusEntry], max: usize, output: &mut String) {
        if entries.len() > Self::DIR_GROUP_THRESHOLD {
            Self::format_entries_grouped(entries, output);
        } else {
            Self::format_entries_listed(entries, max, output);
        }
    }

    /// List entries individually with a cap.
    fn format_entries_listed(entries: &[GitStatusEntry], max: usize, output: &mut String) {
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

    /// Group entries by parent directory, showing status counts per dir.
    /// Example: `src/router/handlers/ (M:5 A:2 D:1)`
    fn format_entries_grouped(entries: &[GitStatusEntry], output: &mut String) {
        use std::collections::BTreeMap;

        // Group by parent directory
        let mut dirs: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for entry in entries {
            let dir = match entry.path.rfind('/') {
                Some(pos) => &entry.path[..=pos],
                None => "./",
            };
            dirs.entry(dir.to_string()).or_default().push(&entry.status);
        }

        for (dir, statuses) in &dirs {
            // Count each status type
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            for s in statuses {
                *counts.entry(s).or_insert(0) += 1;
            }

            let count_str: Vec<String> = counts
                .iter()
                .map(|(status, count)| {
                    if *count == 1 {
                        status.to_string()
                    } else {
                        format!("{}:{}", status, count)
                    }
                })
                .collect();

            output.push_str(&format!("  {} ({})\n", dir, count_str.join(" ")));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(status: &str, path: &str) -> GitStatusEntry {
        GitStatusEntry {
            status: status.to_string(),
            path: path.to_string(),
            new_path: None,
        }
    }

    fn make_entries(status: &str, paths: &[&str]) -> Vec<GitStatusEntry> {
        paths.iter().map(|p| make_entry(status, p)).collect()
    }

    #[test]
    fn test_small_list_shows_individual_files() {
        let entries = make_entries("M", &["src/main.rs", "src/cli.rs", "src/config.rs"]);
        let mut output = String::new();
        ParseHandler::format_entries_capped(&entries, 15, &mut output);
        assert!(output.contains("M src/main.rs"));
        assert!(output.contains("M src/cli.rs"));
        assert!(output.contains("M src/config.rs"));
        assert!(!output.contains("(M:"));
    }

    #[test]
    fn test_large_list_groups_by_directory() {
        // 25 files across 3 directories
        let mut entries = Vec::new();
        for i in 0..10 {
            entries.push(make_entry("M", &format!("src/router/file{}.rs", i)));
        }
        for i in 0..8 {
            entries.push(make_entry("M", &format!("src/formatter/file{}.rs", i)));
        }
        for i in 0..4 {
            entries.push(make_entry("A", &format!("tests/test{}.rs", i)));
        }
        for i in 0..3 {
            entries.push(make_entry("D", &format!("src/router/old{}.rs", i)));
        }

        let mut output = String::new();
        ParseHandler::format_entries_capped(&entries, 15, &mut output);

        // Should group by directory
        assert!(output.contains("src/formatter/"));
        assert!(output.contains("src/router/"));
        assert!(output.contains("tests/"));
        // Should show counts
        assert!(output.contains("M:"));
        // Should NOT list individual files
        assert!(!output.contains("file0.rs"));
    }

    #[test]
    fn test_grouped_shows_status_counts() {
        let mut entries = Vec::new();
        for i in 0..15 {
            entries.push(make_entry("M", &format!("src/handlers/h{}.rs", i)));
        }
        for i in 0..6 {
            entries.push(make_entry("A", &format!("src/handlers/new{}.rs", i)));
        }

        let mut output = String::new();
        ParseHandler::format_entries_capped(&entries, 15, &mut output);

        // Should contain both status types with counts
        assert!(output.contains("M:15"));
        assert!(output.contains("A:6"));
    }

    #[test]
    fn test_grouped_single_status_no_count() {
        let mut entries = Vec::new();
        for i in 0..21 {
            entries.push(make_entry("M", &format!("src/dir{}/file.rs", i)));
        }

        let mut output = String::new();
        ParseHandler::format_entries_capped(&entries, 15, &mut output);

        // Single file per dir should show just "M" not "M:1"
        assert!(output.contains("(M)"));
    }

    #[test]
    fn test_root_files_grouped_as_dot() {
        let mut entries = Vec::new();
        for i in 0..21 {
            entries.push(make_entry("M", &format!("file{}.txt", i)));
        }

        let mut output = String::new();
        ParseHandler::format_entries_capped(&entries, 15, &mut output);

        // Root files should show under "./"
        assert!(output.contains("./"));
        assert!(output.contains("M:21"));
    }
}
