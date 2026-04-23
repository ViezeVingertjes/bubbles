//! Binary operator evaluation: `eval_binary`, `num_op`, `cmp_op`, `value_type_name`.

use crate::compiler::expr::{BinOp, Expr};
use crate::error::{DialogueError, Result};
use crate::value::{Value, VariableStorage};

use super::eval;

pub(super) fn eval_binary<S, F>(
    left: &Expr,
    op: BinOp,
    right: &Expr,
    storage: &S,
    fns: &F,
) -> Result<Value>
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
            (a, b) => Err(DialogueError::TypeMismatch {
                expected: "number or string".into(),
                got: format!("{} and {}", value_type_name(&a), value_type_name(&b)),
                context: "operator `+`".into(),
            }),
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

pub(super) const fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Number(_) => "number",
        Value::Text(_) => "string",
        Value::Bool(_) => "bool",
    }
}

fn num_op(left: Value, right: Value, op: &str, calc: impl Fn(f64, f64) -> f64) -> Result<Value> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if op == "/" && b == 0.0 {
                return Err(DialogueError::ProtocolViolation("division by zero".into()));
            }
            if op == "%" && b == 0.0 {
                return Err(DialogueError::ProtocolViolation("modulo by zero".into()));
            }
            Ok(Value::Number(calc(a, b)))
        }
        (lv, rv) => Err(DialogueError::TypeMismatch {
            expected: "number".into(),
            got: format!("{} and {}", value_type_name(&lv), value_type_name(&rv)),
            context: format!("operator `{op}`"),
        }),
    }
}

fn cmp_op(left: Value, right: Value, op: &str, pred: impl Fn(f64, f64) -> bool) -> Result<Value> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(pred(a, b))),
        (lv, rv) => Err(DialogueError::TypeMismatch {
            expected: "number".into(),
            got: format!("{} and {}", value_type_name(&lv), value_type_name(&rv)),
            context: format!("operator `{op}`"),
        }),
    }
}
