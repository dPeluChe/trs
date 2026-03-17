//! Truncation detection and configuration for reducer output.

use serde::Serialize;

// ============================================================
// Truncation Detection
// ============================================================

/// Information about truncation detection.
///
/// Truncation occurs when output is cut off due to limits, size thresholds,
/// or other constraints. This structure tracks when and why truncation occurred.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TruncationInfo {
    /// Whether the output was truncated.
    pub is_truncated: bool,
    /// Total number of items available before truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_available: Option<usize>,
    /// Number of items included after truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_shown: Option<usize>,
    /// Number of items hidden due to truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_hidden: Option<usize>,
    /// Reason for truncation (e.g., "limit", "size_threshold", "timeout").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Threshold value that caused truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<usize>,
    /// Warning message to display about truncation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl TruncationInfo {
    /// Create a new truncation info indicating no truncation.
    pub fn none() -> Self {
        Self {
            is_truncated: false,
            ..Default::default()
        }
    }

    /// Create truncation info for a limit-based truncation.
    pub fn limited(total: usize, shown: usize, limit: usize) -> Self {
        let hidden = total.saturating_sub(shown);
        Self {
            is_truncated: hidden > 0,
            total_available: Some(total),
            items_shown: Some(shown),
            items_hidden: Some(hidden),
            reason: Some("limit".to_string()),
            threshold: Some(limit),
            warning: if hidden > 0 {
                Some(format!(
                    "Output truncated: showing {} of {} items (limit: {})",
                    shown, total, limit
                ))
            } else {
                None
            },
        }
    }

    /// Create truncation info for a size-based truncation.
    pub fn size_threshold(total_bytes: usize, shown_bytes: usize, threshold_bytes: usize) -> Self {
        Self {
            is_truncated: true,
            total_available: Some(total_bytes),
            items_shown: Some(shown_bytes),
            items_hidden: Some(total_bytes.saturating_sub(shown_bytes)),
            reason: Some("size_threshold".to_string()),
            threshold: Some(threshold_bytes),
            warning: Some(format!(
                "Output truncated: showing {} of {} bytes (threshold: {})",
                shown_bytes, total_bytes, threshold_bytes
            )),
        }
    }

    /// Create truncation info detected from output patterns.
    pub fn detected(pattern: &str, original_size: usize) -> Self {
        Self {
            is_truncated: true,
            total_available: None,
            items_shown: Some(original_size),
            items_hidden: None,
            reason: Some("detected".to_string()),
            threshold: None,
            warning: Some(format!(
                "Truncation detected: output appears to be incomplete (pattern: '{}')",
                pattern
            )),
        }
    }

    /// Check if output appears to be truncated based on common patterns.
    pub fn detect_from_output(output: &str) -> Self {
        // Common truncation indicators
        let truncation_patterns = [
            // Generic truncation markers
            "... truncated",
            "...truncated",
            "[truncated]",
            "<truncated>",
            "output truncated",
            // CLI tool truncation messages
            "results limited to",
            "showing first",
            "showing top",
            "more results available",
            "... and more",
            "...and more",
            "use --limit",
            "use -n to see more",
            // System truncation
            "output too large",
            "output size limit",
            "max output exceeded",
        ];

        let output_lower = output.to_lowercase();
        for pattern in truncation_patterns {
            if output_lower.contains(pattern) {
                return Self::detected(pattern, output.len());
            }
        }

        // Check for incomplete JSON/array patterns
        if Self::detect_incomplete_json(output) {
            return Self::detected("incomplete_json", output.len());
        }

        // Check for cut-off lines
        if Self::detect_cutoff_line(output) {
            return Self::detected("cutoff_line", output.len());
        }

        Self::none()
    }

    /// Detect if output contains incomplete JSON.
    fn detect_incomplete_json(output: &str) -> bool {
        let trimmed = output.trim();

        // Check for incomplete array
        if trimmed.starts_with('[') && !trimmed.ends_with(']') {
            return true;
        }

        // Check for incomplete object
        if trimmed.starts_with('{') && !trimmed.ends_with('}') {
            return true;
        }

        // Check for truncated JSON value
        if trimmed.ends_with("...") || trimmed.ends_with(",") {
            // Likely truncated if it looks like JSON
            if trimmed.starts_with('[') || trimmed.starts_with('{') {
                return true;
            }
        }

        false
    }

    /// Detect if the last line appears to be cut off.
    fn detect_cutoff_line(output: &str) -> bool {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return false;
        }

        let last_line = lines.last().unwrap();

        // Check for common cutoff patterns
        // Line ending with incomplete word or punctuation
        if last_line.ends_with("...") && last_line.len() > 3 {
            return true;
        }

        // Line that looks like it should continue
        // (ends with connecting words or operators)
        let cutoff_indicators = [" and", " or", " with", ",", " -", " +", " &"];
        for indicator in cutoff_indicators {
            if last_line.to_lowercase().ends_with(indicator) {
                return true;
            }
        }

        false
    }

    /// Returns true if truncation was detected.
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    /// Get a human-readable summary of the truncation.
    pub fn summary(&self) -> Option<String> {
        if !self.is_truncated {
            return None;
        }

        if let Some(ref warning) = self.warning {
            Some(warning.clone())
        } else if let (Some(shown), Some(hidden)) = (self.items_shown, self.items_hidden) {
            Some(format!("Truncated: {} items shown, {} hidden", shown, hidden))
        } else {
            Some("Output was truncated".to_string())
        }
    }
}

/// Configuration for truncation detection and handling.
#[derive(Debug, Clone)]
pub struct TruncationConfig {
    /// Maximum number of items to show before truncating.
    pub max_items: Option<usize>,
    /// Maximum output size in bytes before truncating.
    pub max_bytes: Option<usize>,
    /// Whether to detect truncation from output patterns.
    pub detect_patterns: bool,
    /// Whether to include truncation warnings in output.
    pub include_warnings: bool,
}

impl Default for TruncationConfig {
    fn default() -> Self {
        Self {
            max_items: None,
            max_bytes: None,
            detect_patterns: true,
            include_warnings: true,
        }
    }
}

impl TruncationConfig {
    /// Create a new truncation config with item limit.
    pub fn with_max_items(max_items: usize) -> Self {
        Self {
            max_items: Some(max_items),
            ..Default::default()
        }
    }

    /// Create a new truncation config with byte limit.
    pub fn with_max_bytes(max_bytes: usize) -> Self {
        Self {
            max_bytes: Some(max_bytes),
            ..Default::default()
        }
    }

    /// Apply truncation to a list of items.
    pub fn truncate_items<T>(&self, items: Vec<T>) -> (Vec<T>, TruncationInfo) {
        let total = items.len();

        if let Some(limit) = self.max_items {
            if total > limit {
                let shown: Vec<T> = items.into_iter().take(limit).collect();
                return (shown, TruncationInfo::limited(total, limit, limit));
            }
        }

        (items, TruncationInfo::none())
    }

    /// Apply truncation to output string.
    pub fn truncate_output(&self, output: String) -> (String, TruncationInfo) {
        let total_bytes = output.len();

        if let Some(limit) = self.max_bytes {
            if total_bytes > limit {
                let truncated = output.chars().take(limit).collect();
                return (truncated, TruncationInfo::size_threshold(total_bytes, limit, limit));
            }
        }

        // Detect truncation patterns if enabled
        if self.detect_patterns {
            let info = TruncationInfo::detect_from_output(&output);
            if info.is_truncated {
                return (output, info);
            }
        }

        (output, TruncationInfo::none())
    }
}
