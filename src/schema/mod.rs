//! Stable JSON schemas for TARS CLI reducers.
//!
//! This module provides stable, versioned JSON schemas for all reducer outputs.
//! These schemas are designed to be:
//!
//! - **Stable**: Breaking changes require a major version bump
//! - **Documented**: All fields have clear descriptions
//! - **Serializable**: All types implement `Serialize` and `Deserialize`
//! - **Versioned**: Schema version is included in output for compatibility
//!
//! # Schema Categories
//!
//! - **Git**: `GitStatusSchema`, `GitDiffSchema`, `RepositoryStateSchema`
//! - **File System**: `LsOutputSchema`, `FindOutputSchema`
//! - **Search**: `GrepOutputSchema`
//! - **Test Runners**: `TestOutputSchema` (unified for all runners)
//! - **Logs**: `LogsOutputSchema`
//! - **Process**: `ProcessOutputSchema`
//!
//! # Versioning
//!
//! All schemas include a `schema_version` field. The version follows semantic versioning:
//!
//! - **Major version**: Breaking changes (field removal, type changes)
//! - **Minor version**: Additive changes (new optional fields)
//! - **Patch version**: Documentation or internal changes
//!
//! Current schema version: 1.0.0

mod fs;
mod git;
mod logs;
mod process;
mod search;
mod test;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

// Re-export all public types (used by formatters and tests)
#[allow(unused_imports)]
pub use fs::{
    FindCounts, FindEntry, FindError, FindOutputSchema, LsCounts, LsEntry, LsEntryType, LsError,
    LsOutputSchema,
};
#[allow(unused_imports)]
pub use git::{
    GitDiffCounts, GitDiffEntry, GitDiffSchema, GitFileEntry, GitStatusCounts, GitStatusSchema,
    RepositoryStateSchema,
};
#[allow(unused_imports)]
pub use logs::{LogCounts, LogEntry, LogLevel, LogsOutputSchema, RepeatedLine};
pub use process::{ErrorSchema, ProcessOutputSchema};
#[allow(unused_imports)]
pub use search::{
    GrepCounts, GrepFile, GrepMatch, GrepOutputSchema, ReplaceCounts, ReplaceFile, ReplaceMatch,
    ReplaceOutputSchema,
};
#[allow(unused_imports)]
pub use test::{TestOutputSchema, TestResult, TestRunnerType, TestStatus, TestSuite, TestSummary};

// ============================================================
// Schema Version
// ============================================================

/// Current schema version for all output types.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Version information included in all schema outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaVersion {
    /// Schema version string (semver).
    pub version: String,
    /// Schema type identifier.
    #[serde(rename = "type")]
    pub schema_type: String,
}

impl SchemaVersion {
    /// Create a new schema version for the given type.
    pub fn new(schema_type: &str) -> Self {
        Self {
            version: SCHEMA_VERSION.to_string(),
            schema_type: schema_type.to_string(),
        }
    }
}
