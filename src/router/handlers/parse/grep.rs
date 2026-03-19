use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
    /// Handle the grep subcommand.
    pub(crate) fn handle_grep(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the grep output
        let mut grep_output = Self::parse_grep(&input)?;

        // Apply truncation for large result sets (limits from config)
        let limits = &crate::config::config().limits;
        Self::truncate_grep(
            &mut grep_output,
            limits.grep_max_results,
            limits.grep_max_per_file,
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

    /// Truncate grep output if it exceeds the limits.
    ///
    /// This truncates both the number of files and the number of matches per file
    /// to prevent overwhelming output for large result sets.
    pub(crate) fn truncate_grep(
        grep_output: &mut GrepOutput,
        max_files: usize,
        max_matches_per_file: usize,
    ) {
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
}
