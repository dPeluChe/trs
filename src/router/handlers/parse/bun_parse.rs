use super::super::common::CommandResult;
use super::super::types::*;
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

            // Check for test file header: "test/file.test.ts:" (ends with
            // colon) or the failure-recap form "FAIL  test/file.test.js"
            // (bun re-lists failed files at the end without the colon —
            // missing it would drop the failing file + its ✗ tests).
            let is_colon_header = trimmed.ends_with(':')
                && !trimmed.starts_with(|c| c == '✓' || c == '✗' || c == '×' || c == '(');
            let fail_recap_file = trimmed
                .strip_prefix("FAIL")
                .map(str::trim)
                .filter(|rest| !rest.is_empty() && !rest.contains(' ') && rest.contains('/'));
            if is_colon_header || fail_recap_file.is_some() {
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

                let file = match fail_recap_file {
                    Some(f) => f.to_string(),
                    None => trimmed.trim_end_matches(':').to_string(),
                };
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

        // Update summary counts from parsed tests
        Self::update_bun_summary_from_tests(&mut output);

        // Derived AFTER the summary merge: a suite-less parse can still carry
        // counts from the " N pass / N fail" lines (per-test lines from some
        // bun versions don't match the suite grammar). "No tests" must mean
        // truly nothing parsed, and success must respect the failed count.
        let counted = output.summary.tests_total
            + output.summary.tests_passed
            + output.summary.tests_failed
            + output.summary.tests_skipped;
        output.is_empty = output.test_suites.is_empty() && counted == 0;
        output.success =
            output.summary.tests_failed == 0 && output.test_suites.iter().all(|s| s.passed);

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
}
