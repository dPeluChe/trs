use super::common::{CommandContext, CommandError, CommandResult, CommandStats};
use super::types::CommandHandler;
use crate::process::ProcessBuilder;
use crate::OutputFormat;

pub(crate) struct IsCleanHandler;

impl IsCleanHandler {
    /// Check if the git repository is in a clean state.
    pub(crate) fn check_repo_state(check_untracked: bool) -> CommandResult<RepositoryState> {
        // Run git status --porcelain to get machine-readable output
        let output = ProcessBuilder::new("git")
            .args(vec!["status", "--porcelain"])
            .capture_stdout(true)
            .capture_stderr(true)
            .capture_exit_code(true)
            .capture_duration(true)
            .run();

        match output {
            Ok(process_output) => {
                // If git command failed, it might not be a git repository
                if !process_output.success() {
                    return Ok(RepositoryState {
                        is_git_repo: false,
                        is_detached: false,
                        branch: None,
                        is_clean: false,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                        input_bytes: 0,
                    });
                }

                let stdout = process_output.stdout;
                let input_bytes = stdout.len();

                // Empty output means clean repository
                if stdout.trim().is_empty() {
                    return Ok(RepositoryState {
                        is_git_repo: true,
                        is_detached: false,
                        branch: None,
                        is_clean: true,
                        staged_count: 0,
                        unstaged_count: 0,
                        untracked_count: 0,
                        unmerged_count: 0,
                        input_bytes,
                    });
                }

                // Parse porcelain output to count different change types
                let mut staged_count = 0;
                let mut unstaged_count = 0;
                let mut untracked_count = 0;
                let mut unmerged_count = 0;

                for line in stdout.lines() {
                    if line.len() < 2 {
                        continue;
                    }

                    let index_status = line.chars().next().unwrap_or(' ');
                    let worktree_status = line.chars().nth(1).unwrap_or(' ');

                    // Check for unmerged (conflict) states
                    if index_status == 'U'
                        || worktree_status == 'U'
                        || index_status == 'A' && worktree_status == 'A'
                        || index_status == 'D' && worktree_status == 'D'
                    {
                        unmerged_count += 1;
                        continue;
                    }

                    // Check for untracked files
                    if index_status == '?' && worktree_status == '?' {
                        untracked_count += 1;
                        continue;
                    }

                    // Check for staged changes (index status)
                    if index_status != ' ' && index_status != '?' {
                        staged_count += 1;
                    }

                    // Check for unstaged changes (worktree status)
                    if worktree_status != ' ' && worktree_status != '?' {
                        unstaged_count += 1;
                    }
                }

                // Determine if clean based on flags
                let is_clean = if check_untracked {
                    staged_count == 0
                        && unstaged_count == 0
                        && untracked_count == 0
                        && unmerged_count == 0
                } else {
                    staged_count == 0 && unstaged_count == 0 && unmerged_count == 0
                };

                Ok(RepositoryState {
                    is_git_repo: true,
                    is_detached: false,
                    branch: None,
                    is_clean,
                    staged_count,
                    unstaged_count,
                    untracked_count,
                    unmerged_count,
                    input_bytes,
                })
            }
            Err(_) => {
                // git command failed - likely not a git repository
                Ok(RepositoryState {
                    is_git_repo: false,
                    is_detached: false,
                    branch: None,
                    is_clean: false,
                    staged_count: 0,
                    unstaged_count: 0,
                    untracked_count: 0,
                    unmerged_count: 0,
                    input_bytes: 0,
                })
            }
        }
    }

    /// Format repository state for output.
    pub(crate) fn format_output(state: &RepositoryState, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => Self::format_json(state),
            OutputFormat::Compact | OutputFormat::Agent => Self::format_compact(state),
            OutputFormat::Raw => Self::format_raw(state),
            OutputFormat::Csv | OutputFormat::Tsv => Self::format_compact(state),
        }
    }

    pub(crate) fn format_json(state: &RepositoryState) -> String {
        serde_json::json!({
            "is_git_repo": state.is_git_repo,
            "is_clean": state.is_clean,
            "staged_count": state.staged_count,
            "unstaged_count": state.unstaged_count,
            "untracked_count": state.untracked_count,
            "unmerged_count": state.unmerged_count,
        })
        .to_string()
    }

    pub(crate) fn format_compact(state: &RepositoryState) -> String {
        if !state.is_git_repo {
            return "not a git repository\n".to_string();
        }

        if state.is_clean {
            return "clean\n".to_string();
        }

        format!(
            "dirty (staged={} unstaged={} untracked={} unmerged={})\n",
            state.staged_count, state.unstaged_count, state.untracked_count, state.unmerged_count
        )
    }

    pub(crate) fn format_raw(state: &RepositoryState) -> String {
        if state.is_clean {
            "clean\n".to_string()
        } else {
            "dirty\n".to_string()
        }
    }
}

impl CommandHandler for IsCleanHandler {
    type Input = IsCleanInput;

    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult {
        let state = Self::check_repo_state(input.check_untracked.unwrap_or(true))?;

        // Format and print output
        let formatted = Self::format_output(&state, ctx.format);

        // Print stats if requested
        if ctx.stats {
            let total_changes = state.staged_count
                + state.unstaged_count
                + state.untracked_count
                + state.unmerged_count;
            let stats = CommandStats::new()
                .with_reducer("is-clean")
                .with_output_mode(ctx.format)
                .with_input_bytes(state.input_bytes)
                .with_output_bytes(formatted.len())
                .with_items_processed(total_changes)
                .with_extra("Is git repo", state.is_git_repo.to_string())
                .with_extra("Is clean", state.is_clean.to_string())
                .with_extra("Staged", state.staged_count.to_string())
                .with_extra("Unstaged", state.unstaged_count.to_string())
                .with_extra("Untracked", state.untracked_count.to_string())
                .with_extra("Unmerged", state.unmerged_count.to_string());
            stats.print();
        }

        print!("{}", formatted);

        // Exit with appropriate code:
        // 0 - clean
        // 1 - dirty (has changes)
        // 2 - not a git repository
        if !state.is_git_repo {
            return Err(CommandError::ExecutionError {
                message: "not a git repository".to_string(),
                exit_code: Some(2),
            });
        }

        if !state.is_clean {
            return Err(CommandError::ExecutionError {
                message: format!(
                    "repository has changes (staged={} unstaged={} untracked={} unmerged={})",
                    state.staged_count,
                    state.unstaged_count,
                    state.untracked_count,
                    state.unmerged_count
                ),
                exit_code: Some(1),
            });
        }

        Ok(())
    }
}

/// Repository state information.
#[derive(Debug, Clone)]
pub(crate) struct RepositoryState {
    /// Whether this is a git repository.
    is_git_repo: bool,
    /// Whether the repository is in a detached HEAD state.
    #[allow(dead_code)]
    is_detached: bool,
    /// The current branch name (or commit hash if detached).
    #[allow(dead_code)]
    branch: Option<String>,
    /// Whether the repository is clean (no changes).
    is_clean: bool,
    /// Number of staged files.
    staged_count: usize,
    /// Number of unstaged files.
    unstaged_count: usize,
    /// Number of untracked files.
    untracked_count: usize,
    /// Number of unmerged (conflict) files.
    unmerged_count: usize,
    /// Size of git status output in bytes.
    input_bytes: usize,
}

/// Input data for the `is-clean` command.
#[derive(Debug, Clone)]
pub(crate) struct IsCleanInput {
    pub check_untracked: Option<bool>,
}

