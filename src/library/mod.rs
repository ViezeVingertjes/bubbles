//! [`FunctionLibrary`] — built-in functions and host-registration API.

use std::collections::HashMap;

use crate::error::{DialogueError, Result};
use crate::value::Value;

/// A boxed host function callable from dialogue expressions.
pub type HostFn = Box<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static>;

/// Registry of named functions available to expression evaluation.
///
/// Built-in functions (`random`, `dice`, `random_range`, …) are pre-registered.
/// Hosts can add their own via [`FunctionLibrary::register`].
pub struct FunctionLibrary {
    fns: HashMap<String, HostFn>,
}

impl FunctionLibrary {
    /// Creates a library with the built-in functions pre-registered.
    #[must_use]
    pub fn new() -> Self {
        let mut lib = Self {
            fns: HashMap::new(),
        };
        lib.register_builtins();
        lib
    }

    /// Registers a named function.
    ///
    /// Replaces any existing function with the same name.
    pub fn register<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static,
    {
        self.fns.insert(name.into(), Box::new(f));
    }

    /// Calls the function named `name` with `args`, or returns an error if not found.
    ///
    /// # Errors
    /// Returns [`DialogueError::Function`] if the function is unknown or the call fails.
    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        self.fns.get(name).map_or_else(
            || {
                Err(DialogueError::Function {
                    name: name.to_owned(),
                    message: "unknown function".into(),
                })
            },
            |f| f(args),
        )
    }

    fn register_builtins(&mut self) {
        #[cfg(feature = "rand")]
        self.register_rand_builtins();
        self.register_math_builtins();
    }

    fn register_math_builtins(&mut self) {
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

    #[cfg(feature = "rand")]
    fn register_rand_builtins(&mut self) {
        use rand::Rng as _;
        self.register("random", |_args| {
            Ok(Value::Number(rand::rng().random::<f64>()))
        });
        self.register("random_range", |args| {
            let (lo, hi) = require_two_numbers("random_range", &args)?;
            let lo = number_to_i32("random_range", lo, "lo")?;
            let hi = number_to_i32("random_range", hi, "hi")?;
            if lo > hi {
                return Err(DialogueError::Function {
                    name: "random_range".into(),
                    message: format!("lo ({lo}) > hi ({hi})"),
                });
            }
            Ok(Value::Number(f64::from(rand::rng().random_range(lo..=hi))))
        });
        self.register("dice", |args| {
            let (sides, count) = require_two_numbers("dice", &args)?;
            let sides = number_to_u32("dice", sides, "sides")?;
            let count = number_to_u32("dice", count, "count")?;
            if sides == 0 {
                return Err(DialogueError::Function {
                    name: "dice".into(),
                    message: "sides must be > 0".into(),
                });
            }
            let total: u32 = (0..count)
                .map(|_| rand::rng().random_range(1..=sides))
                .sum();
            Ok(Value::Number(f64::from(total)))
        });
    }
}

impl Default for FunctionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ── argument helpers ──────────────────────────────────────────────────────────

fn require_one_number(name: &str, args: &[Value]) -> Result<f64> {
    match args {
        [Value::Number(n)] => Ok(*n),
        _ => Err(DialogueError::Function {
            name: name.to_owned(),
            message: format!("expected 1 number argument, got {args:?}"),
        }),
    }
}

fn require_two_numbers(name: &str, args: &[Value]) -> Result<(f64, f64)> {
    match args {
        [Value::Number(a), Value::Number(b)] => Ok((*a, *b)),
        _ => Err(DialogueError::Function {
            name: name.to_owned(),
            message: format!("expected 2 number arguments, got {args:?}"),
        }),
    }
}

/// Converts a `f64` dialogue number to `i32`, returning an error if the value
/// is non-finite, has a fractional part, or falls outside `i32` range.
fn number_to_i32(fn_name: &str, v: f64, param: &str) -> Result<i32> {
    if !v.is_finite() || v.fract() != 0.0 {
        return Err(DialogueError::Function {
            name: fn_name.to_owned(),
            message: format!("{param} must be a whole number, got {v}"),
        });
    }
    // Safety: v is finite and integer-valued; i32 range check follows.
    #[allow(clippy::cast_possible_truncation)]
    let as_i64 = v as i64;
    i32::try_from(as_i64).map_err(|_| DialogueError::Function {
        name: fn_name.to_owned(),
        message: format!("{param} ({v}) is out of i32 range"),
    })
}

/// Converts a `f64` dialogue number to `u32`, returning an error if the value
/// is non-finite, negative, has a fractional part, or falls outside `u32` range.
fn number_to_u32(fn_name: &str, v: f64, param: &str) -> Result<u32> {
    if !v.is_finite() || v.fract() != 0.0 || v < 0.0 {
        return Err(DialogueError::Function {
            name: fn_name.to_owned(),
            message: format!("{param} must be a non-negative whole number, got {v}"),
        });
    }
    // Safety: v is finite, non-negative, and integer-valued; u32 range check follows.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let as_u64 = v as u64;
    u32::try_from(as_u64).map_err(|_| DialogueError::Function {
        name: fn_name.to_owned(),
        message: format!("{param} ({v}) is out of u32 range"),
    })
}

#[cfg(test)]
mod tests;
