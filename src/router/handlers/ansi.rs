/// Strip ANSI escape codes from a string.
///
/// Handles all common ANSI escape sequence types:
/// - CSI (Control Sequence Introducer): ESC [ ... <final byte>
/// - OSC (Operating System Command): ESC ] ... (BEL or ST)
/// - Simple escape sequences: ESC followed by a single character
/// - Other sequences: ESC (, ESC ), ESC #, etc.
pub(crate) fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\x1b' {
            i += 1;

            if i >= chars.len() {
                break;
            }

            match chars[i] {
                '[' => {
                    i += 1;
                    while i < chars.len() && !(chars[i] >= '@' && chars[i] <= '~') {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                }
                ']' => {
                    i += 1;
                    while i < chars.len() {
                        if chars[i] == '\x07' {
                            i += 1;
                            break;
                        } else if chars[i] == '\x1b' {
                            i += 1;
                            if i < chars.len() && chars[i] == '\\' {
                                i += 1;
                                break;
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                '(' | ')' | '*' | '+' | '-' | '.' | '/' => {
                    i += 1;
                    if i < chars.len() {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Strip emoji characters from a string.
///
/// Removes decorative emoji that waste tokens and confuse non-Claude LLMs.
/// Preserves all ASCII, standard Unicode text, and common symbols.
pub(crate) fn strip_emojis(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if is_emoji(c) {
            while let Some(&next) = chars.peek() {
                if next == '\u{FE0F}' || next == '\u{FE0E}' || next == '\u{200D}' {
                    chars.next();
                } else if is_emoji(next) {
                    chars.next();
                } else {
                    break;
                }
            }
            if result.ends_with(' ') {
                // skip, already have space
            } else if let Some(&next) = chars.peek() {
                if next == ' ' {
                    // next char is space, skip adding one
                } else {
                    // don't add space for inline removal
                }
            }
        } else if c == '\u{FE0F}' || c == '\u{FE0E}' {
            // Skip stray variation selectors
        } else {
            result.push(c);
        }
    }

    result
}

/// Check if a character is a decorative emoji that should be stripped.
///
/// Preserves functional symbols used by CLI tools:
/// - Checkmarks: ✓ ✔ (U+2713, U+2714)
/// - Crosses: ✗ ✕ ✘ (U+2717, U+2715, U+2718)
/// - Bullets: ● ○ ◆ ◇ ▶ ▸ (U+25xx)
/// - Warning: ⚠ (U+26A0)
fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F600..=0x1F64F |
        0x1F300..=0x1F5FF |
        0x1F680..=0x1F6FF |
        0x1F900..=0x1F9FF |
        0x1FA00..=0x1FA6F |
        0x1FA70..=0x1FAFF |
        0x1F1E0..=0x1F1FF |
        0x1F200..=0x1F2FF |
        0xE0001..=0xE007F |
        0x2728 | // ✨ sparkles
        0x274C | // ❌ cross mark
        0x274E | // ❎ cross mark with box
        0x2753 | // ❓ question mark
        0x2754 | // ❔ white question mark
        0x2755 | // ❕ white exclamation mark
        0x2757 | // ❗ exclamation mark
        0x2705 | // ✅ check mark button
        0x2795 | // ➕ plus
        0x2796 | // ➖ minus
        0x2797   // ➗ division
    )
}

/// Sanitize input string by handling control characters.
///
/// - Removes null bytes (0x00)
/// - Replaces other control characters (except newlines and tabs) with spaces
/// - Normalizes multiple consecutive spaces to single space
pub(crate) fn sanitize_control_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;

    for c in s.chars() {
        match c {
            '\x00' => continue,
            '\n' | '\t' | '\r' => {
                result.push(c);
                prev_was_space = false;
            }
            c if c.is_ascii_control() => {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            c => {
                result.push(c);
                prev_was_space = false;
            }
        }
    }

    result
}
