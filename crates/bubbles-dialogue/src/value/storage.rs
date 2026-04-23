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
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

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

    #[test]
    fn hash_map_storage_get_ref_returns_borrowed() {
        let mut s = HashMapStorage::new();
        s.set("$x", Value::Text("hello".into()));
        let cow = s.get_ref("$x").expect("var was set");
        assert!(matches!(cow, Cow::Borrowed(_)));
        assert_eq!(&*cow, &Value::Text("hello".into()));
    }

    /// Tracks reads through each code path so eval can be observed to prefer
    /// `get_ref` over `get` during expression evaluation.
    #[derive(Default)]
    struct CountingStorage {
        inner: HashMap<String, Value>,
        get_calls: Cell<usize>,
        get_ref_calls: Cell<usize>,
    }

    impl VariableStorage for CountingStorage {
        fn get(&self, name: &str) -> Option<Value> {
            self.get_calls.set(self.get_calls.get() + 1);
            self.inner.get(name).cloned()
        }

        fn set(&mut self, name: &str, value: Value) {
            self.inner.insert(name.to_owned(), value);
        }

        fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
            self.get_ref_calls.set(self.get_ref_calls.get() + 1);
            self.inner.get(name).map(Cow::Borrowed)
        }
    }

    #[test]
    fn eval_prefers_get_ref_over_get() {
        use crate::compiler::expr::parse_expr;
        use crate::runtime::eval;

        let mut storage = CountingStorage::default();
        storage.set("$name", Value::Text("Hero".into()));
        storage.set("$hp", Value::Number(100.0));

        let expr = parse_expr("$name + \" has \" + string($hp)").unwrap();
        let _ = eval(&expr, &storage, &|name, args| {
            crate::library::FunctionLibrary::new().call(name, args)
        });

        assert_eq!(
            storage.get_calls.get(),
            0,
            "eval should read via get_ref, not get"
        );
        assert!(
            storage.get_ref_calls.get() >= 2,
            "expected get_ref called at least once per variable read, got {}",
            storage.get_ref_calls.get()
        );
    }
}
