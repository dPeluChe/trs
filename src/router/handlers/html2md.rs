use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::OutputFormat;

pub(crate) struct Html2mdHandler;

impl Html2mdHandler {
    /// Check if the input is a URL.
    pub(crate) fn is_url(input: &str) -> bool {
        input.starts_with("http://") || input.starts_with("https://")
    }

    /// Fetch HTML content from a URL.
    pub(crate) fn fetch_url(&self, url: &str) -> CommandResult<String> {
        use std::io::Read;
        let response = ureq::get(url)
            .call()
            .map_err(|e| CommandError::IoError(format!("Failed to fetch URL '{}': {}", url, e)))?;

        let mut html = String::new();
        response
            .into_body()
            .into_reader()
            .read_to_string(&mut html)
            .map_err(|e| CommandError::IoError(format!("Failed to read response: {}", e)))?;

        Ok(html)
    }

    /// Read HTML content from a file.
    pub(crate) fn read_file(&self, path: &str) -> CommandResult<String> {
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            return Err(CommandError::IoError(format!("File not found: {}", path)));
        }
        std::fs::read_to_string(&path_buf)
            .map_err(|e| CommandError::IoError(format!("Failed to read file '{}': {}", path, e)))
    }

    /// Extract metadata from HTML content.
    pub(crate) fn extract_metadata(&self, html: &str, url_or_file: &str) -> serde_json::Value {
        let mut metadata = serde_json::json!({
            "source": url_or_file,
        });

        // Extract title from <title> tag
        if let Some(title_start) = html.find("<title") {
            if let Some(content_start) = html[title_start..].find('>') {
                let content_start = title_start + content_start + 1;
                if let Some(content_end) = html[content_start..].find("</title>") {
                    let title = &html[content_start..content_start + content_end];
                    metadata["title"] = serde_json::json!(title.trim());
                }
            }
        }

        // Extract meta description
        if let Some(meta_start) = html.find("meta name=\"description\"") {
            let meta_slice = &html[meta_start..];
            if let Some(content_start) = meta_slice.find("content=\"") {
                let content_start = content_start + 9;
                if let Some(content_end) = meta_slice[content_start..].find('"') {
                    let description = &meta_slice[content_start..content_start + content_end];
                    metadata["description"] = serde_json::json!(description);
                }
            }
        }

        // Check if source is URL or file
        if Self::is_url(url_or_file) {
            metadata["type"] = serde_json::json!("url");
        } else {
            metadata["type"] = serde_json::json!("file");
        }

        metadata
    }

    /// HTML tags considered as "noise" that should be skipped during conversion.
    const NOISE_TAGS: &'static [&'static str] = &[
        "script", "style", "noscript", "nav", "header", "footer", "aside", "form", "iframe", "svg",
    ];

    /// Convert HTML to Markdown, filtering out noise elements.
    pub(crate) fn convert_to_markdown(&self, html: &str) -> CommandResult<String> {
        let converter = htmd::HtmlToMarkdownBuilder::new()
            .skip_tags(Self::NOISE_TAGS.to_vec())
            .build();
        converter
            .convert(html)
            .map_err(|e| CommandError::ExecutionError {
                message: format!("Failed to convert HTML to Markdown: {}", e),
                exit_code: Some(1),
            })
    }

    /// Format output based on the output format.
    pub(crate) fn format_output(
        &self,
        markdown: &str,
        metadata: Option<&serde_json::Value>,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => {
                let mut result = serde_json::json!({
                    "markdown": markdown,
                });
                if let Some(meta) = metadata {
                    result["metadata"] = meta.clone();
                }
                format!("{}\n", serde_json::to_string_pretty(&result).unwrap())
            }
            OutputFormat::Compact | OutputFormat::Agent => {
                let mut output = markdown.to_string();
                if let Some(meta) = metadata {
                    output = format!(
                        "---\n{}\n---\n\n{}",
                        serde_json::to_string_pretty(meta).unwrap(),
                        output
                    );
                }
                format!("{}\n", output)
            }
            OutputFormat::Raw | OutputFormat::Csv | OutputFormat::Tsv => {
                format!("{}\n", markdown)
            }
        }
    }
}

impl CommandHandler for Html2mdHandler {
    type Input = Html2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read HTML content from URL or file
        let html = if Self::is_url(&input.input) {
            self.fetch_url(&input.input)?
        } else {
            self.read_file(&input.input)?
        };

        // Extract metadata if requested via --metadata flag OR when using JSON output
        // JSON output always includes metadata for structured data
        let metadata = if input.metadata || ctx.format == OutputFormat::Json {
            Some(self.extract_metadata(&html, &input.input))
        } else {
            None
        };

        // Convert HTML to Markdown
        let markdown = self.convert_to_markdown(&html)?;

        // Format output
        let formatted = self.format_output(&markdown, metadata.as_ref(), ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("html2md")
                .with_output_mode(ctx.format)
                .with_input_bytes(html.len())
                .with_output_bytes(formatted.len())
                .with_extra(
                    "Source type",
                    if Self::is_url(&input.input) {
                        "url"
                    } else {
                        "file"
                    },
                );
            stats.print();
        }

        // Write to output file or stdout
        if let Some(ref output_path) = input.output {
            std::fs::write(output_path, &formatted).map_err(|e| {
                CommandError::IoError(format!(
                    "Failed to write output file '{}': {}",
                    output_path.display(),
                    e
                ))
            })?;
        } else {
            print!("{}", formatted);
        }

        Ok(())
    }
}

/// Input data for the `html2md` command.
#[derive(Debug, Clone)]
pub(crate) struct Html2mdInput {
    pub input: String,
    pub output: Option<std::path::PathBuf>,
    pub metadata: bool,
}

