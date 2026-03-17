use super::super::common::CommandResult;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    // ============================================================
    // NPM Test (Node.js built-in test runner) Parser
    // ============================================================

    /// Parse npm test output into structured data.
    ///
    /// The Node.js built-in test runner (node --test) with spec reporter outputs:
    /// ```text
    /// ▶ test/file.test.js
    ///   ✔ test name (5.123ms)
    ///   ✖ failing test
    ///     code: ...
    ///   ℹ skipped test # SKIP
    ///   ℹ todo test # TODO
    /// ▶ test/file.test.js (12.345ms)
    /// ```
    pub(crate) fn parse_npm_test(input: &str) -> CommandResult<NpmTestOutput> {
        let mut output = NpmTestOutput::default();
        let mut current_suite: Option<NpmTestSuite> = None;
        let mut current_test: Option<NpmTest> = None;
        let mut in_error_details = false;
        let mut error_buffer = String::new();
        let mut indent_stack: Vec<String> = Vec::new(); // Track nested test names

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for npm test output header (e.g., "> project@1.0.0 test")
            if trimmed.starts_with('>') && trimmed.contains("test") {
                continue;
            }

            // Check for summary lines at the end
            // "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
            if trimmed.starts_with("✔ tests") || trimmed.starts_with("✖ tests") {
                Self::parse_npm_test_summary_tests(trimmed, &mut output.summary);
                continue;
            }

            // "✔ test files 2 passed (2)" or "✖ test files 1 failed (2)"
            if trimmed.starts_with("✔ test files") || trimmed.starts_with("✖ test files") {
                Self::parse_npm_test_summary_files(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ tests 4 passed (4)" (alternative format)
            if trimmed.starts_with("ℹ tests") {
                Self::parse_npm_test_summary_tests_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ test files 2 passed (2)" (alternative format)
            if trimmed.starts_with("ℹ test files") {
                Self::parse_npm_test_summary_files_info(trimmed, &mut output.summary);
                continue;
            }

            // "ℹ duration 123ms" or "ℹ duration 1.234s"
            if trimmed.starts_with("ℹ duration") {
                let duration_str = trimmed.strip_prefix("ℹ duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_npm_duration(duration_str);
                continue;
            }

            // Check for test file start: "▶ path/to/test.js"
            if trimmed.starts_with('▶') && !trimmed.contains('(') {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let file = trimmed
                    .strip_prefix('▶')
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                current_suite = Some(NpmTestSuite {
                    file,
                    passed: true,
                    duration: None,
                    tests: Vec::new(),
                });
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Check for test file end with duration: "▶ path/to/test.js (123.456ms)"
            if trimmed.starts_with('▶') && trimmed.contains('(') {
                let duration = Self::extract_npm_suite_duration(trimmed);

                // First, save any pending test
                if let Some(test) = current_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                }

                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                // Save the suite
                if let Some(suite) = current_suite.take() {
                    // Update suite passed status based on tests
                    let has_failures = suite
                        .tests
                        .iter()
                        .any(|t| t.status == NpmTestStatus::Failed);
                    let suite_to_save = NpmTestSuite {
                        passed: !has_failures,
                        ..suite
                    };
                    output.test_suites.push(suite_to_save);
                }
                indent_stack.clear();
                in_error_details = false;
                continue;
            }

            // Parse test results
            // Check if line is inside a test suite (indented or starts with test marker)
            let is_test_line = line.starts_with("  ")
                || line.starts_with("\t")
                || trimmed.starts_with("✔")
                || trimmed.starts_with("✖")
                || trimmed.starts_with("ℹ");

            if is_test_line && current_suite.is_some() {
                // Count indentation level (2 spaces per level)
                let indent = line.chars().take_while(|&c| c == ' ').count() / 2;

                // Adjust indent stack
                while indent_stack.len() > indent {
                    indent_stack.pop();
                }

                // Handle error details (indented more than test line, no marker)
                if in_error_details
                    && !trimmed.starts_with("✔")
                    && !trimmed.starts_with("✖")
                    && !trimmed.starts_with("ℹ")
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
                if let Some(test) = Self::parse_npm_test_line(trimmed, &indent_stack) {
                    // Extract test_name before moving
                    let test_name = test.test_name.clone();
                    let is_failed = test.status == NpmTestStatus::Failed;

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
                .any(|t| t.status == NpmTestStatus::Failed);
            let suite_to_save = NpmTestSuite {
                passed: !has_failures,
                ..suite
            };
            output.test_suites.push(suite_to_save);
        }

        // Set output properties
        output.is_empty = output.test_suites.is_empty();
        output.success = output.test_suites.iter().all(|s| s.passed);

        // Update summary counts from parsed tests
        Self::update_npm_summary_from_tests(&mut output);

        Ok(output)
    }

    /// Parse a single npm test result line.
    pub(crate) fn parse_npm_test_line(line: &str, ancestors: &[String]) -> Option<NpmTest> {
        let line = line.trim_start();

        // Parse passed test: "✔ test name (5.123ms)"
        if line.starts_with("✔") {
            let rest = line.strip_prefix("✔").unwrap_or(line).trim();
            let (name, duration) = Self::split_npm_test_name_and_duration(rest);
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Passed,
                duration,
                error_message: None,
            });
        }

        // Parse failed test: "✖ test name"
        if line.starts_with("✖") {
            let rest = line.strip_prefix("✖").unwrap_or(line).trim();
            let name = rest.to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Failed,
                duration: None,
                error_message: None,
            });
        }

        // Parse skipped test: "ℹ test name # SKIP"
        if line.starts_with("ℹ") && line.contains("# SKIP") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# SKIP")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Skipped,
                duration: None,
                error_message: None,
            });
        }

        // Parse todo test: "ℹ test name # TODO"
        if line.starts_with("ℹ") && line.contains("# TODO") {
            let rest = line.strip_prefix("ℹ").unwrap_or(line).trim();
            let name = rest
                .strip_suffix("# TODO")
                .unwrap_or(rest)
                .trim()
                .to_string();
            return Some(NpmTest {
                name: if ancestors.is_empty() {
                    name.clone()
                } else {
                    format!("{} > {}", ancestors.join(" > "), name)
                },
                test_name: name,
                ancestors: ancestors.to_vec(),
                status: NpmTestStatus::Todo,
                duration: None,
                error_message: None,
            });
        }

        None
    }

    /// Split test name and duration from a line like "test name (5.123ms)"
    pub(crate) fn split_npm_test_name_and_duration(line: &str) -> (String, Option<f64>) {
        // Look for duration at the end: "(5.123ms)" or "(1.234s)"
        if let Some(paren_pos) = line.rfind('(') {
            let name_part = line[..paren_pos].trim();
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1]; // Remove parens
                let duration = Self::parse_npm_duration(duration_str);
                return (name_part.to_string(), duration);
            }
        }
        (line.to_string(), None)
    }

    /// Parse npm duration string (e.g., "5.123ms", "1.234s").
    pub(crate) fn parse_npm_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.ends_with("ms") {
            let num_str = &s[..s.len() - 2];
            num_str.parse::<f64>().ok().map(|n| n / 1000.0)
        } else if s.ends_with('s') {
            let num_str = &s[..s.len() - 1];
            num_str.parse::<f64>().ok()
        } else {
            None
        }
    }

    /// Extract duration from suite end line like "▶ path/to/test.js (123.456ms)"
    pub(crate) fn extract_npm_suite_duration(line: &str) -> Option<f64> {
        if let Some(paren_pos) = line.rfind('(') {
            let duration_part = &line[paren_pos..];
            if duration_part.ends_with("ms)") || duration_part.ends_with("s)") {
                let duration_str = &duration_part[1..duration_part.len() - 1];
                return Self::parse_npm_duration(duration_str);
            }
        }
        None
    }

    /// Parse npm test summary for tests: "✔ tests 4 passed (4)" or "✖ tests 2 failed (4)"
    pub(crate) fn parse_npm_test_summary_tests(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files: "✔ test files 2 passed (2)"
    pub(crate) fn parse_npm_test_summary_files(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches(|c| c == '✔' || c == '✖').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse npm test summary for tests (info format): "ℹ tests 4 passed (4)"
    pub(crate) fn parse_npm_test_summary_tests_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("tests").unwrap_or("").trim();
        Self::parse_npm_counts_with_todo(
            line,
            &mut summary.tests_passed,
            &mut summary.tests_failed,
            &mut summary.tests_skipped,
            &mut summary.tests_todo,
            &mut summary.tests_total,
        );
    }

    /// Parse npm test summary for test files (info format): "ℹ test files 2 passed (2)"
    pub(crate) fn parse_npm_test_summary_files_info(line: &str, summary: &mut NpmTestSummary) {
        let line = line.trim_start_matches('ℹ').trim();
        let line = line.strip_prefix("test files").unwrap_or("").trim();
        Self::parse_npm_counts(
            line,
            &mut summary.suites_passed,
            &mut summary.suites_failed,
            &mut summary.suites_skipped,
            &mut summary.suites_total,
        );
    }

    /// Parse count pattern like "4 passed (4)" or "2 passed 1 failed (3)"
    pub(crate) fn parse_npm_counts(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Parse npm test summary line with todo support.
    pub(crate) fn parse_npm_counts_with_todo(
        line: &str,
        passed: &mut usize,
        failed: &mut usize,
        skipped: &mut usize,
        todo: &mut usize,
        total: &mut usize,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            if let Ok(count) = parts[i].parse::<usize>() {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "passed" => *passed = count,
                        "failed" => *failed = count,
                        "skipped" => *skipped = count,
                        "todo" => *todo = count,
                        _ => {}
                    }
                    i += 2;
                    continue;
                }
            }
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let total_str = &parts[i][1..parts[i].len() - 1];
                if let Ok(t) = total_str.parse::<usize>() {
                    *total = t;
                }
            }
            i += 1;
        }
    }

    /// Update summary counts from parsed tests.
    pub(crate) fn update_npm_summary_from_tests(output: &mut NpmTestOutput) {
        // Only update if summary wasn't already populated from output
        if output.summary.tests_total == 0 {
            for suite in &output.test_suites {
                output.summary.suites_total += 1;
                if suite.passed {
                    output.summary.suites_passed += 1;
                } else {
                    output.summary.suites_failed += 1;
                }

                for test in &suite.tests {
                    output.summary.tests_total += 1;
                    match test.status {
                        NpmTestStatus::Passed => output.summary.tests_passed += 1,
                        NpmTestStatus::Failed => output.summary.tests_failed += 1,
                        NpmTestStatus::Skipped => output.summary.tests_skipped += 1,
                        NpmTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                }
            }
        }
    }

    /// Format npm test output based on the requested format.
    pub(crate) fn format_npm_test(output: &NpmTestOutput, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_npm_test_json(output),
            OutputFormat::Compact => Self::format_npm_test_compact(output),
            OutputFormat::Raw => Self::format_npm_test_raw(output),
            OutputFormat::Agent => Self::format_npm_test_agent(output),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_npm_test_compact(output),
        }
    }

    /// Format npm test output as JSON.
    pub(crate) fn format_npm_test_json(output: &NpmTestOutput) -> String {
        // Extract failing test identifiers (file::test_name format)
        let failed_tests: Vec<_> = output
            .test_suites
            .iter()
            .flat_map(|suite| {
                suite
                    .tests
                    .iter()
                    .filter(|t| t.status == NpmTestStatus::Failed)
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
                        NpmTestStatus::Passed => "passed",
                        NpmTestStatus::Failed => "failed",
                        NpmTestStatus::Skipped => "skipped",
                        NpmTestStatus::Todo => "todo",
                    },
                    "duration": t.duration,
                    "error_message": t.error_message,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "node_version": output.node_version,
        })
        .to_string()
    }

    /// Format npm test output in compact format.
    pub(crate) fn format_npm_test_compact(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        if output.is_empty {
            result.push_str("npm test: no tests found\n");
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
                if test.status == NpmTestStatus::Failed {
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

    /// Format npm test output as raw (just test names with status).
    pub(crate) fn format_npm_test_raw(output: &NpmTestOutput) -> String {
        let mut result = String::new();

        for suite in &output.test_suites {
            let status = if suite.passed { "PASS" } else { "FAIL" };
            result.push_str(&format!("{} {}\n", status, suite.file));

            for test in &suite.tests {
                let test_status = match test.status {
                    NpmTestStatus::Passed => "PASS",
                    NpmTestStatus::Failed => "FAIL",
                    NpmTestStatus::Skipped => "SKIP",
                    NpmTestStatus::Todo => "TODO",
                };
                result.push_str(&format!("  {} {}\n", test_status, test.name));
            }
        }

        result
    }

    /// Format npm test output for AI agent consumption.
    pub(crate) fn format_npm_test_agent(output: &NpmTestOutput) -> String {
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
                    .filter(|t| t.status == NpmTestStatus::Failed)
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
