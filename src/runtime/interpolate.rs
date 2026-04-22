//! Inline `{expr}` substitution for line text.

use crate::compiler::expr::parse_expr;
use crate::error::Result;
use crate::runtime::eval::eval;
use crate::value::VariableStorage;

/// Replaces all `{…}` fragments in `text` with their evaluated values.
///
/// # Errors
/// Propagates parse or evaluation errors from any fragment.
pub fn interpolate<S, F>(text: &str, storage: &S, fns: &F) -> Result<String>
where
    S: VariableStorage,
    F: Fn(&str, Vec<crate::value::Value>) -> Result<crate::value::Value>,
{
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(open) = remaining.find('{') {
        result.push_str(&remaining[..open]);
        let after_open = &remaining[open + 1..];
        let close = after_open.find('}').ok_or_else(|| crate::error::DialogueError::Parse {
            file: "<line>".into(),
            line: 0,
            message: format!("unclosed `{{` in text: `{text}`"),
        })?;
        let expr_src = &after_open[..close];
        let expr = parse_expr(expr_src)?;
        let value = eval(&expr, storage, fns)?;
        result.push_str(&value.to_string());
        remaining = &after_open[close + 1..];
    }
    result.push_str(remaining);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{HashMapStorage, Value, VariableStorage};

    fn no_fns(_: &str, _: Vec<Value>) -> crate::error::Result<Value> {
        Err(crate::error::DialogueError::Runtime("no fns".into()))
    }

    #[test]
    fn plain_text_unchanged() {
        let s = HashMapStorage::new();
        assert_eq!(interpolate("hello", &s, &no_fns).unwrap(), "hello");
    }

    #[test]
    fn single_variable_substituted() {
        let mut s = HashMapStorage::new();
        s.set("$name", Value::Text("Alice".into()));
        assert_eq!(interpolate("Hello {$name}!", &s, &no_fns).unwrap(), "Hello Alice!");
    }

    #[test]
    fn arithmetic_expression_substituted() {
        let s = HashMapStorage::new();
        assert_eq!(interpolate("Result: {1 + 2}", &s, &no_fns).unwrap(), "Result: 3");
    }

    #[test]
    fn multiple_fragments() {
        let mut s = HashMapStorage::new();
        s.set("$a", Value::Number(2.0));
        s.set("$b", Value::Number(3.0));
        assert_eq!(interpolate("{$a} * {$b} = {$a * $b}", &s, &no_fns).unwrap(), "2 * 3 = 6");
    }
}
