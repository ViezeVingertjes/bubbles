//! Compilation pipeline: source text → [`Program`].

pub mod ast;
pub mod expr;
pub mod lexer;
pub(crate) mod parser;
pub mod program;
pub(crate) mod validate;

pub use ast::{BinOp, Expr, IfBranch, LineVariant, Node, OptionItem, Stmt, TextSegment, UnOp};
pub use lexer::{Spanned, Token, tokenise};
pub use program::{Program, VariableDecl};
pub use validate::validate;

use crate::error::Result;

/// Compiles a single `.bub` source string into a [`Program`].
///
/// # Errors
/// Returns [`crate::DialogueError::Parse`] if the source is malformed,
/// or [`crate::DialogueError::DuplicateNode`] if two nodes share a title
/// without `when:` grouping conditions.
pub fn compile(source: &str) -> Result<Program> {
    compile_many(&[("<source>", source)])
}

/// Compiles multiple named `.bub` sources into a single [`Program`].
///
/// Sources are merged in order; duplicate node titles without `when:` grouping
/// conditions cause a [`crate::DialogueError::DuplicateNode`] error.
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
