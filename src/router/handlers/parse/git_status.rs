use super::super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

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
    pub(crate) fn parse_file_entry(
        line: &str,
        section: GitStatusSection,
    ) -> Option<GitStatusEntry> {
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
}
