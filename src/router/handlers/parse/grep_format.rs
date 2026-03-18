//! Grep output formatting functions.
//!
//! Contains format_grep and all format-specific variants (JSON, compact, CSV, TSV, raw).

use super::super::run::RunHandler;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
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

        // Show summary header only for multi-file results or truncated output
        if grep_output.is_truncated {
            output.push_str(&format!(
                "matches: {}/{} files, {}/{} results (truncated)\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else if grep_output.file_count > 1 {
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

        // Add total only for multi-file results or truncated output
        if grep_output.is_truncated {
            output.push_str(&format!(
                "total: {}/{} files, {}/{} matches\n",
                grep_output.files_shown,
                grep_output.total_files,
                grep_output.matches_shown,
                grep_output.total_matches
            ));
        } else if grep_output.file_count > 1 {
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
}
