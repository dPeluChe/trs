use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
    /// Handle the logs subcommand.
    pub(crate) fn handle_logs(file: &Option<std::path::PathBuf>, ctx: &CommandContext) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse the log output
        let logs_output = Self::parse_logs(&input);

        // Format output based on the requested format
        let output = Self::format_logs(&logs_output, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("logs")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(logs_output.total_lines)
                .with_extra("Debug", logs_output.debug_count.to_string())
                .with_extra("Info", logs_output.info_count.to_string())
                .with_extra("Warning", logs_output.warning_count.to_string())
                .with_extra("Error", logs_output.error_count.to_string())
                .with_extra("Fatal", logs_output.fatal_count.to_string())
                .with_extra(
                    "Repeated lines",
                    logs_output.repeated_lines.len().to_string(),
                );
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }

    /// Parse log output into structured data.
    ///
    /// Supports various log formats:
    /// - Common timestamp formats (ISO 8601, syslog, etc.)
    /// - Log levels: DEBUG, INFO, WARN/WARNING, ERROR, FATAL/CRITICAL
    /// - Various formats: `[LEVEL]`, `LEVEL:`, `|LEVEL|`, etc.
    pub(crate) fn parse_logs(input: &str) -> LogsOutput {
        let mut logs_output = LogsOutput::default();
        let mut line_tracker: std::collections::HashMap<String, (usize, usize, usize)> =
            std::collections::HashMap::new();

        for (idx, line) in input.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines but count them
            if trimmed.is_empty() {
                continue;
            }

            // Track repeated lines
            let entry = line_tracker
                .entry(trimmed.to_string())
                .or_insert((0, line_num, line_num));
            entry.0 += 1;
            entry.2 = line_num;

            // Parse the log line
            let log_entry = Self::parse_log_line(trimmed, line_num);
            logs_output.entries.push(log_entry.clone());
            logs_output.total_lines += 1;

            // Count by level
            match log_entry.level {
                LogLevel::Debug => logs_output.debug_count += 1,
                LogLevel::Info => logs_output.info_count += 1,
                LogLevel::Warning => logs_output.warning_count += 1,
                LogLevel::Error => logs_output.error_count += 1,
                LogLevel::Fatal => logs_output.fatal_count += 1,
                LogLevel::Unknown => logs_output.unknown_count += 1,
            }

            // Track recent critical lines (ERROR and FATAL)
            if log_entry.level == LogLevel::Error || log_entry.level == LogLevel::Fatal {
                logs_output.recent_critical.push(log_entry.clone());
                // Keep only the most recent MAX_RECENT_CRITICAL entries
                if logs_output.recent_critical.len() > MAX_RECENT_CRITICAL {
                    logs_output.recent_critical.remove(0);
                }
            }
        }

        // Build repeated lines list (only lines repeated more than once)
        for (line, (count, first_line, last_line)) in line_tracker {
            if count > 1 {
                logs_output.repeated_lines.push(RepeatedLine {
                    line,
                    count,
                    first_line,
                    last_line,
                });
            }
        }

        // Sort repeated lines by first occurrence
        logs_output.repeated_lines.sort_by_key(|r| r.first_line);

        logs_output.is_empty = logs_output.entries.is_empty();
        logs_output
    }

    /// Parse a single log line.
    pub(crate) fn parse_log_line(line: &str, line_number: usize) -> LogEntry {
        let mut entry = LogEntry {
            line: line.to_string(),
            level: LogLevel::Unknown,
            timestamp: None,
            source: None,
            message: line.to_string(),
            line_number,
        };

        // Try to extract timestamp
        entry.timestamp = Self::extract_timestamp(line);

        // Try to extract log level
        entry.level = Self::detect_log_level(line);

        // Extract message (remove timestamp and level prefix)
        entry.message = Self::extract_message(line, &entry.timestamp, &entry.level);

        entry
    }

}
