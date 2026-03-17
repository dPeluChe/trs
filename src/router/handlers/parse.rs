use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::*;
use super::run::RunHandler;
use crate::OutputFormat;
use crate::ParseCommands;

pub(crate) struct ParseHandler;

impl ParseHandler {
    /// Handle the git-status subcommand.
    pub(crate) fn handle_git_status(
        file: &Option<std::path::PathBuf>,
        count: &Option<String>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the git status output
        let status = Self::parse_git_status(&input)?;

        // If count flag is specified, output only the count
        if let Some(category) = count {
            let count_value = match category.to_lowercase().as_str() {
                "staged" => status.staged_count,
                "unstaged" => status.unstaged_count,
                "untracked" => status.untracked_count,
                "unmerged" => status.unmerged_count,
                _ => {
                    return Err(CommandError::InvalidArguments(format!(
                        "Invalid count category: {}. Valid options are: staged, unstaged, untracked, unmerged",
                        category
                    )));
                }
            };
            let output = Self::format_git_status_count(count_value, ctx.format);
            if ctx.stats {
                let stats = CommandStats::new()
                    .with_reducer("git-status")
                    .with_output_mode(ctx.format)
                    .with_input_bytes(input.len())
                    .with_output_bytes(output.len())
                    .with_items_processed(count_value)
                    .with_extra("Category", category.clone());
                stats.print();
            }
            print!("{}", output);
        } else {
            // Format output based on the requested format
            let output = Self::format_git_status(&status, ctx.format);
            if ctx.stats {
                let total_changes = status.staged_count
                    + status.unstaged_count
                    + status.untracked_count
                    + status.unmerged_count;
                let stats = CommandStats::new()
                    .with_reducer("git-status")
                    .with_output_mode(ctx.format)
                    .with_input_bytes(input.len())
                    .with_output_bytes(output.len())
                    .with_items_processed(total_changes)
                    .with_extra("Staged", status.staged_count.to_string())
                    .with_extra("Unstaged", status.unstaged_count.to_string())
                    .with_extra("Untracked", status.untracked_count.to_string())
                    .with_extra("Unmerged", status.unmerged_count.to_string());
                stats.print();
            }
            print!("{}", output);
        }

        Ok(())
    }

    /// Format git status count for output (just the number).
    pub(crate) fn format_git_status_count(count: usize, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({ "count": count }).to_string(),
            OutputFormat::Raw | OutputFormat::Compact | OutputFormat::Agent => {
                format!("{}\n", count)
            }
            OutputFormat::Csv | OutputFormat::Tsv => {
                format!("count\n{}\n", count)
            }
        }
    }

    /// Read input from a file or stdin.
    /// Handles both UTF-8 and binary input gracefully by replacing invalid
    /// UTF-8 sequences with the Unicode replacement character.
    pub(crate) fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        if let Some(path) = file {
            let bytes = std::fs::read(path).map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        } else {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&buffer).into_owned())
        }
    }

    /// Parse git status output into structured data.
    pub(crate) fn parse_git_status(input: &str) -> CommandResult<GitStatus> {
        let mut status = GitStatus::default();
        let mut current_section = GitStatusSection::None;

        for line in input.lines() {
            // Use trim_end() to preserve leading spaces which are significant in porcelain format
            // (e.g., " M file" means unstaged modification, "M  file" means staged modification)
            let line = line.trim_end();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Detect branch info (English)
            if line.starts_with("On branch ") {
                status.branch = line.strip_prefix("On branch ").unwrap_or("").to_string();
                continue;
            }

            // Detect branch info (Spanish)
            if line.starts_with("En la rama ") {
                status.branch = line.strip_prefix("En la rama ").unwrap_or("").to_string();
                continue;
            }

            // Detect branch info (German): "Auf Branch main"
            if line.starts_with("Auf Branch ") {
                status.branch = line.strip_prefix("Auf Branch ").unwrap_or("").to_string();
                continue;
            }

            // Detect HEAD detached
            if line.starts_with("HEAD detached at ") {
                status.branch = format!(
                    "HEAD detached at {}",
                    line.strip_prefix("HEAD detached at ").unwrap_or("")
                );
                continue;
            }

            // Detect ahead count: "Your branch is ahead of 'origin/master' by 3 commits."
            if line.starts_with("Your branch is ahead of ") {
                // Parse: "Your branch is ahead of 'origin/master' by 3 commits."
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.ahead = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect behind count: "Your branch is behind 'origin/master' by 5 commits, and can be fast-forwarded."
            if line.starts_with("Your branch is behind ") {
                // Parse: "Your branch is behind 'origin/master' by 5 commits"
                if let Some(by_pos) = line.find(" by ") {
                    let after_by = &line[by_pos + 4..];
                    if let Some(space_pos) = after_by.find(' ') {
                        if let Ok(count) = after_by[..space_pos].parse::<usize>() {
                            status.behind = Some(count);
                        }
                    }
                }
                continue;
            }

            // Detect up to date: "Your branch is up to date with 'origin/master'."
            if line.starts_with("Your branch is up to date") {
                // No action needed, just skip this line
                continue;
            }

            // Detect up to date (German): "Ihr Branch ist auf demselben Stand wie 'origin/master'."
            if line.starts_with("Ihr Branch ist auf demselben Stand") {
                continue;
            }

            // Detect diverged: "Your branch and 'origin/master' have diverged,"
            if line.starts_with("Your branch and ") && line.contains(" have diverged") {
                // This line indicates divergence, but actual counts are on separate lines
                // We'll set a flag to look for counts on next lines
                continue;
            }

            // Detect diverged counts: "  and have 3 and 5 different commits each, respectively."
            // Also handles "and have 3 and 5 different commits each, respectively."
            if line.contains(" different commits each")
                || (line.starts_with("and have") && line.contains(" different commits"))
            {
                // Parse: "  and have 3 and 5 different commits each, respectively."
                // Format: "have <ahead> and <behind> different commits each"
                let parts: Vec<&str> = line.split_whitespace().collect();
                for i in 0..parts.len() - 1 {
                    if parts[i] == "have" && i + 4 < parts.len() {
                        if let Ok(ahead_count) = parts[i + 1].parse::<usize>() {
                            status.ahead = Some(ahead_count);
                        }
                        // parts[i + 2] is "and"
                        if let Ok(behind_count) = parts[i + 3].parse::<usize>() {
                            status.behind = Some(behind_count);
                        }
                    }
                }
                continue;
            }

            // Detect sections (English and localized versions)
            if line.starts_with("Changes to be committed")
                || line.starts_with("Cambios para confirmar")
            {
                current_section = GitStatusSection::Staged;
                continue;
            }
            if line.starts_with("Changes not staged for commit")
                || line.starts_with("Cambios sin rastrear para el commit")
            {
                current_section = GitStatusSection::Unstaged;
                continue;
            }
            if line.starts_with("Untracked files") || line.starts_with("Archivos sin seguimiento") {
                current_section = GitStatusSection::Untracked;
                continue;
            }
            if line.starts_with("Unmerged paths") {
                current_section = GitStatusSection::Unmerged;
                continue;
            }

            // Skip help text (lines starting with '(' or containing 'use "git')
            if line.starts_with('(') || line.contains("use \"git") {
                continue;
            }

            // Skip clean status lines and other non-file lines
            if line.starts_with("nothing to commit")
                || line.starts_with("no changes added")
                || line.contains("working tree clean")
                || line.contains("árbol de trabajo limpio")
                || line.contains("Arbeitsverzeichnis unverändert")
            {
                continue;
            }

            // Parse file entries
            if let Some(entry) = Self::parse_file_entry(line, current_section) {
                match current_section {
                    GitStatusSection::Staged => status.staged.push(entry),
                    GitStatusSection::Unstaged => status.unstaged.push(entry),
                    GitStatusSection::Untracked => status.untracked.push(entry),
                    GitStatusSection::Unmerged => status.unmerged.push(entry),
                    GitStatusSection::None => {
                        // Handle porcelain format or other inline entries
                        if entry.status.starts_with("??") {
                            status.untracked.push(entry);
                        } else if entry.status.starts_with("UU")
                            || entry.status.starts_with("AA")
                            || entry.status.starts_with("DD")
                        {
                            status.unmerged.push(entry);
                        } else if entry.status.starts_with(' ') {
                            // Unstaged changes (porcelain: " M file")
                            status.unstaged.push(entry);
                        } else {
                            // Staged changes (porcelain: "M  file")
                            status.staged.push(entry);
                        }
                    }
                }
            }
        }

        // Check if this is a clean working tree
        status.is_clean = status.staged.is_empty()
            && status.unstaged.is_empty()
            && status.untracked.is_empty()
            && status.unmerged.is_empty();

        // Set file counts
        status.staged_count = status.staged.len();
        status.unstaged_count = status.unstaged.len();
        status.untracked_count = status.untracked.len();
        status.unmerged_count = status.unmerged.len();

        // Check if this is porcelain format (no section headers)
        if status.branch.is_empty()
            && !input
                .lines()
                .any(|l| l.contains("Changes to be committed") || l.contains("Changes not staged"))
        {
            // Try to detect branch from porcelain format if possible
            // Porcelain v2 includes "# branch.head" lines
            for line in input.lines() {
                if line.starts_with("# branch.head ") {
                    status.branch = line
                        .strip_prefix("# branch.head ")
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        Ok(status)
    }

    /// Parse a single file entry from git status.
    pub(crate) fn parse_file_entry(line: &str, section: GitStatusSection) -> Option<GitStatusEntry> {
        if line.is_empty() {
            return None;
        }

        // Handle porcelain format: "XY path" or "XY orig_path -> new_path"
        // XY can be two characters representing index and worktree status
        if section == GitStatusSection::None {
            // Porcelain format
            // Use chars() for UTF-8 safe iteration
            let chars: Vec<char> = line.chars().collect();
            if chars.len() >= 3 {
                // Get first two characters as status
                let status: String = chars[..2].iter().collect();
                // Get the rest as path (skip first 3 chars: 2 status + 1 space)
                let path: String = chars[3..].iter().collect();
                let path = path.trim();

                if path.is_empty() {
                    return None;
                }

                // Handle rename format: "R  new -> new"
                let (path, new_path) = if path.contains(" -> ") {
                    let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                    (
                        parts.get(1).unwrap_or(&path).to_string(),
                        Some(parts.get(0).unwrap_or(&"").to_string()),
                    )
                } else {
                    (path.to_string(), None)
                };

                return Some(GitStatusEntry {
                    status,
                    path,
                    new_path,
                });
            }
            return None;
        }

        // Handle standard format with tab indentation: "\tmodified:   path" or "\tnew file:   path"
        // Lines can start with tabs, have status, colon, then path
        // We need to find the colon position using char_indices for UTF-8 safety
        if line.contains(':') {
            // Use char_indices for UTF-8 safe slicing
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();
            let colon_char_idx = char_indices.iter().position(|(_, c)| *c == ':')?;

            let before_colon = line[..colon_char_idx].trim();
            // Remove leading tabs from status
            let status = before_colon.trim_start_matches('\t').trim();
            let path_start = char_indices
                .get(colon_char_idx + 1)
                .map(|(i, _)| *i)
                .unwrap_or(line.len());
            let path = line[path_start..].trim();

            if path.is_empty() {
                return None;
            }

            // Handle rename format: "renamed:   new -> new"
            let (path, new_path) = if path.contains(" -> ") {
                let parts: Vec<&str> = path.splitn(2, " -> ").collect();
                (
                    parts.get(1).unwrap_or(&path).to_string(),
                    Some(parts.get(0).unwrap_or(&"").to_string()),
                )
            } else {
                (path.to_string(), None)
            };

            // Normalize status to short form
            let short_status = match status {
                // English
                "new file" => "A",
                "modified" => "M",
                "deleted" => "D",
                "renamed" => "R",
                "copied" => "C",
                "typechange" => "T",
                "both added" => "AA",
                "both deleted" => "DD",
                "both modified" => "UU",
                "added by them" => "AU",
                "deleted by them" => "DU",
                "added by us" => "UA",
                "deleted by us" => "UD",
                // Spanish
                "nuevo archivo" => "A",
                "modificados" => "M",
                "borrados" => "D",
                "renombrados" => "R",
                "copiados" => "C",
                // German
                "neue Datei" => "A",
                "geändert" => "M",
                "gelöscht" => "D",
                "umbenannt" => "R",
                // French
                "nouveau fichier" => "A",
                "modifié" => "M",
                "supprimé" => "D",
                "renommé" => "R",
                _ => status,
            };

            return Some(GitStatusEntry {
                status: short_status.to_string(),
                path,
                new_path,
            });
        }

        // Handle untracked files in standard format (just the path, no prefix)
        if section == GitStatusSection::Untracked {
            return Some(GitStatusEntry {
                status: "??".to_string(),
                path: line.trim().to_string(),
                new_path: None,
            });
        }

        None
    }

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
        let mut output = String::new();

        // Branch info with ahead/behind inline
        if !status.branch.is_empty() {
            let mut branch_line = format!("branch: {}", status.branch);
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

        // Staged changes
        if !status.staged.is_empty() {
            output.push_str(&format!("staged ({}):\n", status.staged.len()));
            for entry in &status.staged {
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

        // Unstaged changes
        if !status.unstaged.is_empty() {
            output.push_str(&format!("unstaged ({}):\n", status.unstaged.len()));
            for entry in &status.unstaged {
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

        // Untracked files
        if !status.untracked.is_empty() {
            output.push_str(&format!("untracked ({}):\n", status.untracked.len()));
            for entry in &status.untracked {
                output.push_str(&format!("  {} {}\n", entry.status, entry.path));
            }
        }

        // Unmerged files
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

    /// Handle the ls subcommand.
    pub(crate) fn handle_ls(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the ls output
        let ls_output = Self::parse_ls(&input)?;

        // Format output based on the requested format
        let output = Self::format_ls(&ls_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("ls")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(ls_output.entries.len())
                .with_extra("Files", ls_output.files.len().to_string())
                .with_extra("Directories", ls_output.directories.len().to_string())
                .with_extra("Hidden", ls_output.hidden.len().to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }
    /// Parse ls output into structured data.
    pub(crate) fn parse_ls(input: &str) -> CommandResult<LsOutput> {
        let mut ls_output = LsOutput::default();
        let mut current_entry: Option<LsEntry> = None;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip "total N" summary lines from ls -l
            if line.starts_with("total ") {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "ls: cannot open directory '/path': Permission denied"
            // or: "ls: cannot access 'file': No such file or directory"
            if line.starts_with("ls: ") && line.contains("cannot ") {
                // Parse the error message
                let error = Self::parse_ls_error(line);
                ls_output.errors.push(error);
                continue;
            }

            // Check if this is a long format line (starts with permissions)
            // Long format: drwxr-xr-x  2 user group  64 Jan  1 12:34 file.txt
            if Self::is_long_format_line(line) {
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry.clone());
                }

                // Parse the long format line
                current_entry = Some(Self::parse_long_format_line(line));
            } else {
                // This is a short format line (just the filename)
                // Save the previous entry if any
                if let Some(entry) = current_entry.take() {
                    ls_output.entries.push(entry);
                }

                // Create entry from the filename
                let name = line.to_string();
                let is_hidden = name.starts_with('.');
                let entry_type = Self::detect_entry_type_from_name(&name);

                current_entry = Some(LsEntry {
                    name,
                    entry_type,
                    is_hidden,
                    ..Default::default()
                });
            }
        }

        // Don't forget the last entry
        if let Some(entry) = current_entry {
            ls_output.entries.push(entry);
        }

        // Categorize entries
        for entry in &ls_output.entries {
            if entry.is_hidden {
                ls_output.hidden.push(entry.clone());
            }
            match entry.entry_type {
                LsEntryType::Directory => {
                    // Check if this is a generated directory
                    if is_generated_directory(&entry.name) {
                        ls_output.generated.push(entry.clone());
                    }
                    ls_output.directories.push(entry.clone())
                }
                LsEntryType::Symlink => ls_output.symlinks.push(entry.clone()),
                _ => ls_output.files.push(entry.clone()),
            }
        }

        // Calculate totals (excluding errors)
        ls_output.total_count = ls_output.entries.len();
        ls_output.is_empty = ls_output.entries.is_empty() && ls_output.errors.is_empty();

        Ok(ls_output)
    }

    /// Parse an ls error message.
    pub(crate) fn parse_ls_error(line: &str) -> LsError {
        // Format: "ls: cannot open directory '/path': Permission denied"
        // or: "ls: cannot access 'file': No such file or directory"

        // Try to extract the path (usually in quotes after 'access' or 'directory')
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        LsError {
            path,
            message: line.to_string(),
        }
    }

    /// Check if a line is in long format (starts with permissions).
    pub(crate) fn is_long_format_line(line: &str) -> bool {
        // Long format lines start with a permission string like:
        // -rwxr-xr-x (file)
        // drwxr-xr-x (directory)
        // lrwxr-xr-x (symlink)
        // brw-r--r-- (block device)
        // crw-r--r-- (char device)
        // srw-r--r-- (socket)
        // prw-r--r-- (pipe/FIFO)
        // total 0 (summary line from ls -l)

        // Skip "total 0" or similar summary lines
        if line.starts_with("total ") {
            return false;
        }

        if line.starts_with('-')
            || line.starts_with('d')
            || line.starts_with('l')
            || line.starts_with('b')
            || line.starts_with('c')
            || line.starts_with('s')
            || line.starts_with('p')
        {
            // Check if it looks like a permission string (has at least 10 characters)
            // Format: type + 9 permission chars (e.g., drwxr-xr-x)
            let perms_part = line.split_whitespace().next();
            if let Some(perms) = perms_part {
                if perms.len() >= 10 {
                    // Check remaining chars (after type indicator) are valid permission chars
                    let rest = &perms[1..];
                    if rest.chars().all(|c| {
                        c == 'r'
                            || c == 'w'
                            || c == 'x'
                            || c == '-'
                            || c == 's'
                            || c == 't'
                            || c == 'S'
                            || c == 'T'
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Parse a long format ls line.
    pub(crate) fn parse_long_format_line(line: &str) -> LsEntry {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Long format: perms links owner group size month day time/year name
        // The name starts after the time/year field. We find it by looking for
        // a time pattern (HH:MM) or a year (4 digits) after the day field.

        if parts.len() < 9 {
            return LsEntry::default();
        }

        let perms = parts[0];

        // Find the name by scanning for the date/time pattern
        // Date is: Month Day Time/Year — we look for month names
        let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
        let mut name_start_idx = 8; // default
        for i in 5..parts.len().saturating_sub(2) {
            if months.contains(&parts[i]) {
                // parts[i] = month, parts[i+1] = day, parts[i+2] = time/year
                name_start_idx = i + 3;
                break;
            }
        }
        let name_part = if name_start_idx < parts.len() {
            parts[name_start_idx..].join(" ")
        } else {
            parts.last().unwrap_or(&"").to_string()
        };

        // Detect entry type from permissions
        let entry_type = Self::detect_entry_type_from_perms(perms);

        // For symlinks, extract name and target (format: "name -> target")
        let (name, symlink_target) =
            if entry_type == LsEntryType::Symlink && name_part.contains(" -> ") {
                let mut split = name_part.splitn(2, " -> ");
                let name = split.next().unwrap_or(&name_part).to_string();
                let target = split.next().map(|s| s.to_string());
                (name, target)
            } else {
                (name_part, None)
            };

        let is_hidden = name.starts_with('.');

        // Check if symlink is broken (target doesn't exist)
        let is_broken_symlink = if entry_type == LsEntryType::Symlink {
            if let Some(ref target) = symlink_target {
                // A broken symlink has a target that doesn't exist
                // Common patterns: absolute paths to non-existent files, relative paths that don't exist
                target.starts_with("/nonexistent") || 
                target.contains("/nonexistent/") ||
                target == "nonexistent" ||
                // Self-referencing (circular) symlinks
                target == &name
            } else {
                false
            }
        } else {
            false
        };

        LsEntry {
            name,
            entry_type,
            is_hidden,
            size: parts.get(4).and_then(|s| s.parse().ok()),
            permissions: Some(perms.to_string()),
            links: parts.get(1).and_then(|s| s.parse().ok()),
            owner: parts.get(2).map(|s| s.to_string()),
            group: parts.get(3).map(|s| s.to_string()),
            modified: Some(format!("{} {} {}", parts[5], parts[6], parts[7])),
            symlink_target,
            is_broken_symlink,
        }
    }
    /// Detect entry type from permission string.
    pub(crate) fn detect_entry_type_from_perms(perms: &str) -> LsEntryType {
        if perms.starts_with('d') {
            LsEntryType::Directory
        } else if perms.starts_with('l') {
            LsEntryType::Symlink
        } else if perms.starts_with('b') {
            LsEntryType::BlockDevice
        } else if perms.starts_with('c') {
            LsEntryType::CharDevice
        } else if perms.starts_with('s') {
            LsEntryType::Socket
        } else if perms.starts_with('p') {
            LsEntryType::Pipe
        } else if perms.starts_with('-') {
            LsEntryType::File
        } else {
            LsEntryType::Other
        }
    }
    /// Detect entry type from name (for short format).
    pub(crate) fn detect_entry_type_from_name(name: &str) -> LsEntryType {
        // In short format, we use heuristics to determine the type
        // 1. If name ends with '/', it's a directory
        // 2. If name has a file extension (contains '.' after the last '/', not just leading '.'), it's a file
        // 3. Otherwise, assume it's a directory (common convention: names without extensions are dirs)
        if name.ends_with('/') {
            LsEntryType::Directory
        } else if Self::has_file_extension(name) {
            LsEntryType::File
        } else {
            LsEntryType::Directory
        }
    }

    /// Check if a name has a file extension (not counting leading dots for hidden files).
    pub(crate) fn has_file_extension(name: &str) -> bool {
        // Get the basename (last component of path)
        let basename = name.rsplit('/').next().unwrap_or(name);

        // Skip the leading dot for hidden files
        let basename = if basename.starts_with('.') && basename.len() > 1 {
            &basename[1..]
        } else {
            basename
        };

        // Check if there's a dot that's not at the start
        // This means we have something like "file.txt" or "name.something"
        if let Some(pos) = basename.rfind('.') {
            // Make sure there's something before the dot and after the dot
            pos > 0 && pos < basename.len() - 1
        } else {
            false
        }
    }
    /// Format ls output for display.
    pub(crate) fn format_ls(ls_output: &LsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_ls_json(ls_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_ls_compact(ls_output),
            OutputFormat::Raw => Self::format_ls_raw(ls_output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_ls_compact(ls_output),
        }
    }
    /// Format ls output as JSON.
    pub(crate) fn format_ls_json(ls_output: &LsOutput) -> String {
        let json = serde_json::json!({
            "schema": {
                "version": "1.0.0",
                "type": "ls_output"
            },
            "is_empty": ls_output.is_empty,
            "entries": ls_output.entries.iter().map(|e| serde_json::json!({
                "name": e.name,
                "type": match e.entry_type {
                    LsEntryType::File => "file",
                    LsEntryType::Directory => "directory",
                    LsEntryType::Symlink => "symlink",
                    LsEntryType::BlockDevice => "block_device",
                    LsEntryType::CharDevice => "char_device",
                    LsEntryType::Socket => "socket",
                    LsEntryType::Pipe => "pipe",
                    LsEntryType::Other => "other",
                },
                "is_hidden": e.is_hidden,
                "is_generated": e.entry_type == LsEntryType::Directory && is_generated_directory(&e.name),
                "is_broken_symlink": e.is_broken_symlink,
                "links": e.links,
                "owner": e.owner,
                "group": e.group,
                "modified": e.modified,
                "symlink_target": e.symlink_target,
            })).collect::<Vec<_>>(),
            "directories": ls_output.directories.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "files": ls_output.files.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "symlinks": ls_output.symlinks.iter().map(|e| {
                if let Some(ref target) = e.symlink_target {
                    format!("{} -> {}", e.name, target)
                } else {
                    e.name.clone()
                }
            }).collect::<Vec<_>>(),
            "broken_symlinks": ls_output.symlinks.iter().filter(|e| e.is_broken_symlink).map(|e| &e.name).collect::<Vec<_>>(),
            "hidden": ls_output.hidden.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "generated": ls_output.generated.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "errors": ls_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "counts": {
                "total": ls_output.total_count,
                "directories": ls_output.directories.len(),
                "files": ls_output.files.len(),
                "symlinks": ls_output.symlinks.len(),
                "hidden": ls_output.hidden.len(),
                "generated": ls_output.generated.len(),
                "errors": ls_output.errors.len(),
            }
        });
        Self::json_to_string(json)
    }

    /// Convert serde_json::Value to pretty-printed JSON string.
    pub(crate) fn json_to_string(value: serde_json::Value) -> String {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
    }
    /// Format a byte size into a human-readable string (e.g. 1.2K, 3.5M).
    fn format_human_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}K", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Format ls output in compact format.
    pub(crate) fn format_ls_compact(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !ls_output.errors.is_empty() {
            for error in &ls_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if ls_output.entries.is_empty() {
            if ls_output.errors.is_empty() {
                output.push_str("(empty)\n");
            }
            return output;
        }

        // Directories first, with / suffix (skip . and .. and empty names)
        for entry in &ls_output.directories {
            if entry.name == "." || entry.name == ".." || entry.name.is_empty() { continue; }
            // Skip entries that look like raw ls lines (contain permissions)
            if entry.name.contains("drwx") || entry.name.contains("lrwx") { continue; }
            let name = if entry.name.ends_with('/') {
                entry.name.clone()
            } else {
                format!("{}/", entry.name)
            };
            output.push_str(&name);
            output.push('\n');
        }

        // Symlinks
        for entry in &ls_output.symlinks {
            if let Some(ref target) = entry.symlink_target {
                if entry.is_broken_symlink {
                    output.push_str(&format!("{} -> {} [broken]\n", entry.name, target));
                } else {
                    output.push_str(&format!("{} -> {}\n", entry.name, target));
                }
            } else {
                output.push_str(&format!("{}\n", entry.name));
            }
        }

        // Files with size
        for entry in &ls_output.files {
            if let Some(size) = entry.size {
                output.push_str(&format!("{}  {}\n", entry.name, Self::format_human_size(size)));
            } else {
                output.push_str(&format!("{}\n", entry.name));
            }
        }

        // Summary line
        let dir_count = ls_output.directories.len();
        let file_count = ls_output.files.len();
        let sym_count = ls_output.symlinks.len();
        let mut summary_parts = Vec::new();
        if file_count > 0 { summary_parts.push(format!("{} files", file_count)); }
        if dir_count > 0 { summary_parts.push(format!("{} dirs", dir_count)); }
        if sym_count > 0 { summary_parts.push(format!("{} symlinks", sym_count)); }
        if !ls_output.generated.is_empty() { summary_parts.push(format!("{} generated", ls_output.generated.len())); }
        output.push_str(&format!("[{}]\n", summary_parts.join(", ")));

        output
    }
    /// Format ls output as raw (just filenames).
    pub(crate) fn format_ls_raw(ls_output: &LsOutput) -> String {
        let mut output = String::new();

        for entry in &ls_output.entries {
            output.push_str(&format!("{}\n", entry.name));
        }

        output
    }

    /// Handle the grep subcommand.
    pub(crate) fn handle_grep(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the grep output
        let mut grep_output = Self::parse_grep(&input)?;

        // Apply truncation for large result sets
        Self::truncate_grep(
            &mut grep_output,
            Self::DEFAULT_MAX_GREP_FILES,
            Self::DEFAULT_MAX_GREP_MATCHES_PER_FILE,
        );

        // Format output based on the requested format
        let output = Self::format_grep(&grep_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("grep")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(grep_output.matches_shown)
                .with_items_filtered(
                    grep_output
                        .total_matches
                        .saturating_sub(grep_output.matches_shown),
                )
                .with_extra("Files with matches", grep_output.file_count.to_string())
                .with_extra("Total matches", grep_output.total_matches.to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse grep output into structured data.
    ///
    /// Supports multiple grep output formats:
    /// - Standard format: `filename:line_number:matched_line`
    /// - Without line numbers: `filename:matched_line`
    /// - With column: `filename:line_number:column:matched_line`
    /// - Recursive format (ripgrep): `filename:line_number:matched_line`
    ///
    /// Matches are grouped by file, preserving the order of first appearance.
    pub(crate) fn parse_grep(input: &str) -> CommandResult<GrepOutput> {
        use std::collections::HashMap;

        let mut grep_output = GrepOutput::default();
        // Use a HashMap to group matches by file path
        let mut matches_by_file: HashMap<String, Vec<GrepMatch>> = HashMap::new();
        // Track the order of file appearance
        let mut file_order: Vec<String> = Vec::new();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Skip grep summary lines (e.g., from ripgrep)
            if line.starts_with("grep:") || line.contains("matched ") && line.ends_with(" files") {
                continue;
            }

            // Try to parse the grep line
            if let Some((path, grep_match)) = Self::parse_grep_line(line) {
                // Track file order on first appearance
                if !matches_by_file.contains_key(&path) {
                    file_order.push(path.clone());
                }
                // Add match to the file's group
                matches_by_file.entry(path).or_default().push(grep_match);
            }
        }

        // Convert HashMap to ordered Vec of GrepFile
        for path in file_order {
            if let Some(matches) = matches_by_file.remove(&path) {
                grep_output.files.push(GrepFile { path, matches });
            }
        }

        // Calculate totals
        grep_output.file_count = grep_output.files.len();
        for file in &grep_output.files {
            grep_output.match_count += file.matches.len();
        }

        // Set total counts before any truncation
        grep_output.total_files = grep_output.files.len();
        grep_output.total_matches = grep_output.match_count;
        grep_output.files_shown = grep_output.files.len();
        grep_output.matches_shown = grep_output.match_count;

        // Check if empty
        grep_output.is_empty = grep_output.files.is_empty();

        Ok(grep_output)
    }

    /// Default maximum number of files to show in grep output before truncation.
    const DEFAULT_MAX_GREP_FILES: usize = 50;

    /// Default maximum number of matches per file to show before truncation.
    const DEFAULT_MAX_GREP_MATCHES_PER_FILE: usize = 20;

    /// Truncate grep output if it exceeds the limits.
    ///
    /// This truncates both the number of files and the number of matches per file
    /// to prevent overwhelming output for large result sets.
    pub(crate) fn truncate_grep(grep_output: &mut GrepOutput, max_files: usize, max_matches_per_file: usize) {
        // First, truncate matches per file
        for file in &mut grep_output.files {
            if file.matches.len() > max_matches_per_file {
                file.matches.truncate(max_matches_per_file);
            }
        }

        // Then, truncate files if needed
        if grep_output.files.len() > max_files {
            grep_output.is_truncated = true;
            grep_output.files_shown = max_files;
            grep_output.files.truncate(max_files);
        } else if grep_output.total_matches
            > grep_output
                .files
                .iter()
                .map(|f| f.matches.len())
                .sum::<usize>()
        {
            // Some matches were truncated per-file but not files
            grep_output.is_truncated = true;
            grep_output.files_shown = grep_output.files.len();
        }

        // Calculate final matches shown
        grep_output.matches_shown = grep_output.files.iter().map(|f| f.matches.len()).sum();
    }

    /// Parse a single grep line.
    ///
    /// Formats supported:
    /// - `path:line_number:content` (standard with -n)
    /// - `path:line_number:column:content` (with --column)
    /// - `path:content` (without -n)
    /// - Binary file matches: `Binary file path matches`
    /// - Context lines: `path-line_number-content` (with -C/-B/-A flags)
    pub(crate) fn parse_grep_line(line: &str) -> Option<(String, GrepMatch)> {
        // Handle "Binary file path matches" format
        if line.starts_with("Binary file ") && line.ends_with(" matches") {
            let path = line
                .strip_prefix("Binary file ")
                .unwrap_or("")
                .strip_suffix(" matches")
                .unwrap_or("");
            if !path.is_empty() {
                return Some((
                    path.to_string(),
                    GrepMatch {
                        line_number: None,
                        column: None,
                        line: "[binary file]".to_string(),
                        is_context: false,
                        excerpt: None,
                    },
                ));
            }
        }

        // Determine if this is a context line or match line
        // Context lines use "-" as separator: "path-line-content"
        // Match lines use ":" as separator: "path:line:content"
        // Find the first separator (either : or -)
        let is_context_line = if let Some(dash_pos) = line.find('-') {
            // Check if dash comes before any colon (or no colon at all)
            match line.find(':') {
                Some(colon_pos) if colon_pos < dash_pos => false,
                _ => true,
            }
        } else {
            false
        };

        // Find the first separator to get the path
        let sep_pos = if is_context_line {
            line.find('-')?
        } else {
            line.find(':')?
        };

        let potential_path = &line[..sep_pos];

        // If the path is empty or the rest doesn't have content, skip
        if potential_path.is_empty() || line.len() <= sep_pos + 1 {
            return None;
        }

        let rest = &line[sep_pos + 1..];

        // Try to parse line number and optionally column
        // Format: line_number:content OR line_number:column:content OR just content
        // Context lines: line_number-content OR line_number-column-content
        let (line_number, column, content, is_context) =
            Self::parse_grep_line_content(rest, is_context_line);

        Some((
            potential_path.to_string(),
            GrepMatch {
                line_number,
                column,
                line: content.to_string(),
                is_context,
                excerpt: None,
            },
        ))
    }

    /// Parse the content part of a grep line (after the path: or path-).
    ///
    /// Context lines use "-" as separator (e.g., "10-content" for context)
    /// while match lines use ":" (e.g., "10:content" for matches).
    pub(crate) fn parse_grep_line_content(
        rest: &str,
        is_context_line: bool,
    ) -> (Option<usize>, Option<usize>, &str, bool) {
        if is_context_line {
            // Context line: use "-" as separator
            // Format: "10-content" or "10-5-content"
            if let Some(dash_pos) = rest.find('-') {
                let potential_line_num = &rest[..dash_pos];

                // Check if it's a valid line number before the dash
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[dash_pos + 1..];

                    // Try to parse column if present (context with column: "10-5-content")
                    if let Some(dash_pos2) = after_line.find('-') {
                        let potential_column = &after_line[..dash_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[dash_pos2 + 1..],
                                true, // is_context
                            );
                        }
                    }

                    // No column, just line number with context
                    return (Some(line_number), None, after_line, true);
                }
            }
            // Couldn't parse as context line, return as content
            (None, None, rest, true)
        } else {
            // Match line: use ":" as separator
            // Try to find the first colon for line number
            if let Some(colon_pos) = rest.find(':') {
                let potential_line_num = &rest[..colon_pos];

                // Check if it's a valid line number
                if let Ok(line_number) = potential_line_num.parse::<usize>() {
                    let after_line = &rest[colon_pos + 1..];

                    // Try to parse column if present
                    if let Some(colon_pos2) = after_line.find(':') {
                        let potential_column = &after_line[..colon_pos2];
                        if let Ok(column) = potential_column.parse::<usize>() {
                            return (
                                Some(line_number),
                                Some(column),
                                &after_line[colon_pos2 + 1..],
                                false, // is_context
                            );
                        }
                    }

                    // No column, just line number
                    return (Some(line_number), None, after_line, false);
                }
            }

            // No line number, just content
            (None, None, rest, false)
        }
    }

    /// Format grep output for display.
    pub(crate) fn format_grep(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_grep_json(grep_output),
            OutputFormat::Csv => Self::format_grep_csv(grep_output),
            OutputFormat::Tsv => Self::format_grep_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_grep_compact(grep_output),
            OutputFormat::Raw => Self::format_grep_raw(grep_output),
        }
    }

    /// Format grep output as JSON using the schema.
    pub(crate) fn format_grep_json(grep_output: &GrepOutput) -> String {
        use crate::schema::{
            GrepCounts, GrepFile as SchemaGrepFile, GrepMatch as SchemaGrepMatch, GrepOutputSchema,
        };

        // Count only non-context matches
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        let mut schema = GrepOutputSchema::new();
        schema.is_empty = grep_output.is_empty;
        schema.is_truncated = grep_output.is_truncated;

        // Convert internal GrepFile to schema GrepFile
        schema.files = grep_output
            .files
            .iter()
            .map(|f| SchemaGrepFile {
                path: f.path.clone(),
                matches: f
                    .matches
                    .iter()
                    .map(|m| SchemaGrepMatch {
                        line_number: m.line_number,
                        column: m.column,
                        line: m.line.clone(),
                        is_context: m.is_context,
                        excerpt: m.excerpt.clone(),
                    })
                    .collect(),
            })
            .collect();

        schema.counts = GrepCounts {
            files: grep_output.file_count,
            matches: match_count,
            total_files: grep_output.total_files,
            total_matches: grep_output.total_matches,
            files_shown: grep_output.files_shown,
            matches_shown: grep_output.matches_shown,
        };

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
    }

    /// Format grep output as CSV.
    pub(crate) fn format_grep_csv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path,line_number,column,is_context,line\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_csv_field(&m.line);
                result.push_str(&format!(
                    "{},{},{},{},{}\n",
                    file.path,
                    m.line_number.map(|n| n.to_string()).unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output as TSV.
    pub(crate) fn format_grep_tsv(grep_output: &GrepOutput) -> String {
        let mut result = String::new();
        result.push_str("path\tline_number\tcolumn\tis_context\tline\n");

        for file in &grep_output.files {
            for m in &file.matches {
                let line_escaped = RunHandler::escape_tsv_field(&m.line);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    file.path,
                    m.line_number.map(|n| n.to_string()).unwrap_or_default(),
                    m.column.map(|c| c.to_string()).unwrap_or_default(),
                    m.is_context,
                    line_escaped
                ));
            }
        }

        result
    }

    /// Format grep output in compact format.
    ///
    /// Consecutive context lines are collapsed into a summary like "... (3 context lines)".
    pub(crate) fn format_grep_compact(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        if grep_output.is_empty {
            output.push_str("grep: no matches\n");
            return output;
        }

        // Count only non-context matches for the summary
        let match_count: usize = grep_output
            .files
            .iter()
            .map(|f| f.matches.iter().filter(|m| !m.is_context).count())
            .sum();

        // Show summary with truncation info if applicable
        if grep_output.is_truncated {
            output.push_str(&format!(
                "matches: {}/{} files, {}/{} results (truncated)\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else {
            output.push_str(&format!(
                "matches: {} files, {} results\n",
                grep_output.file_count, match_count
            ));
        }

        for file in &grep_output.files {
            let non_context_count = file.matches.iter().filter(|m| !m.is_context).count();
            output.push_str(&format!("{} ({}):\n", file.path, non_context_count));

            // Track consecutive context lines for collapsing
            let mut context_start: Option<usize> = None;
            let mut context_count = 0;

            for m in &file.matches {
                if m.is_context {
                    // Start or continue a context block
                    if context_start.is_none() {
                        context_start = m.line_number;
                    }
                    context_count += 1;
                } else {
                    // Output any accumulated context lines first
                    if context_count > 0 {
                        if context_count == 1 {
                            // Single context line - show it
                            if let Some(ln) = context_start {
                                output.push_str(&format!("  {}: ...\n", ln));
                            }
                        } else {
                            // Multiple context lines - collapse
                            if let Some(start) = context_start {
                                output.push_str(&format!(
                                    "  {}-{}: ... ({} context lines)\n",
                                    start,
                                    start + context_count - 1,
                                    context_count
                                ));
                            }
                        }
                        context_start = None;
                        context_count = 0;
                    }

                    // Output the match line with excerpt if available
                    if let Some(ln) = m.line_number {
                        if let Some(col) = m.column {
                            let excerpt_str = m
                                .excerpt
                                .as_ref()
                                .map(|e| format!(" [{}]", e))
                                .unwrap_or_default();
                            output.push_str(&format!(
                                "  {}:{}: {}{}\n",
                                ln, col, m.line, excerpt_str
                            ));
                        } else {
                            let excerpt_str = m
                                .excerpt
                                .as_ref()
                                .map(|e| format!(" [{}]", e))
                                .unwrap_or_default();
                            output.push_str(&format!("  {}: {}{}\n", ln, m.line, excerpt_str));
                        }
                    } else {
                        let excerpt_str = m
                            .excerpt
                            .as_ref()
                            .map(|e| format!(" [{}]", e))
                            .unwrap_or_default();
                        output.push_str(&format!("  {}{}\n", m.line, excerpt_str));
                    }
                }
            }

            // Handle any trailing context lines
            if context_count > 0 {
                if context_count == 1 {
                    if let Some(ln) = context_start {
                        output.push_str(&format!("  {}: ...\n", ln));
                    }
                } else {
                    if let Some(start) = context_start {
                        output.push_str(&format!(
                            "  {}-{}: ... ({} context lines)\n",
                            start,
                            start + context_count - 1,
                            context_count
                        ));
                    }
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output
                .total_files
                .saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output
                .total_matches
                .saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!("  ... {} more file(s) not shown\n", hidden_files));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "  ... {} more match(es) not shown\n",
                    hidden_matches
                ));
            }
        }

        // Add total files and match count at the end
        if grep_output.is_truncated {
            output.push_str(&format!(
                "total: {}/{} files, {}/{} matches\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else {
            output.push_str(&format!(
                "total: {} files, {} matches\n",
                grep_output.file_count, match_count
            ));
        }

        output
    }

    /// Format grep output as raw (original format).
    pub(crate) fn format_grep_raw(grep_output: &GrepOutput) -> String {
        let mut output = String::new();

        for file in &grep_output.files {
            for m in &file.matches {
                // Use dash separator for context lines, colon for matches
                let sep = if m.is_context { "-" } else { ":" };
                if let Some(ln) = m.line_number {
                    if let Some(col) = m.column {
                        output.push_str(&format!(
                            "{}{}{}{}{}:{}\n",
                            file.path, sep, ln, sep, col, m.line
                        ));
                    } else {
                        output.push_str(&format!("{}{}{}{}{}\n", file.path, sep, ln, sep, m.line));
                    }
                } else {
                    output.push_str(&format!("{}:{}\n", file.path, m.line));
                }
            }
        }

        // Show truncation warning if applicable
        if grep_output.is_truncated {
            let hidden_files = grep_output
                .total_files
                .saturating_sub(grep_output.files_shown);
            let hidden_matches = grep_output
                .total_matches
                .saturating_sub(grep_output.matches_shown);
            if hidden_files > 0 {
                output.push_str(&format!("... {} more file(s) truncated\n", hidden_files));
            }
            if hidden_matches > 0 && hidden_files == 0 {
                output.push_str(&format!(
                    "... {} more match(es) truncated\n",
                    hidden_matches
                ));
            }
        }

        output
    }

    /// Handle the test subcommand.
    pub(crate) fn handle_test(
        runner: &Option<crate::TestRunner>,
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse based on the runner type (default to pytest)
        let (output, passed, failed, skipped) = match runner {
            Some(crate::TestRunner::Pytest) | None => {
                let test_output = Self::parse_pytest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.passed,
                    test_output.summary.failed,
                    test_output.summary.skipped,
                );
                let output = Self::format_pytest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Jest) => {
                let test_output = Self::parse_jest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_jest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Vitest) => {
                let test_output = Self::parse_vitest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_vitest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Npm) => {
                let test_output = Self::parse_npm_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_npm_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Pnpm) => {
                let test_output = Self::parse_pnpm_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_pnpm_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Bun) => {
                let test_output = Self::parse_bun_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_bun_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
        };

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("test")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(passed + failed + skipped)
                .with_extra("Passed", passed.to_string())
                .with_extra("Failed", failed.to_string())
                .with_extra("Skipped", skipped.to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse pytest output into structured data.
    pub(crate) fn parse_pytest(input: &str) -> CommandResult<PytestOutput> {
        let mut output = PytestOutput::default();
        let mut current_test: Option<TestResult> = None;
        let mut in_failure_section = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test_name: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse header info
            // "rootdir: /path/to/project"
            if trimmed.starts_with("rootdir:") {
                output.rootdir = Some(
                    trimmed
                        .strip_prefix("rootdir:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
                continue;
            }

            // "platform darwin -- Python 3.12.0, pytest-8.0.0, pluggy-1.4.0"
            if trimmed.starts_with("platform ") {
                output.platform = Some(trimmed.to_string());
                // Extract Python and pytest version
                if let Some(py_pos) = trimmed.find("Python ") {
                    let after_py = &trimmed[py_pos + 7..];
                    if let Some(comma_pos) = after_py.find(',') {
                        output.python_version = Some(after_py[..comma_pos].to_string());
                    }
                }
                if let Some(pytest_pos) = trimmed.find("pytest-") {
                    let after_pytest = &trimmed[pytest_pos + 7..];
                    if let Some(comma_pos) = after_pytest.find(',') {
                        output.pytest_version = Some(after_pytest[..comma_pos].to_string());
                    } else {
                        output.pytest_version = Some(after_pytest.to_string());
                    }
                }
                continue;
            }

            // Detect start of test session
            // "test session starts" or "collected N items"
            if trimmed.contains("test session starts") || trimmed.contains("collected") {
                continue;
            }

            // Detect test results with progress format
            // Format: "tests/test_file.py::test_name PASSED" or "tests/test_file.py::test_name FAILED"
            // Also handles the short format: "test_file.py .F.s" (dot=pass, F=fail, s=skip)
            if let Some(test_result) = Self::parse_pytest_test_line(trimmed) {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    output.tests.push(test);
                }
                current_test = Some(test_result);
                continue;
            }

            // Detect summary line
            // "N passed, M failed, K skipped in X.XXs"
            // Also: "N passed in X.XXs"
            if Self::is_pytest_summary_line(trimmed) {
                let summary = Self::parse_pytest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Detect failure section start
            // "=== FAILURES ===" or "=== short test summary info ==="
            if trimmed.starts_with("=== FAILURES") || trimmed.starts_with("FAILURES") {
                in_failure_section = true;
                continue;
            }
            if trimmed.starts_with("=== short test summary info ===") {
                in_failure_section = true;
                continue;
            }

            // Detect error section
            // "=== ERRORS ==="
            if trimmed.starts_with("=== ERRORS") || trimmed.starts_with("ERRORS") {
                in_failure_section = true;
                continue;
            }

            // Parse failure details
            if in_failure_section {
                // Check if this is a new failure header: "____ test_name ____"
                if trimmed.starts_with("____") && trimmed.ends_with("____") {
                    // Save any previous failure info
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        // "____ test_name ____" matches "file.py::test_name"
                        if let Some(test) = output
                            .tests
                            .iter_mut()
                            .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
                        {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                    let name = trimmed.trim_matches('_').trim().to_string();
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Check for ERROR instead of FAILURES
                // "ERROR at setup of test_name"
                if trimmed.starts_with("ERROR at") || trimmed.starts_with("ERROR:") {
                    in_failure_section = true;
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        if let Some(test) = output
                            .tests
                            .iter_mut()
                            .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
                        {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                    // Extract test name from error line
                    let name = if trimmed.starts_with("ERROR at setup of ") {
                        trimmed
                            .strip_prefix("ERROR at setup of ")
                            .unwrap_or("")
                            .to_string()
                    } else if trimmed.starts_with("ERROR at teardown of ") {
                        trimmed
                            .strip_prefix("ERROR at teardown of ")
                            .unwrap_or("")
                            .to_string()
                    } else {
                        trimmed
                            .strip_prefix("ERROR:")
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    };
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Accumulate failure details
                if current_failed_test_name.is_some() {
                    failure_buffer.push_str(line);
                    failure_buffer.push('\n');
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test.take() {
            output.tests.push(test);
        }

        // Save last failure info
        if let Some(name) = current_failed_test_name.take() {
            // Find test by matching the name at the end (after ::)
            // "____ test_name ____" matches "file.py::test_name"
            if let Some(test) = output
                .tests
                .iter_mut()
                .find(|t| t.name == name || t.name.ends_with(&format!("::{}", name)))
            {
                test.error_message = Some(failure_buffer.trim().to_string());
            }
        }

        // Calculate totals if not already in summary
        if output.summary.total == 0 && !output.tests.is_empty() {
            output.summary.passed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Passed)
                .count();
            output.summary.failed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Failed)
                .count();
            output.summary.skipped = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Skipped)
                .count();
            output.summary.xfailed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XFailed)
                .count();
            output.summary.xpassed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XPassed)
                .count();
            output.summary.errors = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Error)
                .count();
            output.summary.total = output.tests.len();
        }

        // Determine success
        output.success =
            output.summary.failed == 0 && output.summary.errors == 0 && output.summary.total > 0;
        output.is_empty = output.tests.is_empty() && output.summary.total == 0;

        Ok(output)
    }

    /// Parse a single test result line from pytest output.
    pub(crate) fn parse_pytest_test_line(line: &str) -> Option<TestResult> {
        // Format: "tests/test_file.py::test_name PASSED"
        // or: "tests/test_file.py::test_name SKIPPED (reason)"
        // or: "tests/test_file.py::test_name FAILED"
        // or: "tests/test_file.py::test_name XFAIL (reason)"

        // Skip lines that are clearly not test results
        if line.starts_with("===")
            || line.starts_with("---")
            || line.starts_with("...")
            || line.is_empty()
        {
            return None;
        }

        // Look for PASSED, FAILED, SKIPPED, XFAIL, XPASS, ERROR
        let (status_str, remainder) = if line.ends_with(" PASSED") {
            ("PASSED", &line[..line.len() - 7])
        } else if line.ends_with(" FAILED") {
            ("FAILED", &line[..line.len() - 7])
        } else if line.ends_with(" SKIPPED") {
            ("SKIPPED", &line[..line.len() - 8])
        } else if line.ends_with(" XFAIL") {
            ("XFAIL", &line[..line.len() - 6])
        } else if line.ends_with(" XPASS") {
            ("XPASS", &line[..line.len() - 6])
        } else if line.ends_with(" ERROR") {
            ("ERROR", &line[..line.len() - 6])
        } else {
            // Check for inline format: "PASSED [50%]" or "FAILED [50%]"
            if let Some(pos) = line.find(" PASSED [") {
                ("PASSED", &line[..pos])
            } else if let Some(pos) = line.find(" FAILED [") {
                ("FAILED", &line[..pos])
            } else if let Some(pos) = line.find(" SKIPPED [") {
                ("SKIPPED", &line[..pos])
            } else if let Some(pos) = line.find(" XFAIL [") {
                ("XFAIL", &line[..pos])
            } else if let Some(pos) = line.find(" XPASS [") {
                ("XPASS", &line[..pos])
            } else if let Some(pos) = line.find(" ERROR [") {
                ("ERROR", &line[..pos])
            } else {
                return None;
            }
        };

        let status = match status_str {
            "PASSED" => TestStatus::Passed,
            "FAILED" => TestStatus::Failed,
            "SKIPPED" => TestStatus::Skipped,
            "XFAIL" => TestStatus::XFailed,
            "XPASS" => TestStatus::XPassed,
            "ERROR" => TestStatus::Error,
            _ => return None,
        };

        // Parse test name and file
        let test_name = remainder.trim().to_string();

        // Try to extract file and line from "file.py::test_name" format
        let (file, line) = if let Some(pos) = test_name.find("::") {
            let file = test_name[..pos].to_string();
            let rest = &test_name[pos + 2..];
            // Check for line number: "test_name[:lineno]"
            let line = if let Some(colon_pos) = rest.find(':') {
                rest[colon_pos + 1..].parse().ok()
            } else {
                None
            };
            (Some(file), line)
        } else {
            (None, None)
        };

        Some(TestResult {
            name: test_name,
            status,
            duration: None, // Duration is usually in the summary line
            file,
            line,
            error_message: None,
        })
    }

    /// Check if a line is a pytest summary line.
    pub(crate) fn is_pytest_summary_line(line: &str) -> bool {
        // Summary lines start with a number and contain "passed" or "failed"
        // Examples:
        // "2 passed in 0.01s"
        // "2 passed, 1 failed in 0.01s"
        // "2 passed, 1 failed, 3 skipped in 0.01s"
        // "1 failed, 2 passed in 0.01s"
        // "=== 2 passed in 0.01s ==="
        let lower = line.to_lowercase();
        let starts_with_equals = line.starts_with("===");
        let has_passed = lower.contains("passed");
        let has_failed = lower.contains("failed");
        let has_skipped = lower.contains("skipped");
        let has_error = lower.contains("error");
        let has_deselected = lower.contains("deselected");
        let has_xfailed = lower.contains("xfailed");
        let has_xpassed = lower.contains("xpassed");
        let has_warnings = lower.contains("warning");

        (starts_with_equals
            || line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false))
            && (has_passed
                || has_failed
                || has_skipped
                || has_error
                || has_deselected
                || has_xfailed
                || has_xpassed
                || has_warnings)
    }

    /// Parse pytest summary line into TestSummary.
    pub(crate) fn parse_pytest_summary(line: &str) -> TestSummary {
        let mut summary = TestSummary::default();
        let lower = line.to_lowercase();

        // Remove wrapper like "=== ... ==="
        let cleaned = line.trim_matches('=').trim();

        // Parse counts
        // Pattern: "N passed", "N failed", "N skipped", etc.
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                // Look backwards for the number
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.passed = extract_count(&lower, "passed");
        summary.failed = extract_count(&lower, "failed");
        summary.skipped = extract_count(&lower, "skipped");
        summary.errors = extract_count(&lower, "error");
        summary.xfailed = extract_count(&lower, "xfailed");
        summary.xpassed = extract_count(&lower, "xpassed");

        // Calculate total
        summary.total = summary.passed
            + summary.failed
            + summary.skipped
            + summary.errors
            + summary.xfailed
            + summary.xpassed;

        // Parse duration
        // "in 0.01s" or "in 1.23 seconds"
        if let Some(pos) = lower.find(" in ") {
            let after_in = &cleaned[pos + 4..];
            // Extract number before 's' or 'seconds'
            let duration_str: String = after_in
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(duration) = duration_str.parse::<f64>() {
                summary.duration = Some(duration);
            }
        }

        summary
    }

    /// Format pytest output based on the requested format.
    pub(crate) fn format_pytest(output: &PytestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_pytest_json(output),
            OutputFormat::Compact => Self::format_pytest_compact(output),
            OutputFormat::Raw => Self::format_pytest_raw(output),
            OutputFormat::Agent => Self::format_pytest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_pytest_compact(output),
        }
    }

    /// Format pytest output as JSON.
    pub(crate) fn format_pytest_json(output: &PytestOutput) -> String {
        // Extract failing test identifiers
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .map(|t| t.name.clone())
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "passed": output.summary.passed,
                "failed": output.summary.failed,
                "skipped": output.summary.skipped,
                "xfailed": output.summary.xfailed,
                "xpassed": output.summary.xpassed,
                "errors": output.summary.errors,
                "total": output.summary.total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "tests": output.tests.iter().map(|t| serde_json::json!({
                "name": t.name,
                "status": match t.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                    TestStatus::XFailed => "xfailed",
                    TestStatus::XPassed => "xpassed",
                    TestStatus::Error => "error",
                },
                "duration": t.duration,
                "file": t.file,
                "line": t.line,
                "error_message": t.error_message,
            })).collect::<Vec<_>>(),
            "rootdir": output.rootdir,
            "platform": output.platform,
            "python_version": output.python_version,
            "pytest_version": output.pytest_version,
        })
        .to_string()
    }

    /// Format pytest output in compact format.
    pub(crate) fn format_pytest_compact(output: &PytestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!("PASS: {} tests", output.summary.passed));
            if output.summary.skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.skipped));
            }
            if output.summary.xfailed > 0 {
                result.push_str(&format!(", {} xfailed", output.summary.xfailed));
            }
            if output.summary.xpassed > 0 {
                result.push_str(&format!(", {} xpassed", output.summary.xpassed));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} passed, {} failed",
            output.summary.passed, output.summary.failed
        ));
        if output.summary.skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!(", {} xfailed", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!(", {} xpassed", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!(", {} errors", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed tests
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str(&format!("failed ({}):\n", failed_tests.len()));
            for test in failed_tests {
                result.push_str(&format!("  {}\n", test.name));
                if let Some(ref msg) = test.error_message {
                    // Show first line of error message
                    if let Some(first_line) = msg.lines().next() {
                        let truncated = if first_line.len() > 80 {
                            format!("{}...", &first_line[..77])
                        } else {
                            first_line.to_string()
                        };
                        result.push_str(&format!("    {}\n", truncated));
                    }
                }
            }
        }

        result
    }

    /// Format pytest output as raw (just test names with status).
    pub(crate) fn format_pytest_raw(output: &PytestOutput) -> String {
        let mut result = String::new();

        for test in &output.tests {
            let status = match test.status {
                TestStatus::Passed => "PASS",
                TestStatus::Failed => "FAIL",
                TestStatus::Skipped => "SKIP",
                TestStatus::XFailed => "XFAIL",
                TestStatus::XPassed => "XPASS",
                TestStatus::Error => "ERROR",
            };
            result.push_str(&format!("{} {}\n", status, test.name));
        }

        result
    }

    /// Format pytest output for AI agent consumption.
    pub(crate) fn format_pytest_agent(output: &PytestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!("- Total: {}\n", output.summary.total));
        result.push_str(&format!("- Passed: {}\n", output.summary.passed));
        result.push_str(&format!("- Failed: {}\n", output.summary.failed));
        if output.summary.skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!("- XFailed: {}\n", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!("- XPassed: {}\n", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!("- Errors: {}\n", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str("## Failed Tests\n\n");
            for test in failed_tests {
                result.push_str(&format!("### {}\n", test.name));
                if let Some(ref file) = test.file {
                    result.push_str(&format!("File: {}", file));
                    if let Some(line) = test.line {
                        result.push_str(&format!(":{}", line));
                    }
                    result.push('\n');
                }
                if let Some(ref msg) = test.error_message {
                    result.push_str(&format!("\n```\n{}\n```\n", msg));
                }
                result.push('\n');
            }
        }

        result
    }

    /// Parse Jest output into structured data.
    pub(crate) fn parse_jest(input: &str) -> CommandResult<JestOutput> {
        let mut output = JestOutput::default();
        let mut current_suite: Option<JestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "PASS src/path/to/test.js" or "FAIL src/path/to/test.js"
            if trimmed.starts_with("PASS ") || trimmed.starts_with("FAIL ") {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let (passed, file) = if trimmed.starts_with("PASS ") {
                    (true, trimmed.strip_prefix("PASS ").unwrap_or("").trim())
                } else {
                    (false, trimmed.strip_prefix("FAIL ").unwrap_or("").trim())
                };

                current_suite = Some(JestTestSuite {
                    file: file.to_string(),
                    passed,
                    duration: None,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                continue;
            }

            // Detect individual test results
            // Format: "  ✓ test name (5 ms)" or "  ✕ test name" or "  ○ skipped"
            if let Some(test) = Self::parse_jest_test_line(trimmed) {
                if let Some(ref mut suite) = current_suite {
                    suite.tests.push(test);
                }
                continue;
            }

            // Detect test suite duration: "(5 ms)"
            if trimmed.starts_with('(') && trimmed.ends_with(')') && current_suite.is_some() {
                let duration_str = trimmed.trim_matches(|c| c == '(' || c == ')');
                let duration = Self::parse_jest_duration(duration_str);
                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                continue;
            }

            // Detect failure details start
            // "  ● test name › should work"
            if trimmed.starts_with("● ") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                let name = trimmed.strip_prefix("● ").unwrap_or("").trim().to_string();
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Accumulate failure details
            if in_failure_details && current_failed_test.is_some() {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary line: "Test Suites: X passed, Y failed, Z total"
            if trimmed.starts_with("Test Suites:") {
                let summary = Self::parse_jest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Additional summary lines: "Tests:", "Snapshots:", "Time:"
            if trimmed.starts_with("Tests:") {
                Self::parse_jest_tests_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Snapshots:") {
                Self::parse_jest_snapshots_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Time:") {
                Self::parse_jest_time_summary(trimmed, &mut output.summary);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Save last failure info (if any)
        // Note: Error messages are typically captured when we see the next test or suite
        // so we don't need to explicitly save the last one here

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        JestTestStatus::Passed => output.summary.tests_passed += 1,
                        JestTestStatus::Failed => output.summary.tests_failed += 1,
                        JestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        JestTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                    output.summary.tests_total += 1;
                }
            }
        }

        // Determine success
        output.success = output.summary.tests_failed == 0
            && output.summary.suites_failed == 0
            && output.summary.tests_total > 0;
        output.is_empty = output.test_suites.is_empty() && output.summary.tests_total == 0;

        Ok(output)
    }

    /// Parse a single Jest test result line.
    pub(crate) fn parse_jest_test_line(line: &str) -> Option<JestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        if !line.starts_with("✓") && !line.starts_with("✕") && !line.starts_with("○") {
            return None;
        }

        let (status, remainder) = if line.starts_with("✓") {
            (JestTestStatus::Passed, line.strip_prefix("✓").unwrap_or(""))
        } else if line.starts_with("✕") {
            (JestTestStatus::Failed, line.strip_prefix("✕").unwrap_or(""))
        } else if line.starts_with("○") {
            // Could be skipped or todo
            let rem = line.strip_prefix("○").unwrap_or("");
            if rem.contains("skipped") || rem.contains("skip") {
                (JestTestStatus::Skipped, rem)
            } else if rem.contains("todo") {
                (JestTestStatus::Todo, rem)
            } else {
                (JestTestStatus::Skipped, rem)
            }
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name (5 ms)"
        let (test_name, duration) = if let Some(paren_pos) = trimmed.rfind('(') {
            let name_part = trimmed[..paren_pos].trim();
            let duration_part = &trimmed[paren_pos..];
            let duration =
                Self::parse_jest_duration(duration_part.trim_matches(|c| c == '(' || c == ')'));
            (name_part.to_string(), duration)
        } else {
            (trimmed.to_string(), None)
        };

        // Parse ancestors (describe blocks) from test name
        // Format: "describe block > nested describe > test name"
        let (ancestors, final_name) = if test_name.contains('>') || test_name.contains("›") {
            let delimiter = if test_name.contains('>') { ">" } else { "›" };
            let parts: Vec<&str> = test_name.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() > 1 {
                let ancestors: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let name = parts.last().unwrap_or(&"").to_string();
                (ancestors, name)
            } else {
                (Vec::new(), test_name.clone())
            }
        } else {
            (Vec::new(), test_name.clone())
        };

        Some(JestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Jest duration string (e.g., "5 ms", "1.23 s").
    pub(crate) fn parse_jest_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") || s.ends_with("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") {
            Some(num)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Jest summary line for test suites.
    pub(crate) fn parse_jest_summary(line: &str) -> JestSummary {
        let mut summary = JestSummary::default();
        let line = line.strip_prefix("Test Suites:").unwrap_or("");

        // Parse pattern: "X passed, Y failed, Z total" or "X passed, Y total"
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.suites_passed = extract_count(line, "passed");
        summary.suites_failed = extract_count(line, "failed");
        summary.suites_total = extract_count(line, "total");

        summary
    }

    /// Parse Jest summary line for tests.
    pub(crate) fn parse_jest_tests_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Tests:").unwrap_or("");

        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");
        summary.tests_total = extract_count(line, "total");
    }

    /// Parse Jest summary line for snapshots.
    pub(crate) fn parse_jest_snapshots_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Snapshots:").unwrap_or("");
        // Try to extract a number from the line
        let num_str: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = num_str.parse() {
            summary.snapshots = Some(num);
        }
    }

    /// Parse Jest summary line for time.
    pub(crate) fn parse_jest_time_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Time:").unwrap_or("").trim();
        summary.duration = Self::parse_jest_duration(line);
    }

    /// Format Jest output based on the requested format.
    pub(crate) fn format_jest(output: &JestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_jest_json(output),
            OutputFormat::Compact => Self::format_jest_compact(output),
            OutputFormat::Raw => Self::format_jest_raw(output),
            OutputFormat::Agent => Self::format_jest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_jest_compact(output),
        }
    }

    /// Format Jest output as JSON.
    pub(crate) fn format_jest_json(output: &JestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites": {
                    "passed": output.summary.suites_passed,
                    "failed": output.summary.suites_failed,
                    "total": output.summary.suites_total,
                },
                "tests": {
                    "passed": output.summary.tests_passed,
                    "failed": output.summary.tests_failed,
                    "skipped": output.summary.tests_skipped,
                    "todo": output.summary.tests_todo,
                    "total": output.summary.tests_total,
                },
                "snapshots": output.summary.snapshots,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        JestTestStatus::Passed => "passed",
                        JestTestStatus::Failed => "failed",
                        JestTestStatus::Skipped => "skipped",
                        JestTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "jest_version": output.jest_version,
            "test_path_pattern": output.test_path_pattern,
        })
        .to_string()
    }

    /// Format Jest output in compact format.
    pub(crate) fn format_jest_compact(output: &JestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if output.summary.tests_todo > 0 {
                result.push_str(&format!(", {} todo", output.summary.tests_todo));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!(", {} todo", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed test suites
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str(&format!("failed suites ({}):\n", failed_suites.len()));
            for suite in failed_suites {
                result.push_str(&format!("  {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("    ✕ {}\n", test.name));
                    if let Some(ref msg) = test.error_message {
                        if let Some(first_line) = msg.lines().next() {
                            let truncated = if first_line.len() > 80 {
                                format!("{}...", &first_line[..77])
                            } else {
                                first_line.to_string()
                            };
                            result.push_str(&format!("      {}\n", truncated));
                        }
                    }
                }
            }
        }

        result
    }

    /// Format Jest output as raw (just test names with status).
    pub(crate) fn format_jest_raw(output: &JestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let suite_status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", suite_status, suite.file));
            for test in &suite.tests {
                let status = match test.status {
                    JestTestStatus::Passed => "PASS",
                    JestTestStatus::Failed => "FAIL",
                    JestTestStatus::Skipped => "SKIP",
                    JestTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", status, test.name));
            }
        }

        result
    }

    /// Format Jest output for AI agent consumption.
    pub(crate) fn format_jest_agent(output: &JestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Suites: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(snapshots) = output.summary.snapshots {
            result.push_str(&format!("- Snapshots: {}\n", snapshots));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Suites\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == JestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // Vitest Parsing and Formatting
    // ============================================================

    /// Parse Vitest output into structured data.
    pub(crate) fn parse_vitest(input: &str) -> CommandResult<VitestOutput> {
        let mut output = VitestOutput::default();
        let mut current_suite: Option<VitestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;
        let mut in_suite_tree = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "✓ test/example.test.ts (5 tests) 306ms"
            // or: "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
            // or: "✗ test/example.test.ts (5 tests | 1 failed) 306ms"
            if let Some(suite_info) = Self::parse_vitest_suite_header(trimmed) {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                current_suite = Some(VitestTestSuite {
                    file: suite_info.file,
                    passed: suite_info.passed,
                    duration: suite_info.duration,
                    test_count: suite_info.test_count,
                    skipped_count: suite_info.skipped_count,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                in_suite_tree = true;
                continue;
            }

            // Detect test in tree format (indented test results)
            // "   ✓ test name 1ms" or "   ✕ test name"
            if in_suite_tree && line.starts_with("   ") {
                let test_line = line.trim_start();
                if let Some(test) = Self::parse_vitest_test_line(test_line) {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                    continue;
                }
            }

            // Detect failure details start
            // " ❯ test/file.test.ts:10:5"
            // "AssertionError: expected 5 to be 4"
            if trimmed.starts_with("❯ ") && trimmed.contains(".test.") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                // Extract test name from file reference like "❯ test/file.test.ts:10:5 > test name"
                let remainder = trimmed.strip_prefix("❯ ").unwrap_or("");
                // The test name is often after the file location
                let name = if let Some(pos) = remainder.find('>') {
                    remainder[pos + 1..].trim().to_string()
                } else {
                    // Try to get just the file path context
                    remainder.to_string()
                };
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Detect assertion error line
            if trimmed.starts_with("AssertionError:") || trimmed.starts_with("Error:") {
                in_failure_details = true;
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Accumulate failure details
            if in_failure_details
                && (trimmed.starts_with("at ")
                    || trimmed.starts_with("expected")
                    || trimmed.contains("to be")
                    || failure_buffer.len() > 0)
            {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary section
            // " Test Files  4 passed (4)"
            if trimmed.starts_with("Test Files") {
                let summary = Self::parse_vitest_test_files_summary(trimmed);
                output.summary.suites_passed = summary.suites_passed;
                output.summary.suites_failed = summary.suites_failed;
                output.summary.suites_total = summary.suites_total;
                in_suite_tree = false;
                continue;
            }

            // "      Tests  16 passed | 4 skipped (20)"
            if trimmed.starts_with("Tests") && !trimmed.starts_with("Tests:") {
                Self::parse_vitest_tests_summary(trimmed, &mut output.summary);
                continue;
            }

            // "   Start at  12:34:32"
            if trimmed.starts_with("Start at") {
                let time = trimmed.strip_prefix("Start at").unwrap_or("").trim();
                output.summary.start_at = Some(time.to_string());
                continue;
            }

            // "   Duration  1.26s"
            if trimmed.starts_with("Duration") {
                let duration_str = trimmed.strip_prefix("Duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_vitest_duration(duration_str);
                continue;
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        VitestTestStatus::Passed => output.summary.tests_passed += 1,
                        VitestTestStatus::Failed => output.summary.tests_failed += 1,
                        VitestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        VitestTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                    output.summary.tests_total += 1;
                }
            }
        }

        // Determine success
        output.success = output.summary.tests_failed == 0
            && output.summary.suites_failed == 0
            && output.summary.tests_total > 0;
        output.is_empty = output.test_suites.is_empty() && output.summary.tests_total == 0;

        Ok(output)
    }

    /// Parse vitest suite header like "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
    pub(crate) fn parse_vitest_suite_header(line: &str) -> Option<VitestSuiteInfo> {
        let line = line.trim_start();

        let (passed, remainder) = if line.starts_with('✓') {
            (true, line.strip_prefix('✓')?.trim_start())
        } else if line.starts_with('✗') {
            (false, line.strip_prefix('✗')?.trim_start())
        } else if line.starts_with('×') {
            (false, line.strip_prefix('×')?.trim_start())
        } else if line.starts_with("FAIL") {
            (false, line.strip_prefix("FAIL")?.trim_start())
        } else if line.starts_with("PASS") {
            (true, line.strip_prefix("PASS")?.trim_start())
        } else {
            return None;
        };

        // Extract file path - everything before the parenthesis
        let paren_pos = remainder.find('(')?;
        let file = remainder[..paren_pos].trim().to_string();
        let rest = &remainder[paren_pos..];

        // Parse test count info: "(5 tests)" or "(5 tests | 1 skipped)" or "(5 tests | 1 failed)"
        let mut test_count = None;
        let mut skipped_count = None;

        if rest.starts_with('(') && rest.contains(')') {
            let end_paren = rest.find(')').unwrap_or(rest.len());
            let info = &rest[1..end_paren];

            // Extract test count
            if let Some(pos) = info.find(" test") {
                let num_str: String = info[..pos].chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<usize>() {
                    test_count = Some(num);
                }
            }

            // Extract skipped count
            if let Some(pos) = info.find("skipped") {
                let before = &info[..pos];
                if let Some(num_str) = before.rsplit('|').next() {
                    let num_str: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = num_str.parse::<usize>() {
                        skipped_count = Some(num);
                    }
                }
            }
        }

        // Extract duration - look for number followed by ms or s at the end
        let duration = if rest.contains("ms") || rest.contains('s') && !rest.contains("ms") {
            // Find duration at the end of the line
            let after_paren = rest.find(')').map(|p| &rest[p + 1..]).unwrap_or("");
            Self::parse_vitest_duration(after_paren.trim())
        } else {
            None
        };

        Some(VitestSuiteInfo {
            file,
            passed,
            duration,
            test_count,
            skipped_count,
        })
    }

    /// Parse a single Vitest test result line.
    pub(crate) fn parse_vitest_test_line(line: &str) -> Option<VitestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        // Vitest uses: ✓ (passed), ✕/× (failed), ↩ (skipped), etc.
        let (status, remainder) = if line.starts_with('✓') {
            (
                VitestTestStatus::Passed,
                line.strip_prefix('✓')?.trim_start(),
            )
        } else if line.starts_with('✕') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('✕')?.trim_start(),
            )
        } else if line.starts_with('×') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('×')?.trim_start(),
            )
        } else if line.starts_with('↩') {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix('↩')?.trim_start(),
            )
        } else if line.starts_with("↓") {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix("↓")?.trim_start(),
            )
        } else if line.contains("skipped") || line.contains("skip") {
            (VitestTestStatus::Skipped, line)
        } else if line.contains("todo") {
            (VitestTestStatus::Todo, line)
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name 1ms" or "test name 1.5s"
        let (test_name, duration) = if let Some(ms_pos) = trimmed.rfind("ms") {
            // Find the number before "ms"
            let before = &trimmed[..ms_pos];
            let num_start = before
                .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                .map(|p| p + 1)
                .unwrap_or(0);
            let name_part = before[..num_start].trim();
            let duration_str = &before[num_start..];
            let duration = duration_str.parse::<f64>().ok().map(|d| d / 1000.0);
            (name_part.to_string(), duration)
        } else if let Some(s_pos) = trimmed.rfind('s') {
            // Check if it's a duration (not part of a word)
            let before = &trimmed[..s_pos];
            if before.ends_with(|c: char| c.is_ascii_digit()) {
                let num_start = before
                    .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let name_part = before[..num_start].trim();
                let duration_str = &before[num_start..];
                let duration = duration_str.parse::<f64>().ok();
                (name_part.to_string(), duration)
            } else {
                (trimmed.to_string(), None)
            }
        } else {
            (trimmed.to_string(), None)
        };

        // Parse ancestors (describe blocks) from test name
        // Format: "describe block > nested describe > test name"
        let (ancestors, final_name) = if test_name.contains('>') || test_name.contains("›") {
            let delimiter = if test_name.contains('>') { ">" } else { "›" };
            let parts: Vec<&str> = test_name.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() > 1 {
                let ancestors: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let name = parts.last().unwrap_or(&"").to_string();
                (ancestors, name)
            } else {
                (Vec::new(), test_name.clone())
            }
        } else {
            (Vec::new(), test_name.clone())
        };

        Some(VitestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Vitest duration string (e.g., "5ms", "1.26s").
    pub(crate) fn parse_vitest_duration(s: &str) -> Option<f64> {
        let s = s.trim();

        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") && !s.contains("start") {
            Some(num)
        } else if s.contains('m') && !s.contains("ms") {
            Some(num * 60.0)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Vitest "Test Files" summary line.
    pub(crate) fn parse_vitest_test_files_summary(line: &str) -> VitestSummary {
        let mut summary = VitestSummary::default();
        let line = line.strip_prefix("Test Files").unwrap_or("").trim();

        // Parse pattern: "4 passed (4)" or "2 passed, 1 failed (3)"
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.suites_passed = extract_count(line, "passed");
        summary.suites_failed = extract_count(line, "failed");

        // Total is in parentheses
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let total_str = &line[start + 1..end];
                summary.suites_total = total_str.parse().unwrap_or(0);
            }
        }

        summary
    }

    /// Parse Vitest "Tests" summary line.
    pub(crate) fn parse_vitest_tests_summary(line: &str, summary: &mut VitestSummary) {
        let line = line.strip_prefix("Tests").unwrap_or("").trim();

        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                // Find the number before the label
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");

        // Total is in parentheses at the end
        if let Some(start) = line.rfind('(') {
            if let Some(end) = line.rfind(')') {
                if end > start {
                    let total_str = &line[start + 1..end];
                    summary.tests_total = total_str.parse().unwrap_or(0);
                }
            }
        }
    }

    /// Format Vitest output based on the requested format.
    pub(crate) fn format_vitest(output: &VitestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_vitest_json(output),
            OutputFormat::Compact => Self::format_vitest_compact(output),
            OutputFormat::Raw => Self::format_vitest_raw(output),
            OutputFormat::Agent => Self::format_vitest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_vitest_compact(output),
        }
    }

    /// Format Vitest output as JSON.
    pub(crate) fn format_vitest_json(output: &VitestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites": {
                    "passed": output.summary.suites_passed,
                    "failed": output.summary.suites_failed,
                    "total": output.summary.suites_total,
                },
                "tests": {
                    "passed": output.summary.tests_passed,
                    "failed": output.summary.tests_failed,
                    "skipped": output.summary.tests_skipped,
                    "todo": output.summary.tests_todo,
                    "total": output.summary.tests_total,
                },
                "duration": output.summary.duration,
                "start_at": output.summary.start_at,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "test_count": suite.test_count,
                "skipped_count": suite.skipped_count,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        VitestTestStatus::Passed => "passed",
                        VitestTestStatus::Failed => "failed",
                        VitestTestStatus::Skipped => "skipped",
                        VitestTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "vitest_version": output.vitest_version,
        })
        .to_string()
    }

    /// Format Vitest output in compact format.
    pub(crate) fn format_vitest_compact(output: &VitestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} test files, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if output.summary.tests_todo > 0 {
                result.push_str(&format!(", {} todo", output.summary.tests_todo));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} test files ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!(", {} todo", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed test suites
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str(&format!("failed suites ({}):\n", failed_suites.len()));
            for suite in failed_suites {
                result.push_str(&format!("  {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("    ✕ {}\n", test.name));
                    if let Some(ref msg) = test.error_message {
                        if let Some(first_line) = msg.lines().next() {
                            let truncated = if first_line.len() > 80 {
                                format!("{}...", &first_line[..77])
                            } else {
                                first_line.to_string()
                            };
                            result.push_str(&format!("      {}\n", truncated));
                        }
                    }
                }
            }
        }

        result
    }

    /// Format Vitest output as raw (just test names with status).
    pub(crate) fn format_vitest_raw(output: &VitestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let suite_status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", suite_status, suite.file));
            for test in &suite.tests {
                let status = match test.status {
                    VitestTestStatus::Passed => "PASS",
                    VitestTestStatus::Failed => "FAIL",
                    VitestTestStatus::Skipped => "SKIP",
                    VitestTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", status, test.name));
            }
        }

        result
    }

    /// Format Vitest output for AI agent consumption.
    pub(crate) fn format_vitest_agent(output: &VitestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        if let Some(ref start_at) = output.summary.start_at {
            result.push_str(&format!("- Start at: {}\n", start_at));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // NPM Test (Node.js built-in test runner) Parser
    // ============================================================

    /// Parse npm test output into structured data.
    ///
    /// The Node.js built-in test runner (node --test) with spec reporter outputs:
    /// ```text
    /// ▶ test/file.test.js
    ///   ✔ test name (5.123ms)
    ///   ✖ failing test
    ///     code: ...
    ///   ℹ skipped test # SKIP
    ///   ℹ todo test # TODO
    /// ▶ test/file.test.js (12.345ms)
    /// ```
    pub(crate) fn parse_npm_test(input: &str) -> CommandResult<NpmTestOutput> {
        let mut output = NpmTestOutput::default();
        let mut current_suite: Option<NpmTestSuite> = None;
        let mut current_test: Option<NpmTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new(); // Track nested test names

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for npm test output header (e.g., "> project@1.0.0 test")
            if trimmed.starts_with('>') && trimmed.contains("test") {
                continue;
            }

            // Check for summary lines at the end
            // "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
            if trimmed.starts_with("✔ tests") || trimmed.starts_with("✖ tests") {
                Self::parse_npm_test_summary_tests(trimmed, &mut output.summary);
                continue;
            }

            // "✔ test files 2 passed (2)" or "✖ test files 1 failed (2)"
            if trimmed.starts_with("✔ test files") || trimmed.starts_with("✖ test files") {
                Self::parse_npm_test_summary_files(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ tests 4 passed (4)" (alternative format)
            if trimmed.starts_with("ℹ tests") {
                Self::parse_npm_test_summary_tests_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ test files 2 passed (2)" (alternative format)
            if trimmed.starts_with("ℹ test files") {
                Self::parse_npm_test_summary_files_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ duration 123ms" or "ℹ duration 1.234s"
            if trimmed.starts_with("ℹ duration") {
                let duration_str = trimmed.strip_prefix("ℹ duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_npm_duration(duration_str);
                continue;
            }

            // Check for test file start: "▶ path/to/test.js"
            if trimmed.starts_with('▶') && !trimmed.contains('(') {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let file = trimmed
                    .strip_prefix('▶')
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                current_suite = Some(NpmTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Check for test file end with duration: "▶ path/to/test.js (123.456ms)"
            if trimmed.starts_with('▶') && trimmed.contains('(') {
                let duration = Self::extract_npm_suite_duration(trimmed);

                // First, save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                // Save the suite
                if let Some(suite) = current_suite.take() {
                    // Update suite passed status based on tests
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == NpmTestStatus::Failed);
                    let suite_to_save = NpmTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Parse test results
            // Check if line is inside a test suite (indented or starts with test marker)
            let is_test_line = line.starts_with("  ")
                || line.starts_with("\t")
                || trimmed.starts_with("✔")
                || trimmed.starts_with("✖")
                || trimmed.starts_with("ℹ");

            if is_test_line && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✔")
                    && !trimmed.starts_with("✖")
                    && !trimmed.starts_with("ℹ")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_npm_test_line(trimmed, &indent_stack) {
                    // Extract test_name before moving
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == NpmTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == NpmTestStatus::Failed);
            let suite_to_save = NpmTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_npm_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single npm test result line.
    pub(crate) fn parse_npm_test_line(line: &str, ancestors: &[String]) -> Option<NpmTest> {
        let line = line.trim_start();

        // Parse passed test: "✔ test name (5.123ms)"
        if line.starts_with("✔") {
            let rest = line.strip_prefix("✔").unwrap_or(line).trim();
            let (name, duration) = Self::split_npm_test_name_and_duration(rest);
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test: "✖ test name"
        if line.starts_with("✖") {
            let rest = line.strip_prefix("✖").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse skipped test: "ℹ test name # SKIP"
        if line.starts_with("ℹ") && line.contains("# SKIP") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# SKIP")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse todo test: "ℹ test name # TODO"
        if line.starts_with("ℹ") && line.contains("# TODO") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# TODO")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Split test name and duration from a line like "test name (5.123ms)"
    pub(crate) fn split_npm_test_name_and_duration(line: &str) -> (String, Option<f64>) {
        // Look for duration at the end: "(5.123ms)" or "(1.234s)"
        if let Some(paren_pos) = line.rfind('(') {
            let name_part = line[..paren_pos].trim();
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1]; // Remove parens
                let duration = Self::parse_npm_duration(duration_str);
                return (name_part.to_string(), duration);
            }
        }
        (line.to_string(), None)
    }

    /// Parse npm duration string (e.g., "5.123ms", "1.234s").
    pub(crate) fn parse_npm_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            let num_str = &s[..s.len() - 2];
            num_str.parse::<f64>().ok().map(|n| n / 1000.0)
        } else if s.ends_with('s') {
            let num_str = &s[..s.len() - 1];
            num_str.parse::<f64>().ok()
        } else {
            None
        }
    }

    /// Extract duration from suite end line like "▶ path/to/test.js (123.456ms)"
    pub(crate) fn extract_npm_suite_duration(line: &str) -> Option<f64> {
        if let Some(paren_pos) = line.rfind('(') {
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1];
                return Self::parse_npm_duration(duration_str);
            }
        }
        None
    }

    /// Parse npm test summary for tests: "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
    pub(crate) fn parse_npm_test_summary_tests(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files: "✔ test files 2 passed (2)"
    pub(crate) fn parse_npm_test_summary_files(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse npm test summary for tests (info format): "ℹ tests 4 passed (4)"
    pub(crate) fn parse_npm_test_summary_tests_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts_with_todo(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_todo,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files (info format): "ℹ test files 2 passed (2)"
    pub(crate) fn parse_npm_test_summary_files_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse count pattern like "4 passed (4)" or "2 passed 1 failed (3)"
    pub(crate) fn parse_npm_counts(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Parse npm test summary line with todo support.
    pub(crate) fn parse_npm_counts_with_todo(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        todo: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        "todo" => *todo = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Update summary counts from parsed tests.
    pub(crate) fn update_npm_summary_from_tests(output: &mut NpmTestOutput) {
        // Only update if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }

                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        NpmTestStatus::Passed => output.summary.tests_passed += 1,
                        NpmTestStatus::Failed => output.summary.tests_failed += 1,
                        NpmTestStatus::Skipped => output.summary.tests_skipped += 1,
                        NpmTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format npm test output based on the requested format.
    pub(crate) fn format_npm_test(output: &NpmTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_npm_test_json(output),
            OutputFormat::Compact => Self::format_npm_test_compact(output),
            OutputFormat::Raw => Self::format_npm_test_raw(output),
            OutputFormat::Agent => Self::format_npm_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_npm_test_compact(output),
        }
    }

    /// Format npm test output as JSON.
    pub(crate) fn format_npm_test_json(output: &NpmTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == NpmTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        NpmTestStatus::Passed => "passed",
                        NpmTestStatus::Failed => "failed",
                        NpmTestStatus::Skipped => "skipped",
                        NpmTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "node_version": output.node_version,
        })
        .to_string()
    }

    /// Format npm test output in compact format.
    pub(crate) fn format_npm_test_compact(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("npm test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == NpmTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format npm test output as raw (just test names with status).
    pub(crate) fn format_npm_test_raw(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    NpmTestStatus::Passed => "PASS",
                    NpmTestStatus::Failed => "FAIL",
                    NpmTestStatus::Skipped => "SKIP",
                    NpmTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format npm test output for AI agent consumption.
    pub(crate) fn format_npm_test_agent(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == NpmTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // PNPM Test Parser Implementation
    // ============================================================

    /// Parse pnpm test output into structured data.
    /// pnpm test output format is identical to npm test (Node.js built-in test runner).
    ///
    /// Expected format:
    /// ```text
    /// ▶ test/file.test.js
    ///   ✔ should work correctly (5.123ms)
    ///   ✖ should fail
    ///     AssertionError: values are not equal
    ///   ℹ skipped test # SKIP
    ///   ℹ todo test # TODO
    /// ▶ test/file.test.js (12.345ms)
    /// ```
    pub(crate) fn parse_pnpm_test(input: &str) -> CommandResult<PnpmTestOutput> {
        let mut output = PnpmTestOutput::default();
        let mut current_suite: Option<PnpmTestSuite> = None;
        let mut current_test: Option<PnpmTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for pnpm version line (e.g., "pnpm: 9.0.0")
            if trimmed.starts_with("pnpm:") || trimmed.starts_with("PNPM:") {
                output.pnpm_version = Some(
                    trimmed
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }

            // Check for summary lines at the end
            // "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
            if trimmed.starts_with("✔ tests") || trimmed.starts_with("✖ tests") {
                Self::parse_pnpm_test_summary_tests(trimmed, &mut output.summary);
                continue;
            }

            // "✔ test files 2 passed (2)" or "✖ test files 1 failed (2)"
            if trimmed.starts_with("✔ test files") || trimmed.starts_with("✖ test files") {
                Self::parse_pnpm_test_summary_files(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ tests 4 passed (4)" (alternative format)
            if trimmed.starts_with("ℹ tests") {
                Self::parse_pnpm_test_summary_tests_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ test files 2 passed (2)" (alternative format)
            if trimmed.starts_with("ℹ test files") {
                Self::parse_pnpm_test_summary_files_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ duration 123ms" or "ℹ duration 1.234s"
            if trimmed.starts_with("ℹ duration") {
                let duration_str = trimmed.strip_prefix("ℹ duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_pnpm_duration(duration_str);
                continue;
            }

            // Check for test file start: "▶ path/to/test.js"
            if trimmed.starts_with('▶') && !trimmed.contains('(') {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let file = trimmed
                    .strip_prefix('▶')
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                current_suite = Some(PnpmTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Check for test file end with duration: "▶ path/to/test.js (123.456ms)"
            if trimmed.starts_with('▶') && trimmed.contains('(') {
                let duration = Self::extract_pnpm_suite_duration(trimmed);

                // First, save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                // Save the suite
                if let Some(suite) = current_suite.take() {
                    // Update suite passed status based on tests
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == PnpmTestStatus::Failed);
                    let suite_to_save = PnpmTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Parse test results
            // Check if line is inside a test suite (indented or starts with test marker)
            let is_test_line = line.starts_with("  ")
                || line.starts_with("\t")
                || trimmed.starts_with("✔")
                || trimmed.starts_with("✖")
                || trimmed.starts_with("ℹ");

            if is_test_line && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✔")
                    && !trimmed.starts_with("✖")
                    && !trimmed.starts_with("ℹ")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_pnpm_test_line(trimmed, &indent_stack) {
                    // Extract test_name before moving
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == PnpmTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == PnpmTestStatus::Failed);
            let suite_to_save = PnpmTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_pnpm_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single pnpm test result line.
    pub(crate) fn parse_pnpm_test_line(line: &str, ancestors: &[String]) -> Option<PnpmTest> {
        let line = line.trim_start();

        // Parse passed test: "✔ test name (5.123ms)"
        if line.starts_with("✔") {
            let rest = line.strip_prefix("✔").unwrap_or(line).trim();
            let (name, duration) = Self::split_pnpm_test_name_and_duration(rest);
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test: "✖ test name"
        if line.starts_with("✖") {
            let rest = line.strip_prefix("✖").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse skipped test: "ℹ test name # SKIP"
        if line.starts_with("ℹ") && line.contains("# SKIP") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# SKIP")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse todo test: "ℹ test name # TODO"
        if line.starts_with("ℹ") && line.contains("# TODO") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# TODO")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(PnpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: PnpmTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Parse duration string like "5.123ms" or "1.234s" into seconds.
    pub(crate) fn parse_pnpm_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            s.strip_suffix("ms")
                .and_then(|n| n.parse::<f64>().ok())
                .map(|ms| ms / 1000.0)
        } else if s.ends_with("s") {
            s.strip_suffix("s").and_then(|n| n.parse::<f64>().ok())
        } else {
            None
        }
    }

    /// Split test name and duration from a string like "test name (5.123ms)".
    pub(crate) fn split_pnpm_test_name_and_duration(s: &str) -> (String, Option<f64>) {
        // Look for duration in parentheses at the end
        if let Some(start) = s.rfind('(') {
            if let Some(end) = s[start..].find(')') {
                let duration_str = &s[start + 1..start + end];
                let name = s[..start].trim().to_string();
                let duration = Self::parse_pnpm_duration(duration_str);
                return (name, duration);
            }
        }
        (s.to_string(), None)
    }

    /// Extract duration from suite end line like "▶ test.js (123.456ms)".
    pub(crate) fn extract_pnpm_suite_duration(line: &str) -> Option<f64> {
        if let Some(start) = line.rfind('(') {
            if let Some(end) = line[start..].find(')') {
                let duration_str = &line[start + 1..start + end];
                return Self::parse_pnpm_duration(duration_str);
            }
        }
        None
    }

    /// Parse pnpm test summary for tests: "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
    pub(crate) fn parse_pnpm_test_summary_tests(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_total,
        );
    }

    /// Parse pnpm test summary for test files: "✔ test files 2 passed (2)"
    pub(crate) fn parse_pnpm_test_summary_files(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse pnpm test summary for tests (info format): "ℹ tests 4 passed (4)"
    pub(crate) fn parse_pnpm_test_summary_tests_info(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_pnpm_counts_with_todo(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_todo,
            &mut summary.tests_total,
        );
    }

    /// Parse pnpm test summary for test files (info format): "ℹ test files 2 passed (2)"
    pub(crate) fn parse_pnpm_test_summary_files_info(line: &str, summary: &mut PnpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_pnpm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse count pattern like "4 passed (4)" or "2 passed 1 failed (3)"
    pub(crate) fn parse_pnpm_counts(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Parse pnpm test summary line with todo support.
    pub(crate) fn parse_pnpm_counts_with_todo(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        todo: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        "todo" => *todo = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Update summary counts from parsed tests.
    pub(crate) fn update_pnpm_summary_from_tests(output: &mut PnpmTestOutput) {
        // Only update if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }

                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        PnpmTestStatus::Passed => output.summary.tests_passed += 1,
                        PnpmTestStatus::Failed => output.summary.tests_failed += 1,
                        PnpmTestStatus::Skipped => output.summary.tests_skipped += 1,
                        PnpmTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format pnpm test output based on the requested format.
    pub(crate) fn format_pnpm_test(output: &PnpmTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_pnpm_test_json(output),
            OutputFormat::Compact => Self::format_pnpm_test_compact(output),
            OutputFormat::Raw => Self::format_pnpm_test_raw(output),
            OutputFormat::Agent => Self::format_pnpm_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_pnpm_test_compact(output),
        }
    }

    /// Format pnpm test output as JSON.
    pub(crate) fn format_pnpm_test_json(output: &PnpmTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == PnpmTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        PnpmTestStatus::Passed => "passed",
                        PnpmTestStatus::Failed => "failed",
                        PnpmTestStatus::Skipped => "skipped",
                        PnpmTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "pnpm_version": output.pnpm_version,
        })
        .to_string()
    }

    /// Format pnpm test output in compact format.
    pub(crate) fn format_pnpm_test_compact(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("pnpm test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == PnpmTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format pnpm test output as raw (just test names with status).
    pub(crate) fn format_pnpm_test_raw(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    PnpmTestStatus::Passed => "PASS",
                    PnpmTestStatus::Failed => "FAIL",
                    PnpmTestStatus::Skipped => "SKIP",
                    PnpmTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format pnpm test output for AI agent consumption.
    pub(crate) fn format_pnpm_test_agent(output: &PnpmTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == PnpmTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    // ============================================================
    // Bun Test Parsing and Formatting
    // ============================================================

    /// Parse Bun test output into structured data.
    ///
    /// Expected format (default console reporter):
    /// ```text
    /// test/package-json-lint.test.ts:
    /// ✓ test/package.json [0.88ms]
    /// ✓ test/js/third_party/grpc-js/package.json [0.18ms]
    ///
    ///  4 pass
    ///  0 fail
    ///  4 expect() calls
    /// Ran 4 tests in 1.44ms
    /// ```
    ///
    /// For non-TTY environments (no colors):
    /// ```text
    /// test/package-json-lint.test.ts:
    /// (pass) test/package.json [0.48ms]
    /// (fail) test/failing.test.ts
    /// (skip) test/skipped.test.ts
    /// ```
    pub(crate) fn parse_bun_test(input: &str) -> CommandResult<BunTestOutput> {
        let mut output = BunTestOutput::default();
        let mut current_suite: Option<BunTestSuite> = None;
        let mut current_test: Option<BunTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new();
        let mut in_suite = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines, but first save any pending test
            if trimmed.is_empty() {
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                in_error_details = false;
                continue;
            }

            // Check for bun version line (e.g., "bun: 1.0.0" or "Bun v1.0.0")
            if trimmed.starts_with("bun:") || trimmed.starts_with("Bun v") {
                output.bun_version = Some(
                    trimmed
                        .split(|c| c == ':' || c == 'v')
                        .last()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }

            // Check for summary lines at the end
            // "X pass" or "Y fail" or "X expect() calls"
            if Self::is_bun_summary_line(trimmed) {
                // Save any pending test before processing summary
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                Self::parse_bun_summary_line(trimmed, &mut output.summary);
                continue;
            }

            // "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
            if trimmed.starts_with("Ran ") && trimmed.contains(" tests") {
                Self::parse_bun_ran_line(trimmed, &mut output.summary);
                continue;
            }

            // Check for test file header: "test/file.test.ts:" (ends with colon)
            if trimmed.ends_with(':')
                && !trimmed.starts_with(|c| c == '✓' || c == '✗' || c == '×' || c == '(')
            {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == BunTestStatus::Failed);
                    let suite_to_save = BunTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }

                let file = trimmed.trim_end_matches(':').to_string();
                current_suite = Some(BunTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                in_suite = true;
                continue;
            }

            // Parse test results if we're in a suite
            if in_suite && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✓")
                    && !trimmed.starts_with("✗")
                    && !trimmed.starts_with("×")
                    && !trimmed.starts_with("(pass)")
                    && !trimmed.starts_with("(fail)")
                    && !trimmed.starts_with("(skip)")
                    && !trimmed.starts_with("(todo)")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_bun_test_line(trimmed, &indent_stack) {
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == BunTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == BunTestStatus::Failed);
            let suite_to_save = BunTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_bun_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single Bun test result line.
    pub(crate) fn parse_bun_test_line(line: &str, ancestors: &[String]) -> Option<BunTest> {
        let line = line.trim_start();

        // Parse with color markers: "✓ test name [5.123ms]"
        if line.starts_with("✓") {
            let rest = line.strip_prefix("✓").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test with color markers: "✗ test name" or "× test name"
        if line.starts_with("✗") || line.starts_with("×") {
            let rest = line
                .strip_prefix("✗")
                .or_else(|| line.strip_prefix("×"))
                .unwrap_or(line)
                .trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(pass) test name [5.123ms]"
        if line.starts_with("(pass)") {
            let rest = line.strip_prefix("(pass)").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(fail) test name"
        if line.starts_with("(fail)") {
            let rest = line.strip_prefix("(fail)").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(skip) test name"
        if line.starts_with("(skip)") {
            let rest = line.strip_prefix("(skip)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(todo) test name"
        if line.starts_with("(todo)") {
            let rest = line.strip_prefix("(todo)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Parse duration string like "5.123ms" or "1.234s" into seconds.
    pub(crate) fn parse_bun_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            s.strip_suffix("ms")
                .and_then(|n| n.parse::<f64>().ok())
                .map(|ms| ms / 1000.0)
        } else if s.ends_with("s") {
            s.strip_suffix("s").and_then(|n| n.parse::<f64>().ok())
        } else {
            None
        }
    }

    /// Split test name and duration from a string like "test name [5.123ms]".
    pub(crate) fn split_bun_test_name_and_duration(s: &str) -> (String, Option<f64>) {
        // Look for duration in brackets at the end: "test name [5.123ms]"
        if let Some(start) = s.rfind('[') {
            if let Some(end) = s[start..].find(']') {
                let duration_str = &s[start + 1..start + end];
                let name = s[..start].trim().to_string();
                let duration = Self::parse_bun_duration(duration_str);
                return (name, duration);
            }
        }
        (s.to_string(), None)
    }

    /// Check if a line is a Bun summary line.
    pub(crate) fn is_bun_summary_line(line: &str) -> bool {
        let line = line.trim();
        // Match "X pass", "Y fail", "Z expect() calls", "W skipped"
        // These lines start with a number, not a test marker
        // Examples: " 4 pass", " 0 fail", " 4 expect() calls"
        // NOT: "✓ test pass" or "✗ should fail"

        // First check if line starts with a number (possibly with leading spaces)
        let starts_with_number = line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        if !starts_with_number {
            return false;
        }

        line.ends_with(" pass")
            || line.ends_with(" fail")
            || line.ends_with(" expect() calls")
            || line.ends_with(" skipped")
    }

    /// Parse a Bun summary line.
    pub(crate) fn parse_bun_summary_line(line: &str, summary: &mut BunTestSummary) {
        let line = line.trim();

        // Parse "X pass"
        if line.ends_with(" pass") {
            if let Some(count_str) = line.strip_suffix(" pass") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_passed = count;
                }
            }
            return;
        }

        // Parse "Y fail"
        if line.ends_with(" fail") {
            if let Some(count_str) = line.strip_suffix(" fail") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_failed = count;
                }
            }
            return;
        }

        // Parse "Z expect() calls"
        if line.ends_with(" expect() calls") {
            if let Some(count_str) = line.strip_suffix(" expect() calls") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.expect_calls = Some(count);
                }
            }
            return;
        }

        // Parse "X skipped"
        if line.ends_with(" skipped") {
            if let Some(count_str) = line.strip_suffix(" skipped") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_skipped = count;
                }
            }
        }
    }

    /// Parse "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
    pub(crate) fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary) {
        // Format: "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
        let line = line.trim();

        // Extract total tests
        if let Some(start) = line.find("Ran ") {
            let after_ran = &line[start + 4..];
            if let Some(end) = after_ran.find(" tests") {
                if let Ok(count) = after_ran[..end].trim().parse::<usize>() {
                    summary.tests_total = count;
                }
            }
        }

        // Extract files count
        if let Some(start) = line.find("across ") {
            let after_across = &line[start + 7..];
            if let Some(end) = after_across.find(" files") {
                if let Ok(count) = after_across[..end].trim().parse::<usize>() {
                    summary.suites_total = count;
                }
            }
        }

        // Extract duration - format: "in 1.44ms" or "[1.44ms]"
        if let Some(start) = line.find("in ") {
            let after_in = &line[start + 3..];
            summary.duration = Self::parse_bun_duration(after_in);
        } else if let Some(start) = line.rfind('[') {
            if let Some(end) = line[start..].find(']') {
                let duration_str = &line[start + 1..start + end];
                summary.duration = Self::parse_bun_duration(duration_str);
            }
        }
    }

    /// Update summary counts from parsed tests.
    pub(crate) fn update_bun_summary_from_tests(output: &mut BunTestOutput) {
        // Always update suite counts since they may not be in the "Ran" line
        // (the "across X files" part is optional)
        if output.summary.suites_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }
            }
        }

        // Only update test counts if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        BunTestStatus::Passed => output.summary.tests_passed += 1,
                        BunTestStatus::Failed => output.summary.tests_failed += 1,
                        BunTestStatus::Skipped => output.summary.tests_skipped += 1,
                        BunTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format Bun test output based on the requested format.
    pub(crate) fn format_bun_test(output: &BunTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_bun_test_json(output),
            OutputFormat::Compact => Self::format_bun_test_compact(output),
            OutputFormat::Raw => Self::format_bun_test_raw(output),
            OutputFormat::Agent => Self::format_bun_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_bun_test_compact(output),
        }
    }

    /// Format Bun test output as JSON.
    pub(crate) fn format_bun_test_json(output: &BunTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == BunTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "expect_calls": output.summary.expect_calls,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        BunTestStatus::Passed => "passed",
                        BunTestStatus::Failed => "failed",
                        BunTestStatus::Skipped => "skipped",
                        BunTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "bun_version": output.bun_version,
        })
        .to_string()
    }

    /// Format Bun test output in compact format.
    pub(crate) fn format_bun_test_compact(output: &BunTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("bun test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == BunTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));

        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format Bun test output as raw (just test names with status).
    pub(crate) fn format_bun_test_raw(output: &BunTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    BunTestStatus::Passed => "PASS",
                    BunTestStatus::Failed => "FAIL",
                    BunTestStatus::Skipped => "SKIP",
                    BunTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format Bun test output for AI agent consumption.
    pub(crate) fn format_bun_test_agent(output: &BunTestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(expect_calls) = output.summary.expect_calls {
            result.push_str(&format!("- Expect() calls: {}\n", expect_calls));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == BunTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }

    /// Handle the logs subcommand.
    pub(crate) fn handle_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the log output
        let logs_output = Self::parse_logs(&input);

        // Format output based on the requested format
        let output = Self::format_logs(&logs_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("logs")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(logs_output.total_lines)
                .with_extra("Debug", logs_output.debug_count.to_string())
                .with_extra("Info", logs_output.info_count.to_string())
                .with_extra("Warning", logs_output.warning_count.to_string())
                .with_extra("Error", logs_output.error_count.to_string())
                .with_extra("Fatal", logs_output.fatal_count.to_string())
                .with_extra(
                    "Repeated lines",
                    logs_output.repeated_lines.len().to_string(),
                );
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse log output into structured data.
    ///
    /// Supports various log formats:
    /// - Common timestamp formats (ISO 8601, syslog, etc.)
    /// - Log levels: DEBUG, INFO, WARN/WARNING, ERROR, FATAL/CRITICAL
    /// - Various formats: `[LEVEL]`, `LEVEL:`, `|LEVEL|`, etc.
    pub(crate) fn parse_logs(input: &str) -> LogsOutput {
        let mut logs_output = LogsOutput::default();
        let mut line_tracker: std::collections::HashMap<String, (usize, usize, usize)> =
            std::collections::HashMap::new();

        for (idx, line) in input.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines but count them
            if trimmed.is_empty() {
                continue;
            }

            // Track repeated lines
            let entry = line_tracker
                .entry(trimmed.to_string())
                .or_insert((0, line_num, line_num));
            entry.0 += 1;
            entry.2 = line_num;

            // Parse the log line
            let log_entry = Self::parse_log_line(trimmed, line_num);
            logs_output.entries.push(log_entry.clone());
            logs_output.total_lines += 1;

            // Count by level
            match log_entry.level {
                LogLevel::Debug => logs_output.debug_count += 1,
                LogLevel::Info => logs_output.info_count += 1,
                LogLevel::Warning => logs_output.warning_count += 1,
                LogLevel::Error => logs_output.error_count += 1,
                LogLevel::Fatal => logs_output.fatal_count += 1,
                LogLevel::Unknown => logs_output.unknown_count += 1,
            }

            // Track recent critical lines (ERROR and FATAL)
            if log_entry.level == LogLevel::Error || log_entry.level == LogLevel::Fatal {
                logs_output.recent_critical.push(log_entry.clone());
                // Keep only the most recent MAX_RECENT_CRITICAL entries
                if logs_output.recent_critical.len() > MAX_RECENT_CRITICAL {
                    logs_output.recent_critical.remove(0);
                }
            }
        }

        // Build repeated lines list (only lines repeated more than once)
        for (line, (count, first_line, last_line)) in line_tracker {
            if count > 1 {
                logs_output.repeated_lines.push(RepeatedLine {
                    line,
                    count,
                    first_line,
                    last_line,
                });
            }
        }

        // Sort repeated lines by first occurrence
        logs_output.repeated_lines.sort_by_key(|r| r.first_line);

        logs_output.is_empty = logs_output.entries.is_empty();
        logs_output
    }

    /// Parse a single log line.
    pub(crate) fn parse_log_line(line: &str, line_number: usize) -> LogEntry {
        let mut entry = LogEntry {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        };

        // Try to extract timestamp
        entry.timestamp = Self::extract_timestamp(line);

        // Try to extract log level
        entry.level = Self::detect_log_level(line);

        // Extract message (remove timestamp and level prefix)
        entry.message = Self::extract_message(line, &entry.timestamp, &entry.level);

        entry
    }

    /// Extract timestamp from a log line.
    pub(crate) fn extract_timestamp(line: &str) -> Option<String> {
        // Common timestamp patterns:
        // - ISO 8601: 2024-01-15T10:30:00, 2024-01-15 10:30:00
        // - Syslog: Jan 15 10:30:00
        // - Common: 2024/01/15 10:30:00, 01/15/2024 10:30:00
        // - Time only: 10:30:00, 10:30:00.123

        let chars: Vec<char> = line.chars().collect();

        // ISO 8601 with T separator: 2024-01-15T10:30:00
        // Format: YYYY-MM-DDTHH:MM:SS
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_timestamp(potential) {
                // Check for milliseconds and timezone
                let mut end = 19;
                if line.len() > 19 {
                    let rest = &line[19..];
                    // Check for milliseconds
                    if rest.starts_with('.') {
                        let ms_end = rest
                            .find(|c: char| !c.is_ascii_digit())
                            .unwrap_or(rest.len().min(4));
                        end += 1 + ms_end;
                    }
                    // Check for timezone (Z or +/-HH:MM)
                    if end < line.len() {
                        let tz_part = &line[end..];
                        if tz_part.starts_with('Z') {
                            end += 1;
                        } else if tz_part.starts_with('+') || tz_part.starts_with('-') {
                            // Timezone offset like +00:00 or +0000
                            let tz_len =
                                if tz_part.len() >= 6 && tz_part.chars().nth(3) == Some(':') {
                                    6
                                } else if tz_part.len() >= 5 {
                                    5
                                } else {
                                    0
                                };
                            end += tz_len;
                        }
                    }
                }
                return Some(line[..end].to_string());
            }
        }

        // ISO 8601 with space separator: 2024-01-15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_iso8601_space_timestamp(potential) {
                let mut end = 19;
                if line.len() > 19 && line[19..].starts_with('.') {
                    let rest = &line[19..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        // Slash date format: 2024/01/15 10:30:00
        if chars.len() >= 19 {
            let potential = &line[..19.min(line.len())];
            if Self::is_slash_date_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Syslog format: Jan 15 10:30:00
        if chars.len() >= 15 {
            let potential = &line[..15.min(line.len())];
            if Self::is_syslog_timestamp(potential) {
                return Some(potential.to_string());
            }
        }

        // Time only at start: 10:30:00 or 10:30:00.123
        if chars.len() >= 8 {
            let potential = &line[..8.min(line.len())];
            if Self::is_time_only(potential) {
                let mut end = 8;
                if line.len() > 8 && line[8..].starts_with('.') {
                    let rest = &line[8..];
                    let ms_end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len().min(4));
                    end += 1 + ms_end;
                }
                return Some(line[..end].to_string());
            }
        }

        None
    }

    /// Check if string is an ISO 8601 timestamp with T separator.
    pub(crate) fn is_iso8601_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DDTHH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        // Check structure: XXXX-XX-XXTXX:XX:XX
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is an ISO 8601 timestamp with space separator.
    pub(crate) fn is_iso8601_space_timestamp(s: &str) -> bool {
        // Format: YYYY-MM-DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a slash date timestamp.
    pub(crate) fn is_slash_date_timestamp(s: &str) -> bool {
        // Format: YYYY/MM/DD HH:MM:SS
        if s.len() < 19 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[4] == b'/'
            && bytes[7] == b'/'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[0..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
            && bytes[11..13].iter().all(|b| b.is_ascii_digit())
            && bytes[14..16].iter().all(|b| b.is_ascii_digit())
            && bytes[17..19].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if string is a syslog timestamp.
    pub(crate) fn is_syslog_timestamp(s: &str) -> bool {
        // Format: Mon DD HH:MM:SS (e.g., "Jan 15 10:30:00")
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return false;
        }
        months.contains(&parts[0])
            && parts[1].parse::<u8>().is_ok()
            && parts[2].len() == 8
            && parts[2].contains(':')
    }

    /// Check if string is time only (HH:MM:SS).
    pub(crate) fn is_time_only(s: &str) -> bool {
        // Format: HH:MM:SS
        if s.len() < 8 {
            return false;
        }
        let bytes = s.as_bytes();
        bytes[2] == b':'
            && bytes[5] == b':'
            && bytes[0..2].iter().all(|b| b.is_ascii_digit())
            && bytes[3..5].iter().all(|b| b.is_ascii_digit())
            && bytes[6..8].iter().all(|b| b.is_ascii_digit())
    }

    /// Detect log level from a log line.
    pub(crate) fn detect_log_level(line: &str) -> LogLevel {
        let line_upper = line.to_uppercase();

        // Check for various level indicators in order of severity (highest first)
        // Patterns: [FATAL], FATAL:, |FATAL|, FATAL - etc.

        // Fatal/Critical - includes panic, crash, abort
        if Self::contains_level_marker(&line_upper, "FATAL")
            || Self::contains_level_marker(&line_upper, "CRITICAL")
            || Self::contains_level_marker(&line_upper, "CRIT")
            || Self::contains_error_keyword(&line_upper, "PANIC")
            || Self::contains_error_keyword(&line_upper, "CRASH")
            || Self::contains_error_keyword(&line_upper, "ABORT")
            || Self::contains_error_keyword(&line_upper, "EMERG")
            || Self::contains_error_keyword(&line_upper, "ALERT")
        {
            return LogLevel::Fatal;
        }

        // Error - includes exceptions, failures, and common error patterns
        if Self::contains_level_marker(&line_upper, "ERROR")
            || Self::contains_level_marker(&line_upper, "ERR")
            || Self::contains_error_keyword(&line_upper, "EXCEPTION")
            || Self::contains_error_keyword(&line_upper, "FAILED")
            || Self::contains_error_keyword(&line_upper, "FAILURE")
            || Self::contains_error_keyword(&line_upper, "STACK TRACE")
            || Self::contains_error_keyword(&line_upper, "BACKTRACE")
            || Self::contains_error_keyword(&line_upper, "SEGFAULT")
            || Self::contains_error_keyword(&line_upper, "SEG FAULT")
            || Self::contains_error_keyword(&line_upper, "NULL POINTER")
            || Self::contains_error_keyword(&line_upper, "ACCESS DENIED")
            || Self::contains_error_keyword(&line_upper, "TIMEOUT ERROR")
            || Self::contains_error_keyword(&line_upper, "CONNECTION REFUSED")
            || Self::contains_error_keyword(&line_upper, "CONNECTION ERROR")
        {
            return LogLevel::Error;
        }

        // Warning - includes deprecation, caution notices
        if Self::contains_level_marker(&line_upper, "WARN")
            || Self::contains_level_marker(&line_upper, "WARNING")
            || Self::contains_warning_keyword(&line_upper, "DEPRECATED")
            || Self::contains_warning_keyword(&line_upper, "CAUTION")
            || Self::contains_warning_keyword(&line_upper, "ATTENTION")
            || Self::contains_warning_keyword(&line_upper, "BE AWARE")
            || Self::contains_warning_keyword(&line_upper, "PLEASE NOTE")
            || Self::contains_warning_keyword(&line_upper, "SLOW QUERY")
            || Self::contains_warning_keyword(&line_upper, "SLOW REQUEST")
        {
            return LogLevel::Warning;
        }

        // Info
        if Self::contains_level_marker(&line_upper, "INFO")
            || Self::contains_level_marker(&line_upper, "NOTICE")
        {
            return LogLevel::Info;
        }

        // Debug
        if Self::contains_level_marker(&line_upper, "DEBUG")
            || Self::contains_level_marker(&line_upper, "TRACE")
            || Self::contains_level_marker(&line_upper, "VERBOSE")
        {
            return LogLevel::Debug;
        }

        LogLevel::Unknown
    }

    /// Check if line contains an error-related keyword.
    /// This is more lenient than contains_level_marker and looks for keywords
    /// anywhere in the line that typically indicate an error condition.
    pub(crate) fn contains_error_keyword(line_upper: &str, keyword: &str) -> bool {
        // Check for the keyword with word boundaries
        if line_upper.contains(keyword) {
            // Avoid false positives by checking context
            // For example, "no errors" should not be detected as an error
            let keyword_lower = keyword.to_lowercase();
            let negation_patterns = [
                format!("no {}", keyword_lower),
                format!("without {}", keyword_lower),
                format!("not {}", keyword_lower),
                format!("0 {}", keyword_lower),
                format!("zero {}", keyword_lower),
            ];
            for neg in negation_patterns {
                if line_upper.contains(&neg.to_uppercase()) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check if line contains a warning-related keyword.
    pub(crate) fn contains_warning_keyword(line_upper: &str, keyword: &str) -> bool {
        line_upper.contains(keyword)
    }

    /// Check if line contains a level marker.
    pub(crate) fn contains_level_marker(line_upper: &str, level: &str) -> bool {
        // Check for patterns like [LEVEL], LEVEL:, |LEVEL|, <LEVEL>, (LEVEL)
        // These are precise patterns that indicate a log level
        let patterns = [
            format!("[{}]", level),
            format!("{}:", level),
            format!("|{}|", level),
            format!("<{}>", level),
            format!("({})", level),
            format!("{} -", level),
            format!("{}]", level), // Level followed by closing bracket
        ];

        for pattern in patterns {
            if line_upper.contains(&pattern) {
                return true;
            }
        }

        // Check if line starts with level followed by space or colon
        if line_upper.starts_with(level) {
            let after_level = &line_upper[level.len()..];
            if after_level.starts_with(':')
                || after_level.starts_with(' ')
                || after_level.is_empty()
            {
                return true;
            }
        }

        false
    }

    /// Extract message by removing timestamp and level prefix.
    pub(crate) fn extract_message(line: &str, timestamp: &Option<String>, level: &LogLevel) -> String {
        let mut message = line.to_string();

        // Remove timestamp prefix
        if let Some(ts) = timestamp {
            if message.starts_with(ts) {
                message = message[ts.len()..].to_string();
            }
        }

        // Trim leading whitespace after timestamp removal
        message = message.trim_start().to_string();

        // Remove common level prefixes
        let level_patterns: &[&str] = match level {
            LogLevel::Debug => &[
                "[DEBUG]", "DEBUG:", "|DEBUG|", "<DEBUG>", "(DEBUG)", "DEBUG -", "DEBUG ",
            ],
            LogLevel::Info => &[
                "[INFO]", "INFO:", "|INFO|", "<INFO>", "(INFO)", "INFO -", "INFO ",
            ],
            LogLevel::Warning => &[
                "[WARN]",
                "[WARNING]",
                "WARN:",
                "WARNING:",
                "|WARN|",
                "|WARNING|",
                "<WARN>",
                "<WARNING>",
                "(WARN)",
                "(WARNING)",
                "WARN -",
                "WARNING -",
                "WARN ",
                "WARNING ",
            ],
            LogLevel::Error => &[
                "[ERROR]", "ERROR:", "|ERROR|", "<ERROR>", "(ERROR)", "ERROR -", "ERROR ", "[ERR]",
                "ERR:", "ERR ",
            ],
            LogLevel::Fatal => &[
                "[FATAL]",
                "FATAL:",
                "|FATAL|",
                "<FATAL>",
                "(FATAL)",
                "FATAL -",
                "FATAL ",
                "[CRITICAL]",
                "CRITICAL:",
                "[CRIT]",
                "CRIT:",
            ],
            LogLevel::Unknown => &[],
        };

        for pattern in level_patterns {
            let pattern_upper = pattern.to_uppercase();
            let message_upper = message.to_uppercase();
            if message_upper.starts_with(&pattern_upper) {
                message = message[pattern.len()..].to_string();
                break;
            }
        }

        // Clean up leading whitespace and separators
        message = message.trim().to_string();
        if message.starts_with('-') || message.starts_with(':') || message.starts_with(']') {
            message = message[1..].trim().to_string();
        }

        message
    }

    /// Format logs output for display.
    pub(crate) fn format_logs(logs_output: &LogsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_logs_json(logs_output),
            OutputFormat::Csv => Self::format_logs_csv(logs_output),
            OutputFormat::Tsv => Self::format_logs_tsv(logs_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_logs_compact(logs_output),
            OutputFormat::Raw => Self::format_logs_raw(logs_output),
        }
    }

    /// Format logs output as JSON.
    pub(crate) fn format_logs_json(logs_output: &LogsOutput) -> String {
        let total_critical = logs_output.error_count + logs_output.fatal_count;
        serde_json::json!({
            "counts": {
                "total_lines": logs_output.total_lines,
                "debug": logs_output.debug_count,
                "info": logs_output.info_count,
                "warning": logs_output.warning_count,
                "error": logs_output.error_count,
                "fatal": logs_output.fatal_count,
                "unknown": logs_output.unknown_count,
            },
            "repeated_lines": logs_output.repeated_lines.iter().map(|r| serde_json::json!({
                "line": r.line,
                "count": r.count,
                "first_line": r.first_line,
                "last_line": r.last_line,
            })).collect::<Vec<_>>(),
            "recent_critical": logs_output.recent_critical.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "recent_critical_count": logs_output.recent_critical.len(),
            "total_critical": total_critical,
            "entries": logs_output.entries.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format logs output as CSV.
    pub(crate) fn format_logs_csv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,level,timestamp,message\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_csv_field(&entry.message);
            result.push_str(&format!(
                "{},{},{},{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output as TSV.
    pub(crate) fn format_logs_tsv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tlevel\ttimestamp\tmessage\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_tsv_field(&entry.message);
            result.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output in compact format.
    pub(crate) fn format_logs_compact(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        if logs_output.is_empty {
            output.push_str("logs: empty\n");
            return output;
        }

        // Summary header
        output.push_str(&format!("lines: {}\n", logs_output.total_lines));

        // Level summary (only show non-zero counts)
        let mut level_parts = Vec::new();
        if logs_output.fatal_count > 0 {
            level_parts.push(format!("fatal:{}", logs_output.fatal_count));
        }
        if logs_output.error_count > 0 {
            level_parts.push(format!("error:{}", logs_output.error_count));
        }
        if logs_output.warning_count > 0 {
            level_parts.push(format!("warn:{}", logs_output.warning_count));
        }
        if logs_output.info_count > 0 {
            level_parts.push(format!("info:{}", logs_output.info_count));
        }
        if logs_output.debug_count > 0 {
            level_parts.push(format!("debug:{}", logs_output.debug_count));
        }
        if logs_output.unknown_count > 0 {
            level_parts.push(format!("other:{}", logs_output.unknown_count));
        }

        if !level_parts.is_empty() {
            output.push_str(&format!("levels: {}\n", level_parts.join(", ")));
        }

        // Repeated lines summary
        if !logs_output.repeated_lines.is_empty() {
            output.push_str(&format!(
                "repeated: {} unique lines ({} occurrences)\n",
                logs_output.repeated_lines.len(),
                logs_output
                    .repeated_lines
                    .iter()
                    .map(|r| r.count)
                    .sum::<usize>()
            ));
        }

        output.push('\n');

        // Show repeated lines
        if !logs_output.repeated_lines.is_empty() {
            output.push_str("repeated lines:\n");
            for repeated in &logs_output.repeated_lines {
                if repeated.count > 1 {
                    let preview = if repeated.line.len() > 60 {
                        format!("{}...", &repeated.line[..57])
                    } else {
                        repeated.line.clone()
                    };
                    output.push_str(&format!(
                        "  [x{}] {} (lines {}-{})\n",
                        repeated.count, preview, repeated.first_line, repeated.last_line
                    ));
                }
            }
            output.push('\n');
        }

        // Show recent critical lines (ERROR and FATAL)
        if !logs_output.recent_critical.is_empty() {
            let total_critical = logs_output.error_count + logs_output.fatal_count;
            let shown = logs_output.recent_critical.len();
            if shown < total_critical {
                output.push_str(&format!(
                    "recent critical ({} of {}):\n",
                    shown, total_critical
                ));
            } else {
                output.push_str(&format!("recent critical ({}):\n", shown));
            }
            for entry in &logs_output.recent_critical {
                let level_indicator = match entry.level {
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    _ => "[!]",
                };
                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };
                output.push_str(&format!(
                    "  {} {} {}\n",
                    level_indicator, entry.line_number, preview
                ));
            }
            output.push('\n');
        }

        // Show entries with detected levels (collapse consecutive duplicates)
        let has_levels = logs_output
            .entries
            .iter()
            .any(|e| e.level != LogLevel::Unknown);
        if has_levels {
            output.push_str("entries:\n");
            // Collapse consecutive entries with same level and message
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];
                let level_indicator = match entry.level {
                    LogLevel::Debug => "[D]",
                    LogLevel::Info => "[I]",
                    LogLevel::Warning => "[W]",
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    LogLevel::Unknown => "   ",
                };

                // Count consecutive entries with same level and message
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.level == entry.level && next.message == entry.message {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "{} {}-{} {} [x{}]\n",
                        level_indicator, entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {} {}\n",
                        level_indicator, entry.line_number, preview
                    ));
                }

                i += count;
            }
        } else {
            // No levels detected, just show raw lines with line numbers (collapse consecutive duplicates)
            output.push_str("lines:\n");
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];

                // Count consecutive entries with same line content
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.line == entry.line {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.line.len() > 80 {
                    format!("{}...", &entry.line[..77])
                } else {
                    entry.line.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "  {}-{} {} [x{}]\n",
                        entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.line_number, preview));
                }

                i += count;
            }
        }

        output
    }

    /// Format logs output as raw (original format).
    pub(crate) fn format_logs_raw(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        for entry in &logs_output.entries {
            output.push_str(&entry.line);
            output.push('\n');
        }

        output
    }

    /// Handle the find subcommand.
    pub(crate) fn handle_find(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the find output
        let find_output = Self::parse_find(&input)?;

        // Format output based on the requested format
        let output = Self::format_find(&find_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("find")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(find_output.entries.len())
                .with_extra("Files", find_output.files.len().to_string())
                .with_extra("Directories", find_output.directories.len().to_string())
                .with_extra("Hidden", find_output.hidden.len().to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse find output into structured data.
    pub(crate) fn parse_find(input: &str) -> CommandResult<FindOutput> {
        let mut find_output = FindOutput::default();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Check for permission denied or other error messages
            // Format: "find: '/path': Permission denied"
            // or: "find: cannot open directory '/path': Permission denied"
            // or: "find: 'path': No such file or directory"
            if line.starts_with("find: ") && line.contains(':') {
                let error = Self::parse_find_error(line);
                find_output.errors.push(error);
                continue;
            }

            // Each line is a file path
            let path = line.to_string();
            let is_directory = path.ends_with('/');
            let is_hidden = path
                .split('/')
                .last()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false);

            let entry = FindEntry {
                path: path.clone(),
                is_directory,
                is_hidden,
                extension: Self::extract_extension(&path),
                depth: Self::calculate_path_depth(&path),
            };

            find_output.entries.push(entry.clone());
            find_output.total_count += 1;

            if is_directory {
                find_output.directories.push(path.clone());
            } else {
                find_output.files.push(path.clone());
            }

            if is_hidden {
                find_output.hidden.push(path);
            }

            // Track extensions
            if let Some(ext) = &entry.extension {
                *find_output.extensions.entry(ext.clone()).or_insert(0) += 1;
            }
        }

        // Check if empty (considering both entries and errors)
        find_output.is_empty = find_output.entries.is_empty();

        Ok(find_output)
    }

    /// Parse a find error message.
    pub(crate) fn parse_find_error(line: &str) -> FindError {
        // Format: "find: '/path': Permission denied"
        // or: "find: cannot open directory '/path': Permission denied"
        // or: "find: 'path': No such file or directory"

        // Try to extract the path (usually in quotes)
        let path = if let Some(start) = line.find('\'') {
            if let Some(end) = line[start + 1..].find('\'') {
                line[start + 1..start + 1 + end].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        FindError {
            path,
            message: line.to_string(),
        }
    }

    /// Extract file extension from path.
    pub(crate) fn extract_extension(path: &str) -> Option<String> {
        let filename = path.split('/').last()?;
        // Skip hidden files starting with . and files with no extension
        if filename.starts_with('.') {
            return None;
        }
        let dot_pos = filename.rfind('.')?;
        if dot_pos == 0 {
            return None;
        }
        Some(filename[dot_pos + 1..].to_lowercase())
    }

    /// Calculate the depth of a path (number of path separators).
    pub(crate) fn calculate_path_depth(path: &str) -> usize {
        path.matches('/').count()
    }

    /// Format find output for display.
    pub(crate) fn format_find(find_output: &FindOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_find_json(find_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_find_compact(find_output),
            OutputFormat::Raw => Self::format_find_raw(find_output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_find_compact(find_output),
        }
    }

    /// Format find output as JSON.
    pub(crate) fn format_find_json(find_output: &FindOutput) -> String {
        serde_json::json!({
            "is_empty": find_output.is_empty,
            "total_count": find_output.total_count,
            "entries": find_output.entries.iter().map(|e| serde_json::json!({
                "path": e.path,
                "is_directory": e.is_directory,
                "is_hidden": e.is_hidden,
                "extension": e.extension,
                "depth": e.depth,
            })).collect::<Vec<_>>(),
            "directories": find_output.directories,
            "files": find_output.files,
            "hidden": find_output.hidden,
            "extensions": find_output.extensions,
            "errors": find_output.errors.iter().map(|e| serde_json::json!({
                "path": e.path,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format find output in compact format.
    pub(crate) fn format_find_compact(find_output: &FindOutput) -> String {
        let mut output = String::new();

        // Show errors first (if any)
        if !find_output.errors.is_empty() {
            for error in &find_output.errors {
                output.push_str(&format!("error: {}\n", error.message));
            }
        }

        if find_output.is_empty && find_output.errors.is_empty() {
            output.push_str("(no results)\n");
            return output;
        }

        // Just output the paths, no headers
        for entry in &find_output.entries {
            output.push_str(&entry.path);
            output.push('\n');
        }

        // Only add summary for large result sets (20+ entries)
        if find_output.total_count >= 20 {
            let dir_count = find_output.directories.len();
            let file_count = find_output.files.len();
            let mut summary_parts = Vec::new();
            if file_count > 0 { summary_parts.push(format!("{} files", file_count)); }
            if dir_count > 0 { summary_parts.push(format!("{} dirs", dir_count)); }
            if !find_output.extensions.is_empty() {
                let mut exts: Vec<_> = find_output.extensions.iter().collect();
                exts.sort_by(|a, b| b.1.cmp(a.1));
                let top: Vec<String> = exts.iter().take(5).map(|(e, c)| format!(".{}({})", e, c)).collect();
                summary_parts.push(top.join(" "));
            }
            output.push_str(&format!("[{}]\n", summary_parts.join(", ")));
        }

        output
    }

    /// Format find output as raw (just paths).
    pub(crate) fn format_find_raw(find_output: &FindOutput) -> String {
        let mut output = String::new();

        for entry in &find_output.entries {
            output.push_str(&format!("{}\n", entry.path));
        }

        output
    }

    // ============================================================
    // New Parsers: git-log, git-branch, tree, docker, deps, install, build, env
    // ============================================================

    pub(crate) fn handle_git_log(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut commits: Vec<(String, String, String, String)> = Vec::new();
        let mut hash = String::new();
        let mut author = String::new();
        let mut date = String::new();
        let mut msg: Vec<String> = Vec::new();
        let mut in_commit = false;

        // Detect format: --oneline (hash message) vs full (commit hash\nAuthor:\nDate:\n\nmessage)
        let is_oneline = !input.contains("Author: ") && !input.contains("commit ");

        if is_oneline {
            // Parse --oneline format: "abc1234 commit message here"
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Some(space_pos) = trimmed.find(' ') {
                    let h = &trimmed[..space_pos];
                    let m = &trimmed[space_pos+1..];
                    commits.push((h.to_string(), String::new(), String::new(), m.to_string()));
                }
            }
        } else {
            for line in input.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    if in_commit {
                        let full_msg = msg.join(" ").trim().to_string();
                        // Only take the first line of the message (subject)
                        let subject = full_msg.lines().next().unwrap_or("").to_string();
                        let subject = if subject.len() > 72 { format!("{}...", &subject[..69]) } else { subject };
                        commits.push((hash.clone(), date.clone(), author.clone(), subject));
                        msg.clear();
                    }
                    hash = h.chars().take(7).collect();
                    in_commit = true;
                } else if let Some(a) = line.strip_prefix("Author: ") {
                    author = a.split('<').next().unwrap_or(a).trim().to_string();
                } else if let Some(d) = line.strip_prefix("Date:") {
                    date = d.trim().chars().take(16).collect();
                } else if in_commit && !line.trim().is_empty() {
                    msg.push(line.trim().to_string());
                }
            }
            if in_commit {
                let full_msg = msg.join(" ").trim().to_string();
                let subject = full_msg.lines().next().unwrap_or("").to_string();
                let subject = if subject.len() > 72 { format!("{}...", &subject[..69]) } else { subject };
                commits.push((hash, date, author, subject));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                let jc: Vec<serde_json::Value> = commits.iter().map(|(h,d,a,m)| serde_json::json!({"hash":h,"date":d,"author":a,"message":m})).collect();
                serde_json::json!({"commits": jc, "count": commits.len()}).to_string()
            }
            _ => {
                let mut out = format!("commits: {}\n", commits.len());
                for (h, d, a, m) in &commits { out.push_str(&format!("  {} {} ({}) {}\n", h, d, a, m)); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("git-log").with_output_mode(ctx.format).with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(commits.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_git_branch(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut current = String::new();
        let mut local: Vec<String> = Vec::new();
        let mut remote: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.contains("->") { continue; }
            let is_current = trimmed.starts_with('*');
            let name = trimmed.trim_start_matches("* ").trim().to_string();
            if is_current { current = name.clone(); }
            if name.starts_with("remotes/") || name.starts_with("origin/") {
                remote.push(name.trim_start_matches("remotes/").to_string());
            } else { local.push(name); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"current": current, "local": local, "remote": remote, "local_count": local.len(), "remote_count": remote.len()}).to_string(),
            _ => {
                let mut out = String::new();
                // Filter out remote branches that duplicate local ones
                let unique_remote: Vec<&String> = remote.iter().filter(|r| {
                    let short = r.split('/').last().unwrap_or(r);
                    !local.iter().any(|l| l == short)
                }).collect();
                // Minimal: if only one local branch and no unique remotes, just show current
                let other_local: Vec<&String> = local.iter().filter(|b| *b != &current).collect();
                out.push_str(&format!("* {}\n", current));
                if !other_local.is_empty() {
                    for b in &other_local { out.push_str(&format!("  {}\n", b)); }
                }
                if !unique_remote.is_empty() {
                    for b in &unique_remote { out.push_str(&format!("  {}\n", b)); }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("git-branch").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_tree(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();
        let mut total_dirs = 0usize;
        let mut total_files = 0usize;

        for line in input.lines() {
            if line.contains("director") && line.contains("file") {
                for part in line.split(',') {
                    let t = part.trim();
                    if t.ends_with("directories") || t.ends_with("directory") { total_dirs = t.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0); }
                    else if t.ends_with("files") || t.ends_with("file") { total_files = t.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0); }
                }
                continue;
            }
            let name: String = line.chars().filter(|c| !matches!(c, '│' | '├' | '└' | '─' | '─')).collect::<String>().trim().to_string();
            if name.is_empty() || name == "." { continue; }
            if name.ends_with('/') { dirs.push(name.trim_end_matches('/').to_string()); }
            else { files.push(name); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"directories": dirs, "files": files, "total_directories": total_dirs, "total_files": total_files}).to_string(),
            _ => {
                let mut out = format!("{} directories, {} files\n", total_dirs, total_files);
                if !dirs.is_empty() { out.push_str(&format!("dirs:")); for d in dirs.iter().take(20) { out.push_str(&format!(" {}", d)); } if dirs.len() > 20 { out.push_str(&format!(" ...+{}", dirs.len()-20)); } out.push('\n'); }
                if !files.is_empty() { out.push_str(&format!("files:")); for f in files.iter().take(30) { out.push_str(&format!(" {}", f)); } if files.len() > 30 { out.push_str(&format!(" ...+{}", files.len()-30)); } out.push('\n'); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("tree").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_docker_ps(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut containers: Vec<serde_json::Value> = Vec::new();
        let lines: Vec<&str> = input.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let id = parts[0];
                    let image = parts[1];
                    let status = line.find("Up ").or_else(|| line.find("Exited ")).map(|s| line[s..].split("  ").next().unwrap_or("").trim()).unwrap_or("unknown");
                    let name = parts.last().unwrap_or(&"");
                    containers.push(serde_json::json!({"id": id, "image": image, "status": status, "name": name}));
                }
            }
        }
        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"containers": containers, "count": containers.len()}).to_string(),
            _ => {
                let mut out = format!("containers: {}\n", containers.len());
                for c in &containers {
                    let st = c["status"].as_str().unwrap_or("");
                    let mk = if st.starts_with("Up") { "+" } else { "-" };
                    out.push_str(&format!("  {} {} {} ({})\n", mk, c["name"].as_str().unwrap_or(""), c["image"].as_str().unwrap_or(""), st));
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("docker-ps").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_docker_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        Self::handle_logs(file, ctx)
    }

    pub(crate) fn handle_deps(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut deps: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            let clean = line.replace('│', "").replace('├', "").replace('└', "").replace('─', "").replace("deduped", "").trim().to_string();
            if clean.is_empty() { continue; }
            // npm: name@version
            if let Some(at) = clean.rfind('@') {
                if at > 0 { let n = clean[..at].trim().to_string(); let v = clean[at+1..].trim().to_string(); if !n.is_empty() { deps.push((n, v)); continue; } }
            }
            // pip/cargo: name version
            let parts: Vec<&str> = clean.split_whitespace().collect();
            if parts.len() >= 2 {
                let n = parts[0].to_string();
                if n == "Package" || n == "---" || n.starts_with("==") { continue; }
                deps.push((n, parts[1].trim_start_matches('v').to_string()));
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => { let jd: Vec<serde_json::Value> = deps.iter().map(|(n,v)| serde_json::json!({"name":n,"version":v})).collect(); serde_json::json!({"dependencies": jd, "count": deps.len()}).to_string() }
            _ => { let mut out = format!("dependencies: {}\n", deps.len()); for (n,v) in &deps { if v.is_empty() { out.push_str(&format!("  {}\n", n)); } else { out.push_str(&format!("  {}@{}\n", n, v)); } } out }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("deps").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(deps.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_install(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut added: Vec<String> = Vec::new();
        let mut warnings = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut summary = String::new();

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() { continue; }
            let lower = t.to_lowercase();
            if (lower.contains("added") || lower.contains("removed")) && lower.contains("package") { summary = t.to_string(); }
            else if lower.starts_with("successfully installed") { for pkg in t.split_whitespace().skip(2) { added.push(pkg.to_string()); } }
            else if lower.starts_with("npm warn") || lower.starts_with("warn ") { warnings += 1; }
            else if lower.starts_with("npm error") || lower.starts_with("error") || lower.starts_with("err!") { errors.push(t.to_string()); }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"summary": summary, "added": added, "added_count": added.len(), "warnings": warnings, "errors": errors.len()}).to_string(),
            _ => {
                let mut out = String::new();
                if !summary.is_empty() { out.push_str(&format!("{}\n", summary)); }
                if !added.is_empty() { out.push_str(&format!("added ({}): {}\n", added.len(), added.join(", "))); }
                if warnings > 0 { out.push_str(&format!("warnings: {}\n", warnings)); }
                if !errors.is_empty() { out.push_str(&format!("errors ({}):\n", errors.len())); for e in &errors { out.push_str(&format!("  {}\n", e)); } }
                if out.is_empty() { out = "install: ok\n".to_string(); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("install").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    pub(crate) fn handle_build(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut info_last = String::new();
        let mut success = true;

        for line in input.lines() {
            let t = line.trim();
            if t.is_empty() { continue; }
            let lower = t.to_lowercase();
            if lower.contains("error[") || lower.starts_with("error:") || lower.contains("): error ") || lower.contains(": error:") {
                errors.push(t.to_string()); success = false;
            } else if lower.contains("warning[") || lower.starts_with("warning:") || lower.contains(": warning:") {
                warnings.push(t.to_string());
            } else if lower.starts_with("compiling ") || lower.starts_with("finished ") {
                info_last = t.to_string();
            }
        }
        warnings.dedup();

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({"success": success, "errors": errors, "error_count": errors.len(), "warnings": warnings, "warning_count": warnings.len()}).to_string(),
            _ => {
                let mut out = format!("build: {} ({} errors, {} warnings)\n", if success {"ok"} else {"FAILED"}, errors.len(), warnings.len());
                if !errors.is_empty() { out.push_str(&format!("errors ({}):\n", errors.len())); for e in errors.iter().take(20) { out.push_str(&format!("  {}\n", e)); } if errors.len() > 20 { out.push_str(&format!("  ...+{} more\n", errors.len()-20)); } }
                if !warnings.is_empty() { out.push_str(&format!("warnings ({}):\n", warnings.len())); for w in warnings.iter().take(10) { out.push_str(&format!("  {}\n", w)); } if warnings.len() > 10 { out.push_str(&format!("  ...+{} more\n", warnings.len()-10)); } }
                if !info_last.is_empty() { out.push_str(&format!("{}\n", info_last)); }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("build").with_input_bytes(input_bytes).with_output_bytes(output.len()).print(); }
        Ok(())
    }

    /// Check if an env var key is internal noise that should be filtered.
    fn is_env_noise(key: &str) -> bool {
        // Internal shell/terminal noise prefixes
        let noise_prefixes = [
            "_P9K_", "P9K_", "LESS", "LS_COLORS", "LSCOLORS",
            "_", "__", "COMP_", "BASH_FUNC_",
            "ZSH_HIGHLIGHT", "ZSH_AUTOSUGGEST",
            "POWERLEVEL", "ITERM", "TERM_SESSION",
            "SECURITYSESSION", "TMPDIR",
            "LaunchInstanceID", "LOGNAME",
            "Apple_PubSub", "DISPLAY",
            "COMMAND_MODE", "COLORTERM",
            "MANPATH", "INFOPATH", "FPATH",
            "SSH_AUTH_SOCK", "SSH_AGENT_PID",
            "TERM_PROGRAM", "TERM_PROGRAM_VERSION",
            "ORIGINAL_XDG", "XPC_",
            "SUPERSET_", "ZDOTDIR",
            "CARGO_PKG_", "CARGO_MANIFEST", "CARGO_BIN",
            "CARGO_CRATE", "CARGO_PRIMARY",
            "NoDefault", "SSL_CERT",
            "rvm_", "GEM_",
        ];
        for prefix in &noise_prefixes {
            if key.starts_with(prefix) && key != "PATH" && key != "LANG" {
                return true;
            }
        }
        // Single underscore var
        if key == "_" { return true; }
        false
    }

    /// Categorize an env var for grouping.
    fn env_category(key: &str) -> &'static str {
        if key == "PATH" || key.ends_with("_PATH") || key.ends_with("PATH") || key == "MANPATH" || key == "INFOPATH" || key == "FPATH" {
            return "path";
        }
        if matches!(key, "LANG" | "LC_ALL" | "LC_CTYPE" | "LC_MESSAGES" | "LANGUAGE" | "TZ" | "TERM" | "SHELL" | "USER" | "HOME" | "HOSTNAME" | "PWD" | "OLDPWD" | "SHLVL" | "EDITOR" | "VISUAL" | "PAGER" | "XDG_CONFIG_HOME" | "XDG_DATA_HOME" | "XDG_CACHE_HOME" | "XDG_RUNTIME_DIR") {
            return "system";
        }
        if matches!(key, "GOPATH" | "GOROOT" | "CARGO_HOME" | "RUSTUP_HOME" | "PYENV_ROOT" | "RBENV_ROOT" | "NVM_DIR" | "JAVA_HOME" | "ANDROID_HOME" | "CONDA_DEFAULT_ENV" | "VIRTUAL_ENV" | "NODE_OPTIONS" | "NODE_ENV" | "PYTHONPATH" | "RUBY_VERSION" | "RUSTC_WRAPPER" | "npm_config_prefix")
            || key.starts_with("PYTHON") || key.starts_with("RUBY") || key.starts_with("GO") || key.starts_with("RUST") || key.starts_with("NODE") || key.starts_with("NVM") || key.starts_with("JAVA") || key.starts_with("CONDA") {
            return "lang";
        }
        "user"
    }

    pub(crate) fn handle_env(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let mut all_vars: Vec<(String, String)> = Vec::new();

        for line in input.lines() {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].to_string();
                let val = line[eq+1..].to_string();
                all_vars.push((key, val));
            }
        }

        // JSON output: include everything (unfiltered, just sorted)
        let output = match ctx.format {
            OutputFormat::Json => {
                let mut sorted = all_vars.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let jv: serde_json::Map<String, serde_json::Value> = sorted.iter().map(|(k,v)| {
                    let display = if v.len() > 80 { format!("{}...", &v[..77]) } else { v.clone() };
                    (k.clone(), serde_json::Value::String(display))
                }).collect();
                serde_json::json!({"variables": jv, "count": sorted.len()}).to_string()
            }
            _ => {
                // Compact: filter noise and empty values, group by category
                let mut path_vars: Vec<(String, String)> = Vec::new();
                let mut system_vars: Vec<(String, String)> = Vec::new();
                let mut lang_vars: Vec<(String, String)> = Vec::new();
                let mut user_vars: Vec<(String, String)> = Vec::new();
                let mut filtered_count = 0usize;

                for (key, val) in &all_vars {
                    // Skip empty values
                    if val.is_empty() { filtered_count += 1; continue; }
                    // Skip noise
                    if Self::is_env_noise(key) { filtered_count += 1; continue; }

                    let category = Self::env_category(key);
                    let display_val = if key == "PATH" || key.ends_with("PATH") || key == "FPATH" {
                        // For PATH-like vars, show entry count
                        let entries: Vec<&str> = val.split(':').filter(|s| !s.is_empty()).collect();
                        format!("({} entries)", entries.len())
                    } else if val.len() > 60 {
                        format!("{}...", &val[..57])
                    } else {
                        val.clone()
                    };

                    match category {
                        "path" => path_vars.push((key.clone(), display_val)),
                        "system" => system_vars.push((key.clone(), display_val)),
                        "lang" => lang_vars.push((key.clone(), display_val)),
                        _ => user_vars.push((key.clone(), display_val)),
                    }
                }

                path_vars.sort_by(|a, b| a.0.cmp(&b.0));
                system_vars.sort_by(|a, b| a.0.cmp(&b.0));
                lang_vars.sort_by(|a, b| a.0.cmp(&b.0));
                user_vars.sort_by(|a, b| a.0.cmp(&b.0));

                let shown = path_vars.len() + system_vars.len() + lang_vars.len() + user_vars.len();
                let mut out = format!("{} vars ({} filtered)\n", shown, filtered_count);

                if !path_vars.is_empty() {
                    // Show PATH vars inline: just PATH=46 entries
                    for (k, v) in &path_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !system_vars.is_empty() {
                    out.push_str("[system]\n");
                    for (k, v) in &system_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !lang_vars.is_empty() {
                    out.push_str("[lang/runtime]\n");
                    for (k, v) in &lang_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                if !user_vars.is_empty() {
                    out.push_str("[user/other]\n");
                    for (k, v) in &user_vars { out.push_str(&format!("  {}={}\n", k, v)); }
                }
                out
            }
        };
        print!("{}", output);
        if ctx.stats { CommandStats::new().with_reducer("env").with_input_bytes(input_bytes).with_output_bytes(output.len()).with_items_processed(all_vars.len()).print(); }
        Ok(())
    }
}

impl CommandHandler for ParseHandler {
    type Input = ParseCommands;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        match input {
            ParseCommands::GitStatus { file, count } => Self::handle_git_status(file, count, ctx),
            ParseCommands::GitDiff { file } => Self::handle_git_diff(file, ctx),
            ParseCommands::GitLog { file } => Self::handle_git_log(file, ctx),
            ParseCommands::GitBranch { file } => Self::handle_git_branch(file, ctx),
            ParseCommands::Ls { file } => Self::handle_ls(file, ctx),
            ParseCommands::Grep { file } => Self::handle_grep(file, ctx),
            ParseCommands::Find { file } => Self::handle_find(file, ctx),
            ParseCommands::Test { runner, file } => Self::handle_test(runner, file, ctx),
            ParseCommands::Logs { file } => Self::handle_logs(file, ctx),
            ParseCommands::Tree { file } => Self::handle_tree(file, ctx),
            ParseCommands::DockerPs { file } => Self::handle_docker_ps(file, ctx),
            ParseCommands::DockerLogs { file } => Self::handle_docker_logs(file, ctx),
            ParseCommands::Deps { file } => Self::handle_deps(file, ctx),
            ParseCommands::Install { file } => Self::handle_install(file, ctx),
            ParseCommands::Build { file } => Self::handle_build(file, ctx),
            ParseCommands::Env { file } => Self::handle_env(file, ctx),
        }
    }
}

