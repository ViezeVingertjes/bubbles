//! Compilation pipeline: source text → [`Program`].

pub mod lexer;

pub use lexer::{Spanned, Token, tokenise};
