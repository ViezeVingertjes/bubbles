//! Expression evaluator — walks an [`Expr`] AST and produces a [`Value`].

use crate::compiler::expr::{BinOp, Expr, UnOp};
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
            .get(name)
            .ok_or_else(|| DialogueError::UndefinedVariable(name.clone())),
        Expr::Call { name, args } => {
            let evaluated: Result<Vec<Value>> =
                args.iter().map(|a| eval(a, storage, fns)).collect();
            fns(name, evaluated?)
        }
        Expr::Unary { op, expr } => {
            let v = eval(expr, storage, fns)?;
            match op {
                UnOp::Neg => {
                    if let Value::Number(n) = v {
                        Ok(Value::Number(-n))
                    } else {
                        Err(DialogueError::Type(format!("cannot negate {v:?}")))
                    }
                }
                UnOp::Not => Ok(Value::Bool(!v.is_truthy())),
            }
        }
        Expr::Binary { left, op, right } => eval_binary(left, *op, right, storage, fns),
    }
}

fn eval_binary<S, F>(left: &Expr, op: BinOp, right: &Expr, storage: &S, fns: &F) -> Result<Value>
where
    S: VariableStorage,
    F: Fn(&str, Vec<Value>) -> Result<Value>,
{
    // short-circuit for `&&` and `||`
    match op {
        BinOp::And => {
            let lv = eval(left, storage, fns)?;
            if !lv.is_truthy() {
                return Ok(Value::Bool(false));
            }
            return Ok(Value::Bool(eval(right, storage, fns)?.is_truthy()));
        }
        BinOp::Or => {
            let lv = eval(left, storage, fns)?;
            if lv.is_truthy() {
                return Ok(Value::Bool(true));
            }
            return Ok(Value::Bool(eval(right, storage, fns)?.is_truthy()));
        }
        _ => {}
    }

    let lv = eval(left, storage, fns)?;
    let rv = eval(right, storage, fns)?;

    match op {
        BinOp::Add => match (lv, rv) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Text(a + &b)),
            (Value::Text(a), b) => Ok(Value::Text(a + &b.to_string())),
            (a, b) => Err(DialogueError::Type(format!("cannot add {a:?} and {b:?}"))),
        },
        BinOp::Sub => num_op(lv, rv, "-", |x, y| x - y),
        BinOp::Mul => num_op(lv, rv, "*", |x, y| x * y),
        BinOp::Div => num_op(lv, rv, "/", |x, y| x / y),
        BinOp::Rem => num_op(lv, rv, "%", |x, y| x % y),
        BinOp::Eq => Ok(Value::Bool(lv == rv)),
        BinOp::Neq => Ok(Value::Bool(lv != rv)),
        BinOp::Lt => cmp_op(lv, rv, "<", |x: f64, y: f64| x < y),
        BinOp::Lte => cmp_op(lv, rv, "<=", |x: f64, y: f64| x <= y),
        BinOp::Gt => cmp_op(lv, rv, ">", |x: f64, y: f64| x > y),
        BinOp::Gte => cmp_op(lv, rv, ">=", |x: f64, y: f64| x >= y),
        BinOp::And | BinOp::Or => unreachable!("handled above"),
    }
}

fn num_op(left: Value, right: Value, op: &str, calc: impl Fn(f64, f64) -> f64) -> Result<Value> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if op == "/" && b == 0.0 {
                return Err(DialogueError::Runtime("division by zero".into()));
            }
            if op == "%" && b == 0.0 {
                return Err(DialogueError::Runtime("modulo by zero".into()));
            }
            Ok(Value::Number(calc(a, b)))
        }
        (lv, rv) => Err(DialogueError::Type(format!(
            "operator `{op}` requires numbers, got {lv:?} and {rv:?}"
        ))),
    }
}

fn cmp_op(left: Value, right: Value, op: &str, pred: impl Fn(f64, f64) -> bool) -> Result<Value> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(pred(a, b))),
        (lv, rv) => Err(DialogueError::Type(format!(
            "operator `{op}` requires numbers, got {lv:?} and {rv:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::expr::parse_expr;
    use crate::value::HashMapStorage;

    fn no_fns(_: &str, _: Vec<Value>) -> Result<Value> {
        Err(DialogueError::Runtime("no functions registered".into()))
    }

    fn ev(src: &str) -> Value {
        let storage = HashMapStorage::new();
        let expr = parse_expr(src).unwrap();
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
}
