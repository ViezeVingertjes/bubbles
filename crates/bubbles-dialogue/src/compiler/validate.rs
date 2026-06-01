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
            validate_stmts(node.body.as_ref(), program, &node.title)?;
        }
    }
    Ok(())
}

fn validate_stmts(stmts: &[Stmt], program: &Program, node_title: &str) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Jump(target) | Stmt::Detour(target) if !program.node_exists(target) => {
                return Err(DialogueError::Validation(format!(
                    "node '{node_title}' references unknown node '{target}'"
                )));
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                for b in branches {
                    validate_stmts(&b.body, program, node_title)?;
                }
                validate_stmts(else_body, program, node_title)?;
            }
            Stmt::Once {
                body, else_body, ..
            } => {
                validate_stmts(body, program, node_title)?;
                validate_stmts(else_body, program, node_title)?;
            }
            Stmt::Options(items) => {
                for item in items {
                    validate_stmts(&item.body, program, node_title)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
