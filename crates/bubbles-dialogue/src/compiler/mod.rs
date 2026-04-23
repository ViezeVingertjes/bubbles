//! Compilation pipeline: source text → [`Program`].

pub mod ast;
pub mod expr;
pub mod interpolation;
pub mod lexer;
pub(crate) mod parser;
pub mod program;
pub(crate) mod validate;

pub use ast::{BinOp, Expr, IfBranch, LineVariant, Node, OptionItem, Stmt, TextSegment, UnOp};
pub use lexer::{Spanned, Token, tokenise, tokenise_strict};
pub use program::{Program, VariableDecl};
pub use validate::validate;

use crate::error::Result;

/// Compiles a single `.bub` source string into a [`Program`].
///
/// **Validation is not performed.** Unknown jump or detour targets are only
/// caught at runtime as [`crate::DialogueError::UnknownNode`]. Use
/// [`compile_validated`] to catch them eagerly at compile time.
///
/// # Errors
/// Returns [`crate::DialogueError::Parse`] if the source is malformed,
/// or [`crate::DialogueError::DuplicateNode`] if two nodes share a title
/// without `when:` grouping conditions.
pub fn compile(source: &str) -> Result<Program> {
    compile_many(&[("<source>", source)])
}

/// Compiles a single `.bub` source string and then runs [`validate`].
///
/// This is the recommended entry point when you want unknown jump or detour
/// targets to be rejected at compile time rather than at runtime.
///
/// # Errors
/// Returns the same errors as [`compile`], plus [`crate::DialogueError::Validation`]
/// if any jump or detour targets cannot be resolved.
pub fn compile_validated(source: &str) -> Result<Program> {
    compile_many_validated(&[("<source>", source)])
}

/// Compiles multiple named `.bub` sources into a single [`Program`].
///
/// Sources are merged in order; duplicate node titles without `when:` grouping
/// conditions cause a [`crate::DialogueError::DuplicateNode`] error.
///
/// **Validation is not performed.** Use [`compile_many_validated`] to also
/// check that all jump and detour targets exist.
///
/// # Errors
/// Returns a [`crate::DialogueError`] variant on any parse or merge failure.
pub fn compile_many(sources: &[(&str, &str)]) -> Result<Program> {
    let mut all_nodes = Vec::new();
    for (name, source) in sources {
        let nodes = parser::parse(name, source)?;
        all_nodes.extend(nodes);
    }
    Program::from_nodes(all_nodes)
}

/// Compiles multiple named `.bub` sources into a single [`Program`] and then
/// runs [`validate`].
///
/// This is the recommended multi-file entry point when you want unknown jump
/// or detour targets to be rejected immediately rather than at runtime.
///
/// # Errors
/// Returns the same errors as [`compile_many`], plus
/// [`crate::DialogueError::Validation`] if any cross-file jump or detour
/// targets cannot be resolved.
pub fn compile_many_validated(sources: &[(&str, &str)]) -> Result<Program> {
    let prog = compile_many(sources)?;
    validate(&prog)?;
    Ok(prog)
}
