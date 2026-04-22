//! Compile-time validation of cross-node references.

use crate::compiler::ast::Stmt;
use crate::compiler::program::Program;
use crate::error::{DialogueError, Result};

/// Validates all jump and detour targets in `program` refer to existing nodes.
///
/// # Errors
/// Returns [`DialogueError::Validation`] on the first broken reference found.
pub fn validate(program: &Program) -> Result<()> {
    for variants in program.nodes.values() {
        for node in variants {
            validate_stmts(&node.body, program)?;
        }
    }
    Ok(())
}

fn validate_stmts(stmts: &[Stmt], program: &Program) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Jump(target) | Stmt::Detour(target) => {
                if !program.node_exists(target) {
                    return Err(DialogueError::Validation(format!(
                        "reference to unknown node '{target}'"
                    )));
                }
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                for (_, body) in branches {
                    validate_stmts(body, program)?;
                }
                validate_stmts(else_body, program)?;
            }
            Stmt::Once {
                body, else_body, ..
            } => {
                validate_stmts(body, program)?;
                validate_stmts(else_body, program)?;
            }
            Stmt::Options(items) => {
                for item in items {
                    validate_stmts(&item.body, program)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile;

    #[test]
    fn valid_jump_passes() {
        let prog = compile("title: A\n---\n<<jump B>>\n===\ntitle: B\n---\n===\n").unwrap();
        assert!(validate(&prog).is_ok());
    }

    #[test]
    fn unknown_jump_target_fails() {
        let prog = compile("title: A\n---\n<<jump Nonexistent>>\n===\n").unwrap();
        assert!(validate(&prog).is_err());
    }

    #[test]
    fn unknown_jump_inside_if_branch_fails() {
        let prog =
            compile("title: A\n---\n<<if true>>\n    <<jump Ghost>>\n<<endif>>\n===\n").unwrap();
        assert!(validate(&prog).is_err());
    }

    #[test]
    fn unknown_detour_in_else_fails() {
        let prog = compile(
            "title: A\n---\n<<if false>>\n    idle\n<<else>>\n    <<detour Missing>>\n<<endif>>\n===\n",
        )
        .unwrap();
        assert!(validate(&prog).is_err());
    }

    #[test]
    fn unknown_jump_inside_once_fails() {
        let prog =
            compile("title: A\n---\n<<once>>\n    <<jump Nope>>\n<<endonce>>\n===\n").unwrap();
        assert!(validate(&prog).is_err());
    }

    #[test]
    fn unknown_jump_inside_option_body_fails() {
        let prog = compile("title: A\n---\n-> Go\n    <<jump Bad>>\n===\n").unwrap();
        assert!(validate(&prog).is_err());
    }

    #[test]
    fn unknown_jump_inside_once_else_fails() {
        let prog = compile(
            "title: A\n---\n<<once>>\n    ok\n<<else>>\n    <<jump Nope>>\n<<endonce>>\n===\n",
        )
        .unwrap();
        assert!(validate(&prog).is_err());
    }
}
