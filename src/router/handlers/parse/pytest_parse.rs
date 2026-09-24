use super::super::common::CommandResult;
use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
    /// Parse pytest output into structured data.
    pub(crate) fn parse_pytest(input: &str) -> CommandResult<PytestOutput> {
        let mut output = PytestOutput::default();
        let mut current_test: Option<TestResult> = None;
        let mut in_failure_section = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test_name: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse header info
            // "rootdir: /path/to/project"
            if trimmed.starts_with("rootdir:") {
                output.rootdir = Some(
                    trimmed
                        .strip_prefix("rootdir:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
                continue;
            }

            // "platform darwin -- Python 3.12.0, pytest-8.0.0, pluggy-1.4.0"
            if trimmed.starts_with("platform ") {
                output.platform = Some(trimmed.to_string());
                // Extract Python and pytest version
                if let Some(py_pos) = trimmed.find("Python ") {
                    let after_py = &trimmed[py_pos + 7..];
                    if let Some(comma_pos) = after_py.find(',') {
                        output.python_version = Some(after_py[..comma_pos].to_string());
                    }
                }
                if let Some(pytest_pos) = trimmed.find("pytest-") {
                    let after_pytest = &trimmed[pytest_pos + 7..];
                    if let Some(comma_pos) = after_pytest.find(',') {
                        output.pytest_version = Some(after_pytest[..comma_pos].to_string());
                    } else {
                        output.pytest_version = Some(after_pytest.to_string());
                    }
                }
                continue;
            }

            // Detect start of test session
            // "test session starts" or "collected N items"
            if trimmed.contains("test session starts") || trimmed.contains("collected") {
                continue;
            }

            // Detect test results with progress format
            // Format: "tests/test_file.py::test_name PASSED" or "tests/test_file.py::test_name FAILED"
            // Also handles the short format: "test_file.py .F.s" (dot=pass, F=fail, s=skip)
            if let Some(test_result) = Self::parse_pytest_test_line(trimmed) {
                // Save any pending test
                if let Some(test) = current_test.take() {
                    output.tests.push(test);
                }
                current_test = Some(test_result);
                continue;
            }

            // Quiet mode progress line: ".....F..x.s [100%]"
            let quiet_results = Self::parse_pytest_quiet_progress(trimmed);
            if !quiet_results.is_empty() {
                if let Some(test) = current_test.take() {
                    output.tests.push(test);
                }
                output.tests.extend(quiet_results);
                continue;
            }

            // Detect summary line
            // "N passed, M failed, K skipped in X.XXs"
            // Also: "N passed in X.XXs"
            if Self::is_pytest_summary_line(trimmed) {
                let summary = Self::parse_pytest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Section banners are padded to the terminal width
            // (`====== FAILURES ======`), so compare the title, not a prefix.
            let title = trimmed.trim_matches('=').trim();
            if matches!(title, "FAILURES" | "ERRORS")
                || title.starts_with("short test summary info")
            {
                // The last verbose result is still pending here; its failure
                // block is next, so it must be in the list to receive it.
                if let Some(test) = current_test.take() {
                    output.tests.push(test);
                }
                if let Some(name) = current_failed_test_name.take() {
                    Self::attach_failure(&mut output.tests, name, &failure_buffer);
                }
                failure_buffer.clear();
                // The short summary repeats what FAILURES already gave.
                in_failure_section = !title.starts_with("short test summary info");
                continue;
            }

            // Parse failure details
            if in_failure_section {
                // Check if this is a new failure header: "____ test_name ____"
                if trimmed.starts_with("____") && trimmed.ends_with("____") {
                    // Save any previous failure info
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        // "____ test_name ____" matches "file.py::test_name"
                        Self::attach_failure(&mut output.tests, name, &failure_buffer);
                    }
                    let name = trimmed.trim_matches('_').trim().to_string();
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Check for ERROR instead of FAILURES
                // "ERROR at setup of test_name"
                if trimmed.starts_with("ERROR at") || trimmed.starts_with("ERROR:") {
                    in_failure_section = true;
                    if let Some(name) = current_failed_test_name.take() {
                        // Find test by matching the name at the end (after ::)
                        Self::attach_failure(&mut output.tests, name, &failure_buffer);
                    }
                    // Extract test name from error line
                    let name = if trimmed.starts_with("ERROR at setup of ") {
                        trimmed
                            .strip_prefix("ERROR at setup of ")
                            .unwrap_or("")
                            .to_string()
                    } else if trimmed.starts_with("ERROR at teardown of ") {
                        trimmed
                            .strip_prefix("ERROR at teardown of ")
                            .unwrap_or("")
                            .to_string()
                    } else {
                        trimmed
                            .strip_prefix("ERROR:")
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    };
                    current_failed_test_name = Some(name);
                    failure_buffer = String::new();
                    continue;
                }

                // Accumulate failure details
                if current_failed_test_name.is_some() {
                    failure_buffer.push_str(line);
                    failure_buffer.push('\n');
                }
            }
        }

        // Save any pending test
        if let Some(test) = current_test.take() {
            output.tests.push(test);
        }

        // Save last failure info
        if let Some(name) = current_failed_test_name.take() {
            // Find test by matching the name at the end (after ::)
            // "____ test_name ____" matches "file.py::test_name"
            Self::attach_failure(&mut output.tests, name, &failure_buffer);
        }

        // Calculate totals if not already in summary
        if output.summary.total == 0 && !output.tests.is_empty() {
            output.summary.passed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Passed)
                .count();
            output.summary.failed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Failed)
                .count();
            output.summary.skipped = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Skipped)
                .count();
            output.summary.xfailed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XFailed)
                .count();
            output.summary.xpassed = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::XPassed)
                .count();
            output.summary.errors = output
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Error)
                .count();
            output.summary.total = output.tests.len();
        }

        // Determine success
        output.success =
            output.summary.failed == 0 && output.summary.errors == 0 && output.summary.total > 0;
        output.is_empty = output.tests.is_empty() && output.summary.total == 0;

        Ok(output)
    }

    /// Attach a FAILURES-section block to its test. Verbose runs match by name;
    /// quiet runs have unnamed failures, which take the name in order.
    fn attach_failure(tests: &mut [TestResult], name: String, message: &str) {
        let suffix = format!("::{}", name);
        let failed = |t: &TestResult| matches!(t.status, TestStatus::Failed | TestStatus::Error);
        let idx = tests
            .iter()
            .position(|t| t.name == name || t.name.ends_with(&suffix))
            .or_else(|| tests.iter().position(|t| failed(t) && t.name.is_empty()));
        if let Some(i) = idx {
            if tests[i].name.is_empty() {
                tests[i].name = name;
            }
            tests[i].error_message = Some(message.trim().to_string());
        }
    }

    /// Parse a pytest quiet-mode progress line (e.g. ".....F..x.s  [100%]").
    /// Returns individual test results inferred from the progress characters.
    fn parse_pytest_quiet_progress(line: &str) -> Vec<TestResult> {
        // Strip trailing progress indicator like " [100%]" or "  [50%]"
        let chars_part = line
            .find('[')
            .map(|pos| line[..pos].trim_end())
            .unwrap_or(line.trim());

        // Must be only valid progress chars: . F s x X E
        if chars_part.is_empty()
            || !chars_part
                .chars()
                .all(|c| matches!(c, '.' | 'F' | 's' | 'x' | 'X' | 'E'))
        {
            return vec![];
        }

        chars_part
            .chars()
            .map(|c| {
                let status = match c {
                    '.' => TestStatus::Passed,
                    'F' => TestStatus::Failed,
                    's' => TestStatus::Skipped,
                    'x' => TestStatus::XFailed,
                    'X' => TestStatus::XPassed,
                    'E' => TestStatus::Error,
                    _ => TestStatus::Passed,
                };
                // Quiet mode prints no names. Leave it empty: attach_failure
                // fills in the real one from the FAILURES section.
                TestResult {
                    name: String::new(),
                    status,
                    duration: None,
                    file: None,
                    line: None,
                    error_message: None,
                }
            })
            .collect()
    }

    /// Parse a single test result line from pytest output.
    pub(crate) fn parse_pytest_test_line(line: &str) -> Option<TestResult> {
        // Format: "tests/test_file.py::test_name PASSED"
        // or: "tests/test_file.py::test_name SKIPPED (reason)"
        // or: "tests/test_file.py::test_name FAILED"
        // or: "tests/test_file.py::test_name XFAIL (reason)"

        // Skip lines that are clearly not test results
        if line.starts_with("===")
            || line.starts_with("---")
            || line.starts_with("...")
            || line.is_empty()
        {
            return None;
        }

        // Real runs pad the status to a column and append progress:
        // `a.py::t FAILED            [ 98%]`. Drop the progress, then the pad.
        let core = match line.rfind(" [") {
            Some(pos) if line.ends_with("%]") => line[..pos].trim_end(),
            _ => line,
        };
        let (status_str, remainder) = ["PASSED", "FAILED", "SKIPPED", "XFAIL", "XPASS", "ERROR"]
            .iter()
            .find_map(|st| {
                core.strip_suffix(st)
                    .filter(|r| r.ends_with(' '))
                    .map(|r| (*st, r))
            })?;

        let status = match status_str {
            "PASSED" => TestStatus::Passed,
            "FAILED" => TestStatus::Failed,
            "SKIPPED" => TestStatus::Skipped,
            "XFAIL" => TestStatus::XFailed,
            "XPASS" => TestStatus::XPassed,
            "ERROR" => TestStatus::Error,
            _ => return None,
        };

        // Parse test name and file
        let test_name = remainder.trim().to_string();

        // Try to extract file and line from "file.py::test_name" format
        let (file, line) = if let Some(pos) = test_name.find("::") {
            let file = test_name[..pos].to_string();
            let rest = &test_name[pos + 2..];
            // Check for line number: "test_name[:lineno]"
            let line = if let Some(colon_pos) = rest.find(':') {
                rest[colon_pos + 1..].parse().ok()
            } else {
                None
            };
            (Some(file), line)
        } else {
            (None, None)
        };

        Some(TestResult {
            name: test_name,
            status,
            duration: None, // Duration is usually in the summary line
            file,
            line,
            error_message: None,
        })
    }

    /// Check if a line is a pytest summary line.
    pub(crate) fn is_pytest_summary_line(line: &str) -> bool {
        // Summary lines start with a number and contain "passed" or "failed"
        // Examples:
        // "2 passed in 0.01s"
        // "2 passed, 1 failed in 0.01s"
        // "2 passed, 1 failed, 3 skipped in 0.01s"
        // "1 failed, 2 passed in 0.01s"
        // "=== 2 passed in 0.01s ==="
        let lower = line.to_lowercase();
        let starts_with_equals = line.starts_with("===");
        let has_passed = lower.contains("passed");
        let has_failed = lower.contains("failed");
        let has_skipped = lower.contains("skipped");
        let has_error = lower.contains("error");
        let has_deselected = lower.contains("deselected");
        let has_xfailed = lower.contains("xfailed");
        let has_xpassed = lower.contains("xpassed");
        let has_warnings = lower.contains("warning");

        (starts_with_equals
            || line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false))
            && (has_passed
                || has_failed
                || has_skipped
                || has_error
                || has_deselected
                || has_xfailed
                || has_xpassed
                || has_warnings)
    }

    /// Parse pytest summary line into TestSummary.
    pub(crate) fn parse_pytest_summary(line: &str) -> TestSummary {
        let mut summary = TestSummary::default();
        // Search and slice the same string: an index into the padded line
        // used on the trimmed one read the wrong bytes, or panicked.
        let lower = line.trim_matches('=').trim().to_lowercase();

        // Parse counts
        // Pattern: "N passed", "N failed", "N skipped", etc.
        fn extract_count(text: &str, label: &str) -> usize {
            let pattern = format!(" {}", label);
            if let Some(pos) = text.find(&pattern) {
                // Look backwards for the number
                let before = &text[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return last.parse().unwrap_or(0);
                }
            }
            0
        }

        summary.passed = extract_count(&lower, "passed");
        summary.failed = extract_count(&lower, "failed");
        summary.skipped = extract_count(&lower, "skipped");
        summary.errors = extract_count(&lower, "error");
        summary.xfailed = extract_count(&lower, "xfailed");
        summary.xpassed = extract_count(&lower, "xpassed");

        // Calculate total
        summary.total = summary.passed
            + summary.failed
            + summary.skipped
            + summary.errors
            + summary.xfailed
            + summary.xpassed;

        // Parse duration
        // "in 0.01s" or "in 1.23 seconds"
        if let Some(pos) = lower.find(" in ") {
            let after_in = &lower[pos + 4..];
            // Extract number before 's' or 'seconds'
            let duration_str: String = after_in
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(duration) = duration_str.parse::<f64>() {
                summary.duration = Some(duration);
            }
        }

        summary
    }
}
