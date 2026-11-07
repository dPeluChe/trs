//! Versioned JSON schemas for `trs search` and `trs replace` output.
//!
//! Every schema carries a `schema` object, `{version, type}` (semver): a major bump for
//! removed or retyped fields, a minor one for new optional fields.

mod search;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use search::{
    GrepCounts, GrepFile, GrepMatch, GrepOutputSchema, ReplaceCounts, ReplaceFile, ReplaceMatch,
    ReplaceOutputSchema,
};

// ============================================================
// Schema Version
// ============================================================

/// Current schema version.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// The `schema` object in each output.
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
