use super::*;

#[test]
fn test_output_format_default() {
    let cli = Cli::try_parse_from(["trs", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Compact);
}

#[test]
fn test_output_format_json_precedence() {
    let cli = Cli::try_parse_from(["trs", "--json", "--compact", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_output_format_csv() {
    let cli = Cli::try_parse_from(["trs", "--csv", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_output_format_tsv() {
    let cli = Cli::try_parse_from(["trs", "--tsv", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_output_format_agent() {
    let cli = Cli::try_parse_from(["trs", "--agent", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

#[test]
fn test_output_format_raw() {
    let cli = Cli::try_parse_from(["trs", "--raw", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Raw);
}

#[test]
fn test_output_format_compact() {
    let cli = Cli::try_parse_from(["trs", "--compact", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Compact);
}

#[test]
fn test_output_format_precedence_json_over_csv() {
    let cli = Cli::try_parse_from(["trs", "--json", "--csv", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Json);
}

#[test]
fn test_output_format_precedence_csv_over_tsv() {
    let cli = Cli::try_parse_from(["trs", "--csv", "--tsv", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Csv);
}

#[test]
fn test_output_format_precedence_tsv_over_agent() {
    let cli = Cli::try_parse_from(["trs", "--tsv", "--agent", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Tsv);
}

#[test]
fn test_output_format_precedence_agent_over_compact() {
    let cli = Cli::try_parse_from(["trs", "--agent", "--compact", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.output_format(), OutputFormat::Agent);
}

#[test]
fn test_stats_flag() {
    let cli = Cli::try_parse_from(["trs", "--stats", "search", ".", "test"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert!(cli.stats);
}

#[test]
fn test_search_command_parsing() {
    let cli = Cli::try_parse_from([
        "trs",
        "search",
        "/path/to/dir",
        "pattern",
        "--extension",
        "rs",
        "--ignore-case",
        "--context",
        "3",
        "--limit",
        "100",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Search {
            path,
            query,
            extension,
            ignore_case,
            context,
            limit,
        } => {
            assert_eq!(path, &std::path::PathBuf::from("/path/to/dir"));
            assert_eq!(query, "pattern");
            assert_eq!(extension, &Some("rs".to_string()));
            assert!(*ignore_case);
            assert_eq!(context, &Some(3));
            assert_eq!(limit, &Some(100));
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_replace_command_parsing() {
    let cli = Cli::try_parse_from([
        "trs",
        "replace",
        "/path/to/dir",
        "old",
        "new",
        "--extension",
        "ts",
        "--dry-run",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Replace {
            path,
            search,
            replace,
            extension,
            dry_run,
            count,
        } => {
            assert_eq!(path, &std::path::PathBuf::from("/path/to/dir"));
            assert_eq!(search, "old");
            assert_eq!(replace, "new");
            assert_eq!(extension, &Some("ts".to_string()));
            assert!(*dry_run);
            assert!(!*count);
        }
        _ => panic!("Expected Replace command"),
    }
}

#[test]
fn test_replace_command_parsing_with_count() {
    let cli = Cli::try_parse_from([
        "trs",
        "replace",
        "/path/to/dir",
        "old",
        "new",
        "--extension",
        "ts",
        "--dry-run",
        "--count",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Replace {
            path,
            search,
            replace,
            extension,
            dry_run,
            count,
        } => {
            assert_eq!(path, &std::path::PathBuf::from("/path/to/dir"));
            assert_eq!(search, "old");
            assert_eq!(replace, "new");
            assert_eq!(extension, &Some("ts".to_string()));
            assert!(*dry_run);
            assert!(*count);
        }
        _ => panic!("Expected Replace command"),
    }
}

#[test]
fn test_tail_command_parsing() {
    let cli = Cli::try_parse_from([
        "trs",
        "tail",
        "/var/log/app.log",
        "--lines",
        "50",
        "--errors",
        "--follow",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Tail {
            file,
            lines,
            errors,
            follow,
        } => {
            assert_eq!(file, &std::path::PathBuf::from("/var/log/app.log"));
            assert_eq!(*lines, 50);
            assert!(*errors);
            assert!(*follow);
        }
        _ => panic!("Expected Tail command"),
    }
}

#[test]
fn test_clean_command_parsing() {
    let cli = Cli::try_parse_from([
        "trs",
        "clean",
        "--file",
        "input.txt",
        "--no-ansi",
        "--collapse-blanks",
        "--collapse-repeats",
        "--trim",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Clean {
            file,
            no_ansi,
            collapse_blanks,
            collapse_repeats,
            trim,
        } => {
            assert_eq!(file, &Some(std::path::PathBuf::from("input.txt")));
            assert!(*no_ansi);
            assert!(*collapse_blanks);
            assert!(*collapse_repeats);
            assert!(*trim);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_parse_git_status() {
    let cli = Cli::try_parse_from(["trs", "parse", "git-status"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Parse { parser } => match parser {
            ParseCommands::GitStatus { file, .. } => {
                assert!(file.is_none());
            }
            _ => panic!("Expected GitStatus parser"),
        },
        _ => panic!("Expected Parse command"),
    }
}

#[test]
fn test_parse_test_runner() {
    let cli = Cli::try_parse_from(["trs", "parse", "test", "--runner", "pytest"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Parse { parser } => match parser {
            ParseCommands::Test { runner, file } => {
                assert_eq!(*runner, Some(TestRunner::Pytest));
                assert!(file.is_none());
            }
            _ => panic!("Expected Test parser"),
        },
        _ => panic!("Expected Parse command"),
    }
}

#[test]
fn test_html2md_command() {
    let cli = Cli::try_parse_from([
        "trs",
        "html2md",
        "https://example.com",
        "--output",
        "out.md",
        "--metadata",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Html2md {
            input,
            output,
            metadata,
        } => {
            assert_eq!(input, "https://example.com");
            assert_eq!(output, &Some(std::path::PathBuf::from("out.md")));
            assert!(*metadata);
        }
        _ => panic!("Expected Html2md command"),
    }
}

#[test]
fn test_txt2md_command() {
    let cli = Cli::try_parse_from([
        "trs",
        "txt2md",
        "--input",
        "input.txt",
        "--output",
        "out.md",
    ]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    match cli.command.as_ref().unwrap() {
        Commands::Txt2md { input, output } => {
            assert_eq!(input, &Some(std::path::PathBuf::from("input.txt")));
            assert_eq!(output, &Some(std::path::PathBuf::from("out.md")));
        }
        _ => panic!("Expected Txt2md command"),
    }
}

#[cfg(test)]
#[path = "main_tests_precedence.rs"]
mod precedence;
