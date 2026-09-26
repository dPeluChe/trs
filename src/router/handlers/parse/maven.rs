//! Parser for Maven output (`mvn`, `./mvnw`).
//!
//! Keeps what an agent acts on: compiler errors and warnings, each failed
//! test with its message and the frames in the project's own code, the
//! resume hint, and the result. Drops download progress, plugin banners,
//! per-class passing test lines and Maven's help boilerplate. Lines it does
//! not recognize (dependency trees, application output) are kept.

use super::super::ansi::strip_ansi_codes;
use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;
use std::collections::HashSet;

/// Stack frames kept per failed test; the rest are counted.
const MAX_APP_FRAMES: usize = 5;

/// Frames from test runners, the JDK and Maven itself, never the bug.
const FRAMEWORK_FRAMES: &[&str] = &[
    "org.junit.",
    "junit.",
    "org.opentest4j.",
    "org.testng.",
    "org.apache.maven.",
    "org.mockito.internal.",
    "java.base/",
    "java.",
    "jdk.",
    "sun.",
];

const NOISE_PREFIXES: &[&str] = &[
    "Scanning for projects",
    "  from ",
    "skip non existing resourceDirectory",
    "Nothing to compile",
    "Recompiling the module",
    "Compiling ",
    "Using auto detected provider",
    "Using 'UTF-8' encoding",
    "Copying ",
    "Installing ",
    "Deleting ",
    " T E S T S",
    "Running ",
    "Results:",
    "Finished at:",
    "Reactor Build Order:",
    "No tests to run",
    "COMPILATION ERROR",
    "-> [Help ",
    "[Help ",
    "To see the full stack trace",
    "Re-run Maven using the -X",
    "For more information about the errors",
    "Please refer to ",
    "After correcting the problems",
];

#[derive(Default)]
pub(crate) struct Maven {
    pub(crate) lines: Vec<String>,
    pub(crate) build: Option<String>,
    pub(crate) time: Option<String>,
    /// run, failures, errors, skipped
    pub(crate) tests: Option<[usize; 4]>,
    pub(crate) downloads: usize,
    pub(crate) modules: Vec<(String, String)>,
    pub(crate) frames_cut: usize,
    /// App log lines at INFO or below, banners and JVM notices printed by tests.
    pub(crate) hidden_logs: usize,
}

impl ParseHandler {
    pub(crate) fn handle_maven(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = strip_ansi_codes(&Self::read_input(file)?);
        let cwd = std::env::current_dir()
            .map(|d| format!("{}/", d.to_string_lossy()))
            .unwrap_or_default();
        let m = parse_maven(&input, &cwd);
        if m.frames_cut > 0 || m.hidden_logs > 0 {
            crate::parse_out::mark_dropped();
        }
        let output = match ctx.format {
            OutputFormat::Json => format_maven_json(&m),
            _ => format_maven_compact(&m),
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("maven")
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(m.lines.len())
                .print();
        }
        Ok(())
    }
}

#[derive(PartialEq)]
enum Level {
    Info,
    Warning,
    Error,
    None,
}

fn split_level(line: &str) -> (Level, &str) {
    for (tag, level) in [
        ("[INFO]", Level::Info),
        ("[WARNING]", Level::Warning),
        ("[WARN]", Level::Warning),
        ("[ERROR]", Level::Error),
    ] {
        if let Some(rest) = line.strip_prefix(tag) {
            return (level, rest.strip_prefix(' ').unwrap_or(rest));
        }
    }
    (Level::None, line)
}

/// `Tests run: 4, Failures: 1, Errors: 0, Skipped: 0` into its four counts.
fn test_counts(body: &str) -> Option<[usize; 4]> {
    let rest = body.trim_start().strip_prefix("Tests run: ")?;
    let mut out = [0usize; 4];
    for (i, part) in rest.split(", ").take(4).enumerate() {
        let n = part.rsplit(' ').next()?;
        let n = n.split(|c: char| !c.is_ascii_digit()).next()?;
        out[i] = n.parse().ok()?;
    }
    Some(out)
}

/// `com.acme.CartTest.discount -- Time elapsed: 0.01 s <<< FAILURE!` (surefire 3)
/// or `discount(com.acme.CartTest)  Time elapsed: 0.01 sec  <<< FAILURE!` (2.x),
/// as `CartTest.discount` plus the kind.
fn failed_test(body: &str) -> Option<(String, &'static str)> {
    let kind = if body.contains("<<< FAILURE!") {
        "FAIL"
    } else if body.contains("<<< ERROR!") {
        "ERROR"
    } else {
        return None;
    };
    if body.trim_start().starts_with("Tests run:") {
        return None;
    }
    let head = body.split(" -- ").next()?.split("  ").next()?.trim();
    let name = match head.split_once('(') {
        Some((method, class)) => format!("{}.{}", short_class(class.trim_end_matches(')')), method),
        None => {
            let (class, method) = head.rsplit_once('.')?;
            format!("{}.{}", short_class(class), method)
        }
    };
    Some((name, kind))
}

fn quiet_log_re() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(
            r"^(\d{4}-\d{2}-\d{2}[T ])?\d{2}:\d{2}:\d{2}[.,]\d{3}\S*\s.{0,40}?\b(TRACE|DEBUG|INFO)\b",
        )
        .expect("static log pattern")
    })
}

/// The JVM's own notices (agents, native access, Unsafe, reflective access).
/// Not every `WARNING: ` line: java.util.logging prints app warnings that way.
const JVM_NOTICES: &[&str] = &[
    "WARNING: A restricted method",
    "WARNING: java.lang.System::load",
    "WARNING: Use --enable-native-access",
    "WARNING: Restricted methods will be blocked",
    "WARNING: A Java agent has been loaded dynamically",
    "WARNING: If a serviceability tool",
    "WARNING: Dynamic loading of agents",
    "WARNING: A terminally deprecated method in sun.misc.Unsafe",
    "WARNING: sun.misc.Unsafe::",
    "WARNING: Please consider reporting this to the maintainers",
    "WARNING: An illegal reflective access",
    "WARNING: Illegal reflective access by",
    "WARNING: Use --illegal-access=warn",
    "WARNING: All illegal access operations will be denied",
];

fn is_jvm_notice(t: &str) -> bool {
    JVM_NOTICES.iter().any(|p| t.starts_with(p)) || t.contains(" VM warning: ")
}

/// Output a test prints that is not about the test result: framework logs at
/// INFO or below and the Spring banner.
fn is_test_chatter(t: &str) -> bool {
    if t.starts_with(":: Spring Boot ::") || quiet_log_re().is_match(t) {
        return true;
    }
    let visible = t.chars().filter(|c| !c.is_whitespace()).count();
    let alnum = t.chars().filter(|c| c.is_alphanumeric()).count();
    alnum * 10 < visible * 3
}

fn short_class(fqcn: &str) -> &str {
    fqcn.rsplit('.').next().unwrap_or(fqcn)
}

/// `api ........ FAILURE [  0.555 s]` as (`api`, `FAILURE`).
fn reactor_row(body: &str) -> Option<(String, String)> {
    let (name, rest) = body.split_once(" .")?;
    let status = rest.trim_start_matches(['.', ' ']).split(' ').next()?;
    matches!(status, "SUCCESS" | "FAILURE" | "SKIPPED")
        .then(|| (name.trim().to_string(), status.to_string()))
}

pub(crate) fn parse_maven(input: &str, cwd: &str) -> Maven {
    let mut m = Maven::default();
    let mut detailed: HashSet<String> = HashSet::new();
    let mut compile_errors: HashSet<String> = HashSet::new();
    let mut in_compile_errors = false;
    let mut after_goal_failure = false;
    let mut in_reactor = false;
    let mut in_tests = false;
    let mut stack: Option<(usize, usize)> = None; // (frames kept, frames cut)

    for raw in input.lines() {
        // Progress bars redraw with `\r`; only the last redraw is real.
        let line = raw.rsplit('\r').next().unwrap_or(raw);
        let (level, body) = split_level(line.trim_end());
        let short = if cwd.len() > 1 {
            body.replace(cwd, "")
        } else {
            body.to_string()
        };

        if let Some((kept, cut)) = stack.as_mut() {
            if level == Level::None && !body.trim().is_empty() {
                let t = body.trim_start();
                if let Some(frame) = t.strip_prefix("at ") {
                    if FRAMEWORK_FRAMES.iter().any(|p| frame.starts_with(p))
                        || *kept >= MAX_APP_FRAMES
                    {
                        *cut += 1;
                    } else {
                        *kept += 1;
                        m.lines.push(format!("    at {frame}"));
                    }
                } else if !(t.starts_with("...") && t.ends_with(" more")) {
                    m.lines.push(format!("  {}", short.trim()));
                }
                continue;
            }
            if *cut > 0 {
                m.lines.push(format!("    (+{cut} frames)"));
                m.frames_cut += *cut;
            }
            stack = None;
        }

        let t = body.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("T E S T S") {
            in_tests = true;
        } else if t.starts_with("Results:") || t.starts_with("BUILD ") {
            in_tests = false;
        }
        if level == Level::None && (is_jvm_notice(t) || in_tests && is_test_chatter(t)) {
            m.hidden_logs += 1;
            continue;
        }
        if t.starts_with("Downloading from ") || t.starts_with("Progress (") {
            continue;
        }
        if t.starts_with("Downloaded from ") {
            m.downloads += 1;
            continue;
        }
        if let Some(counts) = test_counts(t) {
            if !t.contains(" -- in ") && !t.contains("Time elapsed") {
                let acc = m.tests.get_or_insert([0; 4]);
                for (a, c) in acc.iter_mut().zip(counts) {
                    *a += c;
                }
            }
            continue;
        }
        if let Some((name, kind)) = failed_test(&short) {
            m.lines.push(format!("{kind} {name}"));
            detailed.insert(name);
            stack = Some((0, 0));
            continue;
        }
        if let Some(result) = t.strip_prefix("BUILD ") {
            m.build = Some(result.to_string());
            continue;
        }
        if let Some(time) = t.strip_prefix("Total time:") {
            m.time = Some(time.trim().to_string());
            continue;
        }
        if t.starts_with("Reactor Summary") {
            in_reactor = true;
            continue;
        }
        if in_reactor {
            if let Some(row) = reactor_row(t) {
                m.modules.push(row);
                continue;
            }
            in_reactor = false;
        }
        if level == Level::Info
            && (t.starts_with("--")
                || t.starts_with("Building ")
                    && !t.starts_with("Building jar")
                    && !t.starts_with("Building war"))
        {
            continue;
        }
        // Reactor build order rows: `core        [jar]`.
        if level == Level::Info && t.ends_with(']') && t.contains("  [") && !t.contains(": ") {
            continue;
        }
        if NOISE_PREFIXES
            .iter()
            .any(|p| t.starts_with(p.trim_start()) || body.starts_with(p))
        {
            if t.starts_with("COMPILATION ERROR") {
                in_compile_errors = true;
            }
            continue;
        }
        if level == Level::Info
            && (t.ends_with(" error") || t.ends_with(" errors"))
            && t.split(' ')
                .next()
                .is_some_and(|n| n.parse::<usize>().is_ok())
        {
            in_compile_errors = false;
            continue;
        }
        if t.starts_with("Recompile with -Xlint") || t.contains(": Recompile with -Xlint") {
            continue;
        }
        if let Some(rest) = t.strip_prefix("mvn <args> -rf ") {
            m.lines.push(format!("resume with: mvn <args> -rf {rest}"));
            continue;
        }
        // Results list entries repeat the detailed failure above them.
        if level == Level::Error {
            if matches!(
                t,
                "Failures:" | "Errors:" | "Tests in error:" | "Failed tests:"
            ) {
                continue;
            }
            if body.starts_with("  ") {
                let key = t.split([':', ' ', '(']).next().unwrap_or("");
                if detailed.contains(key) {
                    continue;
                }
            }
        }
        let text = short.trim_end();
        // javac notes repeat the path: `A.java: A.java uses unchecked ...`.
        let text = match text.split_once(": ") {
            Some((p, rest)) if rest.starts_with(p) => rest,
            _ => text,
        };
        let text = text.replace("org.apache.maven.plugins:", "");
        if in_compile_errors {
            compile_errors.insert(text.trim().to_string());
        } else if text.contains("Compilation failure") {
            after_goal_failure = true;
        } else if after_goal_failure && compile_errors.contains(text.trim()) {
            continue;
        }
        let tag = match level {
            Level::Error => "[ERROR] ",
            Level::Warning => "[WARNING] ",
            _ => "",
        };
        m.lines.push(format!("{tag}{text}"));
    }
    if let Some((_, cut)) = stack {
        if cut > 0 {
            m.lines.push(format!("    (+{cut} frames)"));
            m.frames_cut += cut;
        }
    }
    m
}

fn format_maven_compact(m: &Maven) -> String {
    let mut out: Vec<String> = Vec::new();
    if m.downloads > 0 {
        out.push(format!("downloaded {} files", m.downloads));
    }
    out.extend(m.lines.iter().cloned());
    if m.hidden_logs > 0 {
        out.push(format!(
            "(hid {} lines: test logs at INFO/DEBUG, banner, JVM notices)",
            m.hidden_logs
        ));
    }
    if let Some([run, failures, errors, skipped]) = m.tests {
        out.push(format!(
            "tests: {run} run, {failures} failed, {errors} errors, {skipped} skipped"
        ));
    }
    if !m.modules.is_empty() {
        let bad: Vec<String> = m
            .modules
            .iter()
            .filter(|(_, s)| s != "SUCCESS")
            .map(|(n, s)| format!("{n} {s}"))
            .collect();
        out.push(if bad.is_empty() {
            format!("modules: {} SUCCESS", m.modules.len())
        } else {
            format!("modules: {} ({} total)", bad.join(", "), m.modules.len())
        });
    }
    if let Some(build) = &m.build {
        out.push(match &m.time {
            Some(t) => format!("BUILD {build} ({t})"),
            None => format!("BUILD {build}"),
        });
    }
    let text = out.join("\n");
    let mut folded = crate::safe_folds::fold_repeats(&text);
    if !folded.is_empty() {
        folded.push('\n');
    }
    folded
}

fn format_maven_json(m: &Maven) -> String {
    let tests = m.tests.map(|[run, failures, errors, skipped]| {
        serde_json::json!({ "run": run, "failures": failures, "errors": errors, "skipped": skipped })
    });
    let modules: Vec<_> = m
        .modules
        .iter()
        .map(|(n, s)| serde_json::json!({ "name": n, "status": s }))
        .collect();
    serde_json::json!({
        "build": m.build,
        "time": m.time,
        "tests": tests,
        "modules": modules,
        "downloads": m.downloads,
        "hidden_logs": m.hidden_logs,
        "lines": m.lines,
    })
    .to_string()
        + "\n"
}

#[cfg(test)]
#[path = "maven_tests.rs"]
mod tests;
