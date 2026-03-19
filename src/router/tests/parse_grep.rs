use super::*;

// ============================================================
// Grep Parser Tests
// ============================================================

#[test]
fn test_parse_grep_empty() {
    let result = ParseHandler::parse_grep("").unwrap();
    assert!(result.is_empty);
    assert_eq!(result.file_count, 0);
    assert_eq!(result.match_count, 0);
}

#[test]
fn test_parse_grep_single_file_single_match() {
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert!(!result.is_empty);
    assert_eq!(result.file_count, 1);
    assert_eq!(result.match_count, 1);
    assert_eq!(result.files[0].path, "src/main.rs");
    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(result.files[0].matches[0].line, "fn main() {");
}

#[test]
fn test_parse_grep_single_file_multiple_matches() {
    let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.file_count, 1);
    assert_eq!(result.match_count, 2);
    assert_eq!(result.files[0].matches.len(), 2);
    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(result.files[0].matches[1].line_number, Some(45));
}

#[test]
fn test_parse_grep_multiple_files() {
    let input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.file_count, 2);
    assert_eq!(result.match_count, 2);
    assert_eq!(result.files[0].path, "src/main.rs");
    assert_eq!(result.files[1].path, "src/lib.rs");
}

#[test]
fn test_parse_grep_groups_interleaved_files() {
    // Test that matches from the same file are grouped together
    // even when they appear interleaved in the input
    let input = "src/main.rs:10:line one\nsrc/lib.rs:25:line two\nsrc/main.rs:30:line three";
    let result = ParseHandler::parse_grep(input).unwrap();

    // Should have 2 files, not 3
    assert_eq!(result.file_count, 2);
    assert_eq!(result.match_count, 3);

    // Files should preserve order of first appearance
    assert_eq!(result.files[0].path, "src/main.rs");
    assert_eq!(result.files[1].path, "src/lib.rs");

    // main.rs should have both its matches grouped together
    assert_eq!(result.files[0].matches.len(), 2);
    assert_eq!(result.files[0].matches[0].line_number, Some(10));
    assert_eq!(result.files[0].matches[0].line, "line one");
    assert_eq!(result.files[0].matches[1].line_number, Some(30));
    assert_eq!(result.files[0].matches[1].line, "line three");

    // lib.rs should have its single match
    assert_eq!(result.files[1].matches.len(), 1);
    assert_eq!(result.files[1].matches[0].line_number, Some(25));
    assert_eq!(result.files[1].matches[0].line, "line two");
}

#[test]
fn test_parse_grep_with_column() {
    let input = "src/main.rs:42:10:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(result.files[0].matches[0].column, Some(10));
    assert_eq!(result.files[0].matches[0].line, "fn main() {");
}

#[test]
fn test_parse_grep_without_line_number() {
    let input = "src/main.rs:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches[0].line_number, None);
    assert_eq!(result.files[0].matches[0].line, "fn main() {");
}

#[test]
fn test_parse_grep_binary_file() {
    let input = "Binary file target/debug binary matches";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.file_count, 1);
    assert_eq!(result.files[0].path, "target/debug binary");
    assert_eq!(result.files[0].matches[0].line, "[binary file]");
}

#[test]
fn test_parse_grep_format_compact() {
    let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:    println!";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    // Single-file results omit header/footer for compactness
    assert!(!output.contains("matches:"), "single-file should not have header");
    assert!(output.contains("src/main.rs (2):"));
    assert!(output.contains("42: fn main() {"));
    assert!(output.contains("45:     println!"));
}

#[test]
fn test_parse_grep_format_json() {
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Json);

    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["schema"]["type"], "grep_output");
    assert_eq!(json["counts"]["files"], 1);
    assert_eq!(json["counts"]["matches"], 1);
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["matches"][0]["line_number"], 42);
    assert_eq!(json["files"][0]["matches"][0]["line"], "fn main() {");
}

#[test]
fn test_parse_grep_format_csv() {
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Csv);

    assert!(output.starts_with("path,line_number,column,is_context,line\n"));
    assert!(output.contains("src/main.rs,42,,false,fn main() {"));
}

#[test]
fn test_parse_grep_format_tsv() {
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Tsv);

    assert!(output.starts_with("path\tline_number\tcolumn\tis_context\tline\n"));
    assert!(output.contains("src/main.rs\t42\t\tfalse\tfn main() {"));
}

#[test]
fn test_parse_grep_format_raw() {
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

    assert!(output.contains("src/main.rs:42:fn main() {"));
}

#[test]
fn test_parse_grep_empty_compact() {
    let mut result = GrepOutput::default();
    result.is_empty = true;
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    assert!(output.contains("grep: no matches"));
}

#[test]
fn test_parse_grep_line_with_colon_in_content() {
    // Content containing colons should be handled correctly
    let input = "src/main.rs:42:let x = \"http://example.com\";";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(
        result.files[0].matches[0].line,
        "let x = \"http://example.com\";"
    );
}

// ============================================================
// Context Line Tests
// ============================================================

#[test]
fn test_parse_grep_context_line() {
    // Context lines use "-" as separator (from grep -C/-B/-A)
    let input = "src/main.rs-42-context line";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(result.files[0].matches[0].line, "context line");
    assert!(result.files[0].matches[0].is_context);
}

#[test]
fn test_parse_grep_context_line_with_column() {
    // Context line with column info
    let input = "src/main.rs-42-10-context line";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches[0].line_number, Some(42));
    assert_eq!(result.files[0].matches[0].column, Some(10));
    assert_eq!(result.files[0].matches[0].line, "context line");
    assert!(result.files[0].matches[0].is_context);
}

#[test]
fn test_parse_grep_mixed_match_and_context() {
    // Mix of match and context lines
    let input = "src/main.rs-41-context before\nsrc/main.rs:42:match line\nsrc/main.rs-43-context after";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.files[0].matches.len(), 3);

    // First line is context
    assert!(result.files[0].matches[0].is_context);
    assert_eq!(result.files[0].matches[0].line, "context before");

    // Second line is a match
    assert!(!result.files[0].matches[1].is_context);
    assert_eq!(result.files[0].matches[1].line, "match line");

    // Third line is context
    assert!(result.files[0].matches[2].is_context);
    assert_eq!(result.files[0].matches[2].line, "context after");
}

#[test]
fn test_parse_grep_context_is_context_flag_false_for_matches() {
    let input = "src/main.rs:42:match line";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert!(!result.files[0].matches[0].is_context);
}

#[test]
fn test_format_grep_compact_collapse_context_lines() {
    // Multiple consecutive context lines should be collapsed
    let input = "src/main.rs-10-context 1\nsrc/main.rs-11-context 2\nsrc/main.rs-12-context 3\nsrc/main.rs:13:match line";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    // Should collapse 3 context lines into a summary
    assert!(output.contains("10-12: ... (3 context lines)"));
    assert!(output.contains("13: match line"));
}

#[test]
fn test_format_grep_compact_single_context_line() {
    // Single context line should show as "... (1 context lines)" format
    let input = "src/main.rs-10-context line\nsrc/main.rs:11:match line";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    assert!(output.contains("10: ..."));
    assert!(output.contains("11: match line"));
}

#[test]
fn test_format_grep_compact_context_before_and_after() {
    // Context lines before and after match
    let input = "src/main.rs-10-before\nsrc/main.rs:11:match\nsrc/main.rs-12-after";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    assert!(output.contains("10: ..."));
    assert!(output.contains("11: match"));
    assert!(output.contains("12: ..."));
}

#[test]
fn test_format_grep_compact_count_excludes_context() {
    // Match count should exclude context lines
    let input = "src/main.rs-10-context\nsrc/main.rs:11:match\nsrc/main.rs-12-context";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    // Single-file: no header, but file line shows count (1) not (3)
    assert!(output.contains("src/main.rs (1):"));
}

#[test]
fn test_format_grep_compact_trailing_context() {
    // Context lines at the end should be collapsed
    let input = "src/main.rs:10:match\nsrc/main.rs-11-context 1\nsrc/main.rs-12-context 2";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    assert!(output.contains("10: match"));
    assert!(output.contains("11-12: ... (2 context lines)"));
}

#[test]
fn test_format_grep_json_includes_is_context() {
    let input = "src/main.rs-10-context\nsrc/main.rs:11:match";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Json);

    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["files"][0]["matches"][0]["is_context"], true);
    assert_eq!(json["files"][0]["matches"][1]["is_context"], false);
}

#[test]
fn test_format_grep_raw_context_uses_dash() {
    // Raw format should preserve dash separator for context
    let input = "src/main.rs-10-context line";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

    assert!(output.contains("src/main.rs-10-context line"));
}

#[test]
fn test_format_grep_raw_match_uses_colon() {
    // Raw format should use colon for matches
    let input = "src/main.rs:10:match line";
    let result = ParseHandler::parse_grep(input).unwrap();
    let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

    assert!(output.contains("src/main.rs:10:match line"));
}

// ============================================================
// Grep Truncation Tests
// ============================================================

#[test]
fn test_parse_grep_truncation_fields_not_truncated() {
    // Small result set should not be truncated
    let input = "src/main.rs:42:fn main() {";
    let result = ParseHandler::parse_grep(input).unwrap();

    assert_eq!(result.is_truncated, false);
    assert_eq!(result.total_files, 1);
    assert_eq!(result.total_matches, 1);
    assert_eq!(result.files_shown, 1);
    assert_eq!(result.matches_shown, 1);
}

#[test]
fn test_truncate_grep_files() {
    // Create 60 files (exceeds DEFAULT_MAX_GREP_FILES = 50)
    let mut input = String::new();
    for i in 1..=60 {
        input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();

    // Before truncation
    assert_eq!(result.total_files, 60);
    assert_eq!(result.files.len(), 60);

    // Apply truncation
    ParseHandler::truncate_grep(&mut result, 50, 20);

    // After truncation
    assert_eq!(result.is_truncated, true);
    assert_eq!(result.files_shown, 50);
    assert_eq!(result.total_files, 60);
    assert_eq!(result.files.len(), 50);
}

#[test]
fn test_truncate_grep_matches_per_file() {
    // Create 1 file with 25 matches (exceeds DEFAULT_MAX_GREP_MATCHES_PER_FILE = 20)
    let mut input = String::new();
    for i in 1..=25 {
        input.push_str(&format!("src/main.rs:{}:fn func{}() {{\n", i, i));
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();

    // Before truncation
    assert_eq!(result.total_matches, 25);
    assert_eq!(result.files[0].matches.len(), 25);

    // Apply truncation
    ParseHandler::truncate_grep(&mut result, 50, 20);

    // After truncation
    assert_eq!(result.is_truncated, true);
    assert_eq!(result.matches_shown, 20);
    assert_eq!(result.total_matches, 25);
    assert_eq!(result.files[0].matches.len(), 20);
}

#[test]
fn test_truncate_grep_both_limits() {
    // Create 60 files, each with 25 matches
    let mut input = String::new();
    for i in 1..=60 {
        for j in 1..=25 {
            input.push_str(&format!("src/file{}.rs:{}:fn func{}() {{\n", i, j, j));
        }
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();

    // Before truncation: 60 files * 25 matches = 1500 total matches
    assert_eq!(result.total_files, 60);
    assert_eq!(result.total_matches, 1500);

    // Apply truncation
    ParseHandler::truncate_grep(&mut result, 50, 20);

    // After truncation: 50 files * 20 matches = 1000 matches shown
    assert_eq!(result.is_truncated, true);
    assert_eq!(result.files_shown, 50);
    assert_eq!(result.matches_shown, 1000);
    assert_eq!(result.files.len(), 50);
}

#[test]
fn test_format_grep_json_truncation_info() {
    // Create 60 files to trigger truncation
    let mut input = String::new();
    for i in 1..=60 {
        input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();
    ParseHandler::truncate_grep(&mut result, 50, 20);

    let output = ParseHandler::format_grep(&result, OutputFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["counts"]["total_files"], 60);
    assert_eq!(json["counts"]["files_shown"], 50);
}

#[test]
fn test_format_grep_compact_truncation_info() {
    // Create 60 files to trigger truncation
    let mut input = String::new();
    for i in 1..=60 {
        input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();
    ParseHandler::truncate_grep(&mut result, 50, 20);

    let output = ParseHandler::format_grep(&result, OutputFormat::Compact);

    // Check for truncation indicators in compact output
    assert!(output.contains("truncated"));
    assert!(output.contains("50/60"));
    assert!(output.contains("10 more file"));
}

#[test]
fn test_format_grep_raw_truncation_info() {
    // Create 60 files to trigger truncation
    let mut input = String::new();
    for i in 1..=60 {
        input.push_str(&format!("src/file{}.rs:{}:fn func() {{\n", i, i));
    }
    let mut result = ParseHandler::parse_grep(&input).unwrap();
    ParseHandler::truncate_grep(&mut result, 50, 20);

    let output = ParseHandler::format_grep(&result, OutputFormat::Raw);

    // Check for truncation indicator in raw output
    assert!(output.contains("10 more file"));
}

#[test]
fn test_format_grep_json_no_truncation_when_within_limits() {
    // Small result set should not show truncation info
    let input = "src/main.rs:42:fn main() {\nsrc/main.rs:45:println!()";
    let mut result = ParseHandler::parse_grep(input).unwrap();
    ParseHandler::truncate_grep(&mut result, 50, 20);

    let output = ParseHandler::format_grep(&result, OutputFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(json["is_truncated"], false);
    assert!(json["truncation"].is_null());
}
