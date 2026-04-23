//! [`FunctionLibrary`] - built-in functions and host-registration API.

mod i18n;
mod math;
#[cfg(feature = "rand")]
mod rand;

use std::collections::HashMap;

use crate::error::{DialogueError, Result};
use crate::value::Value;

/// A boxed host function callable from dialogue expressions.
pub type HostFn = Box<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static>;

/// Registry of named functions available to expression evaluation.
///
/// Built-in functions (`round`, `dice`, `random_range`, …) are pre-registered.
/// Hosts can add their own via [`FunctionLibrary::register`].
pub struct FunctionLibrary {
    fns: HashMap<String, HostFn>,
}

impl FunctionLibrary {
    /// Creates a library with the built-in functions pre-registered.
    #[must_use]
    pub fn new() -> Self {
        let mut lib = Self {
            fns: HashMap::new(),
        };
        lib.register_builtins();
        lib
    }

    /// Registers a named function.
    ///
    /// Replaces any existing function with the same name.
    pub fn register<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static,
    {
        self.fns.insert(name.into(), Box::new(f));
    }

    /// Calls the function named `name` with `args`, or returns an error if not found.
    ///
    /// # Errors
    /// Returns [`DialogueError::Function`] if the function is unknown or the call fails.
    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        self.fns.get(name).map_or_else(
            || {
                Err(DialogueError::Function {
                    name: name.to_owned(),
                    message: "unknown function".into(),
                })
            },
            |f| f(args),
        )
    }

    fn register_builtins(&mut self) {
        #[cfg(feature = "rand")]
        self.register_rand_builtins();
        self.register_math_builtins();
        self.register_i18n_builtins();
    }
}

impl Default for FunctionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FunctionLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.fns.keys().map(String::as_str).collect();
        f.debug_struct("FunctionLibrary")
            .field("functions", &names)
            .finish()
    }
}

#[cfg(test)]
mod tests;
