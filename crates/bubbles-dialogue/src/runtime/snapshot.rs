//! [`RunnerSnapshot`] - serialisable session state for save / load support.

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
/// **Variable storage is not included** - it is the host's responsibility to
/// serialise `HashMapStorage` (or their own [`VariableStorage`] impl) alongside
/// the snapshot.  Both are `serde`-ready when the `serde` feature is enabled.
///
/// # Save / load example
///
/// Both the snapshot **and** variable storage must be serialised together —
/// neither is complete without the other.
///
/// ```rust
/// # #[cfg(feature = "serde")]
/// # {
/// use bubbles::{HashMapStorage, Runner, Value, VariableStorage, compile};
///
/// let src = "title: A\n---\n<<set $gold = 10>>\nHello!\n===\n";
/// let prog = compile(src).unwrap();
///
/// // ── first session ────────────────────────────────────────────────────
/// let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
/// runner.start("A").unwrap();
/// let _ = runner.next_event(); // NodeStarted
/// let _ = runner.next_event(); // (set $gold side-effect, then Line)
///
/// // Serialise BOTH pieces of mutable state.
/// let snap = runner.snapshot();
/// let snap_json = serde_json::to_string(&snap).unwrap();
/// let vars_json = serde_json::to_string(runner.storage()).unwrap();
///
/// // ── restore in a new session ─────────────────────────────────────────
/// let saved_vars: HashMapStorage = serde_json::from_str(&vars_json).unwrap();
/// let saved_snap: bubbles::RunnerSnapshot = serde_json::from_str(&snap_json).unwrap();
///
/// let mut runner2 = Runner::new(prog, saved_vars);
/// runner2.restore(saved_snap).unwrap();
/// // runner2 is now back at the beginning of node "A" with $gold already set.
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
