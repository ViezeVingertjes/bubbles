//! Runner helpers for reading variable state.

use std::borrow::Cow;

use bubbles::{HashMapStorage, Runner, Value, VariableStorage, compile};

#[test]
fn runner_variable_matches_storage_get() {
    let prog = compile("title: A\n---\nHi.\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.storage_mut().set("$x", Value::Number(7.0));
    assert_eq!(runner.variable("$x"), Some(Value::Number(7.0)));
    assert_eq!(runner.storage().get("$x"), Some(Value::Number(7.0)));
}

#[test]
fn runner_variable_ref_borrows_from_hash_map_storage() {
    let prog = compile("title: A\n---\nHi.\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.storage_mut().set("$name", Value::Text("Pat".into()));
    let cow = runner.variable_ref("$name").expect("set");
    assert!(matches!(cow, Cow::Borrowed(_)));
    assert_eq!(&*cow, &Value::Text("Pat".into()));
}

#[test]
fn runner_all_variables_matches_storage() {
    let prog = compile("title: A\n---\nHi.\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.storage_mut().set("$a", Value::Bool(true));
    let mut from_runner = runner.all_variables();
    let mut from_storage = runner.storage().all_variables();
    from_runner.sort_by(|x, y| x.0.cmp(&y.0));
    from_storage.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(from_runner, from_storage);
}
