use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;

impl ParseHandler {
    pub(crate) fn handle_test(
        runner: &Option<crate::TestRunner>,
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        // Read input from file or stdin
        let input = Self::read_input(file)?;

        // Parse based on the runner type (default to pytest)
        let (output, passed, failed, skipped) = match runner {
            Some(crate::TestRunner::Pytest) | None => {
                let test_output = Self::parse_pytest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.passed,
                    test_output.summary.failed,
                    test_output.summary.skipped,
                );
                let output = Self::format_pytest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Jest) => {
                let test_output = Self::parse_jest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_jest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Vitest) => {
                let test_output = Self::parse_vitest(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_vitest(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Npm) => {
                let test_output = Self::parse_npm_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_npm_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Pnpm) => {
                let test_output = Self::parse_pnpm_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_pnpm_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
            Some(crate::TestRunner::Bun) => {
                let test_output = Self::parse_bun_test(&input)?;
                let (passed, failed, skipped) = (
                    test_output.summary.tests_passed,
                    test_output.summary.tests_failed,
                    test_output.summary.tests_skipped,
                );
                let output = Self::format_bun_test(&test_output, ctx.format);
                (output, passed, failed, skipped)
            }
        };

        // Print stats if requested
        if ctx.stats {
            let stats = CommandStats::new()
                .with_reducer("test")
                .with_output_mode(ctx.format)
                .with_input_bytes(input.len())
                .with_output_bytes(output.len())
                .with_items_processed(passed + failed + skipped)
                .with_extra("Passed", passed.to_string())
                .with_extra("Failed", failed.to_string())
                .with_extra("Skipped", skipped.to_string());
            stats.print();
        }

        print!("{}", output);

        Ok(())
    }
}
