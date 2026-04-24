//! [`VariableStorage`] trait and the default [`HashMapStorage`] implementation.

use std::borrow::Cow;
use std::collections::HashMap;

use super::Value;

/// Pluggable variable storage consumed by the runner.
///
/// Implement this trait to back variables with your game's own data model
/// (e.g. an ECS component, a database row, or a save-file entry).
///
/// # Example
///
/// ```rust
/// use bubbles::{HashMapStorage, Value, VariableStorage};
///
/// let mut s = HashMapStorage::new();
/// s.set("$score", Value::Number(10.0));
/// assert_eq!(s.get("$score"), Some(Value::Number(10.0)));
/// assert_eq!(s.get("$missing"), None);
/// ```
pub trait VariableStorage {
    /// Returns the current value of `name`, or `None` if the variable has not been set.
    ///
    /// This is the ergonomic read path: it always returns an owned [`Value`],
    /// cloning if the backing store holds one by reference.  New impls are
    /// encouraged to override [`get_ref`](Self::get_ref) as well so hot
    /// expression-evaluation paths can avoid cloning [`Value::Text`].
    fn get(&self, name: &str) -> Option<Value>;
    /// Stores `value` under `name`, replacing any previous value.
    fn set(&mut self, name: &str, value: Value);

    /// Returns a reference to the current value of `name`, or `None` if the
    /// variable has not been set.
    ///
    /// The runner prefers this over [`get`](Self::get) during expression
    /// evaluation so string variables can be observed without an allocation.
    /// The default implementation simply forwards to [`get`](Self::get) and
    /// wraps the result in [`Cow::Owned`], so existing implementations keep
    /// working unchanged.  Stores that already own their values (such as
    /// [`HashMapStorage`]) should override this to return [`Cow::Borrowed`].
    fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.get(name).map(Cow::Owned)
    }

    /// Returns every `(variable_name, value)` pair this storage currently holds.
    ///
    /// Intended for debug overlays, save editors, and tests. The default
    /// implementation returns an empty vector; override it when you can
    /// enumerate variables (as [`HashMapStorage`] does).
    fn all_variables(&self) -> Vec<(String, Value)> {
        Vec::new()
    }
}

/// Default in-memory variable store backed by a [`HashMap`].
///
/// # Example
///
/// ```rust
/// use bubbles::{HashMapStorage, Value, VariableStorage};
///
/// let mut storage = HashMapStorage::new();
/// storage.set("$hp", Value::Number(100.0));
/// storage.set("$name", Value::Text("Hero".into()));
///
/// assert_eq!(storage.get("$hp"), Some(Value::Number(100.0)));
/// assert_eq!(storage.get("$name"), Some(Value::Text("Hero".into())));
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HashMapStorage {
    map: HashMap<String, Value>,
}

impl HashMapStorage {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl VariableStorage for HashMapStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.map.get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Value) {
        self.map.insert(name.to_owned(), value);
    }

    fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.map.get(name).map(Cow::Borrowed)
    }

    fn all_variables(&self) -> Vec<(String, Value)> {
        self.map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
