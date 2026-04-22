//! [`SaliencyStrategy`] trait and built-in implementations.

/// A candidate presented to the saliency strategy for selection.
#[derive(Debug, Clone)]
pub struct Candidate<'a> {
    /// Stable id for this variant.
    pub id: &'a str,
    /// Whether this candidate is eligible (guard condition passed, not once-exhausted).
    pub available: bool,
}

/// Selects one candidate from a list of eligible variants.
///
/// Implement this trait to customise line-group and node-group selection behaviour.
pub trait SaliencyStrategy: Send + Sync + 'static {
    /// Returns the index into `candidates` of the chosen variant, or `None` to skip the group.
    fn select(&self, candidates: &[Candidate<'_>]) -> Option<usize>;
}

// ── built-in strategies ───────────────────────────────────────────────────────

/// Picks the first available candidate.
///
/// This is the default strategy when no other is configured.
#[derive(Debug, Clone, Default)]
pub struct FirstAvailable;

impl SaliencyStrategy for FirstAvailable {
    fn select(&self, candidates: &[Candidate<'_>]) -> Option<usize> {
        candidates.iter().position(|c| c.available)
    }
}

/// Picks a random available candidate.
///
/// Only available with the `rand` feature.
#[cfg(feature = "rand")]
#[derive(Debug, Clone, Default)]
pub struct RandomAvailable;

#[cfg(feature = "rand")]
impl SaliencyStrategy for RandomAvailable {
    fn select(&self, candidates: &[Candidate<'_>]) -> Option<usize> {
        use rand::seq::IndexedRandom as _;
        let available: Vec<usize> = candidates
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if c.available { Some(i) } else { None })
            .collect();
        available.choose(&mut rand::rng()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(available: &[bool]) -> Vec<Candidate<'_>> {
        available
            .iter()
            .enumerate()
            .map(|(i, &a)| Candidate {
                id: Box::leak(format!("c{i}").into_boxed_str()),
                available: a,
            })
            .collect()
    }

    #[test]
    fn first_available_picks_first() {
        let s = FirstAvailable;
        assert_eq!(s.select(&make(&[false, true, true])), Some(1));
    }

    #[test]
    fn first_available_none_when_all_unavailable() {
        let s = FirstAvailable;
        assert_eq!(s.select(&make(&[false, false])), None);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn random_available_returns_valid_index() {
        let s = RandomAvailable;
        let candidates = make(&[true, false, true]);
        let idx = s.select(&candidates);
        // should pick either 0 or 2
        assert!(idx == Some(0) || idx == Some(2));
    }
}
