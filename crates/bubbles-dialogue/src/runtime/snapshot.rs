//! [`RunnerSnapshot`] - session state for bookmarks and save / load support.

use std::collections::{BTreeMap, BTreeSet};

use super::rng::RngState;

/// A point-in-time snapshot of the dialogue session's mutable state.
///
/// Use [`Runner::snapshot`](crate::runtime::Runner::snapshot) to capture and
/// [`Runner::restore`](crate::runtime::Runner::restore) to apply.
///
/// The snapshot records:
/// - which node was active (`current_node`),
/// - how many times each node has been visited (`visits`),
/// - which `<<once>>` blocks have already fired (`once_seen`),
/// - where the runner's random stream had reached (`rng`).
///
/// # Canonical bytes
///
/// The collections are ordered, so two snapshots holding the same state
/// serialise to the same bytes in every process. That is what makes save files
/// safe to content-hash, diff, deduplicate, or compare in a golden-file test.
///
/// **Variable storage is not included** - it is the host's responsibility to
/// persist [`HashMapStorage`](crate::HashMapStorage) (or their own
/// [`VariableStorage`](crate::VariableStorage) impl) alongside the snapshot when
/// building a full save file.
///
/// With the `serde` feature, this type also derives `Serialize` / `Deserialize`
/// so you can write it to disk as JSON or another format.
///
/// # Save / load example
///
/// Both the snapshot **and** variable storage must be serialised together when
/// you persist to disk. Neither is complete without the other.
///
/// ```rust
/// # #[cfg(feature = "serde")]
/// # {
/// use bubbles::{HashMapStorage, Runner, Value, VariableStorage, compile};
///
/// let src = "title: A\n---\n<<set $gold = 10>>\nHello!\n===\n";
/// let prog = compile(src).unwrap();
///
/// // First session
/// let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
/// runner.start("A").unwrap();
/// let _ = runner.next_event(); // NodeStarted
/// let _ = runner.next_event(); // (set $gold side-effect, then Line)
///
/// let snap = runner.snapshot();
/// let snap_json = serde_json::to_string(&snap).unwrap();
/// let vars_json = serde_json::to_string(runner.storage()).unwrap();
///
/// let saved_vars: HashMapStorage = serde_json::from_str(&vars_json).unwrap();
/// let saved_snap: bubbles::RunnerSnapshot = serde_json::from_str(&snap_json).unwrap();
///
/// let mut runner2 = Runner::new(prog, saved_vars);
/// runner2.restore(saved_snap).unwrap();
/// // runner2 is back at the beginning of node "A" with $gold already set.
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunnerSnapshot {
    /// The node that was executing when the snapshot was taken.
    ///
    /// When restoring, [`Runner::restore`](crate::runtime::Runner::restore) will
    /// restart execution from the *beginning* of this node. This is intentional:
    /// the in-progress statement list is not serialised because it would
    /// require the full AST to be round-tripped.
    pub current_node: Option<String>,

    /// How many times each node has been visited, ordered by node title.
    pub visits: BTreeMap<String, u32>,

    /// IDs of `<<once>>` blocks (and once-line-variants) that have already
    /// fired and must not fire again, in sorted order.
    pub once_seen: BTreeSet<String>,

    /// Where the runner's generator had reached, so a restored session
    /// continues the same random stream instead of starting a new one.
    ///
    /// Snapshots written before 3.0.0 have no such field; they deserialise
    /// with a fresh OS-seeded stream, which is what those saves did anyway.
    #[cfg_attr(feature = "serde", serde(default = "RngState::from_entropy"))]
    pub rng: RngState,
}
