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
#[path = "eval_tests.rs"]
mod tests;
