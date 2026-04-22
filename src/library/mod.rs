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
        let mut lib = Self { fns: HashMap::new() };
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
            || Err(DialogueError::Function {
                name: name.to_owned(),
                message: "unknown function".into(),
            }),
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
        self.register("clamp", |args| {
            match args.as_slice() {
                [Value::Number(v), Value::Number(lo), Value::Number(hi)] => {
                    Ok(Value::Number(v.clamp(*lo, *hi)))
                }
                _ => Err(DialogueError::Function {
                    name: "clamp".into(),
                    message: format!("expected 3 number arguments, got {:?}", args),
                }),
            }
        });
        self.register("string", |args| {
            match args.into_iter().next() {
                Some(v) => Ok(Value::Text(v.to_string())),
                None => Err(DialogueError::Function {
                    name: "string".into(),
                    message: "expected 1 argument".into(),
                }),
            }
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
            let lo = lo as i64;
            let hi = hi as i64;
            if lo > hi {
                return Err(DialogueError::Function {
                    name: "random_range".into(),
                    message: format!("lo ({lo}) > hi ({hi})"),
                });
            }
            Ok(Value::Number(rand::rng().random_range(lo..=hi) as f64))
        });
        self.register("dice", |args| {
            let (sides, count) = require_two_numbers("dice", &args)?;
            let sides = sides as u64;
            let count = count as u64;
            if sides == 0 {
                return Err(DialogueError::Function {
                    name: "dice".into(),
                    message: "sides must be > 0".into(),
                });
            }
            let total: u64 = (0..count)
                .map(|_| rand::rng().random_range(1..=sides))
                .sum();
            Ok(Value::Number(total as f64))
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
            message: format!("expected 1 number argument, got {:?}", args),
        }),
    }
}

fn require_two_numbers(name: &str, args: &[Value]) -> Result<(f64, f64)> {
    match args {
        [Value::Number(a), Value::Number(b)] => Ok((*a, *b)),
        _ => Err(DialogueError::Function {
            name: name.to_owned(),
            message: format!("expected 2 number arguments, got {:?}", args),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_builtin() {
        let lib = FunctionLibrary::new();
        assert_eq!(lib.call("round", vec![Value::Number(3.7)]).unwrap(), Value::Number(4.0));
    }

    #[test]
    fn min_max_builtins() {
        let lib = FunctionLibrary::new();
        assert_eq!(lib.call("min", vec![Value::Number(2.0), Value::Number(5.0)]).unwrap(), Value::Number(2.0));
        assert_eq!(lib.call("max", vec![Value::Number(2.0), Value::Number(5.0)]).unwrap(), Value::Number(5.0));
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
        assert_eq!(lib.call("double", vec![Value::Number(5.0)]).unwrap(), Value::Number(10.0));
    }

    #[cfg(feature = "rand")]
    #[test]
    fn random_range_within_bounds() {
        let lib = FunctionLibrary::new();
        for _ in 0..20 {
            let v = lib.call("random_range", vec![Value::Number(1.0), Value::Number(6.0)]).unwrap();
            if let Value::Number(n) = v {
                assert!((1.0..=6.0).contains(&n));
            }
        }
    }
}
