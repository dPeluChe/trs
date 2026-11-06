//! Real `node --test` output, which `npm test` and `pnpm test` both relay.
//! Its spec and TAP reporters share one footer (`ℹ tests 4` or `# tests 4`),
//! and that footer is what decides the verdict.

use super::super::types::*;
use crate::path_display::{cwd_prefix, normalize};

#[derive(Default)]
struct Failure {
    file: String,
    name: String,
    error: Option<String>,
}

#[derive(Default)]
pub(crate) struct NodeTestRun {
    tests: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    todo: usize,
    duration: Option<f64>,
    failures: Vec<Failure>,
}

const UNKNOWN_FILE: &str = "node --test";

impl NodeTestRun {
    /// None unless `input` carries the footer, so other reporters keep their parsers.
    pub(crate) fn read(input: &str) -> Option<Self> {
        if !input.contains("ℹ tests ") && !input.contains("# tests ") {
            return None;
        }
        let mut run = Self::default();
        let (mut tests, mut passed) = (None, None);
        for line in input.lines() {
            let line = line.trim();
            let Some(entry) = line.strip_prefix("ℹ ").or(line.strip_prefix("# ")) else {
                continue;
            };
            let Some((key, value)) = entry.split_once(' ') else {
                continue;
            };
            let count = value.parse::<usize>().ok();
            match (key, count) {
                ("tests", Some(_)) => tests = count,
                ("pass", Some(_)) => passed = count,
                ("fail" | "cancelled", Some(n)) => run.failed += n,
                ("skipped", Some(n)) => run.skipped = n,
                ("todo", Some(n)) => run.todo = n,
                ("duration_ms", _) => {
                    run.duration = value.parse::<f64>().ok().map(|ms| ms / 1000.0)
                }
                _ => {}
            }
        }
        run.tests = tests?;
        run.passed = passed?;
        run.failures = failures_of(input, run.failed);
        Some(run)
    }
}

fn failures_of(input: &str, failed: usize) -> Vec<Failure> {
    if failed == 0 {
        return Vec::new();
    }
    let failures = match input.split_once("failing tests:") {
        Some((_, recap)) => spec_failures(recap),
        None => tap_failures(input),
    };
    if failures.is_empty() {
        return vec![Failure {
            file: UNKNOWN_FILE.to_string(),
            ..Failure::default()
        }];
    }
    failures
}

fn file_of(location: &str) -> String {
    let path = normalize(location.rsplitn(3, ':').last().unwrap_or(location));
    let cwd = normalize(&cwd_prefix());
    path.strip_prefix(&cwd).unwrap_or(&path).to_string()
}

fn spec_failures(recap: &str) -> Vec<Failure> {
    let mut failures: Vec<Failure> = Vec::new();
    for line in recap.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(location) = line.strip_prefix("test at ") {
            failures.push(Failure {
                file: file_of(location),
                ..Failure::default()
            });
        } else if let Some(failure) = failures.last_mut() {
            if failure.name.is_empty() {
                let name = line.trim_start_matches('✖').trim();
                failure.name = name.rsplit_once(" (").map_or(name, |(n, _)| n).to_string();
            } else if failure.error.is_none() {
                failure.error = Some(line.to_string());
            }
        }
    }
    failures
}

fn tap_failures(input: &str) -> Vec<Failure> {
    let mut failures: Vec<Failure> = Vec::new();
    let mut lines = input.lines().map(str::trim);
    while let Some(line) = lines.next() {
        if let Some(rest) = line.strip_prefix("not ok ") {
            let name = rest.split_once(" - ").map_or(rest, |(_, n)| n);
            failures.push(Failure {
                file: UNKNOWN_FILE.to_string(),
                name: name.to_string(),
                error: None,
            });
        } else if let Some(failure) = failures.last_mut() {
            if let Some(location) = line.strip_prefix("location: ") {
                failure.file = file_of(location.trim_matches('\''));
            } else if let Some(error) = line.strip_prefix("error: ") {
                let message = if error.starts_with('|') {
                    lines.next()
                } else {
                    Some(error)
                };
                failure.error = message.map(str::to_string);
            }
        }
    }
    failures
}

macro_rules! into_runner_output {
    ($method:ident, $output:ident, $suite:ident, $test:ident, $status:ident) => {
        impl NodeTestRun {
            pub(crate) fn $method(self) -> $output {
                let mut output = $output::default();
                for failure in self.failures {
                    let test = $test {
                        name: failure.name.clone(),
                        test_name: failure.name,
                        ancestors: Vec::new(),
                        status: $status::Failed,
                        duration: None,
                        error_message: failure.error,
                    };
                    match output
                        .test_suites
                        .iter_mut()
                        .find(|s| s.file == failure.file)
                    {
                        Some(suite) => suite.tests.push(test),
                        None => output.test_suites.push($suite {
                            file: failure.file,
                            passed: false,
                            duration: None,
                            tests: vec![test],
                        }),
                    }
                }
                let summary = &mut output.summary;
                summary.suites_total = output.test_suites.len();
                summary.suites_failed = output.test_suites.len();
                summary.tests_total = self.tests;
                summary.tests_passed = self.passed;
                summary.tests_failed = self.failed;
                summary.tests_skipped = self.skipped;
                summary.tests_todo = self.todo;
                summary.duration = self.duration;
                output.is_empty = self.tests == 0;
                output.success = !output.is_empty && self.failed == 0;
                output
            }
        }
    };
}

into_runner_output!(
    into_npm,
    NpmTestOutput,
    NpmTestSuite,
    NpmTest,
    NpmTestStatus
);
into_runner_output!(
    into_pnpm,
    PnpmTestOutput,
    PnpmTestSuite,
    PnpmTest,
    PnpmTestStatus
);
