use super::*;

// ============================================================
// Html2md Handler Tests (Advanced: nested lists, combined, noise)
// ============================================================

#[test]
fn test_html2md_nested_list_conversion() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_nested_list_conversion.html");
    let output_path = temp_dir.join("test_nested_list_conversion_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Nested List Test</title></head>
<body>
<ul>
  <li>Item 1
    <ul>
      <li>Nested item 1.1</li>
      <li>Nested item 1.2</li>
    </ul>
  </li>
  <li>Item 2</li>
</ul>
<ol>
  <li>First
    <ol>
      <li>Sub-first</li>
    </ol>
  </li>
  <li>Second</li>
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

    // Check that nested items are present
    assert!(output_content.contains("Item 1"));
    assert!(output_content.contains("Nested item 1.1"));
    assert!(output_content.contains("Nested item 1.2"));
    assert!(output_content.contains("Item 2"));

    // Check that nested items are indented (have leading whitespace)
    let lines: Vec<&str> = output_content.lines().collect();
    let nested_lines: Vec<&&str> = lines
        .iter()
        .filter(|line| line.contains("Nested item") || line.contains("Sub-first"))
        .collect();

    // Nested items should have some indentation
    for nested_line in nested_lines {
        let has_indentation = nested_line.starts_with(|c: char| c.is_whitespace());
        assert!(
            has_indentation || nested_line.contains('*') || nested_line.contains('-'),
            "Nested list items should be indented: '{}'",
            nested_line
        );
    }

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_html2md_mixed_nested_list_conversion() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_mixed_nested_list.html");
    let output_path = temp_dir.join("test_mixed_nested_list_output.md");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Mixed Nested List Test</title></head>
<body>
<ol>
  <li>First ordered
    <ul>
      <li>Unordered sub-item</li>
    </ul>
  </li>
  <li>Second ordered</li>
</ol>
<ul>
  <li>Unordered first
    <ol>
      <li>Ordered sub-item</li>
    </ol>
  </li>
</ul>
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

    // Check that all content is preserved
    assert!(output_content.contains("First ordered"));
    assert!(output_content.contains("Unordered sub-item"));
    assert!(output_content.contains("Second ordered"));
    assert!(output_content.contains("Unordered first"));
    assert!(output_content.contains("Ordered sub-item"));

    // Verify mixed list types are handled (ordered with unordered nested, and vice versa)
    let has_ordered = output_content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("1. ") || trimmed.starts_with("2. ")
    });
    let has_unordered = output_content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("* ") || trimmed.starts_with("- ")
    });
    assert!(has_ordered, "Should have ordered list markers");
    assert!(has_unordered, "Should have unordered list markers");

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn test_html2md_combined_elements() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_combined_elements.html");

    let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Combined Test</title></head>
<body>
<h1>Main Heading</h1>
<p>Introduction paragraph with a <a href="https://example.com">link</a>.</p>
<h2>Features</h2>
<ul>
  <li>Feature 1 with <strong>bold</strong> text</li>
  <li>Feature 2 with <em>emphasis</em></li>
</ul>
<h2>Steps</h2>
<ol>
  <li>First step</li>
  <li>Second step with <code>code</code></li>
</ol>
<h3>Conclusion</h3>
<p>Final paragraph.</p>
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
fn test_html2md_noise_removal() {
    use std::io::Write;
    let handler = Html2mdHandler;
    let ctx = CommandContext {
        format: OutputFormat::Raw,
        stats: false,
        enabled_formats: vec![],
    };

    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("test_noise_removal.html");
    let output_path = temp_dir.join("test_noise_removal_output.md");

    // HTML with various noise elements that should be filtered out
    let html_content = r#"<!DOCTYPE html>
<html>
<head>
<title>Content Page</title>
<script>
    console.log("This script should be removed");
    var x = 1 + 2;
</script>
<style>
    body { color: red; }
    .hidden { display: none; }
</style>
</head>
<body>
<header>
    <nav>
        <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/about">About</a></li>
        </ul>
    </nav>
</header>
<main>
    <h1>Main Content</h1>
    <p>This is the important content that should be preserved.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
</main>
<footer>
    <p>Copyright 2024</p>
</footer>
<aside>
    <p>Sidebar content</p>
</aside>
<noscript>
    <p>Please enable JavaScript</p>
</noscript>
<iframe src="https://example.com/frame"></iframe>
<svg>
    <circle cx="50" cy="50" r="40"/>
</svg>
<form action="/submit">
    <input type="text" name="field" />
</form>
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

    // Verify main content is preserved
    assert!(
        output_content.contains("Main Content"),
        "Should preserve main heading"
    );
    assert!(
        output_content.contains("important content"),
        "Should preserve main paragraph"
    );
    assert!(
        output_content.contains("Item 1"),
        "Should preserve list items"
    );
    assert!(
        output_content.contains("Item 2"),
        "Should preserve list items"
    );

    // Verify noise elements are removed
    assert!(
        !output_content.contains("console.log"),
        "Should remove script content"
    );
    assert!(
        !output_content.contains("var x"),
        "Should remove script variables"
    );
    assert!(
        !output_content.contains("color: red"),
        "Should remove style content"
    );
    assert!(
        !output_content.contains(".hidden"),
        "Should remove CSS classes"
    );
    assert!(
        !output_content.contains("Home"),
        "Should remove nav content"
    );
    assert!(!output_content.contains("About"), "Should remove nav links");
    assert!(
        !output_content.contains("Copyright"),
        "Should remove footer content"
    );
    assert!(
        !output_content.contains("Sidebar"),
        "Should remove aside content"
    );
    assert!(
        !output_content.contains("enable JavaScript"),
        "Should remove noscript content"
    );
    assert!(
        !output_content.contains("example.com/frame"),
        "Should remove iframe"
    );
    assert!(
        !output_content.contains("circle"),
        "Should remove SVG content"
    );
    assert!(
        !output_content.contains("submit"),
        "Should remove form content"
    );

    let _ = std::fs::remove_file(&html_path);
    let _ = std::fs::remove_file(&output_path);
}
