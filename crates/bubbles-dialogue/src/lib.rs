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
//! **It is deterministic by construction.** Nothing here reads the clock or the
//! environment, ever. It reaches for randomness on its own initiative exactly
//! once: an unseeded runner asks the operating system for a starting seed while
//! it is being built. Pass [`RunnerBuilder::with_seed`] and it reaches for
//! nothing at all. The rest of the nondeterminism is opt-in and retractable:
//!
//! - `rand` is a Cargo feature. Build with `default-features = false` and
//!   `random`, `random_range`, `dice` and `RandomAvailable` are not compiled
//!   in, and the crate has no entropy source: an unseeded runner starts at the
//!   beginning of the stream, so that build replays identically whether or not
//!   you pass a seed, and `with_seed` is how you introduce unpredictability if
//!   you want it. That is also the configuration required to target
//!   `wasm32-unknown-unknown`, since the default build depends on `getrandom`,
//!   so on the web the deterministic build is the only build.
//! - With `rand` on, [`RunnerBuilder::with_seed`] makes randomness
//!   reproducible: `random`, `random_range`, `dice` and `RandomAvailable` all
//!   draw from one [`DialogueRng`] the runner owns, and its position is
//!   recorded in [`Runner::snapshot`]. The three built-ins are also entries in
//!   a [`FunctionLibrary`](library::FunctionLibrary), and
//!   [`register`](library::FunctionLibrary::register) replaces any function of
//!   the same name, so you can install your own generator instead.
//! - Line-group and node-group selection goes through an injected
//!   [`SaliencyStrategy`](saliency::SaliencyStrategy). The default
//!   [`FirstAvailable`](saliency::FirstAvailable) is deterministic;
//!   [`BestLeastRecentlyViewed`](saliency::BestLeastRecentlyViewed) is
//!   deterministic given the same call sequence; `RandomAvailable` is the
//!   opt-in random one.
//!
//! Given the same program, the same starting variables, the same seed, and the
//! same option selections, you get the same sequence of [`DialogueEvent`]s.
//! [`RunnerSnapshot`] and [`HashMapStorage`] serialise to identical bytes for
//! identical state, so saves can be hashed, diffed, or golden-tested.
//!
//! **Known edges.** [`Runner::snapshot`] is a node-granularity bookmark, not a
//! full continuation: [`Runner::restore`] re-enters the recorded node from its
//! beginning, and neither the `<<detour>>` return stack nor saliency history is
//! carried across. The random stream does survive, so rolls continue rather
//! than restart.
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
//! | [`compiler`] | [`compile`], [`compile_many`], [`Program`], [`VariableDecl`](compiler::VariableDecl) |
//! | [`runtime`] | [`Runner`], [`RunnerPhase`], [`DialogueEvent`], [`LineMode`], [`LineProvider`], [`RunnerSnapshot`], [`DialogueRng`], [`RngState`] |
//! | [`library`] | [`FunctionLibrary`](library::FunctionLibrary) and built-in functions |
//! | [`saliency`] | [`SaliencyStrategy`](saliency::SaliencyStrategy), [`FirstAvailable`](saliency::FirstAvailable), [`BestLeastRecentlyViewed`](saliency::BestLeastRecentlyViewed), `RandomAvailable` (`rand` feature) |
//!
//! The lexer, the expression parser, the AST and the markup scanner are
//! deliberately absent from that table. A [`Program`] has no public
//! constructor and no `Deserialize`, and its accessors return only `&str`,
//! `&[String]` and `&[VariableDecl]`, so none of those types was ever
//! reachable through a public signature. Keeping them private means rewriting
//! the parser is not a breaking change.
//!
//! Lint policy is defined once in the workspace root `Cargo.toml` under
//! `[workspace.lints]` and inherited here via `[lints] workspace = true`; we deliberately do
//! not duplicate it in this file.

pub mod compiler;
pub mod error;
pub mod library;
pub mod runtime;
pub mod saliency;
pub mod value;

// `#[doc(inline)]` on every root re-export is load-bearing, not cosmetic. The modules above
// are public, so by default rustdoc documents each item only at its canonical module path and
// leaves `bubbles/struct.Runner.html` unwritten — every link written in the short form 404s.
// Inlining makes the root the canonical path, which is also the path callers are told to
// import (`use bubbles::Runner`). Keep it on anything added here.
#[doc(inline)]
pub use compiler::{Program, compile, compile_many};
#[doc(inline)]
pub use error::{DialogueError, Result};
#[doc(inline)]
pub use runtime::RunnerSnapshot;
// Deprecated, and re-exported for one more major cycle so the warning reaches
// users at the path they already import it from.
#[allow(deprecated)]
#[doc(inline)]
pub use runtime::PassthroughProvider;
#[doc(inline)]
pub use runtime::{
    DialogueEvent, DialogueOption, DialogueRng, HashMapProvider, LineMode, LineProvider,
    MarkupSpan, RngState, Runner, RunnerBuilder, RunnerPhase,
};
#[doc(inline)]
pub use value::{HashMapStorage, Value, VariableStorage};

/// Compiler entry points for this repository's fuzz harness. **Not public API.**
///
/// `fuzz/` is a separate crate, so it cannot reach `pub(crate)` items the way
/// the unit tests do. This module, behind the non-default `fuzzing` feature,
/// hands it the three functions its targets drive directly. Nothing here is
/// covered by the semver contract: it can change or disappear in a patch
/// release, and no crate other than `bubbles-fuzz` should enable the feature.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub mod __fuzz {
    pub use crate::compiler::expr::parse_expr_at;
    pub use crate::compiler::lexer::tokenise;
    pub use crate::compiler::markup::scan_text_segments;
}
