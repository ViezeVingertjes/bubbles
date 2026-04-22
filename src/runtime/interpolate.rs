//! Unit tests for `{expr}` interpolation.
//!
//! The production code path that handled runtime text parsing has been replaced
//! by [`crate::compiler::parser::assignments::parse_interpolated`] (compile time)
//! and [`crate::runtime::runner::Runner::eval_segments`] (runtime). This module
//! keeps independent integration tests so the algorithm's behaviour is still
//! directly verified.

#[cfg(test)]
mod tests {
    use crate::compiler::expr::parse_expr;
    use crate::error::Result;
    use crate::runtime::eval::eval;
    use crate::value::{HashMapStorage, Value, VariableStorage};

    /// Minimal runtime interpolation used only inside these tests.
    fn interpolate<S, F>(text: &str, storage: &S, fns: &F) -> Result<String>
    where
        S: VariableStorage,
        F: Fn(&str, Vec<Value>) -> Result<Value>,
    {
        let mut result = String::with_capacity(text.len());
        let mut remaining = text;

        while let Some(open) = remaining.find('{') {
            result.push_str(&remaining[..open]);
            let after_open = &remaining[open + 1..];
            let close = after_open
                .find('}')
                .ok_or_else(|| crate::error::DialogueError::Parse {
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
        assert_eq!(
            interpolate("Hello {$name}!", &s, &no_fns).unwrap(),
            "Hello Alice!"
        );
    }

    #[test]
    fn arithmetic_expression_substituted() {
        let s = HashMapStorage::new();
        assert_eq!(
            interpolate("Result: {1 + 2}", &s, &no_fns).unwrap(),
            "Result: 3"
        );
    }

    #[test]
    fn multiple_fragments() {
        let mut s = HashMapStorage::new();
        s.set("$a", Value::Number(2.0));
        s.set("$b", Value::Number(3.0));
        assert_eq!(
            interpolate("{$a} * {$b} = {$a * $b}", &s, &no_fns).unwrap(),
            "2 * 3 = 6"
        );
    }

    #[test]
    fn unclosed_brace_errors() {
        let s = HashMapStorage::new();
        let err = interpolate("Hello {name", &s, &no_fns).unwrap_err();
        assert!(err.to_string().contains("unclosed"));
    }

    #[test]
    fn invalid_expr_inside_braces_errors() {
        let s = HashMapStorage::new();
        assert!(interpolate("{1 +}", &s, &no_fns).is_err());
    }
}
