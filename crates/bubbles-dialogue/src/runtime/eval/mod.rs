//! Expression evaluator - walks an [`Expr`] AST and produces a [`Value`].

mod binary;
mod unary;

use crate::compiler::expr::Expr;
use crate::error::{DialogueError, Result};
use crate::value::{Value, VariableStorage};

/// Evaluate an [`Expr`] AST node using `storage` for variable reads and `fns` for function calls.
///
/// `fns` receives the function name and evaluated arguments and must return a [`Value`].
///
/// # Errors
/// Returns [`crate::error::DialogueError`] for undefined variables, type mismatches, or failed
/// function calls.
pub fn eval<S, F>(expr: &Expr, storage: &S, fns: &F) -> Result<Value>
where
    S: VariableStorage,
    F: Fn(&str, Vec<Value>) -> Result<Value>,
{
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Text(s) => Ok(Value::Text(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Var(name) => storage
            .get_ref(name)
            .map(std::borrow::Cow::into_owned)
            .ok_or_else(|| DialogueError::UndefinedVariable(name.clone())),
        Expr::Call { name, args } => {
            let evaluated: Result<Vec<Value>> =
                args.iter().map(|a| eval(a, storage, fns)).collect();
            fns(name, evaluated?)
        }
        Expr::Unary { op, expr } => unary::eval_unary(*op, expr, storage, fns),
        Expr::Binary { left, op, right } => binary::eval_binary(left, *op, right, storage, fns),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::expr::parse_expr_at;
    use crate::value::HashMapStorage;

    fn no_fns(_: &str, _: Vec<Value>) -> Result<Value> {
        Err(DialogueError::Function {
            name: "<none>".into(),
            message: "no functions registered".into(),
        })
    }

    fn ev(src: &str) -> Value {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(src, "<test>", 0).unwrap();
        eval(&expr, &storage, &no_fns).unwrap()
    }

    #[test]
    fn eval_addition() {
        assert_eq!(ev("1 + 2"), Value::Number(3.0));
    }

    #[test]
    fn eval_precedence() {
        assert_eq!(ev("1 + 2 * 3"), Value::Number(7.0));
    }

    #[test]
    fn eval_parentheses() {
        assert_eq!(ev("(1 + 2) * 3"), Value::Number(9.0));
    }

    #[test]
    fn eval_comparison() {
        assert_eq!(ev("3 > 2"), Value::Bool(true));
        assert_eq!(ev("1 >= 2"), Value::Bool(false));
    }

    #[test]
    fn eval_logical_and_short_circuit() {
        assert_eq!(ev("false && true"), Value::Bool(false));
        assert_eq!(ev("true && true"), Value::Bool(true));
    }

    #[test]
    fn eval_logical_or_short_circuit() {
        assert_eq!(ev("true || false"), Value::Bool(true));
        assert_eq!(ev("false || false"), Value::Bool(false));
    }

    #[test]
    fn eval_string_concat() {
        assert_eq!(
            ev(r#""hello" + " world""#),
            Value::Text("hello world".into())
        );
    }

    #[test]
    fn eval_unary_neg() {
        assert_eq!(ev("-3"), Value::Number(-3.0));
    }

    #[test]
    fn eval_unary_not() {
        assert_eq!(ev("!false"), Value::Bool(true));
    }

    #[test]
    fn negate_non_number_errors() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(r#"-"hello""#, "<test>", 0).unwrap();
        let err = eval(&expr, &storage, &no_fns).unwrap_err();
        assert!(
            matches!(err, DialogueError::TypeMismatch { ref context, .. } if context.contains('-')),
            "got {err}"
        );
    }

    #[test]
    fn subtract_non_numbers_errors() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(r#""a" - 1"#, "<test>", 0).unwrap();
        let err = eval(&expr, &storage, &no_fns).unwrap_err();
        assert!(
            matches!(err, DialogueError::TypeMismatch { ref context, .. } if context.contains('-')),
            "got {err}"
        );
    }

    #[test]
    fn compare_non_numbers_errors() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(r#""a" < 1"#, "<test>", 0).unwrap();
        let err = eval(&expr, &storage, &no_fns).unwrap_err();
        assert!(
            matches!(err, DialogueError::TypeMismatch { ref context, .. } if context.contains('<')),
            "got {err}"
        );
    }

    #[test]
    fn modulo_by_zero_errors() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at("5 % 0", "<test>", 0).unwrap();
        let err = eval(&expr, &storage, &no_fns).unwrap_err();
        assert!(err.to_string().contains("modulo"), "got {err}");
    }

    #[test]
    fn multiply_non_numbers_errors() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(r#""x" * 2"#, "<test>", 0).unwrap();
        assert!(eval(&expr, &storage, &no_fns).is_err());
    }

    #[test]
    fn comparison_operators_on_numbers() {
        assert_eq!(ev("1 != 2"), Value::Bool(true));
        assert_eq!(ev("2 != 2"), Value::Bool(false));
        assert_eq!(ev("1 < 2"), Value::Bool(true));
        assert_eq!(ev("2 <= 2"), Value::Bool(true));
        assert_eq!(ev("3 > 2"), Value::Bool(true));
        assert_eq!(ev("3 >= 4"), Value::Bool(false));
    }

    #[test]
    fn div_and_rem_on_numbers() {
        assert_eq!(ev("9 / 2"), Value::Number(4.5));
        assert_eq!(ev("7 % 3"), Value::Number(1.0));
    }

    #[test]
    fn add_number_and_bool_is_type_error() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at("1 + true", "<test>", 0).unwrap();
        assert!(eval(&expr, &storage, &no_fns).is_err());
    }

    #[test]
    fn bool_equality() {
        assert_eq!(ev("true == true"), Value::Bool(true));
        assert_eq!(ev("true == false"), Value::Bool(false));
    }

    #[test]
    fn text_equality_and_inequality() {
        assert_eq!(ev(r#""a" == "a""#), Value::Bool(true));
        assert_eq!(ev(r#""a" != "b""#), Value::Bool(true));
    }

    #[test]
    fn rem_requires_numbers() {
        let storage = HashMapStorage::new();
        let expr = parse_expr_at(r#""x" % 2"#, "<test>", 0).unwrap();
        assert!(eval(&expr, &storage, &no_fns).is_err());
    }
}
