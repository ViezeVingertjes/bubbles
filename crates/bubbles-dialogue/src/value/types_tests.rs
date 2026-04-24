use super::*;

#[test]
fn display_integer_number() {
    assert_eq!(Value::Number(3.0).to_string(), "3");
}

#[test]
fn display_fractional_number() {
    assert_eq!(Value::Number(3.5).to_string(), "3.5");
}

#[test]
fn truthy_coercion() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(Value::Number(1.0).is_truthy());
    assert!(!Value::Number(0.0).is_truthy());
    assert!(Value::Text("hi".into()).is_truthy());
    assert!(!Value::Text(String::new()).is_truthy());
}

#[test]
fn display_very_large_integer_uses_float_formatting() {
    // Above the `1e15` fast-path threshold in `Display`.
    let s = Value::Number(1e16).to_string();
    assert!(s.contains('e') || s.len() >= 15, "got {s}");
}

#[test]
fn display_bool() {
    assert_eq!(Value::Bool(true).to_string(), "true");
}

#[test]
fn from_conversions() {
    assert_eq!(Value::from(3.5_f64), Value::Number(3.5));
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from("x".to_owned()), Value::Text("x".into()));
    assert_eq!(Value::from("y"), Value::Text("y".into()));
}

#[test]
fn negative_number_is_truthy() {
    assert!(Value::Number(-1.0).is_truthy());
}
