//! Input history used to replay the session when the user steps back.
//!
//! Rather than capture a full [`RunnerSnapshot`] + storage blob on every
//! advance, we record the short list of user decisions and re-drive the
//! runner from the top on rewind.  This keeps the data trivially
//! `Clone`-able and gives us deterministic, per-step granularity.

/// One committed input that moved the session forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryStep {
    /// The user advanced past a line (or past a command/node marker).
    Advance,
    /// The user committed the option at this 0-based index.
    SelectOption(usize),
}
