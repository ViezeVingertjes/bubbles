//! [`Candidate`] and the [`SaliencyStrategy`] trait.

/// A candidate presented to the saliency strategy for selection.
#[derive(Debug, Clone)]
pub struct Candidate<'a> {
    /// Stable id for this variant.
    pub id: &'a str,
    /// Whether this candidate is eligible (guard condition passed, not once-exhausted).
    pub available: bool,
}

/// Selects one candidate from a list of available variants.
///
/// Implement this trait to customise line-group and node-group selection behaviour.
/// Strategies may be stateful (e.g. tracking which variants have been recently shown).
pub trait SaliencyStrategy: Send + Sync + 'static {
    /// Returns the index into `candidates` of the chosen variant, or `None` to skip the group.
    fn select(&mut self, candidates: &[Candidate<'_>]) -> Option<usize>;
}
