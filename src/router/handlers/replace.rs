use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::OutputFormat;

#[derive(Debug, Clone)]
pub(crate) struct Replacement {
    line_number: usize,
    original: String,
    replaced: String,
    /// Number of individual matches replaced in this line.
    match_count: usize,
}

/// Handler for the `replace` command.
pub(crate) struct ReplaceHandler;

impl ReplaceHandler {
    /// Default directories to ignore during replace.
    const DEFAULT_IGNORE_DIRS: &'static [&'static str] = &[
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".cache",
        "__pycache__",
        ".venv",
        "venv",
        ".idea",
        ".vscode",
        "vendor",
        "bundle",
        ".tox",
        ".mypy_cache",
        ".pytest_cache",
        "coverage",
        ".next",
        ".nuxt",
    ];

    /// Execute search and replace using ripgrep crates.
    pub(crate) fn execute_replace(
        &self,
        input: &ReplaceInput,
    ) -> CommandResult<Vec<(String, Vec<Replacement>)>> {
        use grep::matcher::Matcher;
        use grep::regex::RegexMatcher;
        use ignore::WalkBuilder;

        // Build the regex matcher
        let matcher =
            RegexMatcher::new(&input.search).map_err(|e| CommandError::ExecutionError {
                message: format!("Invalid regex pattern '{}': {}", input.search, e),
                exit_code: Some(2),
            })?;

        // Shared state for collecting replacements per file
        let mut file_replacements: Vec<(String, Vec<Replacement>)> = Vec::new();

        // Build the directory walker with ignore rules
        let mut walk_builder = WalkBuilder::new(&input.path);

        // Configure walker
        walk_builder
            .hidden(false)
            .git_ignore(true)
            .ignore(true)
            .follow_links(false);

        // Create a set of ignored directory names for filtering
        let ignore_dirs: std::collections::HashSet<&str> =
            Self::DEFAULT_IGNORE_DIRS.iter().copied().collect();

        // Process each file
        for entry_result in walk_builder.build() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip directories
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            // Skip files inside ignored directories
            let should_skip = entry.path().components().any(|c| {
                if let std::path::Component::Normal(name) = c {
                    name.to_str().map(|s| ignore_dirs.contains(s)).unwrap_or(false)
                } else {
                    false
                }
            });
            if should_skip {
                continue;
            }

            // Skip files that don't match the extension filter
            if let Some(ref ext) = input.extension {
                let path_ext = entry.path().extension().and_then(|e| e.to_str());
                if path_ext != Some(ext) {
                    continue;
                }
            }

            let path = entry.path().to_string_lossy().to_string();

            // Read file content
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };

            // Find all matches in this file
            let lines: Vec<&str> = content.lines().collect();
            let mut replacements: Vec<Replacement> = Vec::new();
            let mut modified_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let mut has_changes = false;

            for (line_idx, line) in lines.iter().enumerate() {
                let line_bytes = line.as_bytes();

                // Count matches first, then replace all at once
                let mut offset = 0usize;
                let mut line_match_count = 0usize;

                while let Ok(Some(m)) = matcher.find_at(line_bytes, offset) {
                    line_match_count += 1;
                    let end = m.end();
                    let start = m.start();
                    if end == start {
                        offset = end + 1;
                    } else {
                        offset = end;
                    }
                    if offset >= line_bytes.len() {
                        break;
                    }
                }

                // Use regex crate to do the actual replacement (handles offsets correctly)
                let modified_line = if line_match_count > 0 {
                    has_changes = true;
                    let re = regex::Regex::new(&input.search).unwrap();
                    re.replace_all(line, &input.replace as &str).to_string()
                } else {
                    line.to_string()
                };

                if line_match_count > 0 {
                    replacements.push(Replacement {
                        line_number: line_idx + 1, // 1-indexed
                        original: line.to_string(),
                        replaced: modified_line.clone(),
                        match_count: line_match_count,
                    });
                    modified_lines[line_idx] = modified_line;
                }
            }

            // If there are changes and not dry run, write the file back
            if has_changes {
                if !input.dry_run {
                    let mut new_content = modified_lines.join("\n");
                    // Preserve trailing newline if original had one
                    if content.ends_with('\n') && !new_content.ends_with('\n') {
                        new_content.push('\n');
                    }
                    if let Err(e) = std::fs::write(entry.path(), new_content) {
                        eprintln!("Warning: Failed to write {}: {}", path, e);
                        continue;
                    }
                }
                file_replacements.push((path, replacements));
            }
        }

        Ok(file_replacements)
    }

    /// Format replace output based on the specified format.
    pub(crate) fn format_output(
        replacements: &[(String, Vec<Replacement>)],
        input: &ReplaceInput,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => Self::format_json(replacements, input),
            OutputFormat::Csv => Self::format_csv(replacements, input),
            OutputFormat::Tsv => Self::format_tsv(replacements, input),
            OutputFormat::Compact | OutputFormat::Agent => {
                Self::format_compact(replacements, input)
            }
            OutputFormat::Raw => Self::format_raw(replacements, input),
        }
    }

    /// Format replace output as JSON using the schema.
    pub(crate) fn format_json(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        use crate::schema::{ReplaceCounts, ReplaceFile, ReplaceMatch, ReplaceOutputSchema};

        let files: Vec<ReplaceFile> = replacements
            .iter()
            .map(|(path, reps)| {
                let matches: Vec<ReplaceMatch> = reps
                    .iter()
                    .map(|r| ReplaceMatch::new(r.line_number, &r.original, &r.replaced))
                    .collect();
                ReplaceFile {
                    path: path.clone(),
                    matches,
                }
            })
            .collect();

        let total_replacements: usize = files.iter().map(|f| f.matches.len()).sum();

        let schema = ReplaceOutputSchema::new(&input.search, &input.replace, input.dry_run)
            .with_files(files)
            .with_counts(ReplaceCounts {
                files_affected: replacements.len(),
                total_replacements,
            });

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
    }

    /// Format replace output as CSV.
    pub(crate) fn format_csv(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();
        result.push_str("file,line_number,original,replaced\n");

        for (path, reps) in replacements {
            for rep in reps {
                let original_escaped = Self::escape_csv_field(&rep.original);
                let replaced_escaped = Self::escape_csv_field(&rep.replaced);
                result.push_str(&format!(
                    "{},{},{},{}\n",
                    path, rep.line_number, original_escaped, replaced_escaped
                ));
            }
        }

        // Add summary at the end
        let total_replacements: usize = replacements.iter().map(|(_, r)| r.iter().map(|rep| rep.match_count).sum::<usize>()).sum();
        result.push_str(&format!(
            "\n# Summary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Format replace output as TSV.
    pub(crate) fn format_tsv(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();
        result.push_str("file\tline_number\toriginal\treplaced\n");

        for (path, reps) in replacements {
            for rep in reps {
                let original_escaped = Self::escape_tsv_field(&rep.original);
                let replaced_escaped = Self::escape_tsv_field(&rep.replaced);
                result.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    path, rep.line_number, original_escaped, replaced_escaped
                ));
            }
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.iter().map(|rep| rep.match_count).sum::<usize>()).sum();
        result.push_str(&format!(
            "\n# Summary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Format replace output in compact format.
    pub(crate) fn format_compact(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();

        if replacements.is_empty() {
            if input.dry_run {
                result.push_str("No matches found.\n");
            } else {
                result.push_str("No changes made.\n");
            }
            return result;
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.iter().map(|rep| rep.match_count).sum::<usize>()).sum();

        if input.dry_run {
            result.push_str(&format!(
                "Preview: {} replacements in {} files\n\n",
                total_replacements,
                replacements.len()
            ));
        } else {
            result.push_str(&format!(
                "Replaced {} occurrences in {} files\n\n",
                total_replacements,
                replacements.len()
            ));
        }

        for (path, reps) in replacements {
            result.push_str(&format!("{}:\n", path));
            for rep in reps {
                result.push_str(&format!(
                    "  {}:{}\n",
                    rep.line_number,
                    Self::truncate_line(&rep.replaced, 80)
                ));
            }
            result.push('\n');
        }

        result
    }

    /// Format replace output as raw.
    pub(crate) fn format_raw(replacements: &[(String, Vec<Replacement>)], input: &ReplaceInput) -> String {
        let mut result = String::new();

        for (path, reps) in replacements {
            for rep in reps {
                result.push_str(&format!(
                    "{}:{}: {} -> {}\n",
                    path, rep.line_number, rep.original, rep.replaced
                ));
            }
        }

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.iter().map(|rep| rep.match_count).sum::<usize>()).sum();
        result.push_str(&format!(
            "\nSummary: {} files, {} replacements (dry_run: {})\n",
            replacements.len(),
            total_replacements,
            input.dry_run
        ));

        result
    }

    /// Truncate a line to a maximum length.
    pub(crate) fn truncate_line(line: &str, max_len: usize) -> String {
        if line.len() <= max_len {
            line.to_string()
        } else {
            format!("{}...", &line[..max_len.saturating_sub(3)])
        }
    }

    /// Escape a field for CSV format.
    pub(crate) fn escape_csv_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Escape a field for TSV format.
    pub(crate) fn escape_tsv_field(field: &str) -> String {
        field
            .replace('\t', "\\t")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    /// Format replacement count for output (just the number).
    pub(crate) fn format_count(count: usize, format: OutputFormat) -> String {
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
}

impl CommandHandler for ReplaceHandler {
    type Input = ReplaceInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Execute the replace
        let replacements = self.execute_replace(input)?;

        let total_replacements: usize = replacements.iter().map(|(_, r)| r.iter().map(|rep| rep.match_count).sum::<usize>()).sum();
        let files_count = replacements.len();
        // Calculate input_bytes (total bytes of all original lines)
        let input_bytes: usize = replacements
            .iter()
            .flat_map(|(_, r)| r.iter())
            .map(|r| r.original.len())
            .sum();

        // If count flag is specified, output only the count
        if input.count {
            let output = Self::format_count(total_replacements, ctx.format);
            if ctx.stats {
                let stats = CommandStats::new()
                    .with_reducer("replace")
                    .with_output_mode(ctx.format)
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(output.len())
                    .with_items_processed(total_replacements)
                    .with_extra("Files affected", files_count.to_string())
                    .with_extra("Dry run", input.dry_run.to_string());
                stats.print();
            }
            print!("{}", output);
            return Ok(());
        }

        // Format and print the output
        let output = Self::format_output(&replacements, input, ctx.format);

        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("replace")
                .with_output_mode(ctx.format)
                .with_input_bytes(input_bytes)
                .with_items_processed(total_replacements)
                .with_output_bytes(output.len())
                .with_extra("Files affected", files_count.to_string())
                .with_extra("Dry run", input.dry_run.to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }
}

/// Input data for the `replace` command.
#[derive(Debug, Clone)]
pub(crate) struct ReplaceInput {
    pub path: std::path::PathBuf,
    pub search: String,
    pub replace: String,
    pub extension: Option<String>,
    pub dry_run: bool,
    pub count: bool,
}
