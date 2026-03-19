//! Text parsing logic for txt2md conversion.
//!
//! Contains heading detection, list detection, and text-to-markdown parsing.

use super::Txt2mdHandler;

impl Txt2mdHandler {
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
}
