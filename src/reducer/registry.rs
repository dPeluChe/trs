//! Base reducer implementation and reducer registry.

use serde::Serialize;

use super::{Reducer, ReducerContext, ReducerError, ReducerOutput, ReducerResult};

// ============================================================
// Base Reducer Implementation
// ============================================================

/// Base reducer implementation with common functionality.
///
/// This struct provides default implementations for formatting methods
/// and can be extended by concrete reducer implementations.
pub struct BaseReducer<T: Serialize> {
    /// The name of the reducer.
    name: &'static str,
    /// Phantom data for the generic type parameter.
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + std::fmt::Debug> BaseReducer<T> {
    /// Create a new base reducer with the given name.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Format output as JSON.
    pub fn format_json(output: &T) -> String {
        serde_json::to_string_pretty(&output).unwrap_or_else(|e| {
            format!("{{\"error\": true, \"message\": \"Failed to serialize: {}\"}}", e)
        })
    }

    /// Format output in compact format.
    pub fn format_compact(output: &T) -> String {
        format!("{:?}", output)
    }

    /// Format output in raw format (minimal processing).
    pub fn format_raw(output: &T) -> String {
        format!("{:?}", output)
    }
}

impl<T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug> Reducer for BaseReducer<T> {
    type Input = String;
    type Output = T;

    fn reduce(&self, input: &Self::Input, _context: &ReducerContext) -> ReducerResult<Self::Output> {
        // Default implementation - parse as JSON and deserialize
        match serde_json::from_str(input) {
            Ok(data) => Ok(data),
            Err(e) => Err(ReducerError::InvalidInput(format!(
                "Failed to parse input as {}: {}",
                std::any::type_name::<T>(),
                e
            ))),
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

// ============================================================
// Reducer Registry
// ============================================================

/// Type alias for a reducer function.
type ReducerFn = Box<dyn Fn(&str, &ReducerContext) -> ReducerResult<ReducerOutput>>;

/// A registry for managing and executing reducers.
///
/// The registry provides a centralized place to register reducers by name
/// and execute them with input data.
#[derive(Default)]
pub struct ReducerRegistry {
    reducers: Vec<(&'static str, ReducerFn)>,
}

impl ReducerRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            reducers: Vec::new(),
        }
    }

    /// Register a reducer with the registry.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The reducer type (must implement `Reducer`)
    ///
    /// # Arguments
    ///
    /// * `reducer` - The reducer instance to register
    pub fn register<R>(&mut self, reducer: R)
    where
        R: Reducer<Input = String, Output = ReducerOutput> + 'static,
    {
        let name = reducer.name();
        let reducer_fn = Box::new(move |input: &str, context: &ReducerContext| {
            let input_string = input.to_string();
            reducer.reduce(&input_string, context)
        });

        self.reducers.push((name, reducer_fn));
    }

    /// Execute a reducer by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the reducer to execute
    /// * `input` - The input data to process
    /// * `context` - The context containing output format and stats flag
    ///
    /// # Returns
    ///
    /// Returns the reducer output on success, or a `ReducerError` on failure.
    pub fn execute(
        &self,
        name: &str,
        input: &str,
        context: &ReducerContext,
    ) -> ReducerResult<ReducerOutput> {
        for (reducer_name, reducer_fn) in &self.reducers {
            if reducer_name == &name {
                return reducer_fn(input, context);
            }
        }
        Err(ReducerError::NotImplemented(format!(
            "Reducer '{}' not found",
            name
        )))
    }

    /// Get a list of all registered reducer names.
    pub fn reducer_names(&self) -> Vec<&'static str> {
        self.reducers.iter().map(|(name, _)| *name).collect()
    }
}
