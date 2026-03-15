use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::OutputFormat;

pub(crate) struct Txt2mdInput {
    pub input: Option<std::path::PathBuf>,
    pub output: Option<std::path::PathBuf>,
}

/// Handler for the `txt2md` command.
pub(crate) struct Txt2mdHandler;

impl Txt2mdHandler {
    /// Read text content from a file or stdin.
    pub(crate) fn read_input(&self, input: &Option<std::path::PathBuf>) -> CommandResult<String> {
        if let Some(ref path) = input {
            if !path.exists() {
                return Err(CommandError::IoError(format!(
                    "File not found: {}",
                    path.display()
                )));
            }
            std::fs::read_to_string(path).map_err(|e| {
                CommandError::IoError(format!("Failed to read file '{}': {}", path.display(), e))
            })
        } else {
            use std::io::{self, Read};
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| CommandError::IoError(format!("Failed to read stdin: {}", e)))?;
            Ok(buffer)
        }
    }

    /// Convert plain text to Markdown.
    pub(crate) fn convert_to_markdown(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;
        let mut in_code_block = false;
        let mut in_list = false;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Handle empty lines
            if trimmed.is_empty() {
                // Close list if we were in one
                if in_list {
                    in_list = false;
                }
                result.push(String::new());
                i += 1;
                continue;
            }

            // Check for code block markers (triple backticks or lines with only code-like content)
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                result.push(trimmed.to_string());
                i += 1;
                continue;
            }

            // If we're in a code block, preserve the line as-is
            if in_code_block {
                result.push(line.to_string());
                i += 1;
                continue;
            }

            // Check for indented code (4 spaces or tab)
            if line.starts_with("    ") || line.starts_with('\t') {
                result.push(format!("    {}", trimmed));
                i += 1;
                continue;
            }

            // Detect unordered list items (- or * prefix, possibly nested)
            // Must be checked BEFORE heading detection to avoid misidentifying list items as headings
            if let Some((list_char, indent_level)) = Self::is_unordered_list_item_with_indent(line)
            {
                in_list = true;
                let rest = Self::strip_list_prefix(trimmed, list_char);
                let indent = "  ".repeat(indent_level);
                result.push(format!("{}- {}", indent, Self::format_inline(rest)));

                // Check for continuation lines
                i += 1;
                while i < lines.len() && Self::is_list_continuation(lines[i]) {
                    let cont_trimmed = lines[i].trim();
                    let cont_indent = "  ".repeat(indent_level + 1);
                    result.push(format!(
                        "{}{}",
                        cont_indent,
                        Self::format_inline(cont_trimmed)
                    ));
                    i += 1;
                }
                continue;
            }

            // Detect ordered list items (1., 2., etc., possibly nested)
            // Must be checked BEFORE heading detection to avoid misidentifying list items as headings
            if let Some((number, indent_level)) = Self::is_ordered_list_item_with_indent(line) {
                in_list = true;
                let rest = Self::strip_ordered_prefix(trimmed);
                let indent = "  ".repeat(indent_level);
                result.push(format!(
                    "{}{}. {}",
                    indent,
                    number,
                    Self::format_inline(rest)
                ));

                // Check for continuation lines
                i += 1;
                while i < lines.len() && Self::is_list_continuation(lines[i]) {
                    let cont_trimmed = lines[i].trim();
                    let cont_indent = "  ".repeat(indent_level + 1);
                    result.push(format!(
                        "{}{}",
                        cont_indent,
                        Self::format_inline(cont_trimmed)
                    ));
                    i += 1;
                }
                continue;
            }

            // Detect heading patterns
            // Pattern 1: Line looks like a heading (ALL CAPS, title case, or simple patterns)
            // Check if this is a single-word section heading (preceded by empty line or at start)
            let prev_empty = i == 0 || lines[i - 1].trim().is_empty();
            if Self::is_heading_line(trimmed)
                || (prev_empty && Self::is_single_word_section_heading(trimmed, i, &lines))
            {
                // Close list if we were in one
                if in_list {
                    in_list = false;
                }
                let level = Self::determine_heading_level(trimmed, i, &lines);
                let heading_text = Self::format_heading_text(trimmed);
                result.push(format!("{} {}", "#".repeat(level), heading_text));
                i += 1;
                continue;
            }

            // Pattern 2: Underlined headings (with === or ---)
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if next_line.chars().all(|c| c == '=') && next_line.len() >= 3 {
                    result.push(format!("# {}", trimmed));
                    i += 2;
                    continue;
                }
                if next_line.chars().all(|c| c == '-') && next_line.len() >= 3 {
                    result.push(format!("## {}", trimmed));
                    i += 2;
                    continue;
                }
            }

            // Detect blockquotes (> prefix)
            if trimmed.starts_with('>') {
                let rest = trimmed[1..].trim();
                result.push(format!("> {}", Self::format_inline(rest)));
                i += 1;
                continue;
            }

            // Detect horizontal rules
            if Self::is_horizontal_rule(trimmed) {
                result.push("---".to_string());
                i += 1;
                continue;
            }

            // Regular paragraph text - apply inline formatting
            if in_list {
                in_list = false;
            }
            result.push(Self::format_inline(trimmed));
            i += 1;
        }

        result.join("\n")
    }

    /// Check if a line looks like a heading (ALL CAPS, title case, or simple patterns).
    pub(crate) fn is_heading_line(line: &str) -> bool {
        // Skip lines that are too long to be headings
        if line.len() > 80 {
            return false;
        }

        // Skip lines that start with list markers
        if line.starts_with("- ") || line.starts_with("* ") || line.starts_with('>') {
            return false;
        }

        // Pattern: Numbered section headings like "1. Introduction", "Section 1:", "Chapter 3:"
        if Self::is_numbered_section_heading(line) {
            return true;
        }

        // Pattern: Short lines ending with colon (often labels/headers)
        if line.ends_with(':') && line.len() < 50 {
            let without_colon = line.trim_end_matches(':').trim();
            // Must have some alphabetic content
            if without_colon.chars().any(|c| c.is_alphabetic()) {
                return true;
            }
        }

        // Skip lines that start with numbers (could be ordered list) if not a section heading
        if line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return false;
        }

        // Check if line is mostly uppercase (likely a heading)
        let alpha_chars: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.is_empty() {
            return false;
        }

        let uppercase_count = alpha_chars.iter().filter(|c| c.is_uppercase()).count();
        let ratio = uppercase_count as f64 / alpha_chars.len() as f64;

        // If more than 70% uppercase, it's likely a heading
        if ratio > 0.7 {
            return true;
        }

        // Pattern: Title Case (each word starts with uppercase)
        if Self::is_title_case(line) {
            return true;
        }

        false
    }

    /// Check if line is a numbered section heading (e.g., "1. Introduction", "Section 1:", "Chapter 3:").
    pub(crate) fn is_numbered_section_heading(line: &str) -> bool {
        let line_lower = line.to_lowercase();

        // Pattern: "Section N", "Chapter N", "Part N", "Appendix N" followed by optional text
        let section_patterns = ["section ", "chapter ", "part ", "appendix ", "appendix: "];
        for pattern in section_patterns {
            if let Some(rest) = line_lower.strip_prefix(pattern) {
                // Check if followed by a number or roman numeral
                let rest = rest.trim();
                if rest.is_empty() {
                    continue;
                }
                // Check for digit
                if rest
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return true;
                }
                // Check for roman numeral (I, II, III, IV, V, etc.)
                if Self::starts_with_roman_numeral(rest) {
                    return true;
                }
            }
        }

        // Pattern: "N. Title" where N is a single digit or small number (not a list item)
        // Must have at least 4 words after the number to be a heading, not a list
        // This is more restrictive to avoid false positives on list items
        if let Some(rest) = Self::strip_numbered_prefix(line) {
            let word_count = rest.split_whitespace().count();
            // Headings typically have more descriptive titles (4+ words)
            // List items usually have 1-3 words
            if word_count >= 4 {
                return true;
            }
        }

        false
    }

    /// Strip a numbered prefix like "1. " or "1.1 " from a line.
    pub(crate) fn strip_numbered_prefix(line: &str) -> Option<&str> {
        // Pattern: "N. " or "N.N. " or "N.N.N. "
        let mut chars = line.chars().peekable();
        let mut end_pos = 0;

        // Match sequence of digits and dots
        loop {
            // Skip digits
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    chars.next();
                    end_pos += 1;
                } else {
                    break;
                }
            }
            // Check for dot
            if let Some(&'.') = chars.peek() {
                chars.next();
                end_pos += 1;
            } else {
                break;
            }
        }

        // Must have at least one digit
        if end_pos == 0 {
            return None;
        }

        // Must end with a space after the final dot
        if line.chars().nth(end_pos - 1) == Some('.') {
            if let Some(rest) = line.get(end_pos..) {
                if rest.starts_with(' ') {
                    return Some(rest.trim());
                }
            }
        }

        None
    }

    /// Check if a string starts with a roman numeral.
    pub(crate) fn starts_with_roman_numeral(s: &str) -> bool {
        let roman_numerals = [
            "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii",
            "xiv", "xv", "xvi", "xvii", "xviii", "xix", "xx",
        ];
        let s_lower = s.to_lowercase();
        let first_word = s_lower.split_whitespace().next().unwrap_or("");
        roman_numerals.contains(&first_word.trim_end_matches(':'))
    }

    /// Check if a line is in Title Case (each major word capitalized).
    pub(crate) fn is_title_case(line: &str) -> bool {
        // Skip very short lines
        if line.len() < 10 {
            return false;
        }

        // Skip lines with too many lowercase letters
        let alpha_chars: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.len() < 3 {
            return false;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < 2 {
            return false;
        }

        // Minor words that don't need to be capitalized
        let minor_words = [
            "a", "an", "the", "and", "but", "or", "for", "nor", "on", "at", "to", "by", "in", "of",
            "with", "is", "are", "was", "were", "be",
        ];

        let mut capitalized_count = 0;
        let mut total_words = 0;

        for (i, word) in words.iter().enumerate() {
            let word_lower = word.to_lowercase();
            // Skip minor words in the middle
            if i > 0 && minor_words.contains(&word_lower.as_str()) {
                continue;
            }

            total_words += 1;
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                if first.is_uppercase() {
                    capitalized_count += 1;
                }
            }
        }

        // If most major words are capitalized, it's title case
        if total_words == 0 {
            return false;
        }
        let ratio = capitalized_count as f64 / total_words as f64;
        ratio >= 0.8
    }

    /// Check if a line is a single-word section heading (like "Introduction", "Methods", "Results").
    pub(crate) fn is_single_word_section_heading(line: &str, index: usize, lines: &[&str]) -> bool {
        // Skip lines that are too long
        if line.len() > 30 {
            return false;
        }

        // Skip lines that start with list markers or special characters
        if line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with('>')
            || line.starts_with('#')
        {
            return false;
        }

        // Skip lines that start with numbers (could be ordered list)
        if line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return false;
        }

        // Must be a single word (no spaces)
        if line.contains(' ') {
            return false;
        }

        // Must have alphabetic content
        if !line.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        // First character must be uppercase
        if !line
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            return false;
        }

        // Common section heading words that are likely to be headings
        let common_section_words = [
            "introduction",
            "methods",
            "results",
            "discussion",
            "conclusion",
            "abstract",
            "summary",
            "overview",
            "background",
            "motivation",
            "approach",
            "implementation",
            "evaluation",
            "related",
            "future",
            "appendix",
            "references",
            "acknowledgments",
            "preface",
            "foreword",
            "contents",
            "index",
            "glossary",
            "bibliography",
            "notes",
            "chapter",
            "section",
            "part",
            "prologue",
            "epilogue",
            "setup",
            "installation",
            "usage",
            "examples",
            "configuration",
            "api",
            "tutorial",
            "guide",
            "faq",
            "changelog",
            "history",
            "purpose",
            "scope",
            "limitations",
            "benefits",
            "features",
            "requirements",
            "design",
            "architecture",
            "testing",
            "deployment",
            "maintenance",
            "troubleshooting",
            "support",
            "license",
        ];

        let line_lower = line.to_lowercase();

        // Check if it's a common section word
        if common_section_words.contains(&line_lower.as_str()) {
            return true;
        }

        // Must be preceded by an empty line (section break) - this is checked by caller
        // Look for the next non-empty line to verify this is a heading
        let mut next_idx = index + 1;
        while next_idx < lines.len() && lines[next_idx].trim().is_empty() {
            next_idx += 1;
        }

        // If we're at the end of document, this is a heading
        if next_idx >= lines.len() {
            return true;
        }

        let next_content = lines[next_idx].trim();

        // If next content line looks like content (not a heading pattern), this is likely a heading
        // The content should be longer than the potential heading (a real section heading is short)
        !Self::is_heading_line(next_content) && next_content.len() > line.len()
    }

    /// Determine the heading level based on position and content.
    pub(crate) fn determine_heading_level(line: &str, index: usize, lines: &[&str]) -> usize {
        // First line or near the beginning is usually H1
        if index == 0 {
            return 1;
        }

        // Check if previous line is empty (section break)
        let prev_empty = index > 0 && lines[index - 1].trim().is_empty();

        // Short lines near the beginning are likely H1 or H2
        if line.len() < 30 && prev_empty {
            return if index < 10 { 2 } else { 3 };
        }

        // Default to H2 for section headings
        if prev_empty {
            return 2;
        }

        // Default to H3 for subheadings
        3
    }

    /// Convert text to title case for headings.
    pub(crate) fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>()
                            + &chars.collect::<String>().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format heading text appropriately based on its content.
    pub(crate) fn format_heading_text(line: &str) -> String {
        // Check if this is a numbered section heading - preserve it as-is
        if Self::is_numbered_section_heading(line) {
            // Keep the original format for numbered sections
            return line.to_string();
        }

        // Check if line ends with colon - remove it for cleaner heading
        if line.ends_with(':') {
            let without_colon = line.trim_end_matches(':').trim();
            // Check if mostly uppercase
            let alpha_chars: Vec<char> = without_colon
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect();
            if !alpha_chars.is_empty() {
                let uppercase_count = alpha_chars.iter().filter(|c| c.is_uppercase()).count();
                let ratio = uppercase_count as f64 / alpha_chars.len() as f64;
                if ratio > 0.7 {
                    // Convert to title case
                    return Self::to_title_case(without_colon);
                }
            }
            // Return as-is with proper case
            return without_colon.to_string();
        }

        // Check if mostly uppercase
        let alpha_chars: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
        if !alpha_chars.is_empty() {
            let uppercase_count = alpha_chars.iter().filter(|c| c.is_uppercase()).count();
            let ratio = uppercase_count as f64 / alpha_chars.len() as f64;
            if ratio > 0.7 {
                // Convert to title case
                return Self::to_title_case(line);
            }
        }

        // Already title case or mixed case - preserve it
        line.to_string()
    }

    /// Check if line is an unordered list item (possibly nested).
    /// Returns Some((prefix_char, indent_level)) if it's a list item.
    pub(crate) fn is_unordered_list_item_with_indent(line: &str) -> Option<(char, usize)> {
        // Count leading spaces to determine indent level
        let spaces = line.chars().take_while(|&c| c == ' ').count();
        let indent_level = spaces / 2; // Each 2 spaces = 1 indent level

        let trimmed = line.trim();

        if trimmed.starts_with("- ") {
            Some(('-', indent_level))
        } else if trimmed.starts_with("* ") {
            Some(('*', indent_level))
        } else {
            None
        }
    }

    /// Check if line is an unordered list item.
    pub(crate) fn is_unordered_list_item(line: &str) -> Option<char> {
        if line.starts_with("- ") {
            Some('-')
        } else if line.starts_with("* ") {
            Some('*')
        } else {
            None
        }
    }

    /// Strip unordered list prefix from line.
    pub(crate) fn strip_list_prefix(line: &str, prefix: char) -> &str {
        let prefix_str = format!("{} ", prefix);
        line.strip_prefix(&prefix_str).unwrap_or(line)
    }

    /// Check if line is an ordered list item (possibly nested).
    /// Returns Some((number, indent_level)) if it's a list item.
    pub(crate) fn is_ordered_list_item_with_indent(line: &str) -> Option<(u32, usize)> {
        // Count leading spaces to determine indent level
        let spaces = line.chars().take_while(|&c| c == ' ').count();
        let indent_level = spaces / 2; // Each 2 spaces = 1 indent level

        let trimmed = line.trim();

        // Match patterns like "1.", "2.", "10.", etc.
        let parts: Vec<&str> = trimmed.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        if let Ok(num) = parts[0].parse::<u32>() {
            if parts[1].starts_with(' ') {
                return Some((num, indent_level));
            }
        }
        None
    }

    /// Check if line is an ordered list item.
    pub(crate) fn is_ordered_list_item(line: &str) -> bool {
        // Match patterns like "1.", "2.", "10.", etc.
        let parts: Vec<&str> = line.splitn(2, '.').collect();
        if parts.len() != 2 {
            return false;
        }
        parts[0].parse::<u32>().is_ok() && parts[1].starts_with(' ')
    }

    /// Strip ordered list prefix from line.
    pub(crate) fn strip_ordered_prefix(line: &str) -> &str {
        if let Some(pos) = line.find(". ") {
            &line[pos + 2..]
        } else {
            line
        }
    }

    /// Check if a line is a continuation of a list item (indented but not a list item itself).
    pub(crate) fn is_list_continuation(line: &str) -> bool {
        // A continuation line starts with whitespace but isn't a list item or code
        let trimmed = line.trim();

        // Empty lines are not continuations
        if trimmed.is_empty() {
            return false;
        }

        // Check if the line has leading whitespace
        let has_leading_whitespace = line.starts_with(' ') || line.starts_with('\t');

        // Check if it's NOT a list item itself
        let is_list =
            Self::is_unordered_list_item(trimmed).is_some() || Self::is_ordered_list_item(trimmed);

        // Check if it's not a code block marker
        let is_code_marker = trimmed.starts_with("```") || trimmed.starts_with("~~~");

        has_leading_whitespace && !is_list && !is_code_marker
    }

    /// Check if line is a horizontal rule.
    pub(crate) fn is_horizontal_rule(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 3 {
            return false;
        }
        // Check for patterns like ---, ***, ___
        let first_char = trimmed.chars().next().unwrap();
        (first_char == '-' || first_char == '*' || first_char == '_')
            && trimmed.chars().all(|c| c == first_char)
    }

    /// Apply inline formatting (bold, italic, code).
    pub(crate) fn format_inline(text: &str) -> String {
        let mut result = text.to_string();

        // Detect inline code (text surrounded by backticks)
        // This is already markdown, so preserve it

        // Detect patterns that look like emphasis
        // Words surrounded by * or _ should become italic
        // Words surrounded by ** or __ should become bold

        // For now, we'll do simple pattern detection
        // Look for text like *word* or _word_ and ensure it's italic
        // Look for text like **word** or __word__ and ensure it's bold

        // URL detection - make links clickable
        result = Self::format_urls(&result);

        result
    }

    /// Format URLs as Markdown links.
    pub(crate) fn format_urls(text: &str) -> String {
        // Simple URL pattern matching
        let url_pattern = regex::Regex::new(r"https?://[^\s]+").unwrap();
        url_pattern
            .replace_all(text, |caps: &regex::Captures| {
                let url = &caps[0];
                // Remove trailing punctuation that's likely not part of the URL
                let url = url.trim_end_matches(|c| c == '.' || c == ',' || c == ';' || c == ':');
                format!("<{}>", url)
            })
            .to_string()
    }

    /// Extract metadata from the text.
    pub(crate) fn extract_metadata(
        &self,
        text: &str,
        input_path: &Option<std::path::PathBuf>,
    ) -> serde_json::Value {
        let lines: Vec<&str> = text.lines().collect();
        let word_count: usize = text.split_whitespace().count();
        let line_count = lines.len();

        let mut metadata = serde_json::json!({
            "lines": line_count,
            "words": word_count,
            "characters": text.len(),
        });

        // Try to extract a title from the first non-empty line
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // Use first line as title (truncate if too long)
                let title = if trimmed.len() > 80 {
                    format!("{}...", &trimmed[..77])
                } else {
                    trimmed.to_string()
                };
                metadata["title"] = serde_json::json!(title);
                break;
            }
        }

        // Add source info if input is a file
        if let Some(ref path) = input_path {
            metadata["source"] = serde_json::json!(path.display().to_string());
            metadata["type"] = serde_json::json!("file");
        } else {
            metadata["type"] = serde_json::json!("stdin");
        }

        metadata
    }

    /// Normalize spacing in the markdown output.
    ///
    /// This function:
    /// - Trims trailing whitespace from each line
    /// - Collapses consecutive blank lines into a single blank line
    /// - Removes leading and trailing blank lines
    pub(crate) fn normalize_spacing(&self, text: &str) -> String {
        // Trim trailing whitespace from each line
        let mut result: String = text
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple blank lines into single blank lines
        let lines: Vec<&str> = result.lines().collect();
        let mut collapsed_lines = Vec::new();
        let mut prev_blank = false;

        for line in &lines {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue; // Skip consecutive blank lines
            }
            collapsed_lines.push(*line);
            prev_blank = is_blank;
        }

        result = collapsed_lines.join("\n");

        // Remove leading/trailing blank lines
        result.trim().to_string()
    }

    /// Format output based on the output format.
    pub(crate) fn format_output(
        &self,
        markdown: &str,
        metadata: &serde_json::Value,
        format: OutputFormat,
    ) -> String {
        match format {
            OutputFormat::Json => serde_json::json!({
                "markdown": markdown,
                "metadata": metadata,
            })
            .to_string(),
            OutputFormat::Compact | OutputFormat::Agent => {
                format!(
                    "---\n{}\n---\n\n{}",
                    serde_json::to_string_pretty(metadata).unwrap(),
                    markdown
                )
            }
            OutputFormat::Raw | OutputFormat::Csv | OutputFormat::Tsv => markdown.to_string(),
        }
    }
}

impl CommandHandler for Txt2mdHandler {
    type Input = Txt2mdInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        // Read input text
        let text = self.read_input(&input.input)?;

        // Convert to Markdown
        let markdown = self.convert_to_markdown(&text);

        // Normalize spacing (collapse blank lines, trim trailing whitespace)
        let normalized = self.normalize_spacing(&markdown);

        // Extract metadata
        let metadata = self.extract_metadata(&text, &input.input);

        // Format output
        let formatted = self.format_output(&normalized, &metadata, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("txt2md")
                .with_output_mode(ctx.format)
                .with_input_bytes(text.len())
                .with_output_bytes(formatted.len())
                .with_extra(
                    "Source",
                    input
                        .input
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "stdin".to_string()),
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
            print!("{}\n", formatted);
        }

        Ok(())
    }
}

