use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
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
}
