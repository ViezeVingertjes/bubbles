//! Unit tests for the `plural()` builtin.

use crate::library::FunctionLibrary;
use crate::value::Value;

#[test]
fn plural_singular_count() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "plural",
            vec![
                Value::Number(1.0),
                Value::Text("apple".into()),
                Value::Text("apples".into())
            ]
        )
        .unwrap(),
        Value::Text("apple".into())
    );
}

#[test]
fn plural_plural_count() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "plural",
            vec![
                Value::Number(2.0),
                Value::Text("apple".into()),
                Value::Text("apples".into())
            ]
        )
        .unwrap(),
        Value::Text("apples".into())
    );
}

#[test]
fn plural_zero_is_plural() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "plural",
            vec![
                Value::Number(0.0),
                Value::Text("item".into()),
                Value::Text("items".into())
            ]
        )
        .unwrap(),
        Value::Text("items".into())
    );
}

#[test]
fn plural_negative_one_is_singular() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "plural",
            vec![
                Value::Number(-1.0),
                Value::Text("step".into()),
                Value::Text("steps".into())
            ]
        )
        .unwrap(),
        Value::Text("step".into())
    );
}

#[test]
fn plural_bad_args_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call(
            "plural",
            vec![
                Value::Text("x".into()),
                Value::Text("a".into()),
                Value::Text("b".into())
            ]
        )
        .is_err()
    );
    assert!(lib.call("plural", vec![Value::Number(1.0)]).is_err());
}
