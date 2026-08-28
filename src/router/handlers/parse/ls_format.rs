//! Rendering for ls: the compact, JSON and raw shapes. Parsing lives in ls.rs; this file only turns a parsed LsOutput into text.

use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
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
            if entry.name == "." || entry.name == ".." || entry.name.is_empty() {
                continue;
            }
            // Skip entries that look like raw ls lines (contain permissions)
            if entry.name.contains("drwx") || entry.name.contains("lrwx") {
                continue;
            }
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
                output.push_str(&format!(
                    "{}  {}\n",
                    entry.name,
                    Self::format_human_size(size)
                ));
            } else {
                output.push_str(&format!("{}\n", entry.name));
            }
        }

        // Summary line
        let dir_count = ls_output.directories.len();
        let file_count = ls_output.files.len();
        let sym_count = ls_output.symlinks.len();
        let mut summary_parts = Vec::new();
        if file_count > 0 {
            summary_parts.push(format!("{} files", file_count));
        }
        if dir_count > 0 {
            summary_parts.push(format!("{} dirs", dir_count));
        }
        if sym_count > 0 {
            summary_parts.push(format!("{} symlinks", sym_count));
        }
        if !ls_output.generated.is_empty() {
            summary_parts.push(format!("{} generated", ls_output.generated.len()));
        }
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
