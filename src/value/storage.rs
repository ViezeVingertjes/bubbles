//! [`VariableStorage`] trait and the default [`HashMapStorage`] implementation.

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
    fn get(&self, name: &str) -> Option<Value>;
    /// Stores `value` under `name`, replacing any previous value.
    fn set(&mut self, name: &str, value: Value);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_unset_returns_none() {
        let s = HashMapStorage::new();
        assert!(s.get("$x").is_none());
    }

    #[test]
    fn set_then_get_round_trips() {
        let mut s = HashMapStorage::new();
        s.set("$x", Value::Number(42.0));
        assert_eq!(s.get("$x"), Some(Value::Number(42.0)));
    }

    #[test]
    fn overwrite_updates_value() {
        let mut s = HashMapStorage::new();
        s.set("$x", Value::Bool(true));
        s.set("$x", Value::Bool(false));
        assert_eq!(s.get("$x"), Some(Value::Bool(false)));
    }
}
