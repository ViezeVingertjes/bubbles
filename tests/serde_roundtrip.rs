//! Snapshot/restore tests gated on the `serde` feature.

#[cfg(feature = "serde")]
mod serde_tests {
    use bubbles::{HashMapStorage, Value, VariableStorage};

    #[test]
    fn value_number_round_trips() {
        let v = Value::Number(3.14);
        let json = serde_json::to_string(&v).unwrap();
        let v2: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn value_text_round_trips() {
        let v = Value::Text("hello".into());
        let json = serde_json::to_string(&v).unwrap();
        let v2: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn value_bool_round_trips() {
        let v = Value::Bool(true);
        let json = serde_json::to_string(&v).unwrap();
        let v2: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn hashmap_storage_round_trips() {
        let mut s = HashMapStorage::new();
        s.set("$gold", Value::Number(42.0));
        s.set("$name", Value::Text("Bob".into()));
        s.set("$alive", Value::Bool(true));

        let json = serde_json::to_string(&s).unwrap();
        let s2: HashMapStorage = serde_json::from_str(&json).unwrap();

        assert_eq!(s2.get("$gold"), Some(Value::Number(42.0)));
        assert_eq!(s2.get("$name"), Some(Value::Text("Bob".into())));
        assert_eq!(s2.get("$alive"), Some(Value::Bool(true)));
    }
}
