//! Random built-in functions: `random`, `random_range`, `dice`.

use crate::error::{DialogueError, Result};
use crate::value::Value;

use super::FunctionLibrary;
use super::math::require_two_numbers;

impl FunctionLibrary {
    pub(super) fn register_rand_builtins(&mut self) {
        use rand::RngExt as _;
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

/// Converts a `f64` dialogue number to `i32`, returning an error if the value
/// is non-finite, has a fractional part, or falls outside `i32` range.
fn number_to_i32(fn_name: &str, v: f64, param: &str) -> Result<i32> {
    if !v.is_finite() || v.fract() != 0.0 {
        return Err(DialogueError::Function {
            name: fn_name.to_owned(),
            message: format!("{param} must be a whole number, got {v}"),
        });
    }
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
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let as_u64 = v as u64;
    u32::try_from(as_u64).map_err(|_| DialogueError::Function {
        name: fn_name.to_owned(),
        message: format!("{param} ({v}) is out of u32 range"),
    })
}
