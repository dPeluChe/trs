use super::super::common::CommandResult;
use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
    /// Parse Jest output into structured data.
    pub(crate) fn parse_jest(input: &str) -> CommandResult<JestOutput> {
        let mut output = JestOutput::default();
        let mut current_suite: Option<JestTestSuite> = None;
        let mut in_failure_details = false;
        let mut failure_buffer = String::new();
        let mut current_failed_test: Option<String> = None;

        for line in input.lines() {
            let trimmed = line.trim();

            // Skip empty lines (but preserve them in failure details)
            if trimmed.is_empty() && !in_failure_details {
                continue;
            }

            // Detect test suite header: "PASS src/path/to/test.js" or "FAIL src/path/to/test.js"
            if trimmed.starts_with("PASS ") || trimmed.starts_with("FAIL ") {
                // Save any pending suite
                if let Some(suite) = current_suite.take() {
                    output.test_suites.push(suite);
                }

                let (passed, file) = if trimmed.starts_with("PASS ") {
                    (true, trimmed.strip_prefix("PASS ").unwrap_or("").trim())
                } else {
                    (false, trimmed.strip_prefix("FAIL ").unwrap_or("").trim())
                };

                current_suite = Some(JestTestSuite {
                    file: file.to_string(),
                    passed,
                    duration: None,
                    tests: Vec::new(),
                });
                in_failure_details = false;
                failure_buffer.clear();
                current_failed_test = None;
                continue;
            }

            // Detect individual test results
            // Format: "  ✓ test name (5 ms)" or "  ✕ test name" or "  ○ skipped"
            if let Some(test) = Self::parse_jest_test_line(trimmed) {
                if let Some(ref mut suite) = current_suite {
                    suite.tests.push(test);
                }
                continue;
            }

            // Detect test suite duration: "(5 ms)"
            if trimmed.starts_with('(') && trimmed.ends_with(')') && current_suite.is_some() {
                let duration_str = trimmed.trim_matches(|c| c == '(' || c == ')');
                let duration = Self::parse_jest_duration(duration_str);
                if let Some(ref mut suite) = current_suite {
                    suite.duration = duration;
                }
                continue;
            }

            // Detect failure details start
            // "  ● test name › should work"
            if trimmed.starts_with("● ") {
                in_failure_details = true;
                // Save any previous failure info
                if let Some(name) = current_failed_test.take() {
                    if let Some(ref mut suite) = current_suite {
                        if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                            test.error_message = Some(failure_buffer.trim().to_string());
                        }
                    }
                }
                let name = trimmed.strip_prefix("● ").unwrap_or("").trim().to_string();
                current_failed_test = Some(name);
                failure_buffer = String::new();
                continue;
            }

            // Accumulate failure details
            if in_failure_details && current_failed_test.is_some() {
                failure_buffer.push_str(line);
                failure_buffer.push('\n');
                continue;
            }

            // Detect summary line: "Test Suites: X passed, Y failed, Z total"
            if trimmed.starts_with("Test Suites:") {
                let summary = Self::parse_jest_summary(trimmed);
                output.summary = summary;
                continue;
            }

            // Additional summary lines: "Tests:", "Snapshots:", "Time:"
            if trimmed.starts_with("Tests:") {
                Self::parse_jest_tests_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Snapshots:") {
                Self::parse_jest_snapshots_summary(trimmed, &mut output.summary);
            }
            if trimmed.starts_with("Time:") {
                Self::parse_jest_time_summary(trimmed, &mut output.summary);
            }
        }

        // Save any pending suite
        if let Some(suite) = current_suite.take() {
            output.test_suites.push(suite);
        }

        // Save last failure info (if any)
        // Note: Error messages are typically captured when we see the next test or suite
        // so we don't need to explicitly save the last one here

        // Calculate totals if not already in summary
        if output.summary.suites_total == 0 && !output.test_suites.is_empty() {
            output.summary.suites_passed = output.test_suites.iter().filter(|s| s.passed).count();
            output.summary.suites_failed = output.test_suites.iter().filter(|s| !s.passed).count();
            output.summary.suites_total = output.test_suites.len();

            for suite in &output.test_suites {
                for test in &suite.tests {
                    match test.status {
                        JestTestStatus::Passed => output.summary.tests_passed += 1,
                        JestTestStatus::Failed => output.summary.tests_failed += 1,
                        JestTestStatus::Skipped => output.summary.tests_skipped += 1,
                        JestTestStatus::Todo => output.summary.tests_todo += 1,
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

    /// Parse a single Jest test result line.
    pub(crate) fn parse_jest_test_line(line: &str) -> Option<JestTest> {
        // Trim leading whitespace
        let line = line.trim_start();

        // Skip if doesn't start with proper prefix
        if !line.starts_with("✓") && !line.starts_with("✕") && !line.starts_with("○") {
            return None;
        }

        let (status, remainder) = if line.starts_with("✓") {
            (JestTestStatus::Passed, line.strip_prefix("✓").unwrap_or(""))
        } else if line.starts_with("✕") {
            (JestTestStatus::Failed, line.strip_prefix("✕").unwrap_or(""))
        } else if line.starts_with("○") {
            // Could be skipped or todo
            let rem = line.strip_prefix("○").unwrap_or("");
            if rem.contains("skipped") || rem.contains("skip") {
                (JestTestStatus::Skipped, rem)
            } else if rem.contains("todo") {
                (JestTestStatus::Todo, rem)
            } else {
                (JestTestStatus::Skipped, rem)
            }
        } else {
            return None;
        };

        // Parse test name and duration
        let trimmed = remainder.trim();

        // Extract duration if present: "test name (5 ms)"
        let (test_name, duration) = if let Some(paren_pos) = trimmed.rfind('(') {
            let name_part = trimmed[..paren_pos].trim();
            let duration_part = &trimmed[paren_pos..];
            let duration =
                Self::parse_jest_duration(duration_part.trim_matches(|c| c == '(' || c == ')'));
            (name_part.to_string(), duration)
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

        Some(JestTest {
            name: test_name,
            test_name: final_name,
            ancestors,
            status,
            duration,
            error_message: None,
        })
    }

    /// Parse Jest duration string (e.g., "5 ms", "1.23 s").
    pub(crate) fn parse_jest_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        // Try to extract number and unit
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num: f64 = num_str.parse().ok()?;

        // Convert to seconds based on unit
        if s.contains("ms") || s.ends_with("ms") {
            Some(num / 1000.0)
        } else if s.contains('s') && !s.contains("ms") {
            Some(num)
        } else {
            // Assume milliseconds if no unit
            Some(num / 1000.0)
        }
    }

    /// Parse Jest summary line for test suites.
    pub(crate) fn parse_jest_summary(line: &str) -> JestSummary {
        let mut summary = JestSummary::default();
        let line = line.strip_prefix("Test Suites:").unwrap_or("");

        // Parse pattern: "X passed, Y failed, Z total" or "X passed, Y total"
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
        summary.suites_total = extract_count(line, "total");

        summary
    }

    /// Parse Jest summary line for tests.
    pub(crate) fn parse_jest_tests_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Tests:").unwrap_or("");

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

        summary.tests_passed = extract_count(line, "passed");
        summary.tests_failed = extract_count(line, "failed");
        summary.tests_skipped = extract_count(line, "skipped");
        summary.tests_todo = extract_count(line, "todo");
        summary.tests_total = extract_count(line, "total");
    }

    /// Parse Jest summary line for snapshots.
    pub(crate) fn parse_jest_snapshots_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Snapshots:").unwrap_or("");
        // Try to extract a number from the line
        let num_str: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = num_str.parse() {
            summary.snapshots = Some(num);
        }
    }

    /// Parse Jest summary line for time.
    pub(crate) fn parse_jest_time_summary(line: &str, summary: &mut JestSummary) {
        let line = line.strip_prefix("Time:").unwrap_or("").trim();
        summary.duration = Self::parse_jest_duration(line);
    }
}
