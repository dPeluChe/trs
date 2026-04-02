use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

/// Detected SQL output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbFormat {
    Psql,
    Mysql,
    Sqlite,
}

/// Parsed database query result.
#[derive(Debug, Clone)]
pub(crate) struct DbResult {
    pub format: DbFormat,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    /// Footer text like "(2 rows)" or "2 rows in set (0.00 sec)"
    pub footer: Option<String>,
}

const HEAD_ROWS: usize = 10;
const TAIL_ROWS: usize = 5;
const TRUNCATE_THRESHOLD: usize = 20;

impl ParseHandler {
    /// Detect whether the input is psql, mysql, or sqlite3 tabular output.
    pub(crate) fn detect_db_format(input: &str) -> Option<DbFormat> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // MySQL: first non-empty line starts with +--- and contains +
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('+') && trimmed.contains("---") {
                return Some(DbFormat::Mysql);
            }
            break;
        }

        // Psql: look for a separator line like "----+--------+-----"
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Psql header separator: dashes with + separators, no leading +
            if !trimmed.starts_with('+')
                && trimmed.contains("-+-")
                && trimmed.chars().all(|c| c == '-' || c == '+' || c == ' ')
            {
                return Some(DbFormat::Psql);
            }
            // Also detect if first line has pipe-separated headers (psql style with spaces)
            if trimmed.contains(" | ") {
                return Some(DbFormat::Psql);
            }
            break;
        }

        // Psql: check second line for separator pattern
        if lines.len() >= 2 {
            let second = lines[1].trim();
            if !second.starts_with('+')
                && second.contains("-+-")
                && second.chars().all(|c| c == '-' || c == '+' || c == ' ')
            {
                return Some(DbFormat::Psql);
            }
        }

        // Sqlite: pipe-separated without spaces around pipes, no border chars
        if lines.len() >= 2 {
            let first = lines[0].trim();
            let second = lines[1].trim();
            if first.contains('|')
                && !first.contains(" | ")
                && !first.starts_with('+')
                && !first.starts_with('|')
                && second.contains('|')
            {
                return Some(DbFormat::Sqlite);
            }
        }

        None
    }

    /// Parse psql tabular output.
    fn parse_psql(input: &str) -> DbResult {
        let lines: Vec<&str> = input.lines().collect();
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut footer = None;
        let mut data_started = false;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Footer line: "(N rows)" or "(N row)"
            if trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.contains("row") {
                footer = Some(trimmed.to_string());
                continue;
            }

            // Separator line: skip
            if trimmed.contains("-+-") && trimmed.chars().all(|c| c == '-' || c == '+' || c == ' ')
            {
                data_started = true;
                continue;
            }

            if !data_started && columns.is_empty() {
                // Header row
                columns = trimmed.split('|').map(|s| s.trim().to_string()).collect();
            } else if data_started {
                // Data row
                let cells: Vec<String> = trimmed.split('|').map(|s| s.trim().to_string()).collect();
                rows.push(cells);
            }
        }

        let row_count = rows.len();
        DbResult {
            format: DbFormat::Psql,
            columns,
            rows,
            row_count,
            footer,
        }
    }

    /// Parse mysql tabular output.
    fn parse_mysql(input: &str) -> DbResult {
        let lines: Vec<&str> = input.lines().collect();
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut footer = None;
        let mut header_found = false;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Border line: +----+-----+
            if trimmed.starts_with('+') && trimmed.ends_with('+') && trimmed.contains('-') {
                continue;
            }

            // Footer: "N rows in set" or "N row in set"
            if (trimmed.contains("rows in set") || trimmed.contains("row in set"))
                || (trimmed.contains("rows affected") || trimmed.contains("row affected"))
            {
                footer = Some(trimmed.to_string());
                continue;
            }

            // Data/header rows: | val1 | val2 |
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let cells: Vec<String> = trimmed[1..trimmed.len() - 1]
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .collect();

                if !header_found {
                    columns = cells;
                    header_found = true;
                } else {
                    rows.push(cells);
                }
            }
        }

        let row_count = rows.len();
        DbResult {
            format: DbFormat::Mysql,
            columns,
            rows,
            row_count,
            footer,
        }
    }

    /// Parse sqlite3 pipe-separated output.
    fn parse_sqlite(input: &str) -> DbResult {
        let lines: Vec<&str> = input.lines().collect();
        let mut columns = Vec::new();
        let mut rows = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let cells: Vec<String> = trimmed.split('|').map(|s| s.to_string()).collect();

            if i == 0 {
                columns = cells;
            } else {
                rows.push(cells);
            }
        }

        let row_count = rows.len();
        DbResult {
            format: DbFormat::Sqlite,
            columns,
            rows,
            row_count,
            footer: None,
        }
    }

    /// Parse database query output, auto-detecting format.
    pub(crate) fn parse_db(input: &str) -> DbResult {
        match Self::detect_db_format(input) {
            Some(DbFormat::Psql) => Self::parse_psql(input),
            Some(DbFormat::Mysql) => Self::parse_mysql(input),
            Some(DbFormat::Sqlite) => Self::parse_sqlite(input),
            None => {
                // Fallback: try psql, then mysql, then sqlite
                let psql = Self::parse_psql(input);
                if !psql.columns.is_empty() {
                    return psql;
                }
                let mysql = Self::parse_mysql(input);
                if !mysql.columns.is_empty() {
                    return mysql;
                }
                Self::parse_sqlite(input)
            }
        }
    }

    /// Build sample rows for truncated output.
    pub(crate) fn sample_rows(rows: &[Vec<String>]) -> (Vec<&Vec<String>>, usize) {
        if rows.len() <= TRUNCATE_THRESHOLD {
            let refs: Vec<&Vec<String>> = rows.iter().collect();
            return (refs, 0);
        }
        let head: Vec<&Vec<String>> = rows.iter().take(HEAD_ROWS).collect();
        let tail: Vec<&Vec<String>> = rows.iter().rev().take(TAIL_ROWS).rev().collect();
        let omitted = rows.len() - HEAD_ROWS - TAIL_ROWS;
        let mut sample = head;
        sample.extend(tail);
        (sample, omitted)
    }

    /// Format a row as a compact pipe-delimited line.
    fn format_row_compact(cells: &[String]) -> String {
        cells.join(" | ")
    }

    /// Format compact output.
    fn format_db_compact(result: &DbResult) -> String {
        let col_list = result.columns.join(", ");
        let mut out = format!(
            "{} rows x {} cols  [{}]\n",
            result.row_count,
            result.columns.len(),
            col_list
        );

        let (sample, omitted) = Self::sample_rows(&result.rows);
        for (i, row) in sample.iter().enumerate() {
            if omitted > 0 && i == HEAD_ROWS {
                out.push_str(&format!("[...{} rows omitted]\n", omitted));
            }
            out.push_str(&Self::format_row_compact(row));
            out.push('\n');
        }

        if let Some(ref f) = result.footer {
            out.push_str(f);
            out.push('\n');
        }

        out
    }

    /// Format JSON output.
    pub(crate) fn format_db_json(result: &DbResult) -> String {
        let (sample, _omitted) = Self::sample_rows(&result.rows);
        let sample_json: Vec<serde_json::Value> = sample
            .iter()
            .map(|row| {
                let obj: serde_json::Map<String, serde_json::Value> = result
                    .columns
                    .iter()
                    .zip(row.iter())
                    .map(|(col, val)| (col.clone(), serde_json::Value::String(val.clone())))
                    .collect();
                serde_json::Value::Object(obj)
            })
            .collect();

        let format_name = match result.format {
            DbFormat::Psql => "psql",
            DbFormat::Mysql => "mysql",
            DbFormat::Sqlite => "sqlite3",
        };

        let json = serde_json::json!({
            "format": format_name,
            "columns": result.columns,
            "row_count": result.row_count,
            "sample_rows": sample_json,
        });

        Self::json_to_string(json)
    }

    /// Escape a CSV field (quote if it contains comma, quote, or newline).
    fn csv_escape(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Format CSV output.
    pub(crate) fn format_db_csv(result: &DbResult) -> String {
        let mut out = String::new();
        // Header
        let header: Vec<String> = result.columns.iter().map(|c| Self::csv_escape(c)).collect();
        out.push_str(&header.join(","));
        out.push('\n');

        let (sample, omitted) = Self::sample_rows(&result.rows);
        for (i, row) in sample.iter().enumerate() {
            if omitted > 0 && i == HEAD_ROWS {
                out.push_str(&format!("# [...{} rows omitted]\n", omitted));
            }
            let escaped: Vec<String> = row.iter().map(|c| Self::csv_escape(c)).collect();
            out.push_str(&escaped.join(","));
            out.push('\n');
        }
        out
    }

    /// Format TSV output.
    pub(crate) fn format_db_tsv(result: &DbResult) -> String {
        let mut out = String::new();
        out.push_str(&result.columns.join("\t"));
        out.push('\n');

        let (sample, omitted) = Self::sample_rows(&result.rows);
        for (i, row) in sample.iter().enumerate() {
            if omitted > 0 && i == HEAD_ROWS {
                out.push_str(&format!("# [...{} rows omitted]\n", omitted));
            }
            out.push_str(&row.join("\t"));
            out.push('\n');
        }
        out
    }

    pub(crate) fn handle_db(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();
        let result = Self::parse_db(&input);

        let output = match ctx.format {
            OutputFormat::Json => Self::format_db_json(&result),
            OutputFormat::Csv => Self::format_db_csv(&result),
            OutputFormat::Tsv => Self::format_db_tsv(&result),
            OutputFormat::Raw => input.clone(),
            _ => Self::format_db_compact(&result),
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("db")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(result.row_count)
                .print();
        }
        Ok(())
    }
}
