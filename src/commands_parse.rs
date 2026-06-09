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

        /// Truncate commit subjects to N chars (adds "..."). Default: no truncation.
        /// Useful when piping to a fixed-width context.
        #[arg(long)]
        truncate: Option<usize>,
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

    /// Parse ping output — collapses per-packet lines into a single summary
    /// (host, ratio, loss %, avg/range latency).
    ///
    /// Example: ping -c 3 8.8.8.8 | trs parse ping
    Ping {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse `brew install|upgrade|reinstall` output — drops progress bars
    /// and fetch/pour chatter, keeps the 🍺 install-result lines and errors.
    ///
    /// Example: brew install wget | trs parse brew
    Brew {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse Python traceback output — collapse full paths to basename,
    /// drop code snippet lines under each File frame, keep the stack
    /// frame list and the final ErrorType: message. Output passthrough
    /// when no traceback is detected so normal script output is
    /// untouched.
    ///
    /// Example: python3 script.py 2>&1 | trs parse python-traceback
    #[command(name = "python-traceback", alias = "python")]
    PythonTraceback {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse `ps aux` / `ps -ef` output — truncate multi-kilobyte
    /// COMMAND arguments to the executable basename, sort by CPU
    /// descending, show the top 30 with a summary footer. Agents
    /// scanning for a specific process or the top CPU hogs can skim
    /// the compacted view in 1-2% of the raw bytes.
    ///
    /// Example: ps aux | trs parse ps
    Ps {
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

    /// Parse gh pr checks output
    ///
    /// Example: gh pr checks 123 | trs parse gh-pr-checks
    GhPrChecks {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh run view output
    ///
    /// Example: gh run view 12345 | trs parse gh-run-view
    GhRunView {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse gh pr view output
    ///
    /// Example: gh pr view 123 | trs parse gh-pr-view
    GhPrView {
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

    /// Parse go test output
    ///
    /// Example: go test ./... 2>&1 | trs parse go-test
    GoTest {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse lint output (eslint, clippy, ruff, biome, golangci-lint, tsc)
    ///
    /// Groups issues by file and rule, shows error/warning counts.
    /// Example: cargo clippy 2>&1 | trs parse lint
    Lint {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse database query output (psql, mysql, sqlite3)
    ///
    /// Auto-detects tabular format and extracts columns, rows, and metadata.
    /// Large results are truncated with head/tail sampling.
    ///
    /// Example: psql -c "SELECT * FROM users" | trs parse db
    Db {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git pull / git fetch output — strips remote progress noise,
    /// keeps branch update lines and the file-change summary.
    ///
    /// Example: git pull | trs parse git-pull
    GitPull {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse git commit output — keeps the header and summary line,
    /// collapses per-file create/delete/rename mode lines into counts.
    ///
    /// Example: git commit -m "msg" | trs parse git-commit
    GitCommit {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Parse cargo fmt --check output — collapses per-file diff blocks
    /// into a file list with diff counts.
    ///
    /// Example: cargo fmt --check | trs parse fmt
    Fmt {
        /// Input file (stdin if not specified)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

impl ParseCommands {
    /// Return a copy of this variant with `file` set to `path`.
    pub(crate) fn with_file(self, path: PathBuf) -> Self {
        match self {
            Self::GitStatus { count, .. } => Self::GitStatus {
                file: Some(path),
                count,
            },
            Self::GitDiff { .. } => Self::GitDiff { file: Some(path) },
            Self::GitLog { truncate, .. } => Self::GitLog {
                file: Some(path),
                truncate,
            },
            Self::GitBranch { .. } => Self::GitBranch { file: Some(path) },
            Self::Ls { .. } => Self::Ls { file: Some(path) },
            Self::Grep { .. } => Self::Grep { file: Some(path) },
            Self::Find { .. } => Self::Find { file: Some(path) },
            Self::Test { runner, .. } => Self::Test {
                runner,
                file: Some(path),
            },
            Self::Logs { .. } => Self::Logs { file: Some(path) },
            Self::Tree { .. } => Self::Tree { file: Some(path) },
            Self::DockerPs { .. } => Self::DockerPs { file: Some(path) },
            Self::DockerLogs { .. } => Self::DockerLogs { file: Some(path) },
            Self::Ping { .. } => Self::Ping { file: Some(path) },
            Self::Brew { .. } => Self::Brew { file: Some(path) },
            Self::PythonTraceback { .. } => Self::PythonTraceback { file: Some(path) },
            Self::Ps { .. } => Self::Ps { file: Some(path) },
            Self::Deps { .. } => Self::Deps { file: Some(path) },
            Self::Install { .. } => Self::Install { file: Some(path) },
            Self::Build { .. } => Self::Build { file: Some(path) },
            Self::Env { .. } => Self::Env { file: Some(path) },
            Self::Wc { .. } => Self::Wc { file: Some(path) },
            Self::Download { .. } => Self::Download { file: Some(path) },
            Self::GhPr { .. } => Self::GhPr { file: Some(path) },
            Self::GhIssue { .. } => Self::GhIssue { file: Some(path) },
            Self::GhRun { .. } => Self::GhRun { file: Some(path) },
            Self::GhPrChecks { .. } => Self::GhPrChecks { file: Some(path) },
            Self::GhRunView { .. } => Self::GhRunView { file: Some(path) },
            Self::GhPrView { .. } => Self::GhPrView { file: Some(path) },
            Self::CargoTest { .. } => Self::CargoTest { file: Some(path) },
            Self::GoTest { .. } => Self::GoTest { file: Some(path) },
            Self::Lint { .. } => Self::Lint { file: Some(path) },
            Self::Db { .. } => Self::Db { file: Some(path) },
            Self::GitPull { .. } => Self::GitPull { file: Some(path) },
            Self::GitCommit { .. } => Self::GitCommit { file: Some(path) },
            Self::Fmt { .. } => Self::Fmt { file: Some(path) },
        }
    }
}
