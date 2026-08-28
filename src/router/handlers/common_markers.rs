//! Line classification shared by every handler: which lines read as an
//! error, which as a warning, and which carry a credential that must be
//! redacted. Split out of `common.rs`, where it was more than a third of
//! the file and unrelated to the context/stats types around it.

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
