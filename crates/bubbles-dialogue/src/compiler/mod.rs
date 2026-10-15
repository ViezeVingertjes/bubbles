//! Compilation pipeline: source text → [`Program`].

// The AST, the lexer, the expression parser and the markup scanner are
// implementation detail: `Program` has no public constructor and no
// `Deserialize`, and its accessors hand back only `&str`, `&[String]` and
// `&[VariableDecl]`, so none of these types is reachable from a public
// signature. Keeping them private means a lexer, parser or AST refactor is no
// longer a breaking change. The fuzz harness reaches the three entry points it
// needs through the `fuzzing` feature; see `crate::__fuzz`.
pub(crate) mod ast;
pub(crate) mod expr;
pub(crate) mod lexer;
pub(crate) mod markup;
pub(crate) mod parser;
pub(crate) mod program;
pub(crate) mod validate;

pub use program::{Program, VariableDecl};

use crate::error::Result;
use validate::validate;

/// Compiles a single `.bub` source string into a [`Program`].
///
/// Jump and detour targets are validated immediately; a
/// [`crate::DialogueError::Validation`] error is returned for any reference
/// to a node that does not exist in the compiled program.
///
/// For compiling multiple source files together use [`compile_many`].
///
/// # Errors
/// Returns [`crate::DialogueError::Parse`] if the source is malformed,
/// [`crate::DialogueError::DuplicateNode`] if two nodes share a title without
/// `when:` grouping conditions, or [`crate::DialogueError::Validation`] if a
/// jump or detour target cannot be resolved.
pub fn compile(source: &str) -> Result<Program> {
    compile_many(&[("<source>", source)])
}

/// Compiles multiple named `.bub` sources into a single [`Program`].
///
/// Sources are merged in order; duplicate node titles without `when:` grouping
/// conditions cause a [`crate::DialogueError::DuplicateNode`] error. Jump and
/// detour targets are validated across all sources after merging.
///
/// # Errors
/// Returns a [`crate::DialogueError`] variant on any parse, merge, validation,
/// or empty-source failure.
pub fn compile_many(sources: &[(&str, &str)]) -> Result<Program> {
    let mut all_nodes = Vec::new();
    for (name, source) in sources {
        if source.trim().is_empty() {
            return Err(crate::error::DialogueError::Validation(format!(
                "source is empty ({name})"
            )));
        }
        let nodes = parser::parse(name, source)?;
        all_nodes.extend(nodes);
    }
    let prog = Program::from_nodes(all_nodes)?;
    if prog.node_titles().next().is_none() {
        return Err(crate::error::DialogueError::Validation(
            "no nodes in source".into(),
        ));
    }
    validate(&prog)?;
    Ok(prog)
}
