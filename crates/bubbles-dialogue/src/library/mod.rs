//! [`FunctionLibrary`] - built-in functions and host-registration API.

mod math;
#[cfg(feature = "rand")]
mod rand;
mod text;

use std::collections::HashMap;

use crate::error::{DialogueError, Result};
use crate::runtime::DialogueRng;
use crate::value::Value;

/// A boxed host function callable from dialogue expressions.
#[deprecated(
    since = "3.0.0",
    note = "FunctionLibrary::register is generic over the closure, so the alias is never required"
)]
pub type HostFn = BoxedHostFn;

pub(crate) type BoxedHostFn = Box<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static>;

/// Registry of named functions available to expression evaluation.
///
/// Built-in functions (`round`, `dice`, `random_range`, …) are pre-registered.
/// Hosts can add their own via [`FunctionLibrary::register`].
///
/// Two of the built-ins shape text rather than internationalise it, and both
/// have limits worth knowing before you lean on them. `plural(n, sing, plur)`
/// applies one two-form rule - singular when `|n|` is 1, plural otherwise -
/// which is wrong for languages with other plural categories; supply a
/// CLDR-correct implementation by registering it under the same name.
/// `select(key, "k1:a|k2:b|other:c")` has no escape mechanism for its `|` and
/// `:` separators, so a key containing either, or a value containing `|`,
/// cannot be expressed at all.
pub struct FunctionLibrary {
    fns: HashMap<String, BoxedHostFn>,
}

impl FunctionLibrary {
    /// Creates a library with the built-in functions pre-registered.
    ///
    /// The random built-ins draw from their own generator, seeded from the
    /// operating system. A [`Runner`](crate::Runner) supplies its own
    /// generator instead, so its rolls follow the seed given to
    /// [`RunnerBuilder::with_seed`](crate::RunnerBuilder::with_seed).
    #[must_use]
    pub fn new() -> Self {
        Self::with_rng(&DialogueRng::unseeded())
    }

    /// Creates a library whose random built-ins draw from `rng`.
    pub(crate) fn with_rng(rng: &DialogueRng) -> Self {
        let mut lib = Self {
            fns: HashMap::new(),
        };
        #[cfg(feature = "rand")]
        lib.register_rand_builtins(rng);
        #[cfg(not(feature = "rand"))]
        let _ = rng;
        lib.register_math_builtins();
        lib.register_text_builtins();
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
#[path = "library_tests/mod.rs"]
mod tests;
