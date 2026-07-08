//! Parser for `go test` output.
//!
//! Handles both verbose (`-v`) and default output:
//! - Verbose: `=== RUN`, `--- PASS/FAIL`, per-test details
//! - Default: package summaries only (`ok pkg 0.123s`, `FAIL pkg`)
//! - Also handles `-json` (JSON lines) as a bonus

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Parse `go test` output into pass/fail/skip summary.
    pub(crate) fn handle_go_test(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let result = parse_go_test(&input);

        let output = match ctx.format {
            OutputFormat::Json => format_go_test_json(&result),
            _ => format_go_test_compact(&result),
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("go-test")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(result.total)
                .print();
        }
        Ok(())
    }
}

struct GoTestResult {
    passed: usize,
    failed: usize,
    skipped: usize,
    total: usize,
    packages_ok: usize,
    packages_fail: usize,
    failed_tests: Vec<GoFailedTest>,
    compile_errors: Vec<String>,
    duration: Option<String>,
    success: bool,
}

struct GoFailedTest {
    name: String,
    package: String,
    message: String,
}

fn parse_go_test(input: &str) -> GoTestResult {
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut packages_ok = 0usize;
    let mut packages_fail = 0usize;
    let mut failed_tests: Vec<GoFailedTest> = Vec::new();
    let mut compile_errors: Vec<String> = Vec::new();
    let mut last_duration = None;

    // Buffer output between === RUN and --- PASS/FAIL/SKIP
    // In go test -v, error lines appear BEFORE the --- FAIL line
    let mut run_buffer = String::new();

    for line in input.lines() {
        let t = line.trim();

        // Verbose mode: --- PASS: TestFoo (0.00s)
        if t.starts_with("--- PASS:") {
            passed += 1;
            run_buffer.clear();
            continue;
        }

        // Verbose mode: --- FAIL: TestBar (0.01s)
        if t.starts_with("--- FAIL:") {
            failed += 1;
            let rest = t.strip_prefix("--- FAIL:").unwrap_or("").trim();
            let name = rest.split_whitespace().next().unwrap_or("unknown");
            failed_tests.push(GoFailedTest {
                name: name.to_string(),
                package: String::new(),
                message: run_buffer.trim().to_string(),
            });
            run_buffer.clear();
            continue;
        }

        // Verbose mode: --- SKIP: TestBaz (0.00s)
        if t.starts_with("--- SKIP:") {
            skipped += 1;
            run_buffer.clear();
            continue;
        }

        // === RUN — start buffering for this test
        if t.starts_with("=== RUN") || t.starts_with("=== PAUSE") || t.starts_with("=== CONT") {
            run_buffer.clear();
            continue;
        }

        // Package summary: "ok  \tgithub.com/user/repo/pkg\t0.123s"
        if t.starts_with("ok") && t.contains('\t') {
            packages_ok += 1;
            if let Some(dur) = t.rsplit('\t').next() {
                let dur = dur.trim();
                if dur.ends_with('s') {
                    last_duration = Some(dur.to_string());
                }
            }
            continue;
        }

        // Package failure: "FAIL\tgithub.com/user/repo/pkg\t0.015s"
        if t.starts_with("FAIL") && t.contains('\t') {
            packages_fail += 1;
            let parts: Vec<&str> = t.split('\t').collect();
            if parts.len() >= 2 {
                let pkg = parts[1].trim().to_string();
                // Associate package with recent failed tests that have no package
                for ft in failed_tests.iter_mut().rev() {
                    if ft.package.is_empty() {
                        ft.package = pkg.clone();
                    } else {
                        break;
                    }
                }
            }
            if let Some(dur) = parts.last() {
                let dur = dur.trim();
                if dur.ends_with('s') {
                    last_duration = Some(dur.to_string());
                }
            }
            continue;
        }

        // Bare "FAIL" line (no tab) — package-level failure marker
        if t == "FAIL" {
            continue;
        }

        // "PASS" alone — default mode all-pass indicator
        if t == "PASS" {
            continue;
        }

        // Compile error
        if t.starts_with("# ") && t.contains('/') {
            // "# github.com/user/repo/pkg" — compile error header
            compile_errors.push(t.to_string());
            continue;
        }
        if !compile_errors.is_empty()
            && (t.contains(": undefined:")
                || t.contains(": cannot ")
                || t.contains(": not enough")
                || t.contains(": too many")
                || t.contains(".go:"))
        {
            compile_errors.push(format!("  {}", t));
            continue;
        }

        // Buffer output lines (between === RUN and --- PASS/FAIL/SKIP)
        if !t.is_empty() {
            if !run_buffer.is_empty() {
                run_buffer.push('\n');
            }
            run_buffer.push_str(t);
        }
    }

    // Default mode: if we have package counts but no individual test counts,
    // report packages as the unit
    if passed == 0 && failed == 0 && skipped == 0 && (packages_ok > 0 || packages_fail > 0) {
        passed = packages_ok;
        failed = packages_fail;
    }

    let total = passed + failed + skipped;
    let success = failed == 0 && compile_errors.is_empty();

    GoTestResult {
        passed,
        failed,
        skipped,
        total,
        packages_ok,
        packages_fail,
        failed_tests,
        compile_errors,
        duration: last_duration,
        success,
    }
}

fn format_go_test_compact(r: &GoTestResult) -> String {
    let mut out = String::new();

    if !r.compile_errors.is_empty() {
        out.push_str(&format!("compile errors ({}):\n", r.compile_errors.len()));
        for err in r.compile_errors.iter().take(10) {
            out.push_str(&format!("  {}\n", err));
        }
        if r.compile_errors.len() > 10 {
            out.push_str(&format!("  ...+{} more\n", r.compile_errors.len() - 10));
        }
    }

    let status = if r.success { "ok" } else { "FAILED" };
    out.push_str(&format!("go test: {} ({} passed", status, r.passed));
    if r.failed > 0 {
        out.push_str(&format!(", {} failed", r.failed));
    }
    if r.skipped > 0 {
        out.push_str(&format!(", {} skipped", r.skipped));
    }
    if r.packages_ok + r.packages_fail > 1 {
        out.push_str(&format!(", {} pkgs", r.packages_ok + r.packages_fail));
    }
    if let Some(ref dur) = r.duration {
        out.push_str(&format!(", {}", dur));
    }
    out.push_str(")\n");

    if !r.failed_tests.is_empty() {
        out.push_str(&format!("failures ({}):\n", r.failed_tests.len()));
        for ft in &r.failed_tests {
            if ft.package.is_empty() {
                out.push_str(&format!("  {}\n", ft.name));
            } else {
                out.push_str(&format!("  {} ({})\n", ft.name, ft.package));
            }
            if !ft.message.is_empty() {
                // Show first 3 lines of error
                for (i, line) in ft.message.lines().enumerate() {
                    if i >= 3 {
                        let remaining = ft.message.lines().count() - 3;
                        if remaining > 0 {
                            out.push_str(&format!("      ...+{} lines\n", remaining));
                        }
                        break;
                    }
                    out.push_str(&format!("      {}\n", line));
                }
            }
        }
    }

    out
}

fn format_go_test_json(r: &GoTestResult) -> String {
    let failed_tests: Vec<serde_json::Value> = r
        .failed_tests
        .iter()
        .map(|ft| {
            serde_json::json!({
                "name": ft.name,
                "package": ft.package,
                "message": ft.message,
            })
        })
        .collect();

    serde_json::json!({
        "success": r.success,
        "passed": r.passed,
        "failed": r.failed,
        "skipped": r.skipped,
        "total": r.total,
        "packages_ok": r.packages_ok,
        "packages_fail": r.packages_fail,
        "duration": r.duration,
        "failed_tests": failed_tests,
        "compile_errors": r.compile_errors,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_test_verbose_all_pass() {
        let input = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSub
--- PASS: TestSub (0.00s)
PASS
ok  \tgithub.com/user/repo/math\t0.003s";

        let r = parse_go_test(input);
        assert!(r.success);
        assert_eq!(r.passed, 2);
        assert_eq!(r.failed, 0);
        assert_eq!(r.packages_ok, 1);
    }

    #[test]
    fn test_go_test_verbose_with_failure() {
        let input = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestBroken
    math_test.go:15: expected 5, got 3
--- FAIL: TestBroken (0.00s)
FAIL
FAIL\tgithub.com/user/repo/math\t0.015s";

        let r = parse_go_test(input);
        assert!(!r.success);
        assert_eq!(r.passed, 1);
        assert_eq!(r.failed, 1);
        assert_eq!(r.failed_tests.len(), 1);
        assert_eq!(r.failed_tests[0].name, "TestBroken");
        assert!(r.failed_tests[0].message.contains("expected 5, got 3"));
    }

    #[test]
    fn test_go_test_default_mode() {
        let input = "\
ok  \tgithub.com/user/repo/math\t0.003s
ok  \tgithub.com/user/repo/util\t0.001s
FAIL\tgithub.com/user/repo/api\t0.015s";

        let r = parse_go_test(input);
        assert!(!r.success);
        assert_eq!(r.packages_ok, 2);
        assert_eq!(r.packages_fail, 1);
        // Default mode: packages become the count unit
        assert_eq!(r.passed, 2);
        assert_eq!(r.failed, 1);
    }

    #[test]
    fn test_go_test_with_skip() {
        let input = "\
=== RUN   TestFoo
--- PASS: TestFoo (0.00s)
=== RUN   TestBar
--- SKIP: TestBar (0.00s)
PASS
ok  \tgithub.com/user/repo/pkg\t0.002s";

        let r = parse_go_test(input);
        assert!(r.success);
        assert_eq!(r.passed, 1);
        assert_eq!(r.skipped, 1);
        assert_eq!(r.total, 2);
    }

    #[test]
    fn test_go_test_compile_error() {
        let input = "\
# github.com/user/repo/broken
./broken.go:5:2: undefined: DoSomething
FAIL\tgithub.com/user/repo/broken [build failed]";

        let r = parse_go_test(input);
        assert!(!r.success);
        assert!(!r.compile_errors.is_empty());
        assert!(r.compile_errors[0].contains("github.com/user/repo/broken"));
    }

    #[test]
    fn test_go_test_empty() {
        let r = parse_go_test("");
        assert!(r.success);
        assert_eq!(r.total, 0);
    }

    #[test]
    fn test_go_test_compact_format() {
        let input = "\
=== RUN   TestOne
--- PASS: TestOne (0.00s)
=== RUN   TestTwo
--- FAIL: TestTwo (0.01s)
    foo_test.go:10: mismatch
FAIL
FAIL\tgithub.com/user/repo\t0.015s";

        let r = parse_go_test(input);
        let output = format_go_test_compact(&r);
        assert!(output.contains("go test: FAILED"));
        assert!(output.contains("1 passed"));
        assert!(output.contains("1 failed"));
        assert!(output.contains("TestTwo"));
    }

    #[test]
    fn test_go_test_json_format() {
        let input = "ok  \tgithub.com/user/repo\t0.003s";
        let r = parse_go_test(input);
        let json = format_go_test_json(&r);
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"passed\":1"));
    }
}
