//! `brew install|upgrade|reinstall|uninstall` output compression.
//!
//! Homebrew output is mostly noise: `==> Fetching ...` / `==> Downloading ...`
//! / progress-bar lines / `==> Pouring ...` blocks repeat per dependency.
//! The signal lives in three places and we keep only those:
//! - 🍺 `/opt/homebrew/Cellar/X/Y: N files, Z` lines (per-package install)
//! - `Error:` / `Warning:` lines
//! - the final `==> Summary` block

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    pub(crate) fn handle_brew(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Preserve the 🍺 — it marks install-result lines.
        let input = Self::read_input_raw(file)?;
        let input_bytes = input.len();

        // Fail-open: brew crashes leak useful recovery info we don't want
        // to compress away (e.g. "Permission denied", Ruby stack traces).
        if super::super::common::output_has_failure_signal(&input) {
            crate::parse_out::emit(&input);
            if ctx.stats {
                CommandStats::new()
                    .with_reducer("brew-passthrough")
                    .with_input_bytes(input_bytes)
                    .with_output_bytes(input_bytes)
                    .print();
            }
            return Ok(());
        }

        let mut installed: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        for raw in input.lines() {
            let line = raw.trim();
            if line.is_empty() || is_progress_bar(line) {
                continue;
            }

            // 🍺 <path>: N files, SIZE  → the actual install result per pkg.
            if line.starts_with('\u{1f37a}') {
                installed.push(condense_keg_line(line));
                continue;
            }

            // `==> …` lines: mostly noise (Fetching / Downloading / Pouring /
            // Installing), but errors/warnings sometimes ride the same prefix.
            if line.starts_with("==>") {
                if super::super::common::is_error_line(line) {
                    errors.push(line.to_string());
                } else if super::super::common::is_warning_line(line) {
                    warnings.push(line.to_string());
                }
                continue;
            }

            // Freestanding errors/warnings (not under `==>`).
            if super::super::common::is_error_line(line) {
                errors.push(line.to_string());
            } else if super::super::common::is_warning_line(line) {
                warnings.push(line.to_string());
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "installed": installed,
                "installed_count": installed.len(),
                "errors": errors,
                "warnings": warnings,
            })
            .to_string(),
            _ => format_brew_compact(&installed, &errors, &warnings),
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("brew")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

/// A brew progress bar is a run of `#`, digits, `.`, `%`, and whitespace
/// (e.g. `###  33.4%`). Drop them — pure visual noise.
fn is_progress_bar(line: &str) -> bool {
    line.chars()
        .all(|c| c == '#' || c.is_ascii_digit() || c == '.' || c == '%' || c.is_whitespace())
        && line.contains('#')
}

/// Turn `🍺  /opt/homebrew/Cellar/wget/1.24.5: 90 files, 4.5MB`
/// into  `🍺 wget 1.24.5 (4.5MB, 90 files)` for a tighter readable form.
fn condense_keg_line(line: &str) -> String {
    // Everything after the 🍺 prefix.
    let body = line.trim_start_matches('\u{1f37a}').trim();

    let Some((path, rest)) = body.split_once(':') else {
        return line.to_string();
    };

    // Path is `.../Cellar/<pkg>/<version>` — extract the last two segments.
    let mut parts = path.rsplit('/');
    let version = parts.next().unwrap_or("").trim();
    let pkg = parts.next().unwrap_or("").trim();
    if pkg.is_empty() || version.is_empty() {
        return line.to_string();
    }

    // Swap `N files, SIZE` → `SIZE, N files` so the big number shows first.
    // Split on `", "` (comma-space) — a plain `,` split would break inside
    // thousands-separated counts like `7,560 files`.
    let rest = rest.trim();
    let reordered = match rest.split_once(", ") {
        Some((a, b)) => {
            let a = a.trim();
            let b = b.trim();
            if a.ends_with("files") || a.ends_with("file") {
                format!("{}, {}", b, a)
            } else {
                format!("{}, {}", a, b)
            }
        }
        None => rest.to_string(),
    };

    format!("\u{1f37a} {} {} ({})", pkg, version, reordered)
}

fn format_brew_compact(installed: &[String], errors: &[String], warnings: &[String]) -> String {
    let mut out = String::new();

    if !installed.is_empty() {
        out.push_str(&format!("brew: installed {}\n", installed.len()));
        for pkg in installed {
            out.push_str(&format!("  {}\n", pkg));
        }
    }
    if !warnings.is_empty() {
        out.push_str(&format!("warnings ({}):\n", warnings.len()));
        for w in warnings.iter().take(5) {
            out.push_str(&format!("  {}\n", w));
        }
        if warnings.len() > 5 {
            out.push_str(&format!("  ... +{} more\n", warnings.len() - 5));
        }
    }
    if !errors.is_empty() {
        out.push_str(&format!("errors ({}):\n", errors.len()));
        for e in errors.iter().take(10) {
            out.push_str(&format!("  {}\n", e));
        }
        if errors.len() > 10 {
            out.push_str(&format!("  ... +{} more\n", errors.len() - 10));
        }
    }
    if out.is_empty() {
        out.push_str("brew: no-op (nothing installed/upgraded, no errors)\n");
    }
    out
}
