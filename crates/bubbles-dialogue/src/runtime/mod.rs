//! Runtime execution layer: [`Runner`] and [`DialogueEvent`].

mod builder;
pub(crate) mod eval;
mod event;
mod provider;
mod rng;
mod runner;
mod snapshot;

pub use builder::RunnerBuilder;
pub use event::{DialogueEvent, DialogueOption, LineMode, MarkupSpan};
pub use provider::{HashMapProvider, LineProvider};
// Re-exported for one more major cycle so the deprecation reaches users rather
// than arriving as a missing name.
#[allow(deprecated)]
pub use provider::PassthroughProvider;
pub use rng::{DialogueRng, RngState};
pub use runner::{Runner, RunnerPhase};
pub use snapshot::RunnerSnapshot;
