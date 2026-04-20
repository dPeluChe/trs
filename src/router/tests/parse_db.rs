use super::*;

// ============================================================
// DB Parser Tests — Format Detection
// ============================================================

#[test]
fn test_detect_psql_format() {
    let input = " id | name    | email\n----+---------+-------------------\n  1 | Alice   | alice@example.com\n(1 row)\n";
    let fmt = ParseHandler::detect_db_format(input);
    assert_eq!(
        fmt,
        Some(crate::router::handlers::parse::extra_db::DbFormat::Psql)
    );
}

#[test]
fn test_detect_mysql_format() {
    let input = "+----+---------+-------------------+\n| id | name    | email             |\n+----+---------+-------------------+\n|  1 | Alice   | alice@example.com |\n+----+---------+-------------------+\n1 row in set (0.00 sec)\n";
    let fmt = ParseHandler::detect_db_format(input);
    assert_eq!(
        fmt,
        Some(crate::router::handlers::parse::extra_db::DbFormat::Mysql)
    );
}

#[test]
fn test_detect_sqlite_format() {
    let input = "id|name|email\n1|Alice|alice@example.com\n2|Bob|bob@example.com\n";
    let fmt = ParseHandler::detect_db_format(input);
    assert_eq!(
        fmt,
        Some(crate::router::handlers::parse::extra_db::DbFormat::Sqlite)
    );
}

#[test]
fn test_detect_empty_input() {
    assert_eq!(ParseHandler::detect_db_format(""), None);
    assert_eq!(ParseHandler::detect_db_format("   \n\n"), None);
}

// ============================================================
// DB Parser Tests — Psql Parsing
// ============================================================

#[test]
fn test_parse_psql_basic() {
    let input = " id | name    | email\n----+---------+-------------------\n  1 | Alice   | alice@example.com\n  2 | Bob     | bob@example.com\n(2 rows)\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name", "email"]);
    assert_eq!(result.row_count, 2);
    assert_eq!(result.rows[0], vec!["1", "Alice", "alice@example.com"]);
    assert_eq!(result.rows[1], vec!["2", "Bob", "bob@example.com"]);
    assert_eq!(result.footer, Some("(2 rows)".to_string()));
}

#[test]
fn test_parse_psql_single_row() {
    let input = " id | name\n----+------\n  1 | test\n(1 row)\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name"]);
    assert_eq!(result.row_count, 1);
    assert_eq!(result.footer, Some("(1 row)".to_string()));
}

// ============================================================
// DB Parser Tests — MySQL Parsing
// ============================================================

#[test]
fn test_parse_mysql_basic() {
    let input = "+----+---------+-------------------+\n| id | name    | email             |\n+----+---------+-------------------+\n|  1 | Alice   | alice@example.com |\n|  2 | Bob     | bob@example.com   |\n+----+---------+-------------------+\n2 rows in set (0.00 sec)\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name", "email"]);
    assert_eq!(result.row_count, 2);
    assert_eq!(result.rows[0], vec!["1", "Alice", "alice@example.com"]);
    assert_eq!(result.rows[1], vec!["2", "Bob", "bob@example.com"]);
    assert!(result.footer.is_some());
}

#[test]
fn test_parse_mysql_no_footer() {
    let input = "+----+------+\n| id | name |\n+----+------+\n|  1 | test |\n+----+------+\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name"]);
    assert_eq!(result.row_count, 1);
    assert_eq!(result.footer, None);
}

// ============================================================
// DB Parser Tests — SQLite Parsing
// ============================================================

#[test]
fn test_parse_sqlite_basic() {
    let input = "id|name|email\n1|Alice|alice@example.com\n2|Bob|bob@example.com\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name", "email"]);
    assert_eq!(result.row_count, 2);
    assert_eq!(result.rows[0], vec!["1", "Alice", "alice@example.com"]);
    assert_eq!(result.rows[1], vec!["2", "Bob", "bob@example.com"]);
    assert_eq!(result.footer, None);
}

#[test]
fn test_parse_sqlite_empty_values() {
    let input = "id|name|note\n1|Alice|\n2||test\n";
    let result = ParseHandler::parse_db(input);

    assert_eq!(result.columns, vec!["id", "name", "note"]);
    assert_eq!(result.row_count, 2);
    assert_eq!(result.rows[0], vec!["1", "Alice", ""]);
    assert_eq!(result.rows[1], vec!["2", "", "test"]);
}

// ============================================================
// DB Parser Tests — Truncation
// ============================================================

#[test]
fn test_compact_truncation_large_result() {
    // Build a psql-style output with 30 rows
    let mut input = String::from(" id | val\n----+-----\n");
    for i in 1..=30 {
        input.push_str(&format!("  {} | row{}\n", i, i));
    }
    input.push_str("(30 rows)\n");

    let result = ParseHandler::parse_db(&input);
    assert_eq!(result.row_count, 30);

    // Verify the parse is correct
    assert_eq!(result.rows.len(), 30);

    // Verify sample_rows logic directly
    let (sample, omitted) = ParseHandler::sample_rows(&result.rows);
    assert_eq!(sample.len(), 15); // 10 head + 5 tail
    assert_eq!(omitted, 15); // 30 - 10 - 5
}

#[test]
fn test_compact_no_truncation_small_result() {
    let input = " id | val\n----+-----\n  1 | a\n  2 | b\n  3 | c\n(3 rows)\n";
    let result = ParseHandler::parse_db(input);

    let (sample, omitted) = ParseHandler::sample_rows(&result.rows);
    assert_eq!(sample.len(), 3);
    assert_eq!(omitted, 0);
}

// ============================================================
// DB Parser Tests — Output Formats
// ============================================================

#[test]
fn test_json_output() {
    let input = "id|name\n1|Alice\n2|Bob\n";
    let result = ParseHandler::parse_db(input);
    let json_str = ParseHandler::format_db_json(&result);

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert_eq!(parsed["format"], "sqlite3");
    assert_eq!(parsed["row_count"], 2);
    assert_eq!(parsed["columns"][0], "id");
    assert_eq!(parsed["columns"][1], "name");
    assert_eq!(parsed["sample_rows"].as_array().map(|a| a.len()), Some(2));
}

#[test]
fn test_csv_output() {
    let input = "id|name\n1|Alice\n2|Bob\n";
    let result = ParseHandler::parse_db(input);
    let csv = ParseHandler::format_db_csv(&result);

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], "id,name");
    assert_eq!(lines[1], "1,Alice");
    assert_eq!(lines[2], "2,Bob");
}

#[test]
fn test_tsv_output() {
    let input = "id|name\n1|Alice\n2|Bob\n";
    let result = ParseHandler::parse_db(input);
    let tsv = ParseHandler::format_db_tsv(&result);

    let lines: Vec<&str> = tsv.lines().collect();
    assert_eq!(lines[0], "id\tname");
    assert_eq!(lines[1], "1\tAlice");
    assert_eq!(lines[2], "2\tBob");
}

#[test]
fn test_csv_escaping() {
    let input = "id|note\n1|hello, world\n2|say \"hi\"\n";
    let result = ParseHandler::parse_db(input);
    let csv = ParseHandler::format_db_csv(&result);

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], "id,note");
    assert_eq!(lines[1], "1,\"hello, world\"");
    assert_eq!(lines[2], "2,\"say \"\"hi\"\"\"");
}

// ============================================================
// DB Parser Tests — Classifier
// ============================================================

#[test]
fn test_classifier_psql() {
    let result = crate::classifier::classify_command("psql", &[]);
    assert!(matches!(result, Some(ParseCommands::Db { .. })));
}

#[test]
fn test_classifier_mysql() {
    let result = crate::classifier::classify_command("mysql", &[]);
    assert!(matches!(result, Some(ParseCommands::Db { .. })));
}

#[test]
fn test_classifier_sqlite3() {
    let result = crate::classifier::classify_command("sqlite3", &[]);
    assert!(matches!(result, Some(ParseCommands::Db { .. })));
}

#[test]
fn test_classifier_mariadb() {
    let result = crate::classifier::classify_command("mariadb", &[]);
    assert!(matches!(result, Some(ParseCommands::Db { .. })));
}
