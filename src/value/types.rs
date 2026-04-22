//! The [`Value`] type representing all runtime values.

use core::fmt;

/// All possible runtime values used in expressions and variables.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    /// A floating-point number.
    Number(f64),
    /// A UTF-8 string.
    Text(String),
    /// A boolean.
    Bool(bool),
}

impl Value {
    /// Returns whether the value is truthy under permissive coercion.
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0,
            Self::Text(v) => !v.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(v) => {
                // Omit ".0" suffix so `{$n}` renders as "2" not "2.0".
                #[allow(clippy::cast_possible_truncation)]
                if v.fract() == 0.0 && v.abs() < 1e15 {
                    write!(f, "{}", *v as i64)
                } else {
                    write!(f, "{v}")
                }
            }
            Self::Text(v) => f.write_str(v),
            Self::Bool(v) => write!(f, "{v}"),
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::Text(v.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_integer_number() {
        assert_eq!(Value::Number(3.0).to_string(), "3");
    }

    #[test]
    fn display_fractional_number() {
        assert_eq!(Value::Number(3.5).to_string(), "3.5");
    }

    #[test]
    fn truthy_coercion() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Number(1.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(Value::Text("hi".into()).is_truthy());
        assert!(!Value::Text(String::new()).is_truthy());
    }
}
