use super::ParseCommands;
use std::path::PathBuf;

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
            Self::GhApi { .. } => Self::GhApi { file: Some(path) },
            Self::Ollama { .. } => Self::Ollama { file: Some(path) },
            Self::Du { .. } => Self::Du { file: Some(path) },
            Self::Lsof { .. } => Self::Lsof { file: Some(path) },
            Self::Pgrep { .. } => Self::Pgrep { file: Some(path) },
            Self::Deps { .. } => Self::Deps { file: Some(path) },
            Self::Install { .. } => Self::Install { file: Some(path) },
            Self::Build { .. } => Self::Build { file: Some(path) },
            Self::Aws { .. } => Self::Aws { file: Some(path) },
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
            Self::Maven { .. } => Self::Maven { file: Some(path) },
            Self::Gradle { .. } => Self::Gradle { file: Some(path) },
            Self::Lint { .. } => Self::Lint { file: Some(path) },
            Self::Db { .. } => Self::Db { file: Some(path) },
            Self::GitPull { .. } => Self::GitPull { file: Some(path) },
            Self::GitCommit { .. } => Self::GitCommit { file: Some(path) },
            Self::Fmt { .. } => Self::Fmt { file: Some(path) },
        }
    }
}
