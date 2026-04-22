//! Lightweight engine-agnostic dialogue runtime for Rust games.
//!
//! See the [README](https://github.com/example/bubbles) for a quick-start guide.

#![deny(missing_docs, unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod compiler;
pub mod error;
pub mod library;
pub mod runtime;
pub mod saliency;
pub mod value;

pub use compiler::{Program, compile, compile_many, validate};
pub use error::{DialogueError, Result};
pub use library::FunctionLibrary;
pub use runtime::{DialogueEvent, DialogueOption, HashMapProvider, LineProvider, PassthroughProvider, Runner};
pub use saliency::{Candidate, FirstAvailable, SaliencyStrategy};
pub use value::{HashMapStorage, Value, VariableStorage};
