use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    /// Format Vitest output based on the requested format.
    pub(crate) fn format_vitest(output: &VitestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_vitest_json(output),
            OutputFormat::Compact => Self::format_vitest_compact(output),
            OutputFormat::Raw => Self::format_vitest_raw(output),
            OutputFormat::Agent => Self::format_vitest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_vitest_compact(output),
        }
    }

    /// Format Vitest output as JSON.
    pub(crate) fn format_vitest_json(output: &VitestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites": {
                    "passed": output.summary.suites_passed,
                    "failed": output.summary.suites_failed,
                    "total": output.summary.suites_total,
                },
                "tests": {
                    "passed": output.summary.tests_passed,
                    "failed": output.summary.tests_failed,
                    "skipped": output.summary.tests_skipped,
                    "todo": output.summary.tests_todo,
                    "total": output.summary.tests_total,
                },
                "duration": output.summary.duration,
                "start_at": output.summary.start_at,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "test_count": suite.test_count,
                "skipped_count": suite.skipped_count,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        VitestTestStatus::Passed => "passed",
                        VitestTestStatus::Failed => "failed",
                        VitestTestStatus::Skipped => "skipped",
                        VitestTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "vitest_version": output.vitest_version,
        })
        .to_string()
    }

    /// Format Vitest output in compact format.
    pub(crate) fn format_vitest_compact(output: &VitestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} test files, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if output.summary.tests_todo > 0 {
                result.push_str(&format!(", {} todo", output.summary.tests_todo));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} test files ({} passed, {} failed), {} tests ({} passed, {} failed)",
            output.summary.suites_total,
            output.summary.suites_passed,
            output.summary.suites_failed,
            output.summary.tests_total,
            output.summary.tests_passed,
            output.summary.tests_failed
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!(", {} todo", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed test suites
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str(&format!("failed suites ({}):\n", failed_suites.len()));
            for suite in failed_suites {
                result.push_str(&format!("  {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("    ✕ {}\n", test.name));
                    if let Some(ref msg) = test.error_message {
                        if let Some(first_line) = msg.lines().next() {
                            let truncated = if first_line.len() > 80 {
                                format!("{}...", &first_line[..77])
                            } else {
                                first_line.to_string()
                            };
                            result.push_str(&format!("      {}\n", truncated));
                        }
                    }
                }
            }
        }

        result
    }

    /// Format Vitest output as raw (just test names with status).
    pub(crate) fn format_vitest_raw(output: &VitestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let suite_status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", suite_status, suite.file));
            for test in &suite.tests {
                let status = match test.status {
                    VitestTestStatus::Passed => "PASS",
                    VitestTestStatus::Failed => "FAIL",
                    VitestTestStatus::Skipped => "SKIP",
                    VitestTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", status, test.name));
            }
        }

        result
    }

    /// Format Vitest output for AI agent consumption.
    pub(crate) fn format_vitest_agent(output: &VitestOutput) -> String {
        let mut result = String::new();

        result.push_str("# Test Results\n\n");

        if output.is_empty {
            result.push_str("Status: NO_TESTS\n");
            return result;
        }

        let status = if output.success { "SUCCESS" } else { "FAILURE" };
        result.push_str(&format!("Status: {}\n\n", status));

        // Summary
        result.push_str("## Summary\n");
        result.push_str(&format!(
            "- Test Files: {} passed, {} failed, {} total\n",
            output.summary.suites_passed, output.summary.suites_failed, output.summary.suites_total
        ));
        result.push_str(&format!(
            "- Tests: {} passed, {} failed, {} total\n",
            output.summary.tests_passed, output.summary.tests_failed, output.summary.tests_total
        ));
        if output.summary.tests_skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.tests_skipped));
        }
        if output.summary.tests_todo > 0 {
            result.push_str(&format!("- Todo: {}\n", output.summary.tests_todo));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        if let Some(ref start_at) = output.summary.start_at {
            result.push_str(&format!("- Start at: {}\n", start_at));
        }
        result.push('\n');

        // Failed tests with details
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        if !failed_suites.is_empty() {
            result.push_str("## Failed Test Files\n\n");
            for suite in failed_suites {
                result.push_str(&format!("### {}\n", suite.file));
                let failed_tests: Vec<_> = suite
                    .tests
                    .iter()
                    .filter(|t| t.status == VitestTestStatus::Failed)
                    .collect();
                for test in failed_tests {
                    result.push_str(&format!("- {}", test.name));
                    if let Some(duration) = test.duration {
                        result.push_str(&format!(" ({:.2}s)", duration));
                    }
                    result.push('\n');
                    if let Some(ref msg) = test.error_message {
                        result.push_str(&format!("\n```\n{}\n```\n", msg));
                    }
                }
                result.push('\n');
            }
        }

        result
    }
}
