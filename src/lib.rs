//! Lightweight, engine-agnostic dialogue runtime for Rust games.
//!
//! Write branching dialogue in `.bub` scripts, compile them once, then drive
//! the dialogue from any game loop via a pull-based event API.
//!
//! # Quick start
//!
//! ```rust
//! use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};
//!
//! let source = "title: Greet\n---\nHello!\n===\n";
//! let prog = compile(source).unwrap();
//! let mut runner = Runner::new(prog, HashMapStorage::new());
//! runner.start("Greet").unwrap();
//!
//! assert!(matches!(
//!     runner.next_event().unwrap(),
//!     Some(DialogueEvent::NodeStarted(_))
//! ));
//! assert!(matches!(
//!     runner.next_event().unwrap(),
//!     Some(DialogueEvent::Line { .. })
//! ));
//! assert!(matches!(
//!     runner.next_event().unwrap(),
//!     Some(DialogueEvent::NodeComplete(_))
//! ));
//! assert!(matches!(
//!     runner.next_event().unwrap(),
//!     Some(DialogueEvent::DialogueComplete)
//! ));
//! ```
//!
//! # Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`value`] | [`Value`], [`VariableStorage`], [`HashMapStorage`] |
//! | [`compiler`] | [`compile`], [`compile_many`], [`validate`], [`Program`], [`VariableDecl`] |
//! | [`runtime`] | [`Runner`], [`DialogueEvent`], [`LineProvider`], `RunnerSnapshot` (serde) |
//! | [`library`] | [`FunctionLibrary`] and built-in functions |
//! | [`saliency`] | [`SaliencyStrategy`], [`FirstAvailable`], [`BestLeastRecentlyViewed`], `RandomAvailable` (`rand` feature) |
//!
//! Lint policy is defined once in `Cargo.toml` under `[lints.rust]` /
//! `[lints.clippy]`; we deliberately do not duplicate it here.

pub mod compiler;
pub mod error;
pub mod library;
pub mod runtime;
pub mod saliency;
pub mod value;

pub use compiler::{Program, VariableDecl, compile, compile_many, validate};
pub use error::{DialogueError, Result};
pub use library::FunctionLibrary;
#[cfg(feature = "serde")]
pub use runtime::RunnerSnapshot;
pub use runtime::{
    DialogueEvent, DialogueOption, HashMapProvider, LineProvider, PassthroughProvider, Runner,
};
#[cfg(feature = "rand")]
pub use saliency::RandomAvailable;
pub use saliency::{BestLeastRecentlyViewed, Candidate, FirstAvailable, SaliencyStrategy};
pub use value::{HashMapStorage, Value, VariableStorage};
