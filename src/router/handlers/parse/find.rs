use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
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
}
