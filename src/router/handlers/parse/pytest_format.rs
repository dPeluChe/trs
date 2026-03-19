use super::super::types::*;
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Format pytest output based on the requested format.
    pub(crate) fn format_pytest(output: &PytestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_pytest_json(output),
            OutputFormat::Compact => Self::format_pytest_compact(output),
            OutputFormat::Raw => Self::format_pytest_raw(output),
            OutputFormat::Agent => Self::format_pytest_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_pytest_compact(output),
        }
    }

    /// Format pytest output as JSON.
    pub(crate) fn format_pytest_json(output: &PytestOutput) -> String {
        // Extract failing test identifiers
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .map(|t| t.name.clone())
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "passed": output.summary.passed,
                "failed": output.summary.failed,
                "skipped": output.summary.skipped,
                "xfailed": output.summary.xfailed,
                "xpassed": output.summary.xpassed,
                "errors": output.summary.errors,
                "total": output.summary.total,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "tests": output.tests.iter().map(|t| serde_json::json!({
                "name": t.name,
                "status": match t.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                    TestStatus::XFailed => "xfailed",
                    TestStatus::XPassed => "xpassed",
                    TestStatus::Error => "error",
                },
                "duration": t.duration,
                "file": t.file,
                "line": t.line,
                "error_message": t.error_message,
            })).collect::<Vec<_>>(),
            "rootdir": output.rootdir,
            "platform": output.platform,
            "python_version": output.python_version,
            "pytest_version": output.pytest_version,
        })
        .to_string()
    }

    /// Format pytest output in compact format.
    pub(crate) fn format_pytest_compact(output: &PytestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!("PASS: {} tests", output.summary.passed));
            if output.summary.skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.skipped));
            }
            if output.summary.xfailed > 0 {
                result.push_str(&format!(", {} xfailed", output.summary.xfailed));
            }
            if output.summary.xpassed > 0 {
                result.push_str(&format!(", {} xpassed", output.summary.xpassed));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(" [{:.2}s]", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        result.push_str(&format!(
            "FAIL: {} passed, {} failed",
            output.summary.passed, output.summary.failed
        ));
        if output.summary.skipped > 0 {
            result.push_str(&format!(", {} skipped", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!(", {} xfailed", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!(", {} xpassed", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!(", {} errors", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(" [{:.2}s]", duration));
        }
        result.push('\n');

        // List failed tests
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str(&format!("failed ({}):\n", failed_tests.len()));
            for test in failed_tests {
                result.push_str(&format!("  {}\n", test.name));
                if let Some(ref msg) = test.error_message {
                    // Show first line of error message
                    if let Some(first_line) = msg.lines().next() {
                        let truncated = if first_line.len() > 80 {
                            format!("{}...", &first_line[..77])
                        } else {
                            first_line.to_string()
                        };
                        result.push_str(&format!("    {}\n", truncated));
                    }
                }
            }
        }

        result
    }

    /// Format pytest output as raw (just test names with status).
    pub(crate) fn format_pytest_raw(output: &PytestOutput) -> String {
        let mut result = String::new();

        for test in &output.tests {
            let status = match test.status {
                TestStatus::Passed => "PASS",
                TestStatus::Failed => "FAIL",
                TestStatus::Skipped => "SKIP",
                TestStatus::XFailed => "XFAIL",
                TestStatus::XPassed => "XPASS",
                TestStatus::Error => "ERROR",
            };
            result.push_str(&format!("{} {}\n", status, test.name));
        }

        result
    }

    /// Format pytest output for AI agent consumption.
    pub(crate) fn format_pytest_agent(output: &PytestOutput) -> String {
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
        result.push_str(&format!("- Total: {}\n", output.summary.total));
        result.push_str(&format!("- Passed: {}\n", output.summary.passed));
        result.push_str(&format!("- Failed: {}\n", output.summary.failed));
        if output.summary.skipped > 0 {
            result.push_str(&format!("- Skipped: {}\n", output.summary.skipped));
        }
        if output.summary.xfailed > 0 {
            result.push_str(&format!("- XFailed: {}\n", output.summary.xfailed));
        }
        if output.summary.xpassed > 0 {
            result.push_str(&format!("- XPassed: {}\n", output.summary.xpassed));
        }
        if output.summary.errors > 0 {
            result.push_str(&format!("- Errors: {}\n", output.summary.errors));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
        }
        result.push('\n');

        // Failed tests with details
        let failed_tests: Vec<_> = output
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Failed || t.status == TestStatus::Error)
            .collect();

        if !failed_tests.is_empty() {
            result.push_str("## Failed Tests\n\n");
            for test in failed_tests {
                result.push_str(&format!("### {}\n", test.name));
                if let Some(ref file) = test.file {
                    result.push_str(&format!("File: {}", file));
                    if let Some(line) = test.line {
                        result.push_str(&format!(":{}", line));
                    }
                    result.push('\n');
                }
                if let Some(ref msg) = test.error_message {
                    result.push_str(&format!("\n```\n{}\n```\n", msg));
                }
                result.push('\n');
            }
        }

        result
    }
}
