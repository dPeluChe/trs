//! Log output formatting functions.
//!
//! Contains format_logs and all format-specific variants (JSON, compact, CSV, TSV, raw, agent).

use super::super::run::RunHandler;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Format logs output for display.
    pub(crate) fn format_logs(logs_output: &LogsOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_logs_json(logs_output),
            OutputFormat::Csv => Self::format_logs_csv(logs_output),
            OutputFormat::Tsv => Self::format_logs_tsv(logs_output),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_logs_compact(logs_output),
            OutputFormat::Raw => Self::format_logs_raw(logs_output),
        }
    }

    /// Format logs output as JSON.
    pub(crate) fn format_logs_json(logs_output: &LogsOutput) -> String {
        let total_critical = logs_output.error_count + logs_output.fatal_count;
        serde_json::json!({
            "counts": {
                "total_lines": logs_output.total_lines,
                "debug": logs_output.debug_count,
                "info": logs_output.info_count,
                "warning": logs_output.warning_count,
                "error": logs_output.error_count,
                "fatal": logs_output.fatal_count,
                "unknown": logs_output.unknown_count,
            },
            "repeated_lines": logs_output.repeated_lines.iter().map(|r| serde_json::json!({
                "line": r.line,
                "count": r.count,
                "first_line": r.first_line,
                "last_line": r.last_line,
            })).collect::<Vec<_>>(),
            "recent_critical": logs_output.recent_critical.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
            "recent_critical_count": logs_output.recent_critical.len(),
            "total_critical": total_critical,
            "entries": logs_output.entries.iter().map(|e| serde_json::json!({
                "line_number": e.line_number,
                "level": match e.level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warning",
                    LogLevel::Error => "error",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Unknown => "unknown",
                },
                "timestamp": e.timestamp,
                "message": e.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Format logs output as CSV.
    pub(crate) fn format_logs_csv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number,level,timestamp,message\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_csv_field(&entry.message);
            result.push_str(&format!(
                "{},{},{},{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output as TSV.
    pub(crate) fn format_logs_tsv(logs_output: &LogsOutput) -> String {
        let mut result = String::new();
        result.push_str("line_number\tlevel\ttimestamp\tmessage\n");

        for entry in &logs_output.entries {
            let level_str = match entry.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warning => "warning",
                LogLevel::Error => "error",
                LogLevel::Fatal => "fatal",
                LogLevel::Unknown => "unknown",
            };
            let timestamp = entry.timestamp.as_deref().unwrap_or("");
            let message_escaped = RunHandler::escape_tsv_field(&entry.message);
            result.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                entry.line_number, level_str, timestamp, message_escaped
            ));
        }

        result
    }

    /// Format logs output in compact format.
    pub(crate) fn format_logs_compact(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        if logs_output.is_empty {
            output.push_str("logs: empty\n");
            return output;
        }

        // Summary header
        output.push_str(&format!("lines: {}\n", logs_output.total_lines));

        // Level summary (only show non-zero counts)
        let mut level_parts = Vec::new();
        if logs_output.fatal_count > 0 {
            level_parts.push(format!("fatal:{}", logs_output.fatal_count));
        }
        if logs_output.error_count > 0 {
            level_parts.push(format!("error:{}", logs_output.error_count));
        }
        if logs_output.warning_count > 0 {
            level_parts.push(format!("warn:{}", logs_output.warning_count));
        }
        if logs_output.info_count > 0 {
            level_parts.push(format!("info:{}", logs_output.info_count));
        }
        if logs_output.debug_count > 0 {
            level_parts.push(format!("debug:{}", logs_output.debug_count));
        }
        if logs_output.unknown_count > 0 {
            level_parts.push(format!("other:{}", logs_output.unknown_count));
        }

        if !level_parts.is_empty() {
            output.push_str(&format!("levels: {}\n", level_parts.join(", ")));
        }

        // Repeated lines summary
        if !logs_output.repeated_lines.is_empty() {
            output.push_str(&format!(
                "repeated: {} unique lines ({} occurrences)\n",
                logs_output.repeated_lines.len(),
                logs_output
                    .repeated_lines
                    .iter()
                    .map(|r| r.count)
                    .sum::<usize>()
            ));
        }

        output.push('\n');

        // Show repeated lines
        if !logs_output.repeated_lines.is_empty() {
            output.push_str("repeated lines:\n");
            for repeated in &logs_output.repeated_lines {
                if repeated.count > 1 {
                    let preview = if repeated.line.len() > 60 {
                        format!("{}...", &repeated.line[..57])
                    } else {
                        repeated.line.clone()
                    };
                    output.push_str(&format!(
                        "  [x{}] {} (lines {}-{})\n",
                        repeated.count, preview, repeated.first_line, repeated.last_line
                    ));
                }
            }
            output.push('\n');
        }

        // Show recent critical lines (ERROR and FATAL)
        if !logs_output.recent_critical.is_empty() {
            let total_critical = logs_output.error_count + logs_output.fatal_count;
            let shown = logs_output.recent_critical.len();
            if shown < total_critical {
                output.push_str(&format!(
                    "recent critical ({} of {}):\n",
                    shown, total_critical
                ));
            } else {
                output.push_str(&format!("recent critical ({}):\n", shown));
            }
            for entry in &logs_output.recent_critical {
                let level_indicator = match entry.level {
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    _ => "[!]",
                };
                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };
                output.push_str(&format!(
                    "  {} {} {}\n",
                    level_indicator, entry.line_number, preview
                ));
            }
            output.push('\n');
        }

        // Show entries with detected levels (collapse consecutive duplicates)
        let has_levels = logs_output
            .entries
            .iter()
            .any(|e| e.level != LogLevel::Unknown);
        if has_levels {
            output.push_str("entries:\n");
            // Collapse consecutive entries with same level and message
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];
                let level_indicator = match entry.level {
                    LogLevel::Debug => "[D]",
                    LogLevel::Info => "[I]",
                    LogLevel::Warning => "[W]",
                    LogLevel::Error => "[E]",
                    LogLevel::Fatal => "[F]",
                    LogLevel::Unknown => "   ",
                };

                // Count consecutive entries with same level and message
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.level == entry.level && next.message == entry.message {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.message.len() > 80 {
                    format!("{}...", &entry.message[..77])
                } else {
                    entry.message.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "{} {}-{} {} [x{}]\n",
                        level_indicator, entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {} {}\n",
                        level_indicator, entry.line_number, preview
                    ));
                }

                i += count;
            }
        } else {
            // No levels detected, just show raw lines with line numbers (collapse consecutive duplicates)
            output.push_str("lines:\n");
            let mut i = 0;
            while i < logs_output.entries.len() {
                let entry = &logs_output.entries[i];

                // Count consecutive entries with same line content
                let mut count = 1;
                let mut last_line = entry.line_number;
                while i + count < logs_output.entries.len() {
                    let next = &logs_output.entries[i + count];
                    if next.line == entry.line {
                        count += 1;
                        last_line = next.line_number;
                    } else {
                        break;
                    }
                }

                let preview = if entry.line.len() > 80 {
                    format!("{}...", &entry.line[..77])
                } else {
                    entry.line.clone()
                };

                if count > 1 {
                    output.push_str(&format!(
                        "  {}-{} {} [x{}]\n",
                        entry.line_number, last_line, preview, count
                    ));
                } else {
                    output.push_str(&format!("  {} {}\n", entry.line_number, preview));
                }

                i += count;
            }
        }

        output
    }

    /// Format logs output as raw (original format).
    pub(crate) fn format_logs_raw(logs_output: &LogsOutput) -> String {
        let mut output = String::new();

        for entry in &logs_output.entries {
            output.push_str(&entry.line);
            output.push('\n');
        }

        output
    }
}
