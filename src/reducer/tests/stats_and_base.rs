use super::*;

// ============================================================
// ReducerStats Tests
// ============================================================

#[test]
fn test_reducer_stats_default() {
    let stats = ReducerStats::default();

    assert_eq!(stats.input_bytes, 0);
    assert_eq!(stats.output_bytes, 0);
    assert_eq!(stats.reduction_ratio, 0.0);
    assert_eq!(stats.input_lines, 0);
    assert_eq!(stats.output_lines, 0);
    assert_eq!(stats.input_tokens, 0);
    assert_eq!(stats.output_tokens, 0);
    assert_eq!(stats.token_reduction_ratio, 0.0);
}

#[test]
fn test_reducer_stats_new() {
    let stats = ReducerStats::new(4000, 1000, 200, 50);

    assert_eq!(stats.input_bytes, 4000);
    assert_eq!(stats.output_bytes, 1000);
    assert_eq!(stats.reduction_ratio, 0.25);
    assert_eq!(stats.input_lines, 200);
    assert_eq!(stats.output_lines, 50);
    assert_eq!(stats.input_tokens, 1000); // 4000 / 4
    assert_eq!(stats.output_tokens, 250); // 1000 / 4
    assert_eq!(stats.token_reduction_ratio, 0.25);
}

#[test]
fn test_reducer_stats_zero_input() {
    let stats = ReducerStats::new(0, 0, 0, 0);

    assert_eq!(stats.reduction_ratio, 0.0);
    assert_eq!(stats.token_reduction_ratio, 0.0);
}

// ============================================================
// BaseReducer Tests
// ============================================================

#[test]
fn test_base_reducer_creation() {
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        value: i32,
    }

    let reducer = BaseReducer::<TestData>::new("test");
    assert_eq!(reducer.name(), "test");
}

#[test]
fn test_base_reducer_valid_json_input() {
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        value: i32,
    }

    let reducer = BaseReducer::<TestData>::new("test");
    let context = ReducerContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    let input = r#"{"value": 42}"#.to_string();
    let result = reducer.reduce(&input, &context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestData { value: 42 });
}

#[test]
fn test_base_reducer_invalid_json_input() {
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        value: i32,
    }

    let reducer = BaseReducer::<TestData>::new("test");
    let context = ReducerContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    let input = "not valid json".to_string();
    let result = reducer.reduce(&input, &context);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_invalid_input());
}

// ============================================================
// ReducerRegistry Tests
// ============================================================

#[test]
fn test_reducer_registry_new() {
    let registry = ReducerRegistry::new();
    assert!(registry.reducer_names().is_empty());
}

#[test]
fn test_reducer_registry_default() {
    let registry = ReducerRegistry::default();
    assert!(registry.reducer_names().is_empty());
}

#[test]
fn test_reducer_registry_register() {
    struct TestReducer;

    impl Reducer for TestReducer {
        type Input = String;
        type Output = ReducerOutput;

        fn reduce(
            &self,
            input: &Self::Input,
            _context: &ReducerContext,
        ) -> ReducerResult<Self::Output> {
            Ok(ReducerOutput::new(input.clone()))
        }

        fn name(&self) -> &'static str {
            "test_reducer"
        }
    }

    let mut registry = ReducerRegistry::new();
    registry.register(TestReducer);

    assert_eq!(registry.reducer_names().len(), 1);
    assert!(registry.reducer_names().contains(&"test_reducer"));
}

#[test]
fn test_reducer_registry_execute() {
    struct TestReducer;

    impl Reducer for TestReducer {
        type Input = String;
        type Output = ReducerOutput;

        fn reduce(
            &self,
            input: &Self::Input,
            _context: &ReducerContext,
        ) -> ReducerResult<Self::Output> {
            Ok(ReducerOutput::new(input.clone()).with_summary("Processed"))
        }

        fn name(&self) -> &'static str {
            "test_reducer"
        }
    }

    let mut registry = ReducerRegistry::new();
    registry.register(TestReducer);

    let context = ReducerContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    let result = registry.execute("test_reducer", "test input", &context);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.summary, Some("Processed".to_string()));
}

#[test]
fn test_reducer_registry_execute_not_found() {
    let registry = ReducerRegistry::new();
    let context = ReducerContext {
        format: OutputFormat::Json,
        stats: false,
        enabled_formats: vec![],
    };

    let result = registry.execute("nonexistent", "input", &context);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_not_implemented());
}
