
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

warning: `trs-cli` (bin "trs") generated 2 warnings
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
fn test_parse_tsc_format() {
    let input = "src/components/EmailList.tsx(1,8): error TS6133: 'React' is declared but its value is never read.\n\
src/components/EmailList.tsx(1,38): error TS6133: 'useCallback' is declared but its value is never read.\n\
src/components/EmailListParts.tsx(89,24): error TS2304: Cannot find name 'useCallback'.\n\
src/components/EmailListParts.tsx(7,1): warning TS6133: 'Tooltip' is declared but its value is never read.\n\
Found 3 errors in 2 files.\n";
    let issues = parse_lint_issues(input);
    assert_eq!(issues.len(), 4, "expected 4 issues, got {}", issues.len());
    assert_eq!(issues[0].file, "src/components/EmailList.tsx");
    assert_eq!(issues[0].line, 1);
    assert_eq!(issues[0].col, 8);
    assert_eq!(issues[0].rule, "TS6133");
    assert_eq!(issues[0].level, LintLevel::Error);
    assert_eq!(issues[2].file, "src/components/EmailListParts.tsx");
    assert_eq!(issues[2].rule, "TS2304");
    assert_eq!(issues[3].level, LintLevel::Warning);
}

#[test]
fn test_tsc_compact_output() {
    let input = "src/components/Foo.tsx(1,8): error TS6133: 'React' is declared but its value is never read.\n\
src/components/Foo.tsx(2,1): error TS6133: 'useEffect' is declared but its value is never read.\n\
src/components/Bar.tsx(5,3): warning TS6133: 'X' is declared but its value is never read.\n";
    let issues = parse_lint_issues(input);
    let output = format_lint_compact(&issues, 2, 1);
    assert!(output.contains("lint: 3 (2 errors, 1 warnings) in 2 files"));
    assert!(output.contains("src/components/Foo.tsx (2):"));
    assert!(output.contains("src/components/Bar.tsx (1):"));
    assert!(output.contains("E TS6133"));
    assert!(output.contains("W TS6133"));
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
