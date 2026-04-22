//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod eval;
mod event;
mod interpolate;
mod provider;
mod runner;
#[cfg(feature = "serde")]
mod snapshot;

pub use eval::eval;
pub use event::{DialogueEvent, DialogueOption};
pub use provider::{HashMapProvider, LineProvider, PassthroughProvider};
pub use runner::Runner;
#[cfg(feature = "serde")]
pub use snapshot::RunnerSnapshot;
