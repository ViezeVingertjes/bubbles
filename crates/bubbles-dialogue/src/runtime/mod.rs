//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod builder;
mod eval;
mod event;
mod provider;
mod runner;
mod snapshot;

pub use builder::RunnerBuilder;
pub use eval::eval;
pub use event::{DialogueEvent, DialogueOption, MarkupSpan, line_id_from_tags};
pub use provider::{HashMapProvider, LineProvider, PassthroughProvider};
pub use runner::{Runner, RunnerPhase};
pub use snapshot::RunnerSnapshot;
