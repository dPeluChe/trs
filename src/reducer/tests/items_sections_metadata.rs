use super::*;

// ============================================================
// ReducerItem Tests
// ============================================================

#[test]
fn test_reducer_item_new() {
    let item = ReducerItem::new("key", "value");

    assert_eq!(item.key, "key");
    assert_eq!(item.value, "value");
    assert!(item.label.is_none());
    assert!(item.data.is_none());
}

#[test]
fn test_reducer_item_with_label() {
    let item = ReducerItem::new("key", "value").with_label("label");

    assert_eq!(item.label, Some("label".to_string()));
}

#[test]
fn test_reducer_item_with_data() {
    let data = serde_json::json!({"extra": "info"});
    let item = ReducerItem::new("key", "value").with_data(data.clone());

    assert_eq!(item.data, Some(data));
}

// ============================================================
// ReducerSection Tests
// ============================================================

#[test]
fn test_reducer_section_new() {
    let section = ReducerSection::new("Test Section");

    assert_eq!(section.name, "Test Section");
    assert!(section.count.is_none());
    assert!(section.items.is_empty());
}

#[test]
fn test_reducer_section_with_count() {
    let section = ReducerSection::new("Test").with_count(10);

    assert_eq!(section.count, Some(10));
}

#[test]
fn test_reducer_section_with_items() {
    let items = vec![
        ReducerItem::new("a", "1"),
        ReducerItem::new("b", "2"),
    ];

    let section = ReducerSection::new("Test").with_items(items);

    assert_eq!(section.items.len(), 2);
}

#[test]
fn test_reducer_section_add_item() {
    let mut section = ReducerSection::new("Test");
    section.add_item(ReducerItem::new("a", "1"));

    assert_eq!(section.items.len(), 1);
    assert_eq!(section.items[0].key, "a");
}

// ============================================================
// ReducerMetadata Tests
// ============================================================

#[test]
fn test_reducer_metadata_default() {
    let metadata = ReducerMetadata::default();

    assert!(metadata.reducer.is_empty());
    assert_eq!(metadata.items_processed, 0);
    assert_eq!(metadata.items_filtered, 0);
    assert_eq!(metadata.duration_ms, 0);
    assert!(metadata.custom.is_none());
}
