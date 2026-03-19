use super::*;

// ============================================================
// Txt2md Handler Tests
// ============================================================

#[test]
fn test_txt2md_handler() {
    use std::io::Write;
    let handler = Txt2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    // Create a temp input file
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_txt2md_handler_input.txt");
    let mut file = std::fs::File::create(&input_path).unwrap();
    writeln!(file, "TITLE").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "Some text.").unwrap();
    drop(file);

    let input = Txt2mdInput {
        input: Some(input_path.clone()),
        output: None,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(&input_path);
}

// ============================================================
// Txt2md Normalize Spacing Tests
// ============================================================

#[test]
fn test_txt2md_normalize_spacing_collapses_blank_lines() {
    let handler = Txt2mdHandler;
    // Input with multiple consecutive blank lines
    let input = "Line 1\n\n\n\nLine 2\n\n\nLine 3";
    let result = handler.normalize_spacing(input);
    // Should have only single blank lines between content
    assert_eq!(result, "Line 1\n\nLine 2\n\nLine 3");
}

#[test]
fn test_txt2md_normalize_spacing_trims_trailing_whitespace() {
    let handler = Txt2mdHandler;
    // Input with trailing whitespace on lines
    let input = "Line 1   \nLine 2\t\t\nLine 3   ";
    let result = handler.normalize_spacing(input);
    // Should have no trailing whitespace
    assert_eq!(result, "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_txt2md_normalize_spacing_removes_leading_blank_lines() {
    let handler = Txt2mdHandler;
    // Input with leading blank lines
    let input = "\n\n\nLine 1\nLine 2";
    let result = handler.normalize_spacing(input);
    // Should have no leading blank lines
    assert_eq!(result, "Line 1\nLine 2");
}

#[test]
fn test_txt2md_normalize_spacing_removes_trailing_blank_lines() {
    let handler = Txt2mdHandler;
    // Input with trailing blank lines
    let input = "Line 1\nLine 2\n\n\n";
    let result = handler.normalize_spacing(input);
    // Should have no trailing blank lines
    assert_eq!(result, "Line 1\nLine 2");
}

#[test]
fn test_txt2md_normalize_spacing_empty_input() {
    let handler = Txt2mdHandler;
    let result = handler.normalize_spacing("");
    assert_eq!(result, "");
}

#[test]
fn test_txt2md_normalize_spacing_only_whitespace() {
    let handler = Txt2mdHandler;
    let result = handler.normalize_spacing("   \n\t\n   ");
    assert_eq!(result, "");
}

#[test]
fn test_txt2md_normalize_spacing_single_line() {
    let handler = Txt2mdHandler;
    let result = handler.normalize_spacing("Single line");
    assert_eq!(result, "Single line");
}

#[test]
fn test_txt2md_normalize_spacing_preserves_internal_spacing() {
    let handler = Txt2mdHandler;
    // Internal spacing (between words) should be preserved
    let input = "Line with   multiple   spaces";
    let result = handler.normalize_spacing(input);
    assert_eq!(result, "Line with   multiple   spaces");
}

#[test]
fn test_txt2md_normalize_spacing_complex() {
    let handler = Txt2mdHandler;
    // Complex case with multiple issues
    let input = "\n\n# Heading   \n\n\n\nParagraph text.   \n\n- List item 1   \n\n\n- List item 2\n\n\n";
    let result = handler.normalize_spacing(input);
    // Should normalize all spacing issues
    assert_eq!(
        result,
        "# Heading\n\nParagraph text.\n\n- List item 1\n\n- List item 2"
    );
}

#[test]
fn test_txt2md_normalize_spacing_with_code_block() {
    let handler = Txt2mdHandler;
    // Code blocks should have trailing whitespace trimmed but internal structure preserved
    let input = "```   \ncode line   \n```   ";
    let result = handler.normalize_spacing(input);
    assert_eq!(result, "```\ncode line\n```");
}

#[test]
fn test_txt2md_handler_with_spacing_issues() {
    use std::io::Write;
    let handler = Txt2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    // Create a temp input file with spacing issues
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_txt2md_spacing_input.txt");
    let mut file = std::fs::File::create(&input_path).unwrap();
    writeln!(file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "TITLE").unwrap();
    writeln!(file).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "Some text.").unwrap();
    writeln!(file).unwrap();
    writeln!(file).unwrap();
    drop(file);

    let input = Txt2mdInput {
        input: Some(input_path.clone()),
        output: None,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(&input_path);
}
