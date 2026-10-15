//! An embeddable, deterministic dialogue runtime: scripted conversation as a
//! state machine you drive, not a narrative authoring suite.
//!
//! You write branching dialogue in `.bub` scripts, [`compile`] them to a plain
//! Rust struct, and pull events from a [`Runner`] whenever your application is
//! ready for the next one. There is no bytecode, no binary format, and nothing
//! to load at runtime.
//!
//! On [crates.io](https://crates.io/crates/bubbles-dialogue) the package is **bubbles-dialogue**
//! because the crate name **bubbles** was already taken there. This library is still named
//! **bubbles**: depend on `bubbles-dialogue` in `Cargo.toml` and `use bubbles::…` in code.
//!
//! # Why this library
//!
//! **It embeds without inverting your control flow.** The event API is
//! pull-based: [`Runner::next_event`] advances the state machine and returns,
//! and nothing happens between calls. The library contains no `async`
//! functions, spawns no tasks, and keeps no global or thread-local state, so a
//! [`Runner`] is an ordinary value you own and can hold many of. This is what
//! makes it viable inside somebody else's loop: a Bevy system, a Godot-rust
//! node, the main loop of a plain binary, a browser animation frame under
//! WebAssembly, or, for hosts not written in Rust, a C caller going through
//! the `bubbles-ffi` crate.
//!
//! **It is deterministic by construction.** Nothing here reads the clock, the
//! environment, or a global RNG on its own initiative. The nondeterminism you
//! can have is all opt-in, and all retractable:
//!
//! - `rand` is a Cargo feature. Build with `default-features = false` and no
//!   RNG is compiled in at all. That is also the configuration required to
//!   target `wasm32-unknown-unknown`, since the default build depends on
//!   `getrandom`, so on the web the deterministic build is the only build.
//! - `random`, `random_range` and `dice` are entries in a
//!   [`FunctionLibrary`]. [`FunctionLibrary::register`] replaces any function
//!   of the same name, so you can install a seeded generator you control.
//! - Line-group and node-group selection goes through an injected
//!   [`SaliencyStrategy`]. The default [`FirstAvailable`] is deterministic;
//!   [`BestLeastRecentlyViewed`] is deterministic given the same call
//!   sequence; `RandomAvailable` is the opt-in random one.
//!
//! Given the same program, the same starting variables, and the same option
//! selections, you get the same sequence of [`DialogueEvent`]s.
//!
//! **Known edges.** [`Runner::snapshot`] is a node-granularity bookmark, not a
//! full continuation: [`Runner::restore`] re-enters the recorded node from its
//! beginning, and neither the `<<detour>>` return stack nor saliency history is
//! carried across. Enumeration order from [`Runner::all_variables`] is
//! unspecified for the default [`HashMapStorage`].
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
//! | [`runtime`] | [`Runner`], [`RunnerPhase`], [`DialogueEvent`], [`LineMode`], [`LineProvider`], [`line_id_from_tags`], [`line_mode_from_tags`], [`RunnerSnapshot`], [`DialogueRng`], [`RngState`] |
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
pub use runtime::RunnerSnapshot;
pub use runtime::{
    DialogueEvent, DialogueOption, DialogueRng, HashMapProvider, LineMode, LineProvider,
    MarkupSpan, PassthroughProvider, RngState, Runner, RunnerBuilder, RunnerPhase,
    line_id_from_tags, line_mode_from_tags, option_group_from_tags,
};
#[cfg(feature = "rand")]
pub use saliency::RandomAvailable;
pub use saliency::{BestLeastRecentlyViewed, Candidate, FirstAvailable, SaliencyStrategy};
pub use value::{HashMapStorage, Value, VariableStorage};
