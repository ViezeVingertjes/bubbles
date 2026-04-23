// `ratatui` + `crossterm` together transitively pull in several duplicate
// dependency versions (bitflags 1/2, syn 1/2, thiserror 1/2, etc.).  Those are
// outside our control and stable across ratatui releases; allow the cargo
// lint at the crate level rather than fighting upstream.
#![allow(clippy::multiple_crate_versions)]

//! Writer-focused terminal UI on top of the
//! [`bubbles`](https://docs.rs/bubbles-dialogue) dialogue runtime.
//!
//! The crate is split into a small, pure model and a thin rendering layer so
//! everything can be exercised without a real terminal:
//!
//! - [`AppState`] owns the compiled program and answers questions like
//!   "what line is currently on screen?" or "is the dialogue over?".
//! - [`Intent`] is the input vocabulary.  Tests apply intents directly;
//!   the binary translates key events into them.
//! - [`render`] is the single entry point for drawing an [`AppState`] into a
//!   ratatui frame (real or `TestBackend`).
//!
//! ```no_run
//! use bubbles_tui::{AppState, Intent};
//!
//! let mut state = AppState::from_source("title: A\n---\nHi.\n===\n", "A").unwrap();
//! state.apply(Intent::Advance).unwrap();
//! assert_eq!(state.current_line().unwrap().text, "Hi.");
//! ```

mod app;
mod display;
mod history;
mod ingest;
mod intent;
mod overlay;
mod session;
pub mod terminal;
mod transcript;
mod ui;

pub use app::AppState;
pub use display::{DisplayedLine, DisplayedOption, FocusPanel};
pub use history::HistoryStep;
pub use intent::Intent;
pub use overlay::{ErrorLocation, ErrorOverlay};
pub use transcript::{Transcript, TranscriptEntry};
pub use ui::render;
