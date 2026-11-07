use super::*;

// ============================================================
// All precedence combinations tests
// ============================================================

#[test]
fn test_precedence_json_over_all() {
    // JSON should win over all other formats
    let cli = Cli::try_parse_from([
        "trs",
        "--json",
        "--csv",
        "--tsv",
        "--agent",
        "--compact",
        "--raw",
        "search",
        ".",
        "test",
    ])
    .unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_csv_over_all_except_json() {
    // CSV should win over all except JSON
    let cli = Cli::try_parse_from([
        "trs",
        "--csv",
        "--tsv",
        "--agent",
        "--compact",
        "--raw",
        "search",
        ".",
        "test",
    ])
    .unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_tsv_over_all_except_json_csv() {
    // TSV should win over all except JSON and CSV
    let cli = Cli::try_parse_from([
        "trs",
        "--tsv",
        "--agent",
        "--compact",
        "--raw",
        "search",
        ".",
        "test",
    ])
    .unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_precedence_agent_over_compact_raw() {
    // Agent should win over Compact and Raw
    let cli = Cli::try_parse_from([
        "trs",
        "--agent",
        "--compact",
        "--raw",
        "search",
        ".",
        "test",
    ])
    .unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

#[test]
fn test_precedence_compact_over_raw() {
    // Compact should win over Raw
    let cli = Cli::try_parse_from(["trs", "--compact", "--raw", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Compact);
}

#[test]
fn test_precedence_json_over_csv() {
    let cli = Cli::try_parse_from(["trs", "--json", "--csv", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_json_over_tsv() {
    let cli = Cli::try_parse_from(["trs", "--json", "--tsv", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_json_over_agent() {
    let cli = Cli::try_parse_from(["trs", "--json", "--agent", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_json_over_compact() {
    let cli = Cli::try_parse_from(["trs", "--json", "--compact", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_json_over_raw() {
    let cli = Cli::try_parse_from(["trs", "--json", "--raw", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_csv_over_tsv() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_csv_over_agent() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--agent", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_csv_over_compact() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--compact", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_csv_over_raw() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--raw", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_tsv_over_agent() {
    let cli = Cli::try_parse_from(["trs", "--tsv", "--agent", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_precedence_tsv_over_compact() {
    let cli = Cli::try_parse_from(["trs", "--tsv", "--compact", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_precedence_tsv_over_raw() {
    let cli = Cli::try_parse_from(["trs", "--tsv", "--raw", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_precedence_agent_over_compact() {
    let cli = Cli::try_parse_from(["trs", "--agent", "--compact", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

#[test]
fn test_precedence_agent_over_raw() {
    let cli = Cli::try_parse_from(["trs", "--agent", "--raw", "search", ".", "test"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

// ============================================================
// Tests with different commands (ensure global flags work)
// ============================================================

#[test]
fn test_precedence_with_run_command() {
    let cli = Cli::try_parse_from(["trs", "--json", "--csv", "run", "echo", "hello"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_precedence_with_parse_command() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "parse", "git-status"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_precedence_with_replace_command() {
    let cli =
        Cli::try_parse_from(["trs", "--tsv", "--agent", "replace", ".", "old", "new"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_precedence_with_tail_command() {
    let cli =
        Cli::try_parse_from(["trs", "--agent", "--compact", "tail", "/var/log/test.log"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

#[test]
fn test_precedence_with_clean_command() {
    let cli = Cli::try_parse_from(["trs", "--compact", "--raw", "clean"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Compact);
}

#[test]
fn test_precedence_with_html2md_command() {
    let cli = Cli::try_parse_from(["trs", "--raw", "html2md", "https://example.com"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Raw);
}

#[test]
fn test_precedence_with_txt2md_command() {
    let cli = Cli::try_parse_from(["trs", "--json", "txt2md"]).unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

// ============================================================
// Stdin input tests
// ============================================================

#[test]
fn test_stdin_no_command() {
    // When no command is provided, command should be None
    let cli = Cli::try_parse_from(["trs"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.command.is_none());
}

#[test]
fn test_stdin_with_format_flags() {
    // Format flags should work even without a command
    let cli = Cli::try_parse_from(["trs", "--json"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.output_format(), OutputFormat::Json);

    let cli = Cli::try_parse_from(["trs", "--csv"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.output_format(), OutputFormat::Csv);

    let cli = Cli::try_parse_from(["trs", "--raw"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.output_format(), OutputFormat::Raw);
}
