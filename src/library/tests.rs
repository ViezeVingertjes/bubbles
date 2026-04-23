//! Unit tests for [`super::FunctionLibrary`].

use crate::error::DialogueError;

use super::*;

#[test]
fn default_library_matches_new() {
    assert_eq!(
        FunctionLibrary::new()
            .call("round", vec![Value::Number(2.3)])
            .unwrap(),
        FunctionLibrary::default()
            .call("round", vec![Value::Number(2.3)])
            .unwrap(),
    );
}

#[test]
fn round_builtin() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call("round", vec![Value::Number(3.7)]).unwrap(),
        Value::Number(4.0)
    );
}

#[test]
fn min_max_builtins() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call("min", vec![Value::Number(2.0), Value::Number(5.0)])
            .unwrap(),
        Value::Number(2.0)
    );
    assert_eq!(
        lib.call("max", vec![Value::Number(2.0), Value::Number(5.0)])
            .unwrap(),
        Value::Number(5.0)
    );
}

#[test]
fn unknown_function_errors() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("does_not_exist", vec![]).is_err());
}

#[test]
fn custom_function_registered() {
    let mut lib = FunctionLibrary::new();
    lib.register("double", |args| {
        if let [Value::Number(n)] = args.as_slice() {
            Ok(Value::Number(n * 2.0))
        } else {
            Err(DialogueError::Runtime("double expects one number".into()))
        }
    });
    assert_eq!(
        lib.call("double", vec![Value::Number(5.0)]).unwrap(),
        Value::Number(10.0)
    );
}

#[cfg(feature = "rand")]
#[test]
fn random_range_within_bounds() {
    let lib = FunctionLibrary::new();
    for _ in 0..20 {
        let v = lib
            .call("random_range", vec![Value::Number(1.0), Value::Number(6.0)])
            .unwrap();
        if let Value::Number(n) = v {
            assert!((1.0..=6.0).contains(&n));
        }
    }
}

#[test]
fn floor_ceil_abs() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call("floor", vec![Value::Number(2.9)]).unwrap(),
        Value::Number(2.0)
    );
    assert_eq!(
        lib.call("ceil", vec![Value::Number(2.1)]).unwrap(),
        Value::Number(3.0)
    );
    assert_eq!(
        lib.call("abs", vec![Value::Number(-7.5)]).unwrap(),
        Value::Number(7.5)
    );
}

#[test]
fn clamp_three_numbers() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call(
            "clamp",
            vec![Value::Number(10.0), Value::Number(0.0), Value::Number(5.0)]
        )
        .unwrap(),
        Value::Number(5.0)
    );
}

#[test]
fn clamp_wrong_arity_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call("clamp", vec![Value::Number(1.0), Value::Number(2.0)])
            .is_err()
    );
}

#[test]
fn string_converts_value() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call("string", vec![Value::Number(42.0)]).unwrap(),
        Value::Text("42".into())
    );
}

#[test]
fn string_no_args_errors() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("string", vec![]).is_err());
}

#[test]
fn int_truncates() {
    let lib = FunctionLibrary::new();
    assert_eq!(
        lib.call("int", vec![Value::Number(-3.7)]).unwrap(),
        Value::Number(-3.0)
    );
}

#[test]
fn round_wrong_args_errors() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("round", vec![]).is_err());
    assert!(
        lib.call("round", vec![Value::Number(1.0), Value::Number(2.0)])
            .is_err()
    );
}

#[test]
fn min_wrong_args_errors() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("min", vec![Value::Number(1.0)]).is_err());
}

#[test]
fn max_wrong_args_errors() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("max", vec![Value::Text("a".into())]).is_err());
}

#[test]
fn floor_ceil_abs_int_require_numbers() {
    let lib = FunctionLibrary::new();
    assert!(lib.call("floor", vec![Value::Text("nope".into())]).is_err());
    assert!(lib.call("ceil", vec![]).is_err());
    assert!(lib.call("abs", vec![Value::Bool(true)]).is_err());
    assert!(lib.call("int", vec![Value::Text("1".into())]).is_err());
}

#[cfg(feature = "rand")]
#[test]
fn random_range_lo_gt_hi_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call("random_range", vec![Value::Number(6.0), Value::Number(1.0)])
            .is_err()
    );
}

#[cfg(feature = "rand")]
#[test]
fn random_range_fractional_arg_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call(
            "random_range",
            vec![Value::Number(1.5), Value::Number(6.0)]
        )
        .is_err()
    );
}

#[cfg(feature = "rand")]
#[test]
fn dice_negative_sides_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call("dice", vec![Value::Number(-1.0), Value::Number(3.0)])
            .is_err()
    );
}

#[cfg(feature = "rand")]
#[test]
fn dice_fractional_count_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call("dice", vec![Value::Number(6.0), Value::Number(2.5)])
            .is_err()
    );
}

#[cfg(feature = "rand")]
#[test]
fn dice_zero_sides_errors() {
    let lib = FunctionLibrary::new();
    assert!(
        lib.call("dice", vec![Value::Number(0.0), Value::Number(2.0)])
            .is_err()
    );
}

#[cfg(feature = "rand")]
#[test]
fn dice_rolls_sum() {
    let lib = FunctionLibrary::new();
    let v = lib
        .call("dice", vec![Value::Number(6.0), Value::Number(3.0)])
        .unwrap();
    if let Value::Number(n) = v {
        assert!((3.0..=18.0).contains(&n));
    } else {
        panic!("expected number");
    }
}

// ── plural() ──────────────────────────────────────────────────────────────────

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
    assert!(lib
        .call(
            "plural",
            vec![Value::Text("x".into()), Value::Text("a".into()), Value::Text("b".into())]
        )
        .is_err());
    assert!(lib.call("plural", vec![Value::Number(1.0)]).is_err());
}

#[cfg(feature = "rand")]
#[test]
fn random_builtin_returns_unit_intervalish() {
    let lib = FunctionLibrary::new();
    let Value::Number(n) = lib.call("random", vec![]).unwrap() else {
        panic!("expected number");
    };
    assert!((0.0..=1.0).contains(&n));
}
