//! Internationalisation built-in functions: plural, select.

use crate::error::DialogueError;
use crate::value::Value;

use super::FunctionLibrary;

impl FunctionLibrary {
    pub(super) fn register_i18n_builtins(&mut self) {
        self.register("select", |args| match args.as_slice() {
            [Value::Text(key), Value::Text(mapping)] => {
                let mut fallback: Option<&str> = None;
                for entry in mapping.split('|') {
                    let (k, v) = entry
                        .split_once(':')
                        .ok_or_else(|| DialogueError::Function {
                            name: "select".into(),
                            message: format!("mapping entry {entry:?} has no ':' separator"),
                        })?;
                    if k == "other" {
                        fallback = Some(v);
                    } else if k == key.as_str() {
                        return Ok(Value::Text(v.to_owned()));
                    }
                }
                fallback
                    .map(|v| Value::Text(v.to_owned()))
                    .ok_or_else(|| DialogueError::Function {
                        name: "select".into(),
                        message: format!(
                            "no match for key {key:?} and no 'other' fallback in mapping"
                        ),
                    })
            }
            _ => Err(DialogueError::Function {
                name: "select".into(),
                message: format!("expected (string, string), got {args:?}"),
            }),
        });
        self.register("plural", |args| match args.as_slice() {
            [Value::Number(n), Value::Text(singular), Value::Text(plural)] => {
                let form = if (n.abs() - 1.0).abs() < f64::EPSILON {
                    singular
                } else {
                    plural
                };
                Ok(Value::Text(form.clone()))
            }
            _ => Err(DialogueError::Function {
                name: "plural".into(),
                message: format!("expected (number, string, string), got {args:?}"),
            }),
        });
    }
}
