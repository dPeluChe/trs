//! Loaders for the output fixtures the parser tests read.

mod git_diff;
mod git_status;
mod grep;
mod logs;
mod ls;

#[allow(unused_imports)]
pub use git_diff::*;
#[allow(unused_imports)]
pub use git_status::*;
#[allow(unused_imports)]
pub use grep::*;
#[allow(unused_imports)]
pub use logs::*;
#[allow(unused_imports)]
pub use ls::*;

use std::path::PathBuf;

/// Returns the path to the fixtures directory.
fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixture_data");
    path
}

/// Loads a fixture file by name and returns its contents.
///
/// # Panics
///
/// Panics if the fixture file cannot be read.
pub fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e))
}
