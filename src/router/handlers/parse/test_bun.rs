use super::super::common::CommandResult;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    // ============================================================
    // Bun Test Parsing and Formatting
    // ============================================================

    /// Parse Bun test output into structured data.
    ///
    /// Expected format (default console reporter):
    /// ```text
    /// test/package-json-lint.test.ts:
    /// ✓ test/package.json [0.88ms]
    /// ✓ test/js/third_party/grpc-js/package.json [0.18ms]
    ///
    ///  4 pass
    ///  0 fail
    ///  4 expect() calls
    /// Ran 4 tests in 1.44ms
    /// ```
    ///
    /// For non-TTY environments (no colors):
    /// ```text
    /// test/package-json-lint.test.ts:
    /// (pass) test/package.json [0.48ms]
    /// (fail) test/failing.test.ts
    /// (skip) test/skipped.test.ts
    /// ```
    pub(crate) fn parse_bun_test(input: &str) -> CommandResult<BunTestOutput> {
        let mut output = BunTestOutput::default();
        let mut current_suite: Option<BunTestSuite> = None;
        let mut current_test: Option<BunTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new();
        let mut in_suite = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines, but first save any pending test
            if trimmed.is_empty() {
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                in_error_details = false;
                continue;
            }

            // Check for bun version line (e.g., "bun: 1.0.0" or "Bun v1.0.0")
            if trimmed.starts_with("bun:") || trimmed.starts_with("Bun v") {
                output.bun_version = Some(
                    trimmed
                        .split(|c| c == ':' || c == 'v')
                        .last()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }

            // Check for summary lines at the end
            // "X pass" or "Y fail" or "X expect() calls"
            if Self::is_bun_summary_line(trimmed) {
                // Save any pending test before processing summary
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }
                Self::parse_bun_summary_line(trimmed, &mut output.summary);
                continue;
            }

            // "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
            if trimmed.starts_with("Ran ") && trimmed.contains(" tests") {
                Self::parse_bun_ran_line(trimmed, &mut output.summary);
                continue;
            }

            // Check for test file header: "test/file.test.ts:" (ends with colon)
            if trimmed.ends_with(':')
                && !trimmed.starts_with(|c| c == '✓' || c == '✗' || c == '×' || c == '(')
            {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == BunTestStatus::Failed);
                    let suite_to_save = BunTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }

                let file = trimmed.trim_end_matches(':').to_string();
                current_suite = Some(BunTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                in_suite = true;
                continue;
            }

            // Parse test results if we're in a suite
            if in_suite && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✓")
                    && !trimmed.starts_with("✗")
                    && !trimmed.starts_with("×")
                    && !trimmed.starts_with("(pass)")
                    && !trimmed.starts_with("(fail)")
                    && !trimmed.starts_with("(skip)")
                    && !trimmed.starts_with("(todo)")
                {
                    if let Some(ref mut test) = current_test {
                        if !error_buffer.is_empty() {
                            error_buffer.push('\n');
                        }
                        error_buffer.push_str(trimmed);
                        test.error_message = Some(error_buffer.clone());
                    }
                    continue;
                }

                // Save previous test if we're starting a new one at same or lower indent
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                // Parse test line
                if let Some(test) = Self::parse_bun_test_line(trimmed, &indent_stack) {
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == BunTestStatus::Failed;

                    // Check for failed test to start collecting error details
                    if is_failed {
                        in_error_details = true;
                        error_buffer.clear();
                        current_test = Some(test);
                    } else {
                        in_error_details = false;
                        if let Some(ref mut suite) = current_suite {
                            suite.tests.push(test);
                        }
                    }

                    // Track nested test names
                    indent_stack.push(test_name);
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test {
            if let Some(ref mut suite) = current_suite {
                suite.tests.push(test);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            let has_failures = suite
                .tests
                .iter()
                .any(|t| t.status == BunTestStatus::Failed);
            let suite_to_save = BunTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_bun_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single Bun test result line.
    pub(crate) fn parse_bun_test_line(line: &str, ancestors: &[String]) -> Option<BunTest> {
        let line = line.trim_start();

        // Parse with color markers: "✓ test name [5.123ms]"
        if line.starts_with("✓") {
            let rest = line.strip_prefix("✓").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test with color markers: "✗ test name" or "× test name"
        if line.starts_with("✗") || line.starts_with("×") {
            let rest = line
                .strip_prefix("✗")
                .or_else(|| line.strip_prefix("×"))
                .unwrap_or(line)
                .trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(pass) test name [5.123ms]"
        if line.starts_with("(pass)") {
            let rest = line.strip_prefix("(pass)").unwrap_or(line).trim();
            let (name, duration) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(fail) test name"
        if line.starts_with("(fail)") {
            let rest = line.strip_prefix("(fail)").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(skip) test name"
        if line.starts_with("(skip)") {
            let rest = line.strip_prefix("(skip)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse non-TTY format: "(todo) test name"
        if line.starts_with("(todo)") {
            let rest = line.strip_prefix("(todo)").unwrap_or(line).trim();
            let (name, _) = Self::split_bun_test_name_and_duration(rest);
            return Some(BunTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: BunTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Parse duration string like "5.123ms" or "1.234s" into seconds.
    pub(crate) fn parse_bun_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            s.strip_suffix("ms")
                .and_then(|n| n.parse::<f64>().ok())
                .map(|ms| ms / 1000.0)
        } else if s.ends_with("s") {
            s.strip_suffix("s").and_then(|n| n.parse::<f64>().ok())
        } else {
            None
        }
    }

    /// Split test name and duration from a string like "test name [5.123ms]".
    pub(crate) fn split_bun_test_name_and_duration(s: &str) -> (String, Option<f64>) {
        // Look for duration in brackets at the end: "test name [5.123ms]"
        if let Some(start) = s.rfind('[') {
            if let Some(end) = s[start..].find(']') {
                let duration_str = &s[start + 1..start + end];
                let name = s[..start].trim().to_string();
                let duration = Self::parse_bun_duration(duration_str);
                return (name, duration);
            }
        }
        (s.to_string(), None)
    }

    /// Check if a line is a Bun summary line.
    pub(crate) fn is_bun_summary_line(line: &str) -> bool {
        let line = line.trim();
        // Match "X pass", "Y fail", "Z expect() calls", "W skipped"
        // These lines start with a number, not a test marker
        // Examples: " 4 pass", " 0 fail", " 4 expect() calls"
        // NOT: "✓ test pass" or "✗ should fail"

        // First check if line starts with a number (possibly with leading spaces)
        let starts_with_number = line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        if !starts_with_number {
            return false;
        }

        line.ends_with(" pass")
            || line.ends_with(" fail")
            || line.ends_with(" expect() calls")
            || line.ends_with(" skipped")
    }

    /// Parse a Bun summary line.
    pub(crate) fn parse_bun_summary_line(line: &str, summary: &mut BunTestSummary) {
        let line = line.trim();

        // Parse "X pass"
        if line.ends_with(" pass") {
            if let Some(count_str) = line.strip_suffix(" pass") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_passed = count;
                }
            }
            return;
        }

        // Parse "Y fail"
        if line.ends_with(" fail") {
            if let Some(count_str) = line.strip_suffix(" fail") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_failed = count;
                }
            }
            return;
        }

        // Parse "Z expect() calls"
        if line.ends_with(" expect() calls") {
            if let Some(count_str) = line.strip_suffix(" expect() calls") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.expect_calls = Some(count);
                }
            }
            return;
        }

        // Parse "X skipped"
        if line.ends_with(" skipped") {
            if let Some(count_str) = line.strip_suffix(" skipped") {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    summary.tests_skipped = count;
                }
            }
        }
    }

    /// Parse "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
    pub(crate) fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary) {
        // Format: "Ran X tests in Yms" or "Ran X tests across Y files. [Zms]"
        let line = line.trim();

        // Extract total tests
        if let Some(start) = line.find("Ran ") {
            let after_ran = &line[start + 4..];
            if let Some(end) = after_ran.find(" tests") {
                if let Ok(count) = after_ran[..end].trim().parse::<usize>() {
                    summary.tests_total = count;
                }
            }
        }

        // Extract files count
        if let Some(start) = line.find("across ") {
            let after_across = &line[start + 7..];
            if let Some(end) = after_across.find(" files") {
                if let Ok(count) = after_across[..end].trim().parse::<usize>() {
                    summary.suites_total = count;
                }
            }
        }

        // Extract duration - format: "in 1.44ms" or "[1.44ms]"
        if let Some(start) = line.find("in ") {
            let after_in = &line[start + 3..];
            summary.duration = Self::parse_bun_duration(after_in);
        } else if let Some(start) = line.rfind('[') {
            if let Some(end) = line[start..].find(']') {
                let duration_str = &line[start + 1..start + end];
                summary.duration = Self::parse_bun_duration(duration_str);
            }
        }
    }

    /// Update summary counts from parsed tests.
    pub(crate) fn update_bun_summary_from_tests(output: &mut BunTestOutput) {
        // Always update suite counts since they may not be in the "Ran" line
        // (the "across X files" part is optional)
        if output.summary.suites_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }
            }
        }

        // Only update test counts if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        BunTestStatus::Passed => output.summary.tests_passed += 1,
                        BunTestStatus::Failed => output.summary.tests_failed += 1,
                        BunTestStatus::Skipped => output.summary.tests_skipped += 1,
                        BunTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format Bun test output based on the requested format.
    pub(crate) fn format_bun_test(output: &BunTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_bun_test_json(output),
            OutputFormat::Compact => Self::format_bun_test_compact(output),
            OutputFormat::Raw => Self::format_bun_test_raw(output),
            OutputFormat::Agent => Self::format_bun_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_bun_test_compact(output),
        }
    }

    /// Format Bun test output as JSON.
    pub(crate) fn format_bun_test_json(output: &BunTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == BunTestStatus::Failed)
                    .map(|t| format!("{}::{}", suite.file, t.name))
            })
            .collect();

        serde_json::json!({
            "success": output.success,
            "is_empty": output.is_empty,
            "summary": {
                "suites_passed": output.summary.suites_passed,
                "suites_failed": output.summary.suites_failed,
                "suites_skipped": output.summary.suites_skipped,
                "suites_total": output.summary.suites_total,
                "tests_passed": output.summary.tests_passed,
                "tests_failed": output.summary.tests_failed,
                "tests_skipped": output.summary.tests_skipped,
                "tests_todo": output.summary.tests_todo,
                "tests_total": output.summary.tests_total,
                "expect_calls": output.summary.expect_calls,
                "duration": output.summary.duration,
            },
            "failed_tests": failed_tests,
            "test_suites": output.test_suites.iter().map(|suite| serde_json::json!({
                "file": suite.file,
                "passed": suite.passed,
                "duration": suite.duration,
                "tests": suite.tests.iter().map(|t| serde_json::json!({
                    "name": t.name,
                    "test_name": t.test_name,
                    "ancestors": t.ancestors,
                    "status": match t.status {
                        BunTestStatus::Passed => "passed",
                        BunTestStatus::Failed => "failed",
                        BunTestStatus::Skipped => "skipped",
                        BunTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "bun_version": output.bun_version,
        })
        .to_string()
    }

    /// Format Bun test output in compact format.
    pub(crate) fn format_bun_test_compact(output: &BunTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("bun test: no tests found\n");
            return result;
        }

        // Compact success summary - minimal output when all tests pass
        if output.success {
            result.push_str(&format!(
                "PASS: {} suites, {} tests",
                output.summary.suites_total, output.summary.tests_passed
            ));
            if output.summary.tests_skipped > 0 {
                result.push_str(&format!(", {} skipped", output.summary.tests_skipped));
            }
            if let Some(duration) = output.summary.duration {
                result.push_str(&format!(", {:.2}s", duration));
            }
            result.push('\n');
            return result;
        }

        // Failure-focused summary - detailed output when tests fail
        // Group by passed/failed suites
        let passed_suites: Vec<_> = output.test_suites.iter().filter(|s| s.passed).collect();
        let failed_suites: Vec<_> = output.test_suites.iter().filter(|s| !s.passed).collect();

        // Show failed suites first
        for suite in &failed_suites {
            result.push_str(&format!(
                "FAIL: {} ({} tests)\n",
                suite.file,
                suite.tests.len()
            ));
            for test in &suite.tests {
                if test.status == BunTestStatus::Failed {
                    result.push_str(&format!("  ✖ {}\n", test.test_name));
                }
            }
        }

        // Show passed suites summary
        if !passed_suites.is_empty() {
            result.push_str(&format!(
                "PASS: {} suites, {} tests\n",
                passed_suites.len(),
                passed_suites.iter().map(|s| s.tests.len()).sum::<usize>()
            ));
        }

        // Summary line
        result.push_str(&format!(
            "\n[FAIL] {} suites ({} passed, {} failed), {} tests ({} passed, {} failed)",
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

        if let Some(duration) = output.summary.duration {
            result.push_str(&format!(", {:.2}s", duration));
        }

        result.push('\n');

        result
    }

    /// Format Bun test output as raw (just test names with status).
    pub(crate) fn format_bun_test_raw(output: &BunTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    BunTestStatus::Passed => "PASS",
                    BunTestStatus::Failed => "FAIL",
                    BunTestStatus::Skipped => "SKIP",
                    BunTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format Bun test output for AI agent consumption.
    pub(crate) fn format_bun_test_agent(output: &BunTestOutput) -> String {
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
        if let Some(expect_calls) = output.summary.expect_calls {
            result.push_str(&format!("- Expect() calls: {}\n", expect_calls));
        }
        if let Some(duration) = output.summary.duration {
            result.push_str(&format!("- Duration: {:.2}s\n", duration));
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
                    .filter(|t| t.status == BunTestStatus::Failed)
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
