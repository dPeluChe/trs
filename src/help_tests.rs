use super::*;

#[test]
fn test_get_command_help_search() {
    let help = get_command_help("search");
    assert!(help.is_some());
    assert!(help.unwrap().contains("ripgrep"));
}

#[test]
fn test_get_command_help_replace() {
    let help = get_command_help("replace");
    assert!(help.is_some());
    assert!(help.unwrap().contains("dry-run"));
}

#[test]
fn test_get_command_help_tail() {
    let help = get_command_help("tail");
    assert!(help.is_some());
    assert!(help.unwrap().contains("--follow"));
}

#[test]
fn test_get_command_help_clean() {
    let help = get_command_help("clean");
    assert!(help.is_some());
    assert!(help.unwrap().contains("--no-ansi"));
}

#[test]
fn test_get_command_help_parse() {
    let help = get_command_help("parse");
    assert!(help.is_some());
    assert!(help.unwrap().contains("git-status"));
}

#[test]
fn test_get_command_help_html2md() {
    let help = get_command_help("html2md");
    assert!(help.is_some());
    assert!(help.unwrap().contains("Markdown"));
}

#[test]
fn test_get_command_help_txt2md() {
    let help = get_command_help("txt2md");
    assert!(help.is_some());
    assert!(help.unwrap().contains("plain text"));
}

#[test]
fn test_get_command_help_run() {
    let help = get_command_help("run");
    assert!(help.is_some());
    assert!(help.unwrap().contains("Execute a command"));
}

#[test]
fn test_get_command_help_trim() {
    let help = get_command_help("trim");
    assert!(help.is_some());
    assert!(help.unwrap().contains("whitespace"));
}

#[test]
fn test_get_command_help_unknown() {
    let help = get_command_help("unknown");
    assert!(help.is_none());
}

#[test]
fn test_format_precedence_help() {
    let help = get_format_precedence_help();
    assert!(help.contains("JSON"));
    assert!(help.contains("CSV"));
    assert!(help.contains("TSV"));
    assert!(help.contains("Agent"));
    assert!(help.contains("Compact"));
    assert!(help.contains("Raw"));
}

#[test]
fn test_long_about_not_empty() {
    assert!(!LONG_ABOUT.is_empty());
    assert!(LONG_ABOUT.contains("TARS CLI"));
}
