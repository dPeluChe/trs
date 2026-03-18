//! Configuration system for trs.
//!
//! Loads settings from:
//! 1. `.trs/config.toml` (project-local, highest priority)
//! 2. `~/.trs/config.toml` (user-global)
//! 3. Built-in defaults (lowest priority)

use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get the global config (loaded once, cached).
pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}

/// Top-level configuration.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub limits: Limits,
}

/// Output caps and truncation limits.
/// Users can tune these to avoid LLM retry loops from over-aggressive truncation.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Limits {
    /// Max grep matches shown across all files.
    pub grep_max_results: usize,
    /// Max grep matches shown per file.
    pub grep_max_per_file: usize,
    /// Max staged/modified files shown in git status.
    pub status_max_files: usize,
    /// Max untracked files shown in git status.
    pub status_max_untracked: usize,
    /// Max chars for passthrough fallback (tier 3).
    pub passthrough_max_chars: usize,
    /// Max depth for `trs json`.
    pub json_max_depth: usize,
    /// Max keys per object in `trs json`.
    pub json_keys_per_object: usize,
    /// Max lines for tee output file (0 = unlimited).
    pub tee_max_bytes: usize,
    /// Max tee files to keep.
    pub tee_max_files: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            grep_max_results: 200,
            grep_max_per_file: 25,
            status_max_files: 15,
            status_max_untracked: 10,
            passthrough_max_chars: 2000,
            json_max_depth: 10,
            json_keys_per_object: 30,
            tee_max_bytes: 1_048_576, // 1 MB
            tee_max_files: 20,
        }
    }
}

impl Config {
    /// Load config from project-local or user-global TOML, falling back to defaults.
    fn load() -> Self {
        // Try project-local first
        if let Some(cfg) = Self::try_load(&PathBuf::from(".trs/config.toml")) {
            return cfg;
        }

        // Try user-global
        if let Some(home) = std::env::var("HOME").ok() {
            let global = PathBuf::from(home).join(".trs").join("config.toml");
            if let Some(cfg) = Self::try_load(&global) {
                return cfg;
            }
        }

        // Built-in defaults
        Self::default()
    }

    /// Try to load a config file. Returns None on any error (silent fallback).
    fn try_load(path: &PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        match toml::from_str::<Config>(&content) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                eprintln!("trs: warning: failed to parse {}: {}", path.display(), e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.limits.grep_max_results, 200);
        assert_eq!(cfg.limits.grep_max_per_file, 25);
        assert_eq!(cfg.limits.status_max_files, 15);
        assert_eq!(cfg.limits.passthrough_max_chars, 2000);
        assert_eq!(cfg.limits.tee_max_bytes, 1_048_576);
    }

    #[test]
    fn test_parse_partial_toml() {
        let toml_str = r#"
[limits]
grep_max_results = 500
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.limits.grep_max_results, 500);
        // Other fields use defaults
        assert_eq!(cfg.limits.grep_max_per_file, 25);
        assert_eq!(cfg.limits.status_max_files, 15);
    }

    #[test]
    fn test_parse_empty_toml() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.limits.grep_max_results, 200);
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
[limits]
grep_max_results = 100
grep_max_per_file = 10
status_max_files = 20
status_max_untracked = 5
passthrough_max_chars = 3000
json_max_depth = 5
json_keys_per_object = 15
tee_max_bytes = 524288
tee_max_files = 10
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.limits.grep_max_results, 100);
        assert_eq!(cfg.limits.json_max_depth, 5);
        assert_eq!(cfg.limits.tee_max_files, 10);
    }
}
