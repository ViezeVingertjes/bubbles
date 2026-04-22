//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod event;
mod runner;

pub use event::{DialogueEvent, DialogueOption};
pub use runner::Runner;
