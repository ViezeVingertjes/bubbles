//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod eval;
mod event;
pub(crate) mod interpolate;
mod provider;
mod runner;

pub use eval::eval;
pub use event::{DialogueEvent, DialogueOption};
pub use provider::{HashMapProvider, LineProvider, PassthroughProvider};
pub use runner::Runner;
