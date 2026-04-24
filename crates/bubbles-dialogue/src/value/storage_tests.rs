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
    use crate::compiler::expr::parse_expr_at;
    use crate::runtime::eval;

    let mut storage = CountingStorage::default();
    storage.set("$name", Value::Text("Hero".into()));
    storage.set("$hp", Value::Number(100.0));

    let expr = parse_expr_at("$name + \" has \" + string($hp)", "<test>", 0).unwrap();
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
