use crate::{Cli, OutputFormat};

/// Strip ANSI escape codes from a string.
///
/// This function handles all common ANSI escape sequence types:
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
            // Skip the escape character
            i += 1;

            if i >= chars.len() {
                break;
            }

            match chars[i] {
                // CSI (Control Sequence Introducer): ESC [ ... <final byte>
                '[' => {
                    i += 1;
                    // Skip parameter and intermediate bytes until we reach a final byte
                    // Final bytes are in range 0x40-0x7E (@A-Z[\]^_`a-z{|}~)
                    while i < chars.len() && !(chars[i] >= '@' && chars[i] <= '~') {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip the final byte
                    }
                }
                // OSC (Operating System Command): ESC ] ... (BEL or ST)
                ']' => {
                    i += 1;
                    // Skip until we find BEL (0x07) or ST (ESC \)
                    while i < chars.len() {
                        if chars[i] == '\x07' {
                            // Found BEL, skip it and continue
                            i += 1;
                            break;
                        } else if chars[i] == '\x1b' {
                            // Possible ST sequence (ESC \)
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
                // Character set selection: ESC (, ESC ), ESC *, ESC + followed by a char
                '(' | ')' | '*' | '+' | '-' | '.' | '/' => {
                    i += 1;
                    // Skip the character set identifier
                    if i < chars.len() {
                        i += 1;
                    }
                }
                // Simple two-character escape sequences and other Fe sequences
                // These include: ESC c (RIS), ESC D (IND), ESC E (NEL), ESC H (HTS), etc.
                _ => {
                    // Skip the character after ESC (it's part of the escape sequence)
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

/// Sanitize input string by handling control characters.
///
/// This function:
/// - Removes null bytes (0x00)
/// - Replaces other control characters (except newlines and tabs) with spaces
/// - Normalizes multiple consecutive spaces to single space
/// - Preserves valid Unicode characters
pub(crate) fn sanitize_control_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;

    for c in s.chars() {
        match c {
            // Remove null bytes entirely
            '\x00' => continue,
            // Preserve newlines and tabs
            '\n' | '\t' | '\r' => {
                result.push(c);
                prev_was_space = false;
            }
            // Replace other ASCII control characters with space
            c if c.is_ascii_control() => {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            // Keep all other characters (including Unicode)
            c => {
                result.push(c);
                prev_was_space = false;
            }
        }
    }

    result
}

/// Context passed to command handlers containing global CLI options.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The output format to use for the command result.
    pub format: OutputFormat,
    /// Whether to show execution statistics.
    pub stats: bool,
    /// List of enabled format flags (for warnings/debugging).
    #[allow(dead_code)]
    pub enabled_formats: Vec<OutputFormat>,
}

impl CommandContext {
    /// Create a new command context from CLI options.
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            format: cli.output_format(),
            stats: cli.stats,
            enabled_formats: cli.enabled_format_flags(),
        }
    }

    /// Returns true if multiple format flags were specified.
    #[allow(dead_code)]
    pub fn has_conflicting_formats(&self) -> bool {
        self.enabled_formats.len() > 1
    }
}

/// Result type for command handlers.
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Error type for command execution.
#[derive(Debug, Clone)]
pub enum CommandError {
    /// The command is not yet implemented.
    #[allow(dead_code)]
    NotImplemented(String),
    /// An error occurred during execution with an optional exit code.
    ExecutionError {
        message: String,
        exit_code: Option<i32>,
    },
    /// Invalid arguments provided.
    InvalidArguments(String),
    /// I/O error occurred.
    IoError(String),
}

impl CommandError {
    /// Returns the exit code if this error is associated with a non-zero exit.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandError::ExecutionError { exit_code, .. } => *exit_code,
            _ => None,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            CommandError::ExecutionError { message, .. } => {
                write!(f, "Execution error: {}", message)
            }
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

// ============================================================
// Command Statistics
// ============================================================

/// Estimate the number of tokens from byte count.
/// Uses the common approximation of ~4 characters per token.
pub(crate) fn estimate_tokens(bytes: usize) -> usize {
    // Most tokenizers average around 4 characters per token for English text
    // This is a rough estimate suitable for statistics display
    bytes / 4
}

/// Statistics about command execution.
#[derive(Debug, Clone, Default)]
pub struct CommandStats {
    /// Input size in bytes.
    pub input_bytes: usize,
    /// Output size in bytes.
    pub output_bytes: usize,
    /// Estimated input token count.
    pub input_tokens: usize,
    /// Estimated output token count.
    pub output_tokens: usize,
    /// Number of items processed (matches, files, lines, etc.).
    pub items_processed: usize,
    /// Number of items filtered out.
    pub items_filtered: usize,
    /// Duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// Command name (for run command).
    pub command: Option<String>,
    /// Exit code (for run command).
    pub exit_code: Option<i32>,
    /// Name of the reducer used.
    pub reducer: Option<String>,
    /// Output format mode used.
    pub output_mode: Option<OutputFormat>,
    /// Additional stats as key-value pairs.
    pub extra: Vec<(String, String)>,
}

impl CommandStats {
    /// Create new command stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set input bytes (also calculates estimated tokens).
    pub fn with_input_bytes(mut self, bytes: usize) -> Self {
        self.input_bytes = bytes;
        self.input_tokens = estimate_tokens(bytes);
        self
    }

    /// Set output bytes (also calculates estimated tokens).
    pub fn with_output_bytes(mut self, bytes: usize) -> Self {
        self.output_bytes = bytes;
        self.output_tokens = estimate_tokens(bytes);
        self
    }

    /// Set items processed.
    pub fn with_items_processed(mut self, count: usize) -> Self {
        self.items_processed = count;
        self
    }

    /// Set items filtered.
    pub fn with_items_filtered(mut self, count: usize) -> Self {
        self.items_filtered = count;
        self
    }

    /// Set duration in milliseconds.
    pub fn with_duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    /// Set command name.
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    /// Set exit code.
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Set reducer name.
    pub fn with_reducer(mut self, reducer: impl Into<String>) -> Self {
        self.reducer = Some(reducer.into());
        self
    }

    /// Set output format mode.
    pub fn with_output_mode(mut self, mode: OutputFormat) -> Self {
        self.output_mode = Some(mode);
        self
    }

    /// Add an extra stat.
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.push((key.into(), value.into()));
        self
    }

    /// Calculate reduction percentage.
    pub fn reduction_percent(&self) -> f64 {
        if self.input_bytes == 0 {
            0.0
        } else if self.output_bytes >= self.input_bytes {
            0.0 // No reduction if output is larger or equal
        } else {
            ((self.input_bytes - self.output_bytes) as f64 / self.input_bytes as f64) * 100.0
        }
    }

    /// Calculate token reduction percentage.
    pub fn token_reduction_percent(&self) -> f64 {
        if self.input_tokens == 0 {
            0.0
        } else if self.output_tokens >= self.input_tokens {
            0.0 // No reduction if output is larger or equal
        } else {
            ((self.input_tokens - self.output_tokens) as f64 / self.input_tokens as f64) * 100.0
        }
    }

    /// Print stats to stderr.
    pub fn print(&self) {
        eprintln!("Stats:");
        if let Some(ref cmd) = self.command {
            eprintln!("  Command: {}", cmd);
        }
        if let Some(code) = self.exit_code {
            eprintln!("  Exit code: {}", code);
        }
        if let Some(ref reducer) = self.reducer {
            eprintln!("  Reducer: {}", reducer);
        }
        if let Some(mode) = self.output_mode {
            eprintln!("  Output mode: {}", Self::format_output_mode(mode));
        }
        if self.input_bytes > 0 || self.output_bytes > 0 {
            eprintln!("  Input bytes: {}", self.input_bytes);
            eprintln!("  Output bytes: {}", self.output_bytes);
            let reduction = self.reduction_percent();
            if reduction > 0.0 {
                eprintln!("  Reduction: {:.1}%", reduction);
            }
            // Show token estimation
            if self.input_tokens > 0 || self.output_tokens > 0 {
                eprintln!("  Input tokens (est.): {}", self.input_tokens);
                eprintln!("  Output tokens (est.): {}", self.output_tokens);
                let token_reduction = self.token_reduction_percent();
                if token_reduction > 0.0 {
                    eprintln!("  Token reduction: {:.1}%", token_reduction);
                }
            }
        }
        if self.items_processed > 0 {
            eprintln!("  Items processed: {}", self.items_processed);
        }
        if self.items_filtered > 0 {
            eprintln!("  Items filtered: {}", self.items_filtered);
        }
        if let Some(ms) = self.duration_ms {
            if ms < 1000 {
                eprintln!("  Duration: {}ms", ms);
            } else {
                eprintln!("  Duration: {:.2}s", ms as f64 / 1000.0);
            }
        }
        for (key, value) in &self.extra {
            eprintln!("  {}: {}", key, value);
        }
    }

    /// Format output mode for display.
    pub(crate) fn format_output_mode(mode: OutputFormat) -> &'static str {
        match mode {
            OutputFormat::Raw => "raw",
            OutputFormat::Compact => "compact",
            OutputFormat::Json => "json",
            OutputFormat::Csv => "csv",
            OutputFormat::Tsv => "tsv",
            OutputFormat::Agent => "agent",
        }
    }
}
