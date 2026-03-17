use super::super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

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
}
