//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod builder;
mod eval;
mod event;
mod provider;
mod runner;
#[cfg(feature = "serde")]
mod snapshot;

pub use builder::RunnerBuilder;
pub use eval::eval;
pub use event::{DialogueEvent, DialogueOption, MarkupSpan, line_id_from_tags};
pub use provider::{HashMapProvider, LineProvider, PassthroughProvider};
pub use runner::Runner;
#[cfg(feature = "serde")]
pub use snapshot::RunnerSnapshot;
