//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod builder;
mod eval;
mod event;
mod provider;
mod runner;
mod snapshot;

pub use builder::RunnerBuilder;
pub use eval::eval;
pub use event::{
    DialogueEvent, DialogueOption, LineMode, MarkupSpan, line_id_from_tags, line_mode_from_tags,
    option_group_from_tags,
};
pub use provider::{HashMapProvider, LineProvider, PassthroughProvider};
pub use runner::{Runner, RunnerPhase};
pub use snapshot::RunnerSnapshot;
