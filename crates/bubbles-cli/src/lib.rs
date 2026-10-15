// `ratatui` + `crossterm` together transitively pull in several duplicate
// dependency versions (bitflags 1/2, syn 1/2, thiserror 1/2, etc.).  Those are
// outside our control and stable across ratatui releases; allow the cargo
// lint at the crate level rather than fighting upstream.
#![allow(clippy::multiple_crate_versions)]

//! Command-line front end for the
//! [`bubbles`](https://docs.rs/bubbles-dialogue) dialogue runtime.
//!
//! Two verbs, in order of how often you want them:
//!
//! - [`check`] compiles one or more `.bub` files and reports one diagnostic
//!   per line.  It needs nothing but the runtime's compiler, so it is
//!   available with `--no-default-features` and is the part meant to run in
//!   CI.
//! - The interactive player sits behind the default-on `tui` feature.
//!   [`AppState`] owns the compiled program, [`Intent`] is the input
//!   vocabulary, and [`render`] draws a state into a ratatui frame (real or
//!   `TestBackend`), so the whole UI is testable without a terminal.
//!
//! ```no_run
//! # #[cfg(feature = "tui")] {
//! use bubbles_cli::{AppState, Intent};
//!
//! let mut state = AppState::from_source("title: A\n---\nHi.\n===\n", "A").unwrap();
//! state.apply(Intent::Advance);
//! assert_eq!(state.current_line().unwrap().text, "Hi.");
//! # }
//! ```

pub mod check;
mod diagnostic;
mod source_set;

#[cfg(feature = "tui")]
mod app;
#[cfg(feature = "tui")]
mod display;
#[cfg(feature = "tui")]
mod help;
#[cfg(feature = "tui")]
mod history;
#[cfg(feature = "tui")]
mod ingest;
#[cfg(feature = "tui")]
mod intent;
#[cfg(feature = "tui")]
mod markup;
#[cfg(feature = "tui")]
mod session;
#[cfg(feature = "tui")]
pub mod terminal;
#[cfg(feature = "tui")]
mod transcript;
#[cfg(feature = "tui")]
mod ui;

pub use check::Report;
pub use diagnostic::{Diagnostic, DiagnosticLocation};
pub use source_set::SourceSet;

#[cfg(feature = "tui")]
pub use app::AppState;
#[cfg(feature = "tui")]
pub use display::{DisplayedLine, DisplayedOption, FocusPanel};
#[cfg(feature = "tui")]
pub use help::KEYS;
#[cfg(feature = "tui")]
pub use intent::Intent;
#[cfg(feature = "tui")]
pub use transcript::TranscriptEntry;
#[cfg(feature = "tui")]
pub use ui::render;
