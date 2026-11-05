//! The summary half of the bun test parser: recognizing and reading the trailing counts block, apart from the per-line parsing in bun_parse.rs.

use super::super::types::*;
use super::ParseHandler;

impl ParseHandler {
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

    /// Parse "Ran X test(s) in Yms" or "Ran X test(s) across Y file(s). [Zms]"
    pub(crate) fn parse_bun_ran_line(line: &str, summary: &mut BunTestSummary) {
        let line = line.trim();
        let first_count = |s: &str| s.split_whitespace().next()?.parse::<usize>().ok();

        if let Some(count) = line.strip_prefix("Ran ").and_then(first_count) {
            summary.tests_total = count;
        }
        if let Some(count) = line.split_once("across ").and_then(|(_, r)| first_count(r)) {
            summary.suites_total = count;
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
        // "across X files" is optional.
        if output.summary.suites_total == 0 {
            output.summary.suites_total = output.test_suites.len();
        }
        // bun may omit passing files, so count failures and derive the rest.
        let failed = output.test_suites.iter().filter(|s| !s.passed).count();
        output.summary.suites_failed = failed;
        output.summary.suites_passed = output.summary.suites_total.saturating_sub(failed);

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
}
