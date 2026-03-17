use super::super::common::CommandResult;
use super::super::types::*;
use crate::OutputFormat;
use super::ParseHandler;

impl ParseHandler {
    // ============================================================
    // Vitest Parsing and Formatting
    // ============================================================

    /// Parse Vitest output into structured data.
    pub(crate) fn parse_vitest(input: &str) -> CommandResult<VitestOutput> {
        let mut output = VitestOutput::default();
        let mut current_suite: Option<VitestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;
        let mut in_suite_tree = false;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "✓ test/example.test.ts (5 tests) 306ms"
            // or: "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
            // or: "✗ test/example.test.ts (5 tests | 1 failed) 306ms"
            if let Some(suite_info) = Self::parse_vitest_suite_header(trimmed) {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                current_suite = Some(VitestTestSuite {
                    file: suite_info.file,
                    passed: suite_info.passed,
                    duration: suite_info.duration,
                    test_count: suite_info.test_count,
                    skipped_count: suite_info.skipped_count,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                in_suite_tree = true;
                continue;
            }

            // Detect test in tree format (indented test results)
            // "   ✓ test name 1ms" or "   ✕ test name"
            if in_suite_tree && line.starts_with("   ") {
                let test_line = line.trim_start();
                if let Some(test) = Self::parse_vitest_test_line(test_line) {
                    if let Some(ref mut suite) = current_suite {
                        suite.tests.push(test);
                    }
                    continue;
                }
            }

            // Detect failure details start
            // " ❯ test/file.test.ts:10:5"
            // "AssertionError: expected 5 to be 4"
            if trimmed.starts_with("❯ ") && trimmed.contains(".test.") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                // Extract test name from file reference like "❯ test/file.test.ts:10:5 > test name"
                let remainder = trimmed.strip_prefix("❯ ").unwrap_or("");
                // The test name is often after the file location
                let name = if let Some(pos) = remainder.find('>') {
                    remainder[pos + 1..].trim().to_string()
                } else {
                    // Try to get just the file path context
                    remainder.to_string()
                };
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Detect assertion error line
            if trimmed.starts_with("AssertionError:") || trimmed.starts_with("Error:") {
                in_failure_details = true;
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Accumulate failure details
            if in_failure_details
                && (trimmed.starts_with("at ")
                    || trimmed.starts_with("expected")
                    || trimmed.contains("to be")
                    || failure_buffer.len() > 0)
            {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary section
            // " Test Files  4 passed (4)"
            if trimmed.starts_with("Test Files") {
                let summary = Self::parse_vitest_test_files_summary(trimmed);
                output.summary.suites_passed = summary.suites_passed;
                output.summary.suites_failed = summary.suites_failed;
                output.summary.suites_total = summary.suites_total;
                in_suite_tree = false;
                continue;
            }

            // "      Tests  16 passed | 4 skipped (20)"
            if trimmed.starts_with("Tests") && !trimmed.starts_with("Tests:") {
                Self::parse_vitest_tests_summary(trimmed, &mut output.summary);
                continue;
            }

            // "   Start at  12:34:32"
            if trimmed.starts_with("Start at") {
                let time = trimmed.strip_prefix("Start at").unwrap_or("").trim();
                output.summary.start_at = Some(time.to_string());
                continue;
            }

            // "   Duration  1.26s"
            if trimmed.starts_with("Duration") {
                let duration_str = trimmed.strip_prefix("Duration").unwrap_or("").trim();
                output.summary.duration = Self::parse_vitest_duration(duration_str);
                continue;
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        VitestTestStatus::Passed => output.summary.tests_passed += 1,
                        VitestTestStatus::Failed => output.summary.tests_failed += 1,
                        VitestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        VitestTestStatus::Todo => output.summary.tests_todo += 1,
                    }
                    output.summary.tests_total += 1;
                }
            }
        }

        // Determine success
        output.success = output.summary.tests_failed == 0
            && output.summary.suites_failed == 0
            && output.summary.tests_total > 0;
        output.is_empty = output.test_suites.is_empty() && output.summary.tests_total == 0;

        Ok(output)
    }

    /// Parse vitest suite header like "✓ test/example.test.ts (5 tests | 1 skipped) 306ms"
    pub(crate) fn parse_vitest_suite_header(line: &str) -> Option<VitestSuiteInfo> {
        let line = line.trim_start();

        let (passed, remainder) = if line.starts_with('✓') {
            (true, line.strip_prefix('✓')?.trim_start())
        } else if line.starts_with('✗') {
            (false, line.strip_prefix('✗')?.trim_start())
        } else if line.starts_with('×') {
            (false, line.strip_prefix('×')?.trim_start())
        } else if line.starts_with("FAIL") {
            (false, line.strip_prefix("FAIL")?.trim_start())
        } else if line.starts_with("PASS") {
            (true, line.strip_prefix("PASS")?.trim_start())
        } else {
            return None;
        };

        // Extract file path - everything before the parenthesis
        let paren_pos = remainder.find('(')?;
        let file = remainder[..paren_pos].trim().to_string();
        let rest = &remainder[paren_pos..];

        // Parse test count info: "(5 tests)" or "(5 tests | 1 skipped)" or "(5 tests | 1 failed)"
        let mut test_count = None;
        let mut skipped_count = None;

        if rest.starts_with('(') && rest.contains(')') {
            let end_paren = rest.find(')').unwrap_or(rest.len());
            let info = &rest[1..end_paren];

            // Extract test count
            if let Some(pos) = info.find(" test") {
                let num_str: String = info[..pos].chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<usize>() {
                    test_count = Some(num);
                }
            }

            // Extract skipped count
            if let Some(pos) = info.find("skipped") {
                let before = &info[..pos];
                if let Some(num_str) = before.rsplit('|').next() {
                    let num_str: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = num_str.parse::<usize>() {
                        skipped_count = Some(num);
                    }
                }
            }
        }

        // Extract duration - look for number followed by ms or s at the end
        let duration = if rest.contains("ms") || rest.contains('s') && !rest.contains("ms") {
            // Find duration at the end of the line
            let after_paren = rest.find(')').map(|p| &rest[p + 1..]).unwrap_or("");
            Self::parse_vitest_duration(after_paren.trim())
        } else {
            None
        };

        Some(VitestSuiteInfo {
            file,
            passed,
            duration,
            test_count,
            skipped_count,
        })
    }

    /// Parse a single Vitest test result line.
    pub(crate) fn parse_vitest_test_line(line: &str) -> Option<VitestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        // Vitest uses: ✓ (passed), ✕/× (failed), ↩ (skipped), etc.
        let (status, remainder) = if line.starts_with('✓') {
            (
                VitestTestStatus::Passed,
                line.strip_prefix('✓')?.trim_start(),
            )
        } else if line.starts_with('✕') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('✕')?.trim_start(),
            )
        } else if line.starts_with('×') {
            (
                VitestTestStatus::Failed,
                line.strip_prefix('×')?.trim_start(),
            )
        } else if line.starts_with('↩') {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix('↩')?.trim_start(),
            )
        } else if line.starts_with("↓") {
            (
                VitestTestStatus::Skipped,
                line.strip_prefix("↓")?.trim_start(),
            )
        } else if line.contains("skipped") || line.contains("skip") {
            (VitestTestStatus::Skipped, line)
        } else if line.contains("todo") {
            (VitestTestStatus::Todo, line)
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name 1ms" or "test name 1.5s"
        let (test_name, duration) = if let Some(ms_pos) = trimmed.rfind("ms") {
            // Find the number before "ms"
            let before = &trimmed[..ms_pos];
            let num_start = before
                .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                .map(|p| p + 1)
                .unwrap_or(0);
            let name_part = before[..num_start].trim();
            let duration_str = &before[num_start..];
            let duration = duration_str.parse::<f64>().ok().map(|d| d / 1000.0);
            (name_part.to_string(), duration)
        } else if let Some(s_pos) = trimmed.rfind('s') {
            // Check if it's a duration (not part of a word)
            let before = &trimmed[..s_pos];
            if before.ends_with(|c: char| c.is_ascii_digit()) {
                let num_start = before
                    .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let name_part = before[..num_start].trim();
                let duration_str = &before[num_start..];
                let duration = duration_str.parse::<f64>().ok();
                (name_part.to_string(), duration)
            } else {
                (trimmed.to_string(), None)
            }
        } else {
            (trimmed.to_string(), None)
        };

        // Parse ancestors (describe blocks) from test name
        // Format: "describe block > nested describe > test name"
        let (ancestors, final_name) = if test_name.contains('>') || test_name.contains("›") {
            let delimiter = if test_name.contains('>') { ">" } else { "›" };
            let parts: Vec<&str> = test_name.split(delimiter).map(|s| s.trim()).collect();
            if parts.len() > 1 {
                let ancestors: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let name = parts.last().unwrap_or(&"").to_string();
                (ancestors, name)
            } else {
                (Vec::new(), test_name.clone())
            }
        } else {
            (Vec::new(), test_name.clone())
        };

        Some(VitestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Vitest duration string (e.g., "5ms", "1.26s").
    pub(crate) fn parse_vitest_duration(s: &str) -> Option<f64> {
        let s = s.trim();

        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") && !s.contains("start") {
            Some(num)
        } else if s.contains('m') && !s.contains("ms") {
            Some(num * 60.0)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Vitest "Test Files" summary line.
    pub(crate) fn parse_vitest_test_files_summary(line: &str) -> VitestSummary {
        let mut summary = VitestSummary::default();
        let line = line.strip_prefix("Test Files").unwrap_or("").trim();

        // Parse pattern: "4 passed (4)" or "2 passed, 1 failed (3)"
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.suites_passed = extract_count(line, "passed");
        summary.suites_failed = extract_count(line, "failed");

        // Total is in parentheses
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let total_str = &line[start + 1..end];
                summary.suites_total = total_str.parse().unwrap_or(0);
            }
        }

        summary
    }

    /// Parse Vitest "Tests" summary line.
    pub(crate) fn parse_vitest_tests_summary(line: &str, summary: &mut VitestSummary) {
        let line = line.strip_prefix("Tests").unwrap_or("").trim();

        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                let before = &text[..pos];
                // Find the number before the label
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");

        // Total is in parentheses at the end
        if let Some(start) = line.rfind('(') {
            if let Some(end) = line.rfind(')') {
                if end > start {
                    let total_str = &line[start + 1..end];
                    summary.tests_total = total_str.parse().unwrap_or(0);
                }
            }
        }
    }

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
