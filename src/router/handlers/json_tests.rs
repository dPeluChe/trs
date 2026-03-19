use super::*;

#[test]
fn test_simple_object() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"name":"Alice","age":30,"active":true}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("\"name\": String"));
    assert!(buf.contains("\"age\": Number"));
    assert!(buf.contains("\"active\": Bool"));
}

#[test]
fn test_array_of_objects() {
    let json: serde_json::Value = serde_json::from_str(r#"[{"id":1},{"id":2},{"id":3}]"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[3]"));
}

#[test]
fn test_empty_structures() {
    let json: serde_json::Value = serde_json::from_str(r#"{"items":[],"meta":{}}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[0]"));
    assert!(buf.contains("{}"));
}

#[test]
fn test_nested_depth_limit() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"a":{"b":{"c":{"d":"deep"}}}}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, 2);
    assert!(buf.contains("Object{1 keys}"));
}

#[test]
fn test_long_string_shows_length() {
    let long = "x".repeat(100);
    let json: serde_json::Value =
        serde_json::from_str(&format!(r#"{{"content":"{}"}}"#, long)).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("String[100]"));
}

#[test]
fn test_null_value() {
    let json: serde_json::Value = serde_json::from_str(r#"{"value":null}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Null"));
}

#[test]
fn test_schema_json_output() {
    let json: serde_json::Value = serde_json::from_str(r#"{"name":"test","count":5}"#).unwrap();
    let schema = to_schema_json(&json, 0, usize::MAX);
    assert_eq!(schema["name"], "String");
    assert_eq!(schema["count"], "Number");
}

// ============================================================
// Array sampling tests
// ============================================================

#[test]
fn test_small_array_no_sampling() {
    // Arrays with <=5 items should NOT be sampled
    let json: serde_json::Value =
        serde_json::from_str(r#"[{"x":1},{"x":2},{"x":3},{"x":4},{"x":5}]"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[5]"));
    assert!(!buf.contains("sampled"));
}

#[test]
fn test_large_array_sampling() {
    // Arrays with >5 items should show sampling annotation
    let items: Vec<String> = (0..10).map(|i| format!(r#"{{"val":{}}}"#, i)).collect();
    let json_str = format!("[{}]", items.join(","));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[10]"));
    assert!(buf.contains("sampled: first 3 + last 3"));
}

#[test]
fn test_large_array_sampling_numbers() {
    // Homogeneous number array - still sampled
    let json: serde_json::Value = serde_json::from_str("[1,2,3,4,5,6,7,8,9,10]").unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[10] of Number"));
    assert!(buf.contains("sampled: first 3 + last 3"));
}

#[test]
fn test_sample_array_preserves_error_items() {
    // Error items in the middle should be preserved during sampling
    let json_str = r#"[
        {"status":"ok"},
        {"status":"ok"},
        {"status":"ok"},
        {"status":"ok"},
        {"error":"something broke","status":"fail"},
        {"status":"ok"},
        {"status":"ok"},
        {"status":"ok"},
        {"status":"ok"},
        {"status":"ok"}
    ]"#;
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    if let serde_json::Value::Array(arr) = &json {
        let (sampled, was_sampled) = sample_array(arr);
        assert!(was_sampled);
        // Should include first 3 + error item (index 4) + last 3
        // first 3: indices 0,1,2; last 3: indices 7,8,9; error: index 4
        assert!(sampled.len() >= 7); // 3 + 3 + 1 error
                                     // Verify the error item is in the sample
        let has_error = sampled.iter().any(|v| has_error_keys(v));
        assert!(has_error, "Error item should be preserved in sample");
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_sample_array_exactly_at_threshold() {
    // Exactly 5 items: no sampling
    let json: serde_json::Value = serde_json::from_str("[1,2,3,4,5]").unwrap();
    if let serde_json::Value::Array(arr) = &json {
        let (_sampled, was_sampled) = sample_array(arr);
        assert!(!was_sampled);
    }
}

#[test]
fn test_sample_array_just_above_threshold() {
    // 6 items: sampling kicks in
    let json: serde_json::Value = serde_json::from_str("[1,2,3,4,5,6]").unwrap();
    if let serde_json::Value::Array(arr) = &json {
        let (sampled, was_sampled) = sample_array(arr);
        assert!(was_sampled);
        // first 3 + last 3 = all 6 (overlap), so all items present
        assert_eq!(sampled.len(), 6);
    }
}

#[test]
fn test_large_array_json_schema_output() {
    let items: Vec<String> = (0..10).map(|i| format!(r#"{{"val":{}}}"#, i)).collect();
    let json_str = format!("[{}]", items.join(","));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let schema = to_schema_json(&json, 0, usize::MAX);
    assert_eq!(schema["_type"], "Array");
    assert_eq!(schema["_length"], 10);
    assert!(schema["_sampled"]
        .as_str()
        .unwrap()
        .contains("first 3 + last 3"));
}

// ============================================================
// ID field detection tests
// ============================================================

#[test]
fn test_id_key_detection() {
    assert!(is_id_key("id"));
    assert!(is_id_key("Id"));
    assert!(is_id_key("ID"));
    assert!(is_id_key("uuid"));
    assert!(is_id_key("UUID"));
    assert!(is_id_key("_id"));
    assert!(is_id_key("user_id"));
    assert!(is_id_key("User_Id"));
    assert!(is_id_key("account_id"));
    // Should NOT match
    assert!(!is_id_key("name"));
    assert!(!is_id_key("identity"));
    assert!(!is_id_key("video"));
    assert!(!is_id_key("ideas"));
}

#[test]
fn test_id_annotation_in_structure() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"user_id":123,"name":"Alice","uuid":"abc-def"}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("\"user_id\": Number (id)"));
    assert!(buf.contains("\"uuid\": String (id)"));
    // "name" should NOT have (id)
    assert!(buf.contains("\"name\": String"));
    assert!(!buf.contains("\"name\": String (id)"));
}

#[test]
fn test_id_annotation_nested_object() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"data":{"_id":"abc","title":"test"}}"#).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("\"_id\": String (id)"));
    assert!(!buf.contains("\"title\": String (id)"));
}

#[test]
fn test_id_annotation_json_schema() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"id":1,"name":"test","account_id":42}"#).unwrap();
    let schema = to_schema_json(&json, 0, usize::MAX);
    assert_eq!(schema["id"], "Number (id)");
    assert_eq!(schema["account_id"], "Number (id)");
    assert_eq!(schema["name"], "String");
}

// ============================================================
// Non-JSON rejection with hints
// ============================================================

#[test]
fn test_non_json_hint_toml() {
    let err = serde_json::from_str::<serde_json::Value>("[package]\nname = \"foo\"").unwrap_err();
    let path = Some(PathBuf::from("Cargo.toml"));
    let cmd_err = non_json_hint(&path, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Not JSON"), "got: {}", msg);
            assert!(msg.contains("trs read"), "got: {}", msg);
            assert!(msg.contains("Cargo.toml"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

#[test]
fn test_non_json_hint_yaml() {
    let err = serde_json::from_str::<serde_json::Value>("key: value\n").unwrap_err();
    let path = Some(PathBuf::from("config.yaml"));
    let cmd_err = non_json_hint(&path, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Not JSON"), "got: {}", msg);
            assert!(msg.contains("trs read"), "got: {}", msg);
            assert!(msg.contains("config.yaml"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

#[test]
fn test_non_json_hint_yml() {
    let err = serde_json::from_str::<serde_json::Value>("key: value\n").unwrap_err();
    let path = Some(PathBuf::from("config.yml"));
    let cmd_err = non_json_hint(&path, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Not JSON"), "got: {}", msg);
            assert!(msg.contains("trs read"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

#[test]
fn test_non_json_hint_csv() {
    let err = serde_json::from_str::<serde_json::Value>("a,b,c\n1,2,3\n").unwrap_err();
    let path = Some(PathBuf::from("data.csv"));
    let cmd_err = non_json_hint(&path, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Not JSON"), "got: {}", msg);
            assert!(msg.contains("trs parse deps"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

#[test]
fn test_non_json_hint_unknown_ext() {
    let err = serde_json::from_str::<serde_json::Value>("not json at all").unwrap_err();
    let path = Some(PathBuf::from("data.txt"));
    let cmd_err = non_json_hint(&path, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Invalid JSON"), "got: {}", msg);
            assert!(!msg.contains("Not JSON"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

#[test]
fn test_non_json_hint_no_file() {
    let err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let cmd_err = non_json_hint(&None, &err);
    match cmd_err {
        CommandError::InvalidArguments(msg) => {
            assert!(msg.contains("Invalid JSON"), "got: {}", msg);
        }
        other => panic!("Expected InvalidArguments, got: {:?}", other),
    }
}

// ============================================================
// Error key detection tests
// ============================================================

#[test]
fn test_has_error_keys() {
    let obj: serde_json::Value = serde_json::from_str(r#"{"error":"bad","code":500}"#).unwrap();
    assert!(has_error_keys(&obj));

    let obj: serde_json::Value =
        serde_json::from_str(r#"{"exception":"NPE","trace":"..."}"#).unwrap();
    assert!(has_error_keys(&obj));

    let obj: serde_json::Value = serde_json::from_str(r#"{"failed":true}"#).unwrap();
    assert!(has_error_keys(&obj));

    let obj: serde_json::Value = serde_json::from_str(r#"{"failure_reason":"timeout"}"#).unwrap();
    assert!(has_error_keys(&obj));

    let obj: serde_json::Value = serde_json::from_str(r#"{"name":"ok","status":"good"}"#).unwrap();
    assert!(!has_error_keys(&obj));

    // Non-objects never match
    let arr: serde_json::Value = serde_json::from_str("[1,2,3]").unwrap();
    assert!(!has_error_keys(&arr));
}

// ============================================================
// Combined features test
// ============================================================

#[test]
fn test_combined_large_array_with_id_fields() {
    // Large array of objects with ID fields: should sample + annotate IDs
    let items: Vec<String> = (0..10)
        .map(|i| format!(r#"{{"user_id":{},"name":"user{}"}}"#, i, i))
        .collect();
    let json_str = format!(r#"{{"users":{}}}"#, format!("[{}]", items.join(",")));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let mut buf = String::new();
    format_structure(&json, &mut buf, 0, usize::MAX);
    assert!(buf.contains("Array[10]"), "should show array length");
    assert!(buf.contains("sampled"), "should indicate sampling");
    assert!(buf.contains("(id)"), "should annotate id fields");
}
