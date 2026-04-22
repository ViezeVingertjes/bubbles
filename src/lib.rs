//! Lightweight engine-agnostic dialogue runtime for Rust games.
//!
//! See the [README](https://github.com/example/bubbles) for a quick-start guide.

#![deny(missing_docs, unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod compiler;
pub mod error;
pub mod runtime;
pub mod value;

pub use compiler::{Program, compile, compile_many};
pub use error::{DialogueError, Result};
pub use runtime::{DialogueEvent, DialogueOption, Runner};
pub use value::{HashMapStorage, Value, VariableStorage};
