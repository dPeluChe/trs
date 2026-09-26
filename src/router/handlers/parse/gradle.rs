//! Parser for Gradle output (`gradle`, `./gradlew`).
//!
//! Keeps failed tasks, compiler errors with their source line, warnings,
//! failed tests, test counts and the result. Drops tasks that did nothing
//! new (UP-TO-DATE, NO-SOURCE, ...), the `* Try:` hints and the second copy
//! of compiler errors inside `* What went wrong:`. Gradle prints a failed
//! test as just `AssertionFailedError at CartTest.java:7`; the message is in
//! the JUnit XML report it just wrote, so it is read from there.

use super::super::ansi::strip_ansi_codes;
use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;
use std::collections::HashSet;
use std::path::Path;

const NOISE_PREFIXES: &[&str] = &[
    "Starting a Gradle Daemon",
    "Daemon will be stopped",
    "To honour the JVM settings",
    "[Incubating] Problems report is available",
    "You can use '--warning-mode all'",
    "For more on this, please refer to",
    "* What went wrong:",
    "Note: Recompile with -Xlint",
    "> Configure project",
];

#[derive(Default)]
pub(crate) struct Gradle {
    pub(crate) lines: Vec<String>,
    pub(crate) quiet_tasks: usize,
    pub(crate) cut: bool,
}

impl ParseHandler {
    pub(crate) fn handle_gradle(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = strip_ansi_codes(&Self::read_input(file)?);
        let cwd = std::env::current_dir().unwrap_or_default();
        let g = parse_gradle(&input, &cwd);
        if g.cut {
            crate::parse_out::mark_dropped();
        }
        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({ "lines": g.lines, "quiet_tasks": g.quiet_tasks }).to_string()
                    + "\n"
            }
            _ => format_gradle_compact(&g),
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gradle")
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(g.lines.len())
                .print();
        }
        Ok(())
    }
}

/// `> Task :api:test FAILED` as (`:api:test`, `FAILED`); outcome may be empty.
fn task_line(t: &str) -> Option<(&str, &str)> {
    let rest = t.strip_prefix("> Task ")?;
    Some(rest.split_once(' ').unwrap_or((rest, "")))
}

/// `CartTest > discount() FAILED` as (`CartTest`, `discount()`).
fn failed_test(t: &str) -> Option<(&str, &str)> {
    let rest = t.strip_suffix(" FAILED")?;
    let (class, method) = rest.split_once(" > ")?;
    (!class.contains(' ')).then_some((class, method))
}

/// Failure message for `class.method` from the JUnit XML report of `task`.
fn report_message(root: &Path, task: &str, class: &str, method: &str) -> Option<String> {
    let mut parts: Vec<&str> = task.trim_start_matches(':').split(':').collect();
    let task_name = parts.pop()?;
    let dir = parts
        .iter()
        .fold(root.to_path_buf(), |d, p| d.join(p))
        .join("build/test-results")
        .join(task_name);
    let suffix = format!(".{class}.xml");
    let file = std::fs::read_dir(&dir).ok()?.flatten().find(|e| {
        let name = e.file_name().to_string_lossy().into_owned();
        name.ends_with(&suffix) || name == format!("TEST-{class}.xml")
    })?;
    let xml = std::fs::read_to_string(file.path()).ok()?;
    let case = xml.find(&format!("<testcase name=\"{}\"", xml_escape(method)))?;
    let body = &xml[case..];
    // `<testcase .../>` passed; reading on would take the next test's failure.
    if body[..body.find('>')?].ends_with('/') {
        return None;
    }
    let body = &body[..body.find("</testcase>").unwrap_or(body.len())];
    let attr = body.split("message=\"").nth(1)?;
    let msg = xml_unescape(&attr[..attr.find('"')?]);
    let first = msg.lines().next()?.to_string();
    // `org.opentest4j.AssertionFailedError: expected ...`: the type is on the line above.
    let text = match first.split_once(": ") {
        Some((ty, rest)) if !ty.contains(' ') && ty.contains('.') => rest.to_string(),
        _ => first,
    };
    Some(text.chars().take(300).collect())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#10;", "\n")
        .replace("&amp;", "&")
}

pub(crate) fn parse_gradle(input: &str, cwd: &Path) -> Gradle {
    let prefix = format!("{}/", cwd.to_string_lossy());
    let mut g = Gradle::default();
    let mut seen: HashSet<String> = HashSet::new();
    let mut in_try = false;
    let mut in_wrong = false;
    let mut after_warning = 0u8;
    let mut last_test_task = String::new();
    let mut pending: Option<(String, String)> = None;

    for raw in input.lines() {
        let line = raw.replace("file://", "").replace(&prefix, "");
        let t = line.trim();
        if t.is_empty() {
            in_try = false;
            continue;
        }
        if let Some((class, method)) = pending.take() {
            g.lines.push(line.trim_end().to_string());
            if let Some(msg) = report_message(cwd, &last_test_task, &class, &method) {
                g.lines.push(format!("        {msg}"));
            }
            continue;
        }
        if after_warning > 0 {
            after_warning -= 1;
            g.cut = true;
            continue;
        }
        if t.starts_with("* Try:") {
            in_try = true;
            continue;
        }
        if in_try && t.starts_with('>') {
            continue;
        }
        in_try = false;
        if t.starts_with("* What went wrong:") {
            in_wrong = true;
            continue;
        }
        if t.chars().all(|c| c == '=' || c == '-') {
            in_wrong = false;
            continue;
        }
        if let Some((task, outcome)) = task_line(t) {
            if outcome == "FAILED" {
                g.lines.push(t.to_string());
                last_test_task = task.to_string();
            } else {
                g.quiet_tasks += 1;
            }
            continue;
        }
        if let Some((class, method)) = failed_test(t) {
            g.lines.push(t.to_string());
            pending = Some((class.to_string(), method.to_string()));
            continue;
        }
        if NOISE_PREFIXES.iter().any(|p| t.starts_with(p))
            || t.contains(" actionable task")
            || t.ends_with(": Task failed with an exception.")
        {
            continue;
        }
        // Warnings echo the source line and a caret; the path:line is enough.
        if t.contains(": warning: ") {
            after_warning = 2;
        }
        // `* What went wrong:` repeats the compiler output, indented by two.
        if in_wrong && seen.contains(t) {
            continue;
        }
        if !in_wrong {
            seen.insert(t.to_string());
        }
        g.lines.push(line.trim_end().to_string());
    }
    g
}

fn format_gradle_compact(g: &Gradle) -> String {
    let mut text = g.lines.join("\n");
    if g.quiet_tasks > 0 && !g.lines.is_empty() {
        text = format!("({} tasks ok)\n{text}", g.quiet_tasks);
    }
    let mut folded = crate::safe_folds::fold_repeats(&text);
    if !folded.is_empty() {
        folded.push('\n');
    }
    folded
}

#[cfg(test)]
#[path = "gradle_tests.rs"]
mod tests;
