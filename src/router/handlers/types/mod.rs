//! Shared data structures and types for command handlers.
//!
//! Contains all parsed output types for git, ls, find, grep, test runners,
//! and log parsers, plus the CommandHandler trait.

pub(crate) mod fs;
pub(crate) mod git;
pub(crate) mod grep_types;
pub(crate) mod logs;
pub(crate) mod test_types_core;
pub(crate) mod test_types_runners;

// Re-export everything so `use super::types::*` keeps working.
pub(crate) use fs::*;
pub(crate) use git::*;
pub(crate) use grep_types::*;
pub(crate) use logs::*;
pub(crate) use test_types_core::*;
pub(crate) use test_types_runners::*;

use super::common::{CommandContext, CommandResult};

// ============================================================
// CommandHandler Trait
// ============================================================

/// Trait for command handlers that parse and reduce command output.
pub trait CommandHandler {
    type Input;
    fn execute(&self, input: &Self::Input, ctx: &CommandContext) -> CommandResult;
}
