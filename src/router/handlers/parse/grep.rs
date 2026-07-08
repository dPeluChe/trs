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

        crate::parse_out::emit(&output);

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

        // Two-pass scan: prefer ':N:' patterns (match lines) over '-N-' (context lines).
        // This correctly handles paths that contain dashes (e.g. src/my-module/foo.rs:10:content)
        // because we look for the FIRST separator+digits+separator sequence per type.

        // Pass 1: match line — find ':N:' or ':N$'
        if let Some((path, lineno, rest)) = find_sep_digits(line, b':') {
            // Optional column: rest may start with 'N:content'
            let (column, content) = extract_column(rest);
            return Some((
                path,
                GrepMatch {
                    line_number: lineno,
                    column,
                    line: content,
                    is_context: false,
                    excerpt: None,
                },
            ));
        }

        // Pass 2: context line — find '-N-' or '-N$'
        if let Some((path, lineno, rest)) = find_sep_digits(line, b'-') {
            // Optional column for context lines: rest may start with 'N-content'
            let (column, content) = extract_column_sep(rest, b'-');
            return Some((
                path,
                GrepMatch {
                    line_number: lineno,
                    column,
                    line: content,
                    is_context: true,
                    excerpt: None,
                },
            ));
        }

        // Fallback: path:content without line number (grep without -n)
        if let Some(pos) = line.find(':') {
            let path = &line[..pos];
            if !path.is_empty() {
                return Some((
                    path.to_string(),
                    GrepMatch {
                        line_number: None,
                        column: None,
                        line: line[pos + 1..].to_string(),
                        is_context: false,
                        excerpt: None,
                    },
                ));
            }
        }

        None
    }
}

/// Scan `line` for the first `sep + pure_digits + (sep | eol)` sequence where
/// the path before the separator is non-empty. Returns (path, lineno, content_after).
fn find_sep_digits(line: &str, sep: u8) -> Option<(String, Option<usize>, String)> {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b != sep || i == 0 {
            continue;
        }
        let digit_start = i + 1;
        let digit_end = bytes[digit_start..]
            .iter()
            .position(|&d| !d.is_ascii_digit())
            .map(|p| digit_start + p)
            .unwrap_or(bytes.len());
        if digit_end == digit_start {
            continue; // no digits follow the separator
        }
        let terminated = digit_end == bytes.len() || bytes[digit_end] == sep;
        if !terminated {
            continue;
        }
        let path = line[..i].to_string();
        let lineno = line[digit_start..digit_end].parse::<usize>().ok();
        let content = if digit_end < bytes.len() {
            line[digit_end + 1..].to_string()
        } else {
            String::new()
        };
        return Some((path, lineno, content));
    }
    None
}

/// Extract an optional column number from a `"N:rest"` string (match lines).
/// Returns `(Some(N), rest)` if the prefix is a valid usize; otherwise `(None, original)`.
fn extract_column(s: String) -> (Option<usize>, String) {
    extract_column_sep(s, b':')
}

/// Extract an optional column from `"N<sep>rest"` for any separator byte.
fn extract_column_sep(s: String, sep: u8) -> (Option<usize>, String) {
    let bytes = s.as_bytes();
    let num_end = bytes
        .iter()
        .position(|&b| !b.is_ascii_digit())
        .unwrap_or(bytes.len());
    if num_end > 0 && num_end < bytes.len() && bytes[num_end] == sep {
        if let Ok(col) = s[..num_end].parse::<usize>() {
            return (Some(col), s[num_end + 1..].to_string());
        }
    }
    (None, s)
}
