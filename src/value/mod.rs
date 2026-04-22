//! Value model: [`Value`] enum, [`VariableStorage`] trait, and [`HashMapStorage`].

mod storage;
mod types;

pub use storage::{HashMapStorage, VariableStorage};
pub use types::Value;
