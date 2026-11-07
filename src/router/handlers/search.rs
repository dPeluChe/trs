use super::common::{
    check_root, finish_partial, project_walker, readable, CommandContext, CommandError,
    CommandResult, CommandStats,
};
use super::parse::ParseHandler;
use super::types::*;
use crate::OutputFormat;

pub(crate) struct SearchHandler;

impl SearchHandler {
    /// Default maximum number of files to show in output before truncation.
    const DEFAULT_MAX_FILES: usize = 50;

    /// Execute high-performance search using ripgrep crates.
    pub(crate) fn execute_search(
        &self,
        input: &SearchInput,
        format: OutputFormat,
    ) -> CommandResult<(GrepOutput, Vec<String>)> {
        use grep::matcher::Matcher;
        use grep::regex::RegexMatcher;
        use grep::searcher::Searcher;
        use grep::searcher::SearcherBuilder;
        use grep::searcher::Sink;
        use std::sync::{Arc, Mutex};

        check_root(&input.path)?;

        // Build the regex matcher
        let matcher = if input.ignore_case {
            RegexMatcher::new(&format!("(?i){}", input.query))
        } else {
            RegexMatcher::new(&input.query)
        }
        .map_err(|e| CommandError::ExecutionError {
            message: format!("Invalid regex pattern '{}': {}", input.query, e),
            exit_code: Some(2),
        })?;

        /// A single match result with column information.
        #[derive(Debug, Clone)]
        struct MatchResult {
            line_number: usize,
            column: usize,
            line: String,
            excerpt: String,
            is_context: bool,
        }

        /// Custom sink to capture match positions and excerpts.
        struct MatchSink {
            matches: Vec<MatchResult>,
            matcher: RegexMatcher,
        }

        impl MatchSink {
            fn new(matcher: RegexMatcher) -> Self {
                Self {
                    matches: Vec::new(),
                    matcher,
                }
            }
        }

        impl Sink for MatchSink {
            type Error = std::io::Error;

            fn matched(
                &mut self,
                _searcher: &Searcher,
                mat: &grep::searcher::SinkMatch<'_>,
            ) -> Result<bool, Self::Error> {
                let line_number = mat.line_number().unwrap_or(0) as usize;
                let line_bytes = mat.bytes();
                let line_str = String::from_utf8_lossy(line_bytes);
                let line = line_str.to_string();

                // Find the column position and extract the excerpt
                let (column, excerpt) = if let Ok(Some(m)) = self.matcher.find(line_bytes) {
                    let col = m.start();
                    let excerpt_bytes = &line_bytes[m.start()..m.end()];
                    let excerpt_str = String::from_utf8_lossy(excerpt_bytes);
                    // Calculate character column (not byte offset) for display
                    let char_col =
                        String::from_utf8_lossy(&line_bytes[..col.min(line_bytes.len())])
                            .chars()
                            .count();
                    (char_col + 1, excerpt_str.to_string()) // 1-indexed
                } else {
                    (1, String::new())
                };

                self.matches.push(MatchResult {
                    line_number,
                    column,
                    line: line.trim_end().to_string(),
                    excerpt,
                    is_context: false,
                });
                Ok(true)
            }

            fn context(
                &mut self,
                _searcher: &Searcher,
                ctx: &grep::searcher::SinkContext<'_>,
            ) -> Result<bool, Self::Error> {
                let line_number = ctx.line_number().unwrap_or(0) as usize;
                let line_bytes = ctx.bytes();
                let line_str = String::from_utf8_lossy(line_bytes);

                self.matches.push(MatchResult {
                    line_number,
                    column: 0,
                    line: line_str.trim_end().to_string(),
                    excerpt: String::new(),
                    is_context: true,
                });
                Ok(true)
            }
        }

        // Shared state for collecting matches per file
        let file_matches: Arc<Mutex<Vec<(String, Vec<MatchResult>)>>> =
            Arc::new(Mutex::new(Vec::new()));

        let mut failures: Vec<String> = Vec::new();

        // Search each file
        for entry in project_walker(&input.path).build() {
            let Some(entry) = readable(entry, &mut failures) else {
                continue;
            };

            // Skip directories
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            // Skip files that don't match the extension filter
            if let Some(ref ext) = input.extension {
                let path_ext = entry.path().extension().and_then(|e| e.to_str());
                if path_ext != Some(ext) {
                    continue;
                }
            }

            // Forward-slash output for cross-platform consistency (#60).
            let path = crate::path_display::display_path(entry.path());

            // Create a new searcher for each file
            let mut searcher_builder = SearcherBuilder::new();
            searcher_builder.line_number(true);

            // Configure context if requested
            if let Some(ctx) = input.context {
                searcher_builder.before_context(ctx);
                searcher_builder.after_context(ctx);
            }

            let mut searcher = searcher_builder.build();

            // Create sink with matcher clone
            let mut sink = MatchSink::new(matcher.clone());

            // Search with our custom sink
            if let Err(e) = searcher.search_path(&matcher, entry.path(), &mut sink) {
                failures.push(format!("{path}: {e}"));
            }

            if !sink.matches.is_empty() {
                file_matches.lock().unwrap().push((path, sink.matches));
            }
        }

        // Collect results into GrepOutput
        let collected = Arc::try_unwrap(file_matches)
            .expect("All references should be dropped")
            .into_inner()
            .expect("Mutex should not be poisoned");

        // Convert to GrepFile structures with excerpts
        let mut files: Vec<GrepFile> = collected
            .into_iter()
            .map(|(path, match_results)| {
                let grep_matches: Vec<GrepMatch> = match_results
                    .into_iter()
                    .map(|mr| GrepMatch {
                        line_number: Some(mr.line_number),
                        column: if mr.is_context { None } else { Some(mr.column) },
                        line: mr.line,
                        is_context: mr.is_context,
                        excerpt: if mr.excerpt.is_empty() || mr.is_context {
                            None
                        } else {
                            Some(mr.excerpt)
                        },
                    })
                    .collect();
                GrepFile {
                    path,
                    matches: grep_matches,
                }
            })
            .collect();

        // Sort files by path
        files.sort_by(|a, b| a.path.cmp(&b.path));

        // Calculate counts
        let file_count = files.len();
        let match_count: usize = files.iter().map(GrepFile::match_count).sum();

        // Calculate input_bytes (total bytes of all matched lines)
        let input_bytes: usize = files
            .iter()
            .flat_map(|f| f.matches.iter())
            .map(|m| m.line.len())
            .sum();

        // Apply truncation
        match input.limit {
            Some(limit) => Self::keep_first_results(&mut files, limit),
            None => files.truncate(format.cap(Self::DEFAULT_MAX_FILES)),
        }

        let files_shown = files.len();
        let matches_shown: usize = files.iter().map(GrepFile::match_count).sum();
        let is_truncated = files_shown < file_count || matches_shown < match_count;
        if is_truncated {
            crate::parse_out::mark_dropped();
        }

        let output = GrepOutput {
            files,
            file_count,
            match_count,
            is_empty: file_count == 0,
            has_unread: false,
            is_truncated,
            total_files: file_count,
            total_matches: match_count,
            files_shown,
            matches_shown,
            input_bytes,
        };
        Ok((output, failures))
    }

    fn keep_first_results(files: &mut Vec<GrepFile>, limit: usize) {
        let mut left = limit;
        files.retain_mut(|file| {
            if left == 0 {
                return false;
            }
            file.keep_matches(left);
            left -= file.match_count();
            true
        });
    }

    /// Format search output for display.
    pub(crate) fn format_output(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => ParseHandler::format_grep_json(grep_output),
            OutputFormat::Csv => Self::format_csv(grep_output),
            OutputFormat::Tsv => Self::format_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(grep_output),
            OutputFormat::Raw => Self::format_raw(grep_output),
        }
    }

    /// Format search output as CSV.
    pub(crate) fn format_csv(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_csv(grep_output)
    }

    /// Format search output as TSV.
    pub(crate) fn format_tsv(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_tsv(grep_output)
    }

    /// Format search output in compact format.
    pub(crate) fn format_compact(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_compact(grep_output)
    }

    /// Format search output as raw (ripgrep output).
    pub(crate) fn format_raw(grep_output: &GrepOutput) -> String {
        ParseHandler::format_grep_raw(grep_output)
    }
}

impl CommandHandler for SearchHandler {
    type Input = SearchInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Execute the search
        let (grep_output, failures) = self.execute_search(input, ctx.format)?;
        let output = Self::format_output(&grep_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("search")
                .with_output_mode(ctx.format)
                .with_input_bytes(grep_output.input_bytes)
                .with_items_processed(grep_output.matches_shown)
                .with_items_filtered(
                    grep_output
                        .total_matches
                        .saturating_sub(grep_output.matches_shown),
                )
                .with_output_bytes(output.len())
                .with_extra("Files searched", grep_output.total_files.to_string())
                .with_extra("Files with matches", grep_output.file_count.to_string())
                .with_extra("Total matches", grep_output.total_matches.to_string());
            stats.print();
        }

        finish_partial(&output, &failures, ctx.format)
    }
}

/// Input data for the `search` command.
#[derive(Debug, Clone)]
pub(crate) struct SearchInput {
    pub path: std::path::PathBuf,
    pub query: String,
    pub extension: Option<String>,
    pub ignore_case: bool,
    pub context: Option<usize>,
    pub limit: Option<usize>,
}
