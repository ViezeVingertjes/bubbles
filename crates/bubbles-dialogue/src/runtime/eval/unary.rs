//! Unary operator evaluation.

use crate::compiler::expr::{Expr, UnOp};
use crate::error::{DialogueError, Result};
use crate::value::{Value, VariableStorage};

use super::binary::value_type_name;
use super::eval;

pub(super) fn eval_unary<S, F>(op: UnOp, expr: &Expr, storage: &S, fns: &F) -> Result<Value>
where
    S: VariableStorage,
    F: Fn(&str, Vec<Value>) -> Result<Value>,
{
    let v = eval(expr, storage, fns)?;
    match op {
        UnOp::Neg => {
            if let Value::Number(n) = v {
                Ok(Value::Number(-n))
            } else {
                Err(DialogueError::TypeMismatch {
                    expected: "number".into(),
                    got: value_type_name(&v).into(),
                    context: "unary `-`".into(),
                })
            }
        }
        UnOp::Not => Ok(Value::Bool(!v.is_truthy())),
    }
}
