//! Random built-in functions: `random`, `random_range`, `dice`.

use crate::error::{DialogueError, Result};
use crate::runtime::DialogueRng;
use crate::value::Value;

use super::FunctionLibrary;
use super::math::require_two_numbers;

impl FunctionLibrary {
    /// Registers the random built-ins against `rng`.
    ///
    /// Each closure keeps its own handle to the same stream, which is how a
    /// seeded runner replays every roll.
    pub(super) fn register_rand_builtins(&mut self, rng: &DialogueRng) {
        let random = rng.clone();
        self.register("random", move |_args| Ok(Value::Number(random.next_f64())));

        let random_range = rng.clone();
        self.register("random_range", move |args| {
            let (lo, hi) = require_two_numbers("random_range", &args)?;
            let lo = number_to_i32("random_range", lo, "lo")?;
            let hi = number_to_i32("random_range", hi, "hi")?;
            if lo > hi {
                return Err(DialogueError::Function {
                    name: "random_range".into(),
                    message: format!("lo ({lo}) > hi ({hi})"),
                });
            }
            let rolled = random_range.range_inclusive(i64::from(lo), i64::from(hi));
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Number(rolled as f64))
        });

        let dice = rng.clone();
        self.register("dice", move |args| {
            let (sides, count) = require_two_numbers("dice", &args)?;
            let sides = number_to_u32("dice", sides, "sides")?;
            let count = number_to_u32("dice", count, "count")?;
            if sides == 0 {
                return Err(DialogueError::Function {
                    name: "dice".into(),
                    message: "sides must be > 0".into(),
                });
            }
            // Summing in u64 keeps the worst case (`dice(u32::MAX, u32::MAX)`)
            // from overflowing, which the previous u32 accumulator did.
            #[allow(clippy::cast_sign_loss)]
            let total: u64 = (0..count)
                .map(|_| dice.range_inclusive(1, i64::from(sides)) as u64)
                .sum();
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Number(total as f64))
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
