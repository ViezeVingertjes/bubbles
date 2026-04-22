//! [`VariableStorage`] trait and the default [`HashMapStorage`] implementation.

use std::collections::HashMap;

use super::Value;

/// Pluggable variable storage consumed by the runner.
pub trait VariableStorage {
    /// Returns the value of a variable, or `None` if unset.
    fn get(&self, name: &str) -> Option<Value>;
    /// Sets a variable.
    fn set(&mut self, name: &str, value: Value);
}

/// Default in-memory variable store backed by a [`HashMap`].
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
