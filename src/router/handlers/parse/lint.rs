use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;
use std::collections::BTreeMap;

/// A single lint issue.
struct LintIssue {
    file: String,
    line: usize,
    col: usize,
    level: LintLevel,
    rule: String,
    message: String,
}

#[derive(Clone, Copy, PartialEq)]
enum LintLevel {
    Error,
    Warning,
}

impl ParseHandler {
    /// Parse lint output from various linters and format compactly.
    /// Supports: eslint, cargo clippy, ruff, biome, golangci-lint, tsc, generic.
    pub(crate) fn handle_lint(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let issues = parse_lint_issues(&input);
        let errors = issues
            .iter()
            .filter(|i| i.level == LintLevel::Error)
            .count();
        let warnings = issues
            .iter()
            .filter(|i| i.level == LintLevel::Warning)
            .count();

        let output = match ctx.format {
            OutputFormat::Json => format_lint_json(&issues, errors, warnings),
            _ => format_lint_compact(&issues, errors, warnings),
        };

        print!("{}", output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("lint")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(issues.len())
                .print();
        }
        Ok(())
    }
}

/// Parse lint issues from mixed linter output.
fn parse_lint_issues(input: &str) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Clippy/rustc format: "warning: message" then " --> file:line:col"
        if (line.starts_with("warning: ")
            || line.starts_with("error: ")
            || line.starts_with("error["))
            && !line.contains("generated")
            && !line.contains("aborting")
            && !line.contains("could not compile")
        {
            let level = if line.starts_with("error") {
                LintLevel::Error
            } else {
                LintLevel::Warning
            };
            let message = line
                .trim_start_matches("error[")
                .split(']')
                .last()
                .unwrap_or(line)
                .trim_start_matches("warning: ")
                .trim_start_matches("error: ")
                .trim_start_matches(": ")
                .to_string();

            // Look for " --> file:line:col" on next lines
            if i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                if let Some(loc) = next.strip_prefix("--> ") {
                    // Parse "file:line:col" from right to avoid path colons
                    let parts: Vec<&str> = loc.rsplitn(3, ':').collect();
                    if parts.len() >= 3 {
                        let col = parts[0].parse().unwrap_or(0);
                        let ln = parts[1].parse().unwrap_or(0);
                        let file = parts[2].to_string();
                        let rule = extract_clippy_rule(&lines, i);
                        issues.push(LintIssue {
                            file,
                            line: ln,
                            col,
                            level,
                            rule,
                            message,
                        });
                    }
                }
            }
            i += 1;
            continue;
        }

        // ESLint/biome format: "  line:col  level  message  rule"
        if line.starts_with(|c: char| c.is_ascii_digit()) {
            let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace()).collect();
            if parts.len() >= 2 {
                if let Some((ln_str, col_str)) = parts[0].split_once(':') {
                    let ln = ln_str.parse().unwrap_or(0);
                    let col = col_str.parse().unwrap_or(0);
                    let rest = parts[1].trim();
                    let (level, rest) = if rest.starts_with("error") {
                        (
                            LintLevel::Error,
                            rest.strip_prefix("error").unwrap_or(rest).trim(),
                        )
                    } else if rest.starts_with("warning") {
                        (
                            LintLevel::Warning,
                            rest.strip_prefix("warning").unwrap_or(rest).trim(),
                        )
                    } else {
                        (LintLevel::Warning, rest)
                    };
                    // Last word is typically the rule
                    let (message, rule) = if let Some(last_space) = rest.rfind("  ") {
                        (
                            rest[..last_space].trim().to_string(),
                            rest[last_space..].trim().to_string(),
                        )
                    } else {
                        (rest.to_string(), String::new())
                    };
                    // Need file context — look backwards for a file path line
                    let file = find_eslint_file_context(&lines, i);
                    issues.push(LintIssue {
                        file,
                        line: ln,
                        col,
                        level,
                        rule,
                        message,
                    });
                }
            }
        }

        // ruff/pylint format: "file.py:line:col: RULE message"
        if line.contains(".py:")
            || line.contains(".ts:")
            || line.contains(".js:")
            || line.contains(".go:")
        {
            if let Some(issue) = parse_colon_format(line) {
                issues.push(issue);
            }
        }

        i += 1;
    }

    issues
}

/// Extract clippy lint rule name from surrounding lines (look for "= help: ... visit .../index.html#rule_name").
fn extract_clippy_rule(lines: &[&str], start: usize) -> String {
    for j in start..lines.len().min(start + 10) {
        let l = lines[j].trim();
        if l.contains("clippy::") || l.contains("#[warn(") || l.contains("#[deny(") {
            if let Some(pos) = l.find("clippy::") {
                let rule = &l[pos..];
                let end = rule
                    .find(|c: char| c == ')' || c == ']' || c == '`')
                    .unwrap_or(rule.len());
                return rule[..end].to_string();
            }
        }
        if l.contains("index.html#") {
            if let Some(pos) = l.rfind('#') {
                return l[pos + 1..].to_string();
            }
        }
        // Stop at next warning/error
        if j > start && (l.starts_with("warning:") || l.starts_with("error:")) {
            break;
        }
    }
    String::new()
}

/// Parse "file:line:col: RULE message" format (ruff, pylint, golangci-lint).
fn parse_colon_format(line: &str) -> Option<LintIssue> {
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() < 4 {
        return None;
    }
    let file = parts[0].trim().to_string();
    let ln = parts[1].trim().parse().unwrap_or(0);
    let col_str = parts[2].trim();
    let col = col_str.parse().unwrap_or(0);

    let rest = if parts.len() > 3 { parts[3].trim() } else { "" };
    // "E401 message" or "error: message" or just "message"
    let (rule, message, level) = if rest
        .starts_with(|c: char| c.is_ascii_uppercase() && c != 'E' && c != 'W')
        || rest.starts_with("E")
        || rest.starts_with("W")
        || rest.starts_with("F")
        || rest.starts_with("C")
    {
        let (rule, msg) = rest.split_once(' ').unwrap_or((rest, ""));
        let level = if rule.starts_with('E') || rule.starts_with('F') {
            LintLevel::Error
        } else {
            LintLevel::Warning
        };
        (rule.to_string(), msg.trim().to_string(), level)
    } else if rest.starts_with("error") {
        let msg = rest.strip_prefix("error:").unwrap_or(rest).trim();
        (String::new(), msg.to_string(), LintLevel::Error)
    } else if rest.starts_with("warning") {
        let msg = rest.strip_prefix("warning:").unwrap_or(rest).trim();
        (String::new(), msg.to_string(), LintLevel::Warning)
    } else {
        (String::new(), rest.to_string(), LintLevel::Warning)
    };

    if ln == 0 && col == 0 {
        return None;
    }

    Some(LintIssue {
        file,
        line: ln,
        col,
        level,
        rule,
        message,
    })
}

/// Look backwards from an ESLint issue line to find the file path.
fn find_eslint_file_context(lines: &[&str], from: usize) -> String {
    for j in (0..from).rev() {
        let l = lines[j].trim();
        if l.contains('/') && !l.starts_with(|c: char| c.is_ascii_digit()) && !l.is_empty() {
            return l.to_string();
        }
    }
    "unknown".to_string()
}

/// Format lint issues in compact grouped output.
fn format_lint_compact(issues: &[LintIssue], errors: usize, warnings: usize) -> String {
    if issues.is_empty() {
        return "lint: clean\n".to_string();
    }

    let mut output = String::new();

    // Header
    let mut parts = Vec::new();
    if errors > 0 {
        parts.push(format!("{} errors", errors));
    }
    if warnings > 0 {
        parts.push(format!("{} warnings", warnings));
    }

    // Count unique files
    let mut files: BTreeMap<&str, Vec<&LintIssue>> = BTreeMap::new();
    for issue in issues {
        files.entry(&issue.file).or_default().push(issue);
    }
    output.push_str(&format!(
        "lint: {} ({}) in {} files\n",
        issues.len(),
        parts.join(", "),
        files.len()
    ));

    // Group by file
    for (file, file_issues) in &files {
        output.push_str(&format!("{} ({}):\n", file, file_issues.len()));
        for issue in file_issues {
            let marker = if issue.level == LintLevel::Error {
                "E"
            } else {
                "W"
            };
            if issue.rule.is_empty() {
                output.push_str(&format!(
                    "  {} {}:{} {}\n",
                    marker, issue.line, issue.col, issue.message
                ));
            } else {
                output.push_str(&format!(
                    "  {} {} {}:{}\n",
                    marker, issue.rule, issue.line, issue.col
                ));
            }
        }
    }

    output
}

/// Format lint issues as JSON.
fn format_lint_json(issues: &[LintIssue], errors: usize, warnings: usize) -> String {
    let json_issues: Vec<serde_json::Value> = issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "file": i.file,
                "line": i.line,
                "col": i.col,
                "level": if i.level == LintLevel::Error { "error" } else { "warning" },
                "rule": i.rule,
                "message": i.message,
            })
        })
        .collect();

    serde_json::json!({
        "total": issues.len(),
        "errors": errors,
        "warnings": warnings,
        "issues": json_issues,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clippy_format() {
        let input = r#"warning: unused import: `OutputFormat`
 --> src/classifier_exec.rs:8:23
  |
8 | use crate::{Commands, OutputFormat};
  |                       ^^^^^^^^^^^^
  = note: `#[warn(unused_imports)]` on by default

warning: redundant closure
  --> src/router/mod.rs:45:35
   |
45 |         .map(|s| s.to_string())
   |                   ^^^^^^^^^^^^ help: replace
   = help: for further information visit https://rust-lang.github.io/rust-clippy/index.html#redundant_closure

warning: `tars-cli` (bin "trs") generated 2 warnings
"#;
        let issues = parse_lint_issues(input);
        assert!(
            issues.len() >= 2,
            "Expected at least 2 issues, got {}",
            issues.len()
        );
        assert_eq!(issues[0].file, "src/classifier_exec.rs");
        assert_eq!(issues[0].line, 8);
    }

    #[test]
    fn test_parse_ruff_colon_format() {
        let input = "src/main.py:10:5: F401 `os` imported but unused\nsrc/main.py:15:1: E302 expected 2 blank lines\n";
        let issues = parse_lint_issues(input);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].rule, "F401");
        assert_eq!(issues[0].line, 10);
        assert_eq!(issues[1].rule, "E302");
    }

    #[test]
    fn test_format_compact_clean() {
        let output = format_lint_compact(&[], 0, 0);
        assert_eq!(output, "lint: clean\n");
    }

    #[test]
    fn test_format_compact_grouped() {
        let issues = vec![
            LintIssue {
                file: "src/a.rs".into(),
                line: 10,
                col: 5,
                level: LintLevel::Error,
                rule: "E001".into(),
                message: "bad".into(),
            },
            LintIssue {
                file: "src/a.rs".into(),
                line: 20,
                col: 1,
                level: LintLevel::Warning,
                rule: "W001".into(),
                message: "meh".into(),
            },
            LintIssue {
                file: "src/b.rs".into(),
                line: 5,
                col: 3,
                level: LintLevel::Error,
                rule: "E002".into(),
                message: "worse".into(),
            },
        ];
        let output = format_lint_compact(&issues, 2, 1);
        assert!(output.contains("lint: 3 (2 errors, 1 warnings)"));
        assert!(output.contains("src/a.rs (2):"));
        assert!(output.contains("src/b.rs (1):"));
        assert!(output.contains("E E001 10:5"));
        assert!(output.contains("W W001 20:1"));
    }

    #[test]
    fn test_format_json() {
        let issues = vec![LintIssue {
            file: "a.py".into(),
            line: 1,
            col: 1,
            level: LintLevel::Error,
            rule: "F401".into(),
            message: "unused".into(),
        }];
        let output = format_lint_json(&issues, 1, 0);
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["total"], 1);
        assert_eq!(json["errors"], 1);
        assert_eq!(json["issues"][0]["rule"], "F401");
    }
}
