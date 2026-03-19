use clap::Subcommand;
use std::path::PathBuf;

use super::TestRunner;

#[derive(Debug, Subcommand)]
pub enum ParseCommands {
    /// Parse git status output
    ///
    /// Transforms git status output into structured format showing
    /// branch info, staged/unstaged files, and untracked files.
    ///
    /// Example: git status | trs parse git-status
    GitStatus {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Output only the count for the specified category (staged, unstaged, untracked, unmerged)
        /// Default: unstaged
        #[arg(long)]
        count: Option<String>,
    },

    /// Parse git diff output
    ///
    /// Transforms git diff output into structured format showing
    /// changed files and summary statistics.
    ///
    /// Example: git diff | trs parse git-diff
    GitDiff {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse ls output
    ///
    /// Transforms ls output into structured format separating
    /// directories, files, and hidden items.
    ///
    /// Example: ls -la | trs parse ls
    Ls {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse grep output
    ///
    /// Transforms grep results into structured format grouping
    /// matches by file with line numbers.
    ///
    /// Example: grep -rn "pattern" . | trs parse grep
    Grep {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse find output
    ///
    /// Transforms find results into structured format categorizing
    /// files, directories, and other entries by type.
    ///
    /// Example: find . -name "*.rs" | trs parse find
    Find {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse test runner output
    ///
    /// Transforms test runner output into structured format showing
    /// passed/failed/skipped counts and execution time.
    ///
    /// Supported runners: pytest, jest, vitest, npm, pnpm, bun
    ///
    /// Example: pytest | trs parse test --runner pytest
    Test {
        /// Test runner type (pytest, jest, vitest, npm, pnpm, bun)
        #[arg(short = 't', long, value_enum)]
        runner: Option<TestRunner>,

        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse log/tail output
    ///
    /// Transforms log streams into structured format detecting
    /// repeated lines and error/warning levels.
    ///
    /// Example: tail -f /var/log/app.log | trs parse logs
    Logs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git log output
    ///
    /// Example: git log | trs parse git-log
    GitLog {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git branch output
    ///
    /// Example: git branch -a | trs parse git-branch
    GitBranch {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse tree command output
    ///
    /// Example: tree | trs parse tree
    Tree {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse docker ps output
    ///
    /// Example: docker ps | trs parse docker-ps
    DockerPs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse docker logs output
    ///
    /// Example: docker logs container | trs parse docker-logs
    DockerLogs {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse dependency list output (npm ls, pip list, cargo tree)
    ///
    /// Example: npm ls | trs parse deps
    Deps {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse package install output (npm install, pip install, cargo build)
    ///
    /// Example: npm install | trs parse install
    Install {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse build output (cargo build, tsc, gcc, make)
    ///
    /// Example: cargo build | trs parse build
    Build {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse environment variables
    ///
    /// Example: env | trs parse env
    Env {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse wc (word count) output
    ///
    /// Example: wc file.txt | trs parse wc
    Wc {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse wget/curl download output
    ///
    /// Example: curl -v https://example.com 2>&1 | trs parse download
    Download {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh pr list output
    ///
    /// Example: gh pr list | trs parse gh-pr
    GhPr {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh issue list output
    ///
    /// Example: gh issue list | trs parse gh-issue
    GhIssue {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh run list output
    ///
    /// Example: gh run list | trs parse gh-run
    GhRun {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse cargo test output
    ///
    /// Example: cargo test 2>&1 | trs parse cargo-test
    CargoTest {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}
