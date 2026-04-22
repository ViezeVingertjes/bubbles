//! Value model: [`Value`] enum, [`VariableStorage`] trait, and [`HashMapStorage`].

mod storage;
mod value;

pub use storage::{HashMapStorage, VariableStorage};
pub use value::Value;
