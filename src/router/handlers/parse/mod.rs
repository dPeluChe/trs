use super::common::{CommandContext, CommandError, CommandResult};
use super::types::*;
use crate::ParseCommands;

pub(crate) mod git;
pub(crate) mod ls;
pub(crate) mod grep;
pub(crate) mod find;
pub(crate) mod test;
pub(crate) mod test_pytest;
pub(crate) mod test_jest;
pub(crate) mod test_vitest;
pub(crate) mod test_npm;
pub(crate) mod test_pnpm;
pub(crate) mod test_bun;
pub(crate) mod logs;
pub(crate) mod extra;

pub(crate) struct ParseHandler;

impl ParseHandler {
    /// Read input from a file or stdin.
    /// Handles both UTF-8 and binary input gracefully by replacing invalid
    /// UTF-8 sequences with the Unicode replacement character.
    pub(crate) fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use std::io::{self, Read};

        if let Some(path) = file {
            let bytes = std::fs::read(path).map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        } else {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| CommandError::IoError(e.to_string()))?;
            Ok(String::from_utf8_lossy(&buffer).into_owned())
        }
    }

    /// Convert serde_json::Value to pretty-printed JSON string.
    pub(crate) fn json_to_string(value: serde_json::Value) -> String {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
    }

    /// Format a byte size into a human-readable string (e.g. 1.2K, 3.5M).
    pub(crate) fn format_human_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}K", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl CommandHandler for ParseHandler {
    type Input = ParseCommands;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        match input {
            ParseCommands::GitStatus { file, count } => Self::handle_git_status(file, count, ctx),
            ParseCommands::GitDiff { file } => Self::handle_git_diff(file, ctx),
            ParseCommands::GitLog { file } => Self::handle_git_log(file, ctx),
            ParseCommands::GitBranch { file } => Self::handle_git_branch(file, ctx),
            ParseCommands::Ls { file } => Self::handle_ls(file, ctx),
            ParseCommands::Grep { file } => Self::handle_grep(file, ctx),
            ParseCommands::Find { file } => Self::handle_find(file, ctx),
            ParseCommands::Test { runner, file } => Self::handle_test(runner, file, ctx),
            ParseCommands::Logs { file } => Self::handle_logs(file, ctx),
            ParseCommands::Tree { file } => Self::handle_tree(file, ctx),
            ParseCommands::DockerPs { file } => Self::handle_docker_ps(file, ctx),
            ParseCommands::DockerLogs { file } => Self::handle_docker_logs(file, ctx),
            ParseCommands::Deps { file } => Self::handle_deps(file, ctx),
            ParseCommands::Install { file } => Self::handle_install(file, ctx),
            ParseCommands::Build { file } => Self::handle_build(file, ctx),
            ParseCommands::Env { file } => Self::handle_env(file, ctx),
        }
    }
}
