//! Unit tests for the `select()` builtin.

use crate::library::FunctionLibrary;
use crate::value::Value;

#[test]
fn select_known_key() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "select",
            vec![
                Value::Text("m".into()),
                Value::Text("m:He|f:She|other:They".into())
            ]
        )
        .unwrap(),
        Value::Text("He".into())
    );
}

#[test]
fn select_second_key() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "select",
            vec![
                Value::Text("f".into()),
                Value::Text("m:He|f:She|other:They".into())
            ]
        )
        .unwrap(),
        Value::Text("She".into())
    );
}

#[test]
fn select_falls_back_to_other() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "select",
            vec![
                Value::Text("nb".into()),
                Value::Text("m:He|f:She|other:They".into())
            ]
        )
        .unwrap(),
        Value::Text("They".into())
    );
}

#[test]
fn select_missing_other_key_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call(
            "select",
            vec![Value::Text("x".into()), Value::Text("m:He|f:She".into())]
        )
        .is_err()
    );
}

#[test]
fn select_malformed_entry_errors() {
    let lib = FunctionLibrary::new();
    // "badentry" has no colon, so parsing fails
    assert!(
        lib.call(
            "select",
            vec![
                Value::Text("m".into()),
                Value::Text("badentry|other:Fallback".into())
            ]
        )
        .is_err()
    );
}

#[test]
fn select_bad_args_errors() {
    let lib = FunctionLibrary::new();
    // first arg must be a string
    assert!(
        lib.call(
            "select",
            vec![Value::Number(1.0), Value::Text("a:A|other:B".into())]
        )
        .is_err()
    );
    // second arg must be a string
    assert!(
        lib.call("select", vec![Value::Text("a".into()), Value::Number(1.0)])
            .is_err()
    );
}

#[test]
fn select_value_may_contain_colon() {
    // Only the first colon on each entry is the separator.
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "select",
            vec![
                Value::Text("time".into()),
                Value::Text("time:12:00|other:??".into())
            ]
        )
        .unwrap(),
        Value::Text("12:00".into())
    );
}
