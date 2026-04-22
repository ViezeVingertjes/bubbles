//! Property tests for the expression evaluator — arithmetic and boolean laws.

use bubbles::compiler::expr::parse_expr;
use bubbles::runtime::eval as eval_fn;
use bubbles::{DialogueError, HashMapStorage, Value, VariableStorage};
use proptest::prelude::*;

// ── helpers ───────────────────────────────────────────────────────────────────

fn no_fns(_: &str, _: Vec<Value>) -> Result<Value, DialogueError> {
    Err(DialogueError::Runtime("no functions in proptest".into()))
}

fn eval(src: &str) -> Value {
    let expr = parse_expr(src).unwrap();
    let storage = HashMapStorage::new();
    eval_fn(&expr, &storage, &no_fns).unwrap()
}

fn eval_with(src: &str, vars: &[(&str, Value)]) -> Value {
    let expr = parse_expr(src).unwrap();
    let mut storage = HashMapStorage::new();
    for (k, v) in vars {
        storage.set(k, v.clone());
    }
    eval_fn(&expr, &storage, &no_fns).unwrap()
}

// ── arithmetic laws ───────────────────────────────────────────────────────────

proptest! {
    /// Commutativity of addition: a + b == b + a
    #[test]
    fn addition_commutes(a in -1000.0f64..1000.0, b in -1000.0f64..1000.0) {
        let l = eval(&format!("{a} + {b}"));
        let r = eval(&format!("{b} + {a}"));
        prop_assert_eq!(l, r);
    }

    /// Commutativity of multiplication: a * b == b * a
    #[test]
    fn multiplication_commutes(a in -1000.0f64..1000.0, b in -1000.0f64..1000.0) {
        let l = eval(&format!("{a} * {b}"));
        let r = eval(&format!("{b} * {a}"));
        prop_assert_eq!(l, r);
    }

    /// Additive identity: a + 0 == a
    #[test]
    fn additive_identity(a in -1000.0f64..1000.0) {
        prop_assert_eq!(eval(&format!("{a} + 0")), Value::Number(a));
    }

    /// Multiplicative identity: a * 1 == a
    #[test]
    fn multiplicative_identity(a in -1000.0f64..1000.0) {
        prop_assert_eq!(eval(&format!("{a} * 1")), Value::Number(a));
    }

    /// Subtraction identity: a - 0 == a
    #[test]
    fn subtraction_identity(a in -1000.0f64..1000.0) {
        prop_assert_eq!(eval(&format!("{a} - 0")), Value::Number(a));
    }

    /// Double negation: -(-a) == a
    #[test]
    fn double_negation(a in -1000.0f64..1000.0) {
        prop_assert_eq!(eval(&format!("-(-{a})")), Value::Number(a));
    }

    /// Distributivity: a * (b + c) == a * b + a * c
    #[test]
    fn distributivity(
        a in -100.0f64..100.0,
        b in -100.0f64..100.0,
        c in -100.0f64..100.0,
    ) {
        let l = eval(&format!("{a} * ({b} + {c})"));
        let r = eval(&format!("{a} * {b} + {a} * {c}"));
        // Use tolerance due to floating-point rounding.
        if let (Value::Number(lv), Value::Number(rv)) = (l, r) {
            prop_assert!((lv - rv).abs() < 1e-9, "distributivity: {lv} != {rv}");
        }
    }
}

// ── boolean laws ──────────────────────────────────────────────────────────────

proptest! {
    /// De Morgan's first law: !(a && b) == !a || !b
    #[test]
    fn de_morgan_and(a in any::<bool>(), b in any::<bool>()) {
        let l = eval(&format!("!({a} && {b})"));
        let r = eval(&format!("!{a} || !{b}"));
        prop_assert_eq!(l, r);
    }

    /// De Morgan's second law: !(a || b) == !a && !b
    #[test]
    fn de_morgan_or(a in any::<bool>(), b in any::<bool>()) {
        let l = eval(&format!("!({a} || {b})"));
        let r = eval(&format!("!{a} && !{b}"));
        prop_assert_eq!(l, r);
    }

    /// Double logical negation: !!a == a
    #[test]
    fn double_logical_negation(a in any::<bool>()) {
        prop_assert_eq!(eval(&format!("!!{a}")), Value::Bool(a));
    }

    /// AND with false absorbs: false && a == false
    #[test]
    fn and_false_absorbs(a in any::<bool>()) {
        prop_assert_eq!(eval(&format!("false && {a}")), Value::Bool(false));
    }

    /// OR with true absorbs: true || a == true
    #[test]
    fn or_true_absorbs(a in any::<bool>()) {
        prop_assert_eq!(eval(&format!("true || {a}")), Value::Bool(true));
    }
}

// ── comparison laws ───────────────────────────────────────────────────────────

proptest! {
    /// Reflexivity: a == a
    #[test]
    fn equality_reflexive(a in -1000.0f64..1000.0) {
        prop_assert_eq!(eval(&format!("{a} == {a}")), Value::Bool(true));
    }

    /// Symmetry of equality: (a == b) == (b == a)
    #[test]
    fn equality_symmetric(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let l = eval(&format!("{a} == {b}"));
        let r = eval(&format!("{b} == {a}"));
        prop_assert_eq!(l, r);
    }

    /// Trichotomy: exactly one of a < b, a == b, a > b
    #[test]
    fn trichotomy(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let lt = eval(&format!("{a} < {b}")) == Value::Bool(true);
        let eq = eval(&format!("{a} == {b}")) == Value::Bool(true);
        let gt = eval(&format!("{a} > {b}")) == Value::Bool(true);
        let exactly_one = (lt as u8) + (eq as u8) + (gt as u8) == 1;
        prop_assert!(exactly_one);
    }

    /// Negation flips comparison: (a < b) == !(a >= b)
    #[test]
    fn negation_flips_lt(a in -100.0f64..100.0, b in -100.0f64..100.0) {
        let lt = eval(&format!("{a} < {b}"));
        let ge_negated = eval(&format!("!({a} >= {b})"));
        prop_assert_eq!(lt, ge_negated);
    }
}

// ── variable reads ────────────────────────────────────────────────────────────

proptest! {
    /// Reading a Number variable round-trips correctly.
    #[test]
    fn number_variable_round_trips(v in -1000.0f64..1000.0) {
        let result = eval_with("$x", &[("$x", Value::Number(v))]);
        prop_assert_eq!(result, Value::Number(v));
    }

    /// Reading a Bool variable round-trips correctly.
    #[test]
    fn bool_variable_round_trips(v in any::<bool>()) {
        let result = eval_with("$flag", &[("$flag", Value::Bool(v))]);
        prop_assert_eq!(result, Value::Bool(v));
    }
}
