use super::*;

// ============================================================
// Html2md Handler Tests (Basic)
// ============================================================

#[test]
fn test_html2md_handler() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    // Create a temporary HTML file for testing
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_input.html");
    let output_path = temp_dir.join("test_html2md_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<h1>Hello World</h1>
<p>This is a <strong>test</strong> paragraph.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: Some(output_path.clone()),
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    // Verify output file was created and contains markdown
    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("Hello World"));
    assert!(output_content.contains("test"));

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_html2md_handler_with_metadata() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    // Create a temporary HTML file for testing
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_meta_input.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Title</title>
<meta name="description" content="Test description">
</head>
<body>
<h1>Heading</h1>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: None,
        metadata: true,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_handler_json_includes_metadata_automatically() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    // Create a temporary HTML file for testing
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_html2md_auto_meta.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Auto Metadata Title</title>
<meta name="description" content="Auto metadata description">
</head>
<body>
<h1>Content</h1>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    // Test with metadata=false but JSON output should still include metadata
    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: None,
        metadata: false, // Explicitly set to false
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    // Cleanup
    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_is_url() {
    assert!(Html2mdHandler::is_url("http://example.com"));
    assert!(Html2mdHandler::is_url("https://example.com"));
    assert!(!Html2mdHandler::is_url("/path/to/file.html"));
    assert!(!Html2mdHandler::is_url("file.html"));
}

#[test]
fn test_html2md_file_not_found() {
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let input = Html2mdInput {
        input: "/nonexistent/path/to/file.html".to_string(),
        output: None,
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(matches!(result, Err(CommandError::IoError(_))));
}

#[test]
fn test_html2md_heading_conversion() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_heading_conversion.html");
    let output_path = temp_dir.join("test_heading_conversion_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Test</title></head>
<body>
<h1>Heading 1</h1>
<h2>Heading 2</h2>
<h3>Heading 3</h3>
<h4>Heading 4</h4>
<h5>Heading 5</h5>
<h6>Heading 6</h6>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: Some(output_path.clone()),
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("# Heading 1"));
    assert!(output_content.contains("## Heading 2"));
    assert!(output_content.contains("### Heading 3"));
    assert!(output_content.contains("#### Heading 4"));
    assert!(output_content.contains("##### Heading 5"));
    assert!(output_content.contains("###### Heading 6"));

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_html2md_heading_with_inline_elements() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_heading_inline.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Heading Inline Test</title></head>
<body>
<h1>Heading with <em>emphasis</em></h1>
<h2>Heading with <strong>bold</strong></h2>
<h3>Heading with <code>code</code></h3>
<h4>Heading with <a href="https://example.com">link</a></h4>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: None,
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    let _ = std::fs::remove_file(&html_path);
}

#[test]
fn test_html2md_link_conversion() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_link_conversion.html");
    let output_path = temp_dir.join("test_link_conversion_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Link Test</title></head>
<body>
<p>Visit <a href="https://example.com">Example</a> for more info.</p>
<p>Check <a href="https://rust-lang.org">Rust</a> language.</p>
<p><a href="/relative/path">Relative link</a> works too.</p>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: Some(output_path.clone()),
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("[Example](https://example.com)"));
    assert!(output_content.contains("[Rust](https://rust-lang.org)"));
    assert!(output_content.contains("[Relative link](/relative/path)"));

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_html2md_list_conversion() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_list_conversion.html");
    let output_path = temp_dir.join("test_list_conversion_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>List Test</title></head>
<body>
<ul>
  <li>Unordered item 1</li>
  <li>Unordered item 2</li>
  <li>Unordered item 3</li>
</ul>
<ol>
  <li>Ordered item 1</li>
  <li>Ordered item 2</li>
  <li>Ordered item 3</li>
</ol>
</body>
</html>"#;

    let mut file = std::fs::File::create(&html_path).unwrap();
    file.write_all(html_content.as_bytes()).unwrap();
    drop(file);

    let input = Html2mdInput {
        input: html_path.to_string_lossy().to_string(),
        output: Some(output_path.clone()),
        metadata: false,
    };

    let result = handler.execute(&input, &ctx);
    assert!(result.is_ok());

    let output_content = std::fs::read_to_string(&output_path).unwrap();

    // Check for unordered list markdown formatting (asterisk or dash prefix)
    // The output should contain list markers like "*   " or "-   " or "* " or "- "
    let has_unordered_markers = output_content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("* ")
            || trimmed.starts_with("- ")
            || trimmed.starts_with("*   ")
            || trimmed.starts_with("-   ")
    });
    assert!(
        has_unordered_markers,
        "Output should contain unordered list markers (* or -)"
    );

    // Check for ordered list markdown formatting (number followed by period)
    let has_ordered_markers = output_content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("1. ") || trimmed.starts_with("2. ") || trimmed.starts_with("3. ")
    });
    assert!(
        has_ordered_markers,
        "Output should contain ordered list markers (1., 2., 3.)"
    );

    // Check that the actual content is preserved
    assert!(output_content.contains("Unordered item 1"));
    assert!(output_content.contains("Unordered item 2"));
    assert!(output_content.contains("Unordered item 3"));
    assert!(output_content.contains("Ordered item 1"));
    assert!(output_content.contains("Ordered item 2"));
    assert!(output_content.contains("Ordered item 3"));

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}
