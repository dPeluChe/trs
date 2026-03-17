//! Schema formatting methods for the Agent formatter.

use super::AgentFormatter;
use super::helpers::{truncate, format_duration};

#[allow(dead_code)]
impl AgentFormatter {
    /// Format a GitStatusSchema into agent-optimized output.
    pub fn format_git_status(status: &crate::schema::GitStatusSchema) -> String {
        let mut output = String::new();
        output.push_str("# Git Status\n\n");

        if !status.branch.is_empty() {
            output.push_str(&format!("- branch: {}\n", status.branch));
            if let Some(ahead) = status.ahead {
                if ahead > 0 {
                    output.push_str(&format!("- ahead: {}\n", ahead));
                }
            }
            if let Some(behind) = status.behind {
                if behind > 0 {
                    output.push_str(&format!("- behind: {}\n", behind));
                }
            }
        }

        if status.is_clean {
            output.push_str("- status: clean\n");
            return output;
        }

        output.push_str("- status: dirty\n");
        output.push_str(&format!("- staged: {}\n", status.counts.staged));
        output.push_str(&format!("- unstaged: {}\n", status.counts.unstaged));
        output.push_str(&format!("- untracked: {}\n", status.counts.untracked));
        output.push_str(&format!("- unmerged: {}\n", status.counts.unmerged));

        if !status.staged.is_empty() {
            output.push_str(&format!("\n## Staged ({})\n", status.staged.len()));
            for entry in &status.staged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        if !status.unstaged.is_empty() {
            output.push_str(&format!("\n## Unstaged ({})\n", status.unstaged.len()));
            for entry in &status.unstaged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        if !status.untracked.is_empty() {
            output.push_str(&format!("\n## Untracked ({})\n", status.untracked.len()));
            for entry in &status.untracked {
                output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
            }
        }

        if !status.unmerged.is_empty() {
            output.push_str(&format!("\n## Unmerged ({})\n", status.unmerged.len()));
            for entry in &status.unmerged {
                if let Some(ref old_path) = entry.old_path {
                    output.push_str(&format!(
                        "  - [{}] {} -> {}\n",
                        entry.status, old_path, entry.path
                    ));
                } else {
                    output.push_str(&format!("  - [{}] {}\n", entry.status, entry.path));
                }
            }
        }

        output
    }

    /// Format a GitDiffSchema into agent-optimized output.
    pub fn format_git_diff(diff: &crate::schema::GitDiffSchema) -> String {
        if diff.is_empty {
            return "# Git Diff\n\n- status: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Git Diff\n\n");

        output.push_str(&format!("- files changed: {}\n", diff.counts.total_files));
        output.push_str(&format!("- insertions: {}\n", diff.total_additions));
        output.push_str(&format!("- deletions: {}\n", diff.total_deletions));

        output.push_str(&format!("\n## Files ({})\n", diff.files.len()));
        for file in &diff.files {
            output.push_str(&format!(
                "- [{}] {} (+{} -{})\n",
                file.change_type, file.path, file.additions, file.deletions
            ));
        }

        if diff.is_truncated {
            output.push_str(&format!(
                "\n- truncated: showing {} of {} files\n",
                diff.counts.files_shown, diff.counts.total_files
            ));
        }

        output
    }

    /// Format a LsOutputSchema into agent-optimized output.
    pub fn format_ls(ls: &crate::schema::LsOutputSchema) -> String {
        if ls.is_empty {
            return "# Directory Listing\n\n- result: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Directory Listing\n\n");

        output.push_str(&format!("- total: {}\n", ls.counts.total));
        output.push_str(&format!("- directories: {}\n", ls.counts.directories));
        output.push_str(&format!("- files: {}\n", ls.counts.files));
        output.push_str(&format!("- symlinks: {}\n", ls.counts.symlinks));
        if ls.counts.hidden > 0 {
            output.push_str(&format!("- hidden: {}\n", ls.counts.hidden));
        }

        if !ls.directories.is_empty() {
            output.push_str(&format!("\n## Directories ({})\n", ls.directories.len()));
            for dir in &ls.directories {
                output.push_str(&format!("- {}\n", dir));
            }
        }

        if !ls.files.is_empty() {
            output.push_str(&format!("\n## Files ({})\n", ls.files.len()));
            for file in &ls.files {
                output.push_str(&format!("- {}\n", file));
            }
        }

        if !ls.symlinks.is_empty() {
            output.push_str(&format!("\n## Symlinks ({})\n", ls.symlinks.len()));
            for symlink in &ls.symlinks {
                if let Some(entry) = ls.entries.iter().find(|e| &e.name == symlink) {
                    if let Some(ref target) = entry.symlink_target {
                        if entry.is_broken_symlink {
                            output.push_str(&format!("- {} -> {} [broken]\n", symlink, target));
                        } else {
                            output.push_str(&format!("- {} -> {}\n", symlink, target));
                        }
                    } else {
                        output.push_str(&format!("- {}\n", symlink));
                    }
                } else {
                    output.push_str(&format!("- {}\n", symlink));
                }
            }
        }

        if !ls.hidden.is_empty() {
            output.push_str(&format!("\n## Hidden ({})\n", ls.hidden.len()));
            for hidden in &ls.hidden {
                output.push_str(&format!("- {}\n", hidden));
            }
        }

        if !ls.generated.is_empty() {
            output.push_str(&format!("\n## Generated ({})\n", ls.generated.len()));
            for gen in &ls.generated {
                output.push_str(&format!("- {}\n", gen));
            }
        }

        if !ls.errors.is_empty() {
            output.push_str(&format!("\n## Errors ({})\n", ls.errors.len()));
            for error in &ls.errors {
                output.push_str(&format!("- {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a GrepOutputSchema into agent-optimized output.
    pub fn format_grep(grep: &crate::schema::GrepOutputSchema) -> String {
        if grep.is_empty {
            return "# Search Results\n\n- result: no matches\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Search Results\n\n");

        output.push_str(&format!("- files: {}\n", grep.counts.files));
        output.push_str(&format!("- matches: {}\n", grep.counts.matches));

        for file in &grep.files {
            output.push_str(&format!("\n## {}\n", file.path));
            output.push_str(&format!("- matches: {}\n", file.matches.len()));
            for m in &file.matches {
                if m.is_context {
                    if let Some(line) = m.line_number {
                        output.push_str(&format!("  - line {} [context]\n", line));
                    }
                } else if let Some(line) = m.line_number {
                    output.push_str(&format!(
                        "  - line {}: {}\n",
                        line,
                        truncate(m.line.trim(), 80)
                    ));
                } else {
                    output.push_str(&format!("  - {}\n", truncate(m.line.trim(), 80)));
                }
            }
        }

        if grep.is_truncated {
            output.push_str(&format!(
                "\n- truncated: showing {} of {} files\n",
                grep.counts.files_shown, grep.counts.total_files
            ));
        }

        output
    }

    /// Format a FindOutputSchema into agent-optimized output.
    pub fn format_find(find: &crate::schema::FindOutputSchema) -> String {
        if find.is_empty {
            return "# Find Results\n\n- result: no matches\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Find Results\n\n");

        output.push_str(&format!("- total: {}\n", find.counts.total));
        output.push_str(&format!("- directories: {}\n", find.counts.directories));
        output.push_str(&format!("- files: {}\n", find.counts.files));

        if !find.directories.is_empty() {
            output.push_str(&format!("\n## Directories ({})\n", find.directories.len()));
            for dir in &find.directories {
                output.push_str(&format!("- {}\n", dir));
            }
        }

        if !find.files.is_empty() {
            output.push_str(&format!("\n## Files ({})\n", find.files.len()));
            for file in &find.files {
                output.push_str(&format!("- {}\n", file));
            }
        }

        if !find.hidden.is_empty() {
            output.push_str(&format!("\n## Hidden ({})\n", find.hidden.len()));
            for hidden in &find.hidden {
                output.push_str(&format!("- {}\n", hidden));
            }
        }

        if !find.extensions.is_empty() {
            output.push_str(&format!("\n## Extensions\n"));
            let mut exts: Vec<_> = find.extensions.iter().collect();
            exts.sort_by(|a, b| b.1.cmp(a.1));
            for (ext, count) in exts {
                output.push_str(&format!("- .{}: {}\n", ext, count));
            }
        }

        if !find.errors.is_empty() {
            output.push_str(&format!("\n## Errors ({})\n", find.errors.len()));
            for error in &find.errors {
                output.push_str(&format!("- {}: {}\n", error.path, error.message));
            }
        }

        output
    }

    /// Format a TestOutputSchema into agent-optimized output.
    pub fn format_test_output(test: &crate::schema::TestOutputSchema) -> String {
        if test.is_empty {
            return "# Test Results\n\n- result: no tests\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Test Results\n\n");

        output.push_str(&format!("- runner: {}\n", test.runner));
        if let Some(ref version) = test.runner_version {
            output.push_str(&format!("- version: {}\n", version));
        }

        output.push_str(&format!(
            "- status: {}\n",
            if test.success { "passed" } else { "failed" }
        ));

        output.push_str(&format!("- total: {}\n", test.summary.total));
        output.push_str(&format!("- passed: {}\n", test.summary.passed));
        output.push_str(&format!("- failed: {}\n", test.summary.failed));
        if test.summary.skipped > 0 {
            output.push_str(&format!("- skipped: {}\n", test.summary.skipped));
        }
        if test.summary.xfailed > 0 {
            output.push_str(&format!("- xfailed: {}\n", test.summary.xfailed));
        }
        if test.summary.xpassed > 0 {
            output.push_str(&format!("- xpassed: {}\n", test.summary.xpassed));
        }
        if test.summary.errors > 0 {
            output.push_str(&format!("- errors: {}\n", test.summary.errors));
        }
        if let Some(duration) = test.summary.duration_ms {
            output.push_str(&format!("- duration: {}\n", format_duration(duration)));
        }

        if test.summary.suites_total > 0 {
            output.push_str(&format!(
                "\n- suites: {} ({} passed, {} failed)\n",
                test.summary.suites_total, test.summary.suites_passed, test.summary.suites_failed
            ));
        }

        if !test.success {
            output.push_str("\n## Failed Tests\n");
            for suite in &test.test_suites {
                if !suite.passed {
                    for t in &suite.tests {
                        if t.status == crate::schema::TestStatus::Failed {
                            output.push_str(&format!("- {}", t.name));
                            if let Some(ref file) = t.file {
                                output.push_str(&format!(" ({})", file));
                                if let Some(line) = t.line {
                                    output.push_str(&format!(":{}", line));
                                }
                            }
                            output.push('\n');
                            if let Some(ref msg) = t.error_message {
                                for line in msg.lines().take(5) {
                                    output.push_str(&format!("  > {}\n", line));
                                }
                            }
                        }
                    }
                }
            }
        }

        output
    }

    /// Format a LogsOutputSchema into agent-optimized output.
    pub fn format_logs(logs: &crate::schema::LogsOutputSchema) -> String {
        if logs.is_empty {
            return "# Log Output\n\n- result: empty\n".to_string();
        }

        let mut output = String::new();
        output.push_str("# Log Output\n\n");

        output.push_str(&format!("- total lines: {}\n", logs.counts.total_lines));

        output.push_str(&format!("- error: {}\n", logs.counts.error));
        output.push_str(&format!("- warning: {}\n", logs.counts.warning));
        output.push_str(&format!("- info: {}\n", logs.counts.info));
        output.push_str(&format!("- debug: {}\n", logs.counts.debug));

        if !logs.repeated_lines.is_empty() {
            output.push_str(&format!(
                "\n## Repeated Lines ({})\n",
                logs.repeated_lines.len()
            ));
            for repeated in &logs.repeated_lines {
                output.push_str(&format!(
                    "- lines {}-{} [x{}]: {}\n",
                    repeated.first_line, repeated.last_line, repeated.count, repeated.line
                ));
            }
        }

        if !logs.recent_critical.is_empty() {
            let critical_count = logs.counts.error + logs.counts.fatal;
            output.push_str(&format!(
                "\n## Recent Critical ({}/{})\n",
                logs.recent_critical.len(),
                critical_count
            ));
            for entry in &logs.recent_critical {
                let level = match entry.level {
                    crate::schema::LogLevel::Error => "ERROR",
                    crate::schema::LogLevel::Fatal => "FATAL",
                    _ => "!",
                };
                output.push_str(&format!(
                    "- line {} [{}]: {}\n",
                    entry.line_number,
                    level,
                    truncate(&entry.message, 80)
                ));
            }
        }

        output
    }

    /// Format a RepositoryStateSchema into agent-optimized output.
    pub fn format_repository_state(state: &crate::schema::RepositoryStateSchema) -> String {
        let mut output = String::new();
        output.push_str("# Repository State\n\n");

        if !state.is_git_repo {
            output.push_str("- is_git_repo: false\n");
            return output;
        }

        output.push_str("- is_git_repo: true\n");

        if let Some(ref branch) = state.branch {
            if state.is_detached {
                output.push_str(&format!("- branch: {} (detached)\n", branch));
            } else {
                output.push_str(&format!("- branch: {}\n", branch));
            }
        }

        if state.is_clean {
            output.push_str("- status: clean\n");
        } else {
            output.push_str("- status: dirty\n");
            output.push_str(&format!("- staged: {}\n", state.counts.staged));
            output.push_str(&format!("- unstaged: {}\n", state.counts.unstaged));
            output.push_str(&format!("- untracked: {}\n", state.counts.untracked));
            output.push_str(&format!("- unmerged: {}\n", state.counts.unmerged));
        }

        output
    }

    /// Format a ProcessOutputSchema into agent-optimized output.
    pub fn format_process(process: &crate::schema::ProcessOutputSchema) -> String {
        let mut output = String::new();
        output.push_str("# Process Output\n\n");

        output.push_str(&format!("- command: {}\n", process.command));
        if !process.args.is_empty() {
            output.push_str(&format!("- args: {}\n", process.args.join(" ")));
        }
        output.push_str(&format!(
            "- status: {}\n",
            if process.success { "success" } else { "failed" }
        ));
        if let Some(code) = process.exit_code {
            output.push_str(&format!("- exit_code: {}\n", code));
        }
        output.push_str(&format!("- duration_ms: {}\n", process.duration_ms));
        if process.timed_out {
            output.push_str("- timed_out: true\n");
        }

        if !process.stdout.is_empty() {
            output.push_str("\n## stdout\n");
            output.push_str(&format!("```\n{}```\n", process.stdout));
        }

        if !process.stderr.is_empty() {
            output.push_str("\n## stderr\n");
            output.push_str(&format!("```\n{}```\n", process.stderr));
        }

        output
    }

    /// Format an ErrorSchema into agent-optimized output.
    pub fn format_error_schema(error: &crate::schema::ErrorSchema) -> String {
        let mut output = String::new();
        output.push_str("# Error\n\n");

        output.push_str(&format!("- message: {}\n", error.message));
        if let Some(ref error_type) = error.error_type {
            output.push_str(&format!("- type: {}\n", error_type));
        }
        if let Some(code) = error.exit_code {
            output.push_str(&format!("- exit_code: {}\n", code));
        }

        if !error.context.is_empty() {
            output.push_str("\n## Context\n");
            for (key, value) in &error.context {
                output.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        output
    }
}
