//! Heading detection heuristics for txt2md parsing.

use super::Txt2mdHandler;

impl Txt2mdHandler {
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
}
