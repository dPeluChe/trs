//! List, rule, and inline detection heuristics for txt2md parsing.

use super::Txt2mdHandler;

impl Txt2mdHandler {
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
}
