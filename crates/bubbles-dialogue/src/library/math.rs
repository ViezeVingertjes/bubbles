//! Math built-in functions: round, floor, ceil, min, max, abs, clamp, string, int.

use crate::error::{DialogueError, Result};
use crate::value::Value;

use super::FunctionLibrary;

pub(super) fn require_one_number(name: &str, args: &[Value]) -> Result<f64> {
    match args {
        [Value::Number(n)] => Ok(*n),
        _ => Err(DialogueError::Function {
            name: name.to_owned(),
            message: format!("expected 1 number argument, got {args:?}"),
        }),
    }
}

pub(super) fn require_two_numbers(name: &str, args: &[Value]) -> Result<(f64, f64)> {
    match args {
        [Value::Number(a), Value::Number(b)] => Ok((*a, *b)),
        _ => Err(DialogueError::Function {
            name: name.to_owned(),
            message: format!("expected 2 number arguments, got {args:?}"),
        }),
    }
}

impl FunctionLibrary {
    pub(super) fn register_math_builtins(&mut self) {
        self.register("round", |args| {
            let n = require_one_number("round", &args)?;
            Ok(Value::Number(n.round()))
        });
        self.register("floor", |args| {
            let n = require_one_number("floor", &args)?;
            Ok(Value::Number(n.floor()))
        });
        self.register("ceil", |args| {
            let n = require_one_number("ceil", &args)?;
            Ok(Value::Number(n.ceil()))
        });
        self.register("min", |args| {
            let (a, b) = require_two_numbers("min", &args)?;
            Ok(Value::Number(a.min(b)))
        });
        self.register("max", |args| {
            let (a, b) = require_two_numbers("max", &args)?;
            Ok(Value::Number(a.max(b)))
        });
        self.register("abs", |args| {
            let n = require_one_number("abs", &args)?;
            Ok(Value::Number(n.abs()))
        });
        self.register("clamp", |args| match args.as_slice() {
            [Value::Number(v), Value::Number(lo), Value::Number(hi)] => {
                Ok(Value::Number(v.clamp(*lo, *hi)))
            }
            _ => Err(DialogueError::Function {
                name: "clamp".into(),
                message: format!("expected 3 number arguments, got {args:?}"),
            }),
        });
        self.register("string", |args| {
            args.into_iter().next().map_or_else(
                || {
                    Err(DialogueError::Function {
                        name: "string".into(),
                        message: "expected 1 argument".into(),
                    })
                },
                |v| Ok(Value::Text(v.to_string())),
            )
        });
        self.register("int", |args| {
            let n = require_one_number("int", &args)?;
            Ok(Value::Number(n.trunc()))
        });
    }
}
