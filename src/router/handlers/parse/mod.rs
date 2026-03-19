use super::common::{CommandContext, CommandError, CommandResult};
use super::types::*;
use crate::ParseCommands;

pub(crate) mod bun_format;
pub(crate) mod bun_parse;
pub(crate) mod extra_cargo_test;
pub(crate) mod extra_download;
pub(crate) mod extra_env;
pub(crate) mod extra_services;
pub(crate) mod extra_system;
pub(crate) mod find;
pub(crate) mod git_branch;
pub(crate) mod git_diff;
pub(crate) mod git_diff_format;
pub(crate) mod git_log;
pub(crate) mod git_status;
pub(crate) mod git_status_format;
pub(crate) mod grep;
pub(crate) mod grep_format;
pub(crate) mod jest_format;
pub(crate) mod jest_parse;
pub(crate) mod logs;
pub(crate) mod logs_format;
pub(crate) mod logs_helpers;
pub(crate) mod ls;
pub(crate) mod npm_format;
pub(crate) mod npm_parse;
pub(crate) mod pnpm_format;
pub(crate) mod pnpm_parse;
pub(crate) mod pytest_format;
pub(crate) mod pytest_parse;
pub(crate) mod test;
pub(crate) mod vitest_format;
pub(crate) mod vitest_parse;

pub(crate) struct ParseHandler;

impl ParseHandler {
    /// Read input from a file or stdin.
    /// Handles both UTF-8 and binary input gracefully by replacing invalid
    /// UTF-8 sequences with the Unicode replacement character.
    pub(crate) fn read_input(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
        use super::common::strip_emojis;
        use std::io::{self, Read};

        let raw = if let Some(path) = file {
            let bytes = std::fs::read(path).map_err(|e| CommandError::IoError(e.to_string()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        } else {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|e| CommandError::IoError(e.to_string()))?;
            String::from_utf8_lossy(&buffer).into_owned()
        };

        // Strip emojis by default — they waste tokens and confuse non-Claude LLMs
        Ok(strip_emojis(&raw))
    }

    /// Read input without emoji stripping (for parsers that need emoji context).
    /// Use sparingly — only when emojis carry semantic meaning (e.g., status indicators).
    pub(crate) fn read_input_raw(file: &Option<std::path::PathBuf>) -> CommandResult<String> {
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
            ParseCommands::Wc { file } => Self::handle_wc(file, ctx),
            ParseCommands::Download { file } => Self::handle_download(file, ctx),
            ParseCommands::GhPr { file } => Self::handle_gh_pr(file, ctx),
            ParseCommands::GhIssue { file } => Self::handle_gh_issue(file, ctx),
            ParseCommands::GhRun { file } => Self::handle_gh_run(file, ctx),
            ParseCommands::CargoTest { file } => Self::handle_cargo_test(file, ctx),
        }
    }
}
