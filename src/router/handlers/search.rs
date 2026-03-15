use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::*;
use super::parse::ParseHandler;
use crate::OutputFormat;

pub(crate) struct SearchHandler;

impl SearchHandler {
    /// Default directories to ignore during search.
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

    /// Default maximum number of files to show in output before truncation.
    const DEFAULT_MAX_FILES: usize = 50;

    /// Execute high-performance search using ripgrep crates.
    pub(crate) fn execute_search(&self, input: &SearchInput) -> CommandResult<GrepOutput> {
        use grep::matcher::Matcher;
        use grep::regex::RegexMatcher;
        use grep::searcher::Searcher;
        use grep::searcher::SearcherBuilder;
        use grep::searcher::Sink;
        use ignore::WalkBuilder;
        use std::sync::{Arc, Mutex};

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

        // Build the directory walker with ignore rules
        let mut walk_builder = WalkBuilder::new(&input.path);

        // Add custom ignore patterns for common directories
        for dir in Self::DEFAULT_IGNORE_DIRS {
            walk_builder.add_ignore(format!("!{}/", dir));
        }

        // Configure walker
        walk_builder
            .hidden(false) // Don't skip hidden files by default
            .git_ignore(true) // Respect .gitignore
            .ignore(true) // Respect .ignore files
            .follow_links(false); // Don't follow symlinks

        // Search each file
        for entry_result in walk_builder.build() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue, // Skip files we can't access
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

            let path = entry.path().to_string_lossy().to_string();

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
            let search_result = searcher.search_path(&matcher, entry.path(), &mut sink);

            // Ignore search errors (binary files, permission issues, etc.)
            if search_result.is_ok() && !sink.matches.is_empty() {
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
        let match_count: usize = files.iter().map(|f| f.matches.len()).sum();

        // Calculate input_bytes (total bytes of all matched lines)
        let input_bytes: usize = files
            .iter()
            .flat_map(|f| f.matches.iter())
            .map(|m| m.line.len())
            .sum();

        // Apply truncation
        let max_files = input.limit.unwrap_or(Self::DEFAULT_MAX_FILES);
        let is_truncated = files.len() > max_files;

        let total_files = files.len();
        let total_matches = match_count;

        // Truncate files if needed
        if files.len() > max_files {
            files.truncate(max_files);
        }

        let files_shown = files.len();
        let matches_shown: usize = files.iter().map(|f| f.matches.len()).sum();

        Ok(GrepOutput {
            files,
            file_count,
            match_count,
            is_empty: file_count == 0,
            is_truncated,
            total_files,
            total_matches,
            files_shown,
            matches_shown,
            input_bytes,
        })
    }

    /// Format search output for display.
    pub(crate) fn format_output(grep_output: &GrepOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(grep_output),
            OutputFormat::Csv => Self::format_csv(grep_output),
            OutputFormat::Tsv => Self::format_tsv(grep_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(grep_output),
            OutputFormat::Raw => Self::format_raw(grep_output),
        }
    }

    /// Format search output as JSON using the schema.
    pub(crate) fn format_json(grep_output: &GrepOutput) -> String {
        use crate::schema::{
            GrepCounts, GrepFile as SchemaGrepFile, GrepMatch as SchemaGrepMatch, GrepOutputSchema,
        };

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
            matches: grep_output.match_count,
            total_files: grep_output.total_files,
            total_matches: grep_output.total_matches,
            files_shown: grep_output.files_shown,
            matches_shown: grep_output.matches_shown,
        };

        serde_json::to_string_pretty(&schema).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("Failed to serialize: {}", e)}).to_string()
        })
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
        let grep_output = self.execute_search(input)?;

        // Print stats if requested
        if ctx.stats {
            let output = Self::format_output(&grep_output, ctx.format);
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
            print!("{}", output);
        } else {
            // Format and print the output
            let output = Self::format_output(&grep_output, ctx.format);
            print!("{}", output);
        }

        Ok(())
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
