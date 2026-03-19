use super::*;

#[test]
fn test_detect_language() {
    assert_eq!(detect_language(&PathBuf::from("main.rs")), Language::Rust);
    assert_eq!(detect_language(&PathBuf::from("app.py")), Language::Python);
    assert_eq!(detect_language(&PathBuf::from("index.ts")), Language::JavaScript);
    assert_eq!(detect_language(&PathBuf::from("data.json")), Language::Data);
    assert_eq!(detect_language(&PathBuf::from("config.yaml")), Language::Data);
    assert_eq!(detect_language(&PathBuf::from("Cargo.toml")), Language::Data);
}

#[test]
fn test_minimal_filter_strips_comments() {
    let input = "use std::io;\n// This is a comment\nfn main() {\n    // another\n    println!(\"hi\");\n}\n";
    let result = filter_minimal(input, Language::Rust);
    assert!(!result.contains("// This is a comment"));
    assert!(!result.contains("// another"));
    assert!(result.contains("fn main()"));
    assert!(result.contains("println!"));
}

#[test]
fn test_minimal_filter_preserves_todo() {
    let input = "// TODO: fix this\n// regular comment\nfn foo() {}\n";
    let result = filter_minimal(input, Language::Rust);
    assert!(result.contains("TODO"));
    assert!(!result.contains("regular comment"));
}

#[test]
fn test_aggressive_filter_rust() {
    let input = r#"use std::io;

/// Doc comment
pub fn hello(name: &str) -> String {
    let greeting = format!("Hello, {}!", name);
    println!("{}", greeting);
    greeting
}

pub struct Config {
    pub name: String,
    pub value: i32,
}

const MAX: usize = 100;
"#;
    let result = filter_aggressive(input, Language::Rust);
    assert!(result.contains("use std::io"));
    assert!(result.contains("pub fn hello"));
    assert!(result.contains("pub struct Config"));
    assert!(result.contains("const MAX"));
    assert!(!result.contains("let greeting"));
    assert!(!result.contains("println!"));
}

#[test]
fn test_aggressive_filter_python() {
    let input = r#"import os
from pathlib import Path

MAX_SIZE = 1024

class MyClass:
    def __init__(self):
        self.value = 0
        self.name = "test"

    def method(self):
        result = self.value + 1
        return result

def standalone():
    x = 10
    return x
"#;
    let result = filter_aggressive(input, Language::Python);
    assert!(result.contains("import os"));
    assert!(result.contains("from pathlib"));
    assert!(result.contains("MAX_SIZE = 1024"));
    assert!(result.contains("class MyClass"));
    assert!(result.contains("def __init__"));
    assert!(result.contains("def method"));
    assert!(result.contains("def standalone"));
    assert!(!result.contains("self.value = 0"));
    assert!(!result.contains("x = 10"));
}

#[test]
fn test_data_files_passthrough() {
    let json = r#"{"key": "value", "// not a comment": true}"#;
    let result = filter_minimal(json, Language::Data);
    // Data files are handled at the caller level, but minimal shouldn't touch them
    // since is_comment_line returns false for Language::Unknown-like
    assert!(result.contains("// not a comment"));
}

#[test]
fn test_line_range_head() {
    let lines = vec!["a", "b", "c", "d", "e"];
    assert_eq!(apply_line_range(&lines, Some(3), None), vec!["a", "b", "c"]);
}

#[test]
fn test_line_range_tail() {
    let lines = vec!["a", "b", "c", "d", "e"];
    assert_eq!(apply_line_range(&lines, None, Some(2)), vec!["d", "e"]);
}

#[test]
fn test_count_braces_ignores_strings() {
    assert_eq!(count_braces(r#"let s = "{}"; {"#), 1);
    assert_eq!(count_braces("fn foo() {"), 1);
    assert_eq!(count_braces("}"), -1);
    assert_eq!(count_braces("{ }"), 0);
}
