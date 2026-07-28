use crate::{Cli, OutputFormat};

pub(crate) use super::ansi::{sanitize_control_chars, strip_ansi_codes, strip_emojis};

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
    /// Create a default context with Compact format (for fast-path bypass).
    pub fn default_compact() -> Self {
        Self {
            format: OutputFormat::Compact,
            stats: false,
            enabled_formats: vec![],
        }
    }

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

/// Exit status of the command trs just ran, for parsers that summarize
/// success/failure. Ambient because the parse layer is reached through a
/// generic Router route, and threading it would touch every handler.
/// Unset means no child ran (standalone `trs parse --file`), so parsers keep
/// their text-only behavior there.
mod child_exit {
    use std::sync::atomic::{AtomicI32, Ordering};
    const UNSET: i32 = i32::MIN;
    static CODE: AtomicI32 = AtomicI32::new(UNSET);

    pub(crate) fn set(code: i32) {
        CODE.store(code, Ordering::Relaxed);
    }
    pub(crate) fn get() -> Option<i32> {
        match CODE.load(Ordering::Relaxed) {
            UNSET => None,
            v => Some(v),
        }
    }
}

pub(crate) use child_exit::set as set_child_exit;

/// The command trs ran exited non-zero. A hard fact — parsers use it to
/// override text heuristics, which cannot see every tool's error dialect.
pub(crate) fn child_failed() -> bool {
    matches!(child_exit::get(), Some(c) if c != 0)
}

/// Exit code of the command trs ran, when one ran.
pub(crate) fn child_exit_code() -> Option<i32> {
    child_exit::get()
}

/// Estimate the number of tokens from byte count.
/// Uses the common approximation of ~4 characters per token.
pub(crate) fn estimate_tokens(bytes: usize) -> usize {
    // Most tokenizers average around 4 characters per token for English text
    // This is a rough estimate suitable for statistics display
    bytes / 4
}

/// Quote a field for CSV output, per RFC 4180: wrap in double quotes when it
/// holds a comma, quote, CR or LF, doubling any embedded quote. Shared by the
/// run / replace / tail / parse handlers that emit CSV directly.
pub(crate) fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
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

// ============================================================
// Multilingual error / warning detection
// ============================================================

/// Marker strings that flag a line as "this is an error". Covers English plus
/// the locales most likely to surface in tool output (rustc/cargo, npm, pip,
/// system utilities localized to the user's env).
///
/// Match is case-insensitive on an already-lowercased haystack — callers are
/// expected to `.to_ascii_lowercase()` their line first (or use the helpers
/// below which do it for them).
///
/// Non-ASCII scripts (Chinese/Japanese/Russian) keep their native casing
/// because to_ascii_lowercase is a no-op on them.
const ERROR_MARKERS: &[&str] = &[
    // English (canonical)
    "error:",
    "error[",
    "err!",
    // German
    "fehler:",
    // French
    "erreur:",
    // Spanish
    "error:", // same word but keeps the list self-documenting
    // Portuguese / Italian
    "erro:",
    "errore:",
    // Russian
    "ошибка:",
    // Chinese (simplified + traditional)
    "错误",
    "錯誤",
    // Japanese
    "エラー:",
    "エラー",
    // Korean
    "오류:",
];

const WARNING_MARKERS: &[&str] = &[
    "warning:",
    "warning[",
    "warn ",
    "warn:",
    "warnung:",        // German
    "avertissement:",  // French
    "attention:",      // French (informal)
    "advertencia:",    // Spanish
    "aviso:",          // Portuguese
    "avviso:",         // Italian
    "警告:",           // Chinese / Japanese
    "предупреждение:", // Russian
    "경고:",           // Korean
];

/// True if the line looks like an error message in any supported locale.
/// Checks both the prefix case (typical for compiler/linter output like
/// `error: mismatched types`) and the embedded case (like
/// `src/foo.rs:5:3: error: ...`).
pub(crate) fn is_error_line(line: &str) -> bool {
    let lower = line.trim_start().to_ascii_lowercase();
    for marker in ERROR_MARKERS {
        if lower.starts_with(marker) {
            return true;
        }
        // Embedded: "<path>:<line>:<col>: error: ..." / "fehler: ..."
        let needle_prefix = format!(": {}", marker);
        if lower.contains(&needle_prefix) {
            return true;
        }
    }
    has_coded_diagnostic(&lower, "error")
}

/// Diagnostics that put a code between the severity word and the colon —
/// `error TS2322:` (tsc), `error C2065:` (MSVC), `warning CS0168:` (C#).
/// The plain `error:` markers miss these, which is how a failing `tsc` build
/// once summarized as zero errors. Requiring the trailing colon on the code
/// keeps prose like "error handling improved" from matching.
fn has_coded_diagnostic(lower: &str, word: &str) -> bool {
    let needle = format!("{} ", word);
    let mut from = 0;
    while let Some(pos) = lower[from..].find(&needle) {
        let at = from + pos;
        // Must start the line or follow a separator, not be a word tail
        // ("mirror error" is fine; "syntaxerror foo:" should not match here).
        let boundary = at == 0
            || matches!(
                lower[..at].chars().next_back(),
                Some(' ') | Some(':') | Some('\t') | Some('[') | Some('(')
            );
        let rest = &lower[at + needle.len()..];
        let code: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if boundary
            && !code.is_empty()
            && code.chars().any(|c| c.is_ascii_digit())
            && rest[code.len()..].starts_with(':')
        {
            return true;
        }
        from = at + needle.len();
    }
    false
}

/// True if the line looks like a warning message in any supported locale.
pub(crate) fn is_warning_line(line: &str) -> bool {
    let lower = line.trim_start().to_ascii_lowercase();
    for marker in WARNING_MARKERS {
        if lower.starts_with(marker) {
            return true;
        }
        let needle_prefix = format!(": {}", marker);
        if lower.contains(&needle_prefix) {
            return true;
        }
    }
    has_coded_diagnostic(&lower, "warning")
}

/// True if the output contains a hard-failure signal: exit stack trace, panic,
/// fatal, or known crash patterns. Used by the fail-open gate so we don't
/// silently compress away a broken subprocess's output.
pub(crate) fn output_has_failure_signal(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    const FATAL_MARKERS: &[&str] = &[
        "panic:",
        "panicked at",
        "fatal:",
        "fatal error",
        "traceback (most recent call last)", // Python
        "exception in thread",               // Java
        "uncaught exception",                // Node
        "segmentation fault",
        "stack overflow",
        "killed (program exited", // some wrappers
        "abort trap",
    ];
    FATAL_MARKERS.iter().any(|m| lower.contains(m))
}

// ============================================================
// Credential preservation (NOT redaction)
// ============================================================

/// Marker substrings that flag a line as "may contain credentials — don't
/// silently drop during compression". This is a PRESERVATION mechanism so
/// debugging info (connection strings, auth headers) survives the
/// summarization step. It is NOT a redactor — the line still goes to the
/// agent verbatim.
const CREDENTIAL_MARKERS: &[&str] = &[
    "aws_access_key_id",
    "aws_secret_access_key",
    "AKIA", // AWS access key prefix (IAM user)
    "ASIA", // AWS temp session key prefix (STS)
    "ghp_", // GitHub personal access token
    "github_pat_",
    "gho_",     // GitHub OAuth token
    "ghu_",     // GitHub user-to-server token
    "sk_live_", // Stripe live secret
    "sk_test_",
    "rk_live_",
    "Bearer ",
    "Authorization:",
    "X-Api-Key:",
    "x-api-key:",
    "api_key=",
    "api-key=",
    "apikey=",
    "password=",
    "passwd=",
    "PRIVATE KEY",
    "BEGIN RSA PRIVATE KEY",
    "BEGIN OPENSSH PRIVATE KEY",
];

/// True if the line contains any pattern we want to preserve through
/// compression. Cheap substring checks plus two structural heuristics
/// (URL basic-auth, JWT shape).
pub(crate) fn contains_credential(line: &str) -> bool {
    // Huge lines are probably binary blobs — skip; the scan would be slow
    // and the false-positive rate is high.
    if line.len() > 2000 {
        return false;
    }

    for marker in CREDENTIAL_MARKERS {
        if line.contains(marker) {
            return true;
        }
    }

    // URL basic-auth: scheme://user:pass@host. We want to catch
    // `postgresql://u:p@h/db`, `redis://x:y@h`, `mongodb+srv://...` etc.
    if let Some(proto_end) = line.find("://") {
        let after = &line[proto_end + 3..];
        if let Some(at_pos) = after.find('@') {
            let creds = &after[..at_pos];
            if !creds.is_empty() && creds.contains(':') && !creds.contains('/') {
                return true;
            }
        }
    }

    // JWT: `eyJ` (base64 of `{"`) followed by two dots within a short span.
    if let Some(pos) = line.find("eyJ") {
        let tail = &line[pos..];
        let dots = tail.chars().take(500).filter(|c| *c == '.').count();
        if dots >= 2 {
            return true;
        }
    }

    false
}
