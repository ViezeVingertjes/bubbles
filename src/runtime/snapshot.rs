//! [`RunnerSnapshot`] — serialisable session state for save / load support.

use std::collections::{HashMap, HashSet};

/// A point-in-time snapshot of the dialogue session's mutable state.
///
/// Use `Runner::snapshot` to capture and `Runner::restore` to apply.
///
/// The snapshot records:
/// - which node was active (`current_node`),
/// - how many times each node has been visited (`visits`),
/// - which `<<once>>` blocks have already fired (`once_seen`).
///
/// **Variable storage is not included** — it is the host's responsibility to
/// serialise `HashMapStorage` (or their own [`VariableStorage`] impl) alongside
/// the snapshot.  Both are `serde`-ready when the `serde` feature is enabled.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "serde")]
/// # {
/// use bubbles::{compile, HashMapStorage, Runner};
///
/// let src = "title: A\n---\nLine one.\n===\n";
/// let prog = compile(src).unwrap();
/// let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
/// runner.start("A").unwrap();
/// let _ = runner.next_event(); // NodeStarted
/// let _ = runner.next_event(); // Line
///
/// let snap = runner.snapshot();
/// let json = serde_json::to_string(&snap).unwrap();
///
/// // … later, in a new game session …
/// let mut runner2 = Runner::new(prog, HashMapStorage::new());
/// let snap2: bubbles::RunnerSnapshot = serde_json::from_str(&json).unwrap();
/// runner2.restore(snap2).unwrap();
/// # }
/// ```
///
/// [`VariableStorage`]: crate::value::VariableStorage
#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunnerSnapshot {
    /// The node that was executing when the snapshot was taken.
    ///
    /// When restoring, `Runner::restore` will restart execution from the
    /// *beginning* of this node.  This is intentional: the in-progress
    /// statement list is not serialised because it would require the full
    /// AST to be round-tripped.
    pub current_node: Option<String>,

    /// How many times each node has been visited.
    pub visits: HashMap<String, u32>,

    /// IDs of `<<once>>` blocks (and once-line-variants) that have already
    /// fired and must not fire again.
    pub once_seen: HashSet<String>,
}
