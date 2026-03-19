use super::*;

// ============================================================
// JSON Formatter Tests
// ============================================================

#[test]
fn test_json_format_message() {
    let output = JsonFormatter::format_message("branch", "main");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
}

#[test]
fn test_json_format_key_value() {
    let output = JsonFormatter::format_key_value("count", 42);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["count"], 42);
}

#[test]
fn test_json_format_object() {
    let output = JsonFormatter::format_object(&[
        ("branch", serde_json::json!("main")),
        ("is_clean", serde_json::json!(true)),
        ("count", serde_json::json!(5)),
    ]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["is_clean"], true);
    assert_eq!(json["count"], 5);
}

#[test]
fn test_json_format_counts() {
    let output = JsonFormatter::format_counts(&[("passed", 10), ("failed", 2)]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 10);
    assert_eq!(json["failed"], 2);
}

#[test]
fn test_json_format_counts_with_zeros() {
    let output = JsonFormatter::format_counts(&[("passed", 0), ("failed", 2)]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 0);
    assert_eq!(json["failed"], 2);
}

#[test]
fn test_json_format_section() {
    let items = vec!["file1.rs", "file2.rs"];
    let output = JsonFormatter::format_section("staged", &items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["staged"].is_array());
    assert_eq!(json["staged"][0], "file1.rs");
    assert_eq!(json["staged"][1], "file2.rs");
}

#[test]
fn test_json_format_item() {
    let output = JsonFormatter::format_item("M", "src/main.rs");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["status"], "M");
    assert_eq!(json["path"], "src/main.rs");
}

#[test]
fn test_json_format_item_renamed() {
    let output = JsonFormatter::format_item_renamed("R", "old.rs", "new.rs");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["status"], "R");
    assert_eq!(json["path"], "new.rs");
    assert_eq!(json["old_path"], "old.rs");
}

#[test]
fn test_json_format_test_summary() {
    let output = JsonFormatter::format_test_summary(10, 2, 1, 1500);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["passed"], 10);
    assert_eq!(json["failed"], 2);
    assert_eq!(json["skipped"], 1);
    assert_eq!(json["total"], 13);
    assert_eq!(json["duration_ms"], 1500);
}

#[test]
fn test_json_format_status() {
    let success_output = JsonFormatter::format_status(true);
    let success_json: serde_json::Value = serde_json::from_str(&success_output).unwrap();
    assert_eq!(success_json["success"], true);

    let failure_output = JsonFormatter::format_status(false);
    let failure_json: serde_json::Value = serde_json::from_str(&failure_output).unwrap();
    assert_eq!(failure_json["success"], false);
}

#[test]
fn test_json_format_failures() {
    let failures = vec!["test_one".to_string(), "test_two".to_string()];
    let output = JsonFormatter::format_failures(&failures);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["failures"].is_array());
    assert_eq!(json["count"], 2);
}

#[test]
fn test_json_format_failures_empty() {
    let failures: Vec<String> = vec![];
    let output = JsonFormatter::format_failures(&failures);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json["failures"].is_array());
    assert_eq!(json["count"], 0);
}

#[test]
fn test_json_format_log_levels() {
    let output = JsonFormatter::format_log_levels(2, 5, 10, 3);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], 2);
    assert_eq!(json["warn"], 5);
    assert_eq!(json["info"], 10);
    assert_eq!(json["debug"], 3);
    assert_eq!(json["total"], 20);
}

#[test]
fn test_json_format_log_levels_with_zeros() {
    let output = JsonFormatter::format_log_levels(0, 5, 0, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], 0);
    assert_eq!(json["warn"], 5);
    assert_eq!(json["total"], 5);
}

#[test]
fn test_json_format_grep_match() {
    let output = JsonFormatter::format_grep_match("src/main.rs", Some(42), "fn main()");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert_eq!(json["line"], 42);
    assert_eq!(json["content"], "fn main()");
}

#[test]
fn test_json_format_grep_match_no_line() {
    let output = JsonFormatter::format_grep_match("src/main.rs", None, "match found");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert!(json["line"].is_null());
}

#[test]
fn test_json_format_grep_file() {
    let output = JsonFormatter::format_grep_file("src/main.rs", 5);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["file"], "src/main.rs");
    assert_eq!(json["match_count"], 5);
}

#[test]
fn test_json_format_diff_file() {
    let output = JsonFormatter::format_diff_file("src/main.rs", "M", 10, 5);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["path"], "src/main.rs");
    assert_eq!(json["change_type"], "M");
    assert_eq!(json["additions"], 10);
    assert_eq!(json["deletions"], 5);
}

#[test]
fn test_json_format_diff_summary() {
    let output = JsonFormatter::format_diff_summary(3, 25, 10);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["files_changed"], 3);
    assert_eq!(json["insertions"], 25);
    assert_eq!(json["deletions"], 10);
}

#[test]
fn test_json_format_clean() {
    let output = JsonFormatter::format_clean();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], true);
}

#[test]
fn test_json_format_dirty() {
    let output = JsonFormatter::format_dirty(2, 3, 5, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], false);
    assert_eq!(json["staged"], 2);
    assert_eq!(json["unstaged"], 3);
    assert_eq!(json["untracked"], 5);
    assert_eq!(json["unmerged"], 0);
}

#[test]
fn test_json_format_branch_with_tracking() {
    let output = JsonFormatter::format_branch_with_tracking("main", 0, 0);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "main");
    assert_eq!(json["ahead"], 0);
    assert_eq!(json["behind"], 0);

    let output = JsonFormatter::format_branch_with_tracking("feature", 3, 2);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["branch"], "feature");
    assert_eq!(json["ahead"], 3);
    assert_eq!(json["behind"], 2);
}

#[test]
fn test_json_format_empty() {
    let output = JsonFormatter::format_empty();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["empty"], true);
}

#[test]
fn test_json_format_truncated() {
    let output = JsonFormatter::format_truncated(10, 50);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_truncated"], true);
    assert_eq!(json["shown"], 10);
    assert_eq!(json["total"], 50);
}

#[test]
fn test_json_format_error() {
    let output = JsonFormatter::format_error("Something went wrong");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], true);
    assert_eq!(json["message"], "Something went wrong");
}

#[test]
fn test_json_format_error_with_code() {
    let output = JsonFormatter::format_error_with_code("Command failed", 1);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["error"], true);
    assert_eq!(json["message"], "Command failed");
    assert_eq!(json["exit_code"], 1);
}

#[test]
fn test_json_format_not_implemented() {
    let output = JsonFormatter::format_not_implemented("Feature X");
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["not_implemented"], true);
    assert_eq!(json["message"], "Feature X");
}

#[test]
fn test_json_format_command_result() {
    let output = JsonFormatter::format_command_result(
        "echo",
        &["hello".to_string(), "world".to_string()],
        "hello world\n",
        "",
        0,
        10,
    );
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["command"], "echo");
    assert!(json["args"].is_array());
    assert_eq!(json["stdout"], "hello world\n");
    assert_eq!(json["stderr"], "");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["duration_ms"], 10);
}

#[test]
fn test_json_format_list() {
    let items = vec!["file1.rs", "file2.rs"];
    let output = JsonFormatter::format_list(&items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0], "file1.rs");
    assert_eq!(json[1], "file2.rs");
}

#[test]
fn test_json_format_count() {
    let output = JsonFormatter::format_count(42);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["count"], 42);
}

#[test]
fn test_json_format_flag() {
    let output = JsonFormatter::format_flag("is_clean", true);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["is_clean"], true);
}

#[test]
fn test_json_format_array() {
    #[derive(serde::Serialize)]
    struct Item {
        name: &'static str,
        value: usize,
    }
    let items = vec![
        Item {
            name: "first",
            value: 1,
        },
        Item {
            name: "second",
            value: 2,
        },
    ];
    let output = JsonFormatter::format_array(&items);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "first");
    assert_eq!(json[1]["value"], 2);
}
