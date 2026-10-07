use super::*;
use crate::OutputFormat;

fn split(args: &[&str]) -> (Vec<String>, Option<OutputFormat>, bool) {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    split_format_flags(&args)
}

fn kept(args: &[&str]) -> Vec<String> {
    split(args).0
}

#[test]
fn a_flag_the_program_defines_goes_to_the_program() {
    // `gh run list --json x` used to reach gh as `gh run list x`.
    let (args, format, _) = split(&["gh", "run", "list", "--json", "id,url"]);
    assert_eq!(args, ["gh", "run", "list", "--json", "id,url"]);
    assert!(format.is_none());
    assert_eq!(kept(&["npm", "ls", "--json"]), ["npm", "ls", "--json"]);
    assert_eq!(
        kept(&["rg", "x", "--json", "--stats"]),
        ["rg", "x", "--json", "--stats"]
    );
}

#[test]
fn git_raw_is_gits_for_diff_and_log_even_after_global_options() {
    assert_eq!(
        kept(&["git", "log", "--raw", "-1"]),
        ["git", "log", "--raw", "-1"]
    );
    assert_eq!(
        kept(&["git", "-C", "repo", "diff", "--raw"]),
        ["git", "-C", "repo", "diff", "--raw"]
    );
}

#[test]
fn a_flag_trs_formats_for_is_still_trs() {
    let (args, format, _) = split(&["git", "status", "--json"]);
    assert_eq!(args, ["git", "status"]);
    assert!(matches!(format, Some(OutputFormat::Json)));
    let (args, format, stats) = split(&["git", "status", "--raw", "--stats"]);
    assert_eq!(args, ["git", "status"]);
    assert!(matches!(format, Some(OutputFormat::Raw)) && stats);
}

#[test]
fn commands_trs_has_no_formatter_for_keep_every_flag() {
    assert_eq!(
        kept(&[
            "echo",
            "a",
            "--json",
            "--raw",
            "--csv",
            "--tsv",
            "--agent",
            "--compact",
            "--stats"
        ]),
        [
            "echo",
            "a",
            "--json",
            "--raw",
            "--csv",
            "--tsv",
            "--agent",
            "--compact",
            "--stats"
        ]
    );
    assert_eq!(
        kept(&["terraform", "output", "--json"]),
        ["terraform", "output", "--json"]
    );
    assert_eq!(
        kept(&["kubectl", "get", "--raw", "/api"]),
        ["kubectl", "get", "--raw", "/api"]
    );
}

#[test]
fn nothing_after_a_double_dash_is_trs() {
    assert_eq!(
        kept(&["git", "log", "--", "--json"]),
        ["git", "log", "--", "--json"]
    );
}

#[test]
fn args_without_our_flags_are_untouched() {
    let (args, format, stats) = split(&["git", "status", "-sb"]);
    assert_eq!(args, ["git", "status", "-sb"]);
    assert!(format.is_none() && !stats);
}

#[test]
fn structured_output_requests_are_recognized() {
    let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(has_structured_output_flag(&a(&[
        "run", "list", "--json", "id"
    ])));
    assert!(has_structured_output_flag(&a(&["status", "--porcelain"])));
    assert!(!has_structured_output_flag(&a(&["status", "-sb"])));
}
