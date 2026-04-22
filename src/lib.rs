//! Lightweight engine-agnostic dialogue runtime for Rust games.
//!
//! See the [README](https://github.com/example/bubbles) for a quick-start guide.

#![deny(missing_docs, unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod compiler;
pub mod value;

pub use value::{HashMapStorage, Value, VariableStorage};
