//! Git-related schema formatting methods for the Compact formatter.

use super::CompactFormatter;

#[allow(dead_code)]
impl CompactFormatter {
    /// Format a GitStatusSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::{GitStatusSchema, GitFileEntry};
    /// let mut status = GitStatusSchema::new("main");
    /// status.is_clean = true;
    /// let output = CompactFormatter::format_git_status(&status);
    /// assert!(output.contains("branch: main"));
    /// assert!(output.contains("status: clean"));
    /// ```
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        let mut output = String::new();

        // Branch info
        if !status.branch.is_empty() {
            output.push_str(&Self::format_branch_with_tracking(
                &status.branch,
                status.ahead.unwrap_or(0),
                status.behind.unwrap_or(0),
            ));
        }

        // Clean state
        if status.is_clean {
            output.push_str(&Self::format_clean());
            return output;
        }

        // Summary line with counts
        output.push_str(&Self::format_counts(
            "counts",
            &[
                ("staged", status.counts.staged),
                ("unstaged", status.counts.unstaged),
                ("untracked", status.counts.untracked),
                ("unmerged", status.counts.unmerged),
            ],
        ));

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&Self::format_section_header(
                "staged",
                Some(status.staged.len()),
            ));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        // Unstaged changes
        if !status.unstaged.is_empty() {
            output.push_str(&Self::format_section_header(
                "unstaged",
                Some(status.unstaged.len()),
            ));
            for entry in &status.unstaged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        // Untracked files
        if !status.untracked.is_empty() {
            output.push_str(&Self::format_section_header(
                "untracked",
                Some(status.untracked.len()),
            ));
            for entry in &status.untracked {
                output.push_str(&Self::format_item(&entry.status, &entry.path));
            }
        }

        // Unmerged files
        if !status.unmerged.is_empty() {
            output.push_str(&Self::format_section_header(
                "unmerged",
                Some(status.unmerged.len()),
            ));
            for entry in &status.unmerged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&Self::format_item_renamed(
                        &entry.status,
                        old_path,
                        &entry.path,
                    ));
                } else {
                    output.push_str(&Self::format_item(&entry.status, &entry.path));
                }
            }
        }

        output
    }

    /// Format a GitDiffSchema into compact output.
    ///
    /// # Example
    ///
    /// ```
    /// use tars_cli::formatter::CompactFormatter;
    /// use tars_cli::schema::GitDiffSchema;
    /// let diff = GitDiffSchema::new();
    /// let output = CompactFormatter::format_git_diff(&diff);
    /// assert!(output.contains("diff: empty"));
    /// ```
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        if diff.is_empty {
            return "diff: empty\n".to_string();
        }

        let mut output = String::new();

        // List files with changes
        for file in &diff.files {
            output.push_str(&Self::format_diff_file(
                &file.path,
                &file.change_type,
                file.additions,
                file.deletions,
            ));
        }

        // Summary
        output.push_str(&Self::format_diff_summary(
            diff.counts.total_files,
            diff.total_additions,
            diff.total_deletions,
        ));

        // Truncation warning if needed
        if diff.is_truncated {
            output.push_str(&Self::format_truncated(
                diff.counts.files_shown,
                diff.counts.total_files,
            ));
        }

        output
    }
}
