//! [`SaliencyStrategy`] trait and built-in implementations.

use std::collections::HashMap;

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

// ── built-in strategies ───────────────────────────────────────────────────────

/// Picks the first available candidate.
///
/// This is the default strategy when no other is configured.
///
/// # Example
///
/// ```rust
/// use bubbles::saliency::{Candidate, FirstAvailable, SaliencyStrategy};
///
/// let mut s = FirstAvailable;
/// let candidates = vec![
///     Candidate { id: "a", available: false },
///     Candidate { id: "b", available: true },
///     Candidate { id: "c", available: true },
/// ];
/// assert_eq!(s.select(&candidates), Some(1));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FirstAvailable;

impl SaliencyStrategy for FirstAvailable {
    fn select(&mut self, candidates: &[Candidate<'_>]) -> Option<usize> {
        candidates.iter().position(|c| c.available)
    }
}

/// Picks a random available candidate.
///
/// Only available with the `rand` feature.
///
/// # Example
///
/// ```rust
/// use bubbles::saliency::{Candidate, RandomAvailable, SaliencyStrategy};
///
/// let mut s = RandomAvailable;
/// let candidates = vec![
///     Candidate { id: "a", available: false },
///     Candidate { id: "b", available: true },
///     Candidate { id: "c", available: true },
/// ];
/// let idx = s.select(&candidates);
/// assert!(idx == Some(1) || idx == Some(2));
/// ```
#[cfg(feature = "rand")]
#[derive(Debug, Clone, Default)]
pub struct RandomAvailable;

#[cfg(feature = "rand")]
impl SaliencyStrategy for RandomAvailable {
    fn select(&mut self, candidates: &[Candidate<'_>]) -> Option<usize> {
        use rand::seq::IndexedRandom as _;
        let available: Vec<usize> = candidates
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if c.available { Some(i) } else { None })
            .collect();
        available.choose(&mut rand::rng()).copied()
    }
}

/// Picks the available candidate that was least recently selected.
///
/// Candidates that have never been shown are treated as if they were last shown at turn 0,
/// which gives them priority over candidates that have already been shown. Among candidates
/// with equal "last seen" turns the one with the lower index wins.
///
/// This strategy is ideal for NPC barks and variation lines where you want maximum
/// variety before repeating content.
///
/// # Example
///
/// ```rust
/// use bubbles::saliency::{BestLeastRecentlyViewed, Candidate, SaliencyStrategy};
///
/// let mut s = BestLeastRecentlyViewed::default();
/// let candidates = vec![
///     Candidate { id: "a", available: true },
///     Candidate { id: "b", available: true },
///     Candidate { id: "c", available: true },
/// ];
///
/// // First call — all unseen, picks index 0.
/// assert_eq!(s.select(&candidates), Some(0));
/// // Second call — "a" was just seen, picks "b" at index 1.
/// assert_eq!(s.select(&candidates), Some(1));
/// // Third call — picks "c" at index 2.
/// assert_eq!(s.select(&candidates), Some(2));
/// // Fourth call — all seen, wraps back to "a" (oldest).
/// assert_eq!(s.select(&candidates), Some(0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct BestLeastRecentlyViewed {
    /// Maps candidate id → the turn on which it was last selected.
    last_seen: HashMap<String, u64>,
    /// Monotonically increasing counter incremented on every selection.
    turn: u64,
}

impl BestLeastRecentlyViewed {
    /// Creates a fresh strategy with no history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SaliencyStrategy for BestLeastRecentlyViewed {
    fn select(&mut self, candidates: &[Candidate<'_>]) -> Option<usize> {
        let idx = candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.available)
            .min_by_key(|(_, c)| self.last_seen.get(c.id).copied().unwrap_or(0))
            .map(|(i, _)| i)?;

        self.turn += 1;
        self.last_seen
            .insert(candidates[idx].id.to_owned(), self.turn);
        Some(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(ids: &[&'static str]) -> Vec<Candidate<'static>> {
        ids.iter()
            .map(|&id| Candidate {
                id,
                available: true,
            })
            .collect()
    }

    fn cand_mask(ids: &[&'static str], mask: &[bool]) -> Vec<Candidate<'static>> {
        ids.iter()
            .zip(mask.iter())
            .map(|(&id, &available)| Candidate { id, available })
            .collect()
    }

    // ── FirstAvailable ───────────────────────────────────────────────────────

    #[test]
    fn first_available_picks_first() {
        let mut s = FirstAvailable;
        let cs = cand_mask(&["a", "b", "c"], &[false, true, true]);
        assert_eq!(s.select(&cs), Some(1));
    }

    #[test]
    fn first_available_none_when_all_unavailable() {
        let mut s = FirstAvailable;
        let cs = cand_mask(&["a", "b"], &[false, false]);
        assert_eq!(s.select(&cs), None);
    }

    #[test]
    fn first_available_empty() {
        let mut s = FirstAvailable;
        assert_eq!(s.select(&[]), None);
    }

    // ── RandomAvailable ──────────────────────────────────────────────────────

    #[cfg(feature = "rand")]
    #[test]
    fn random_available_returns_valid_index() {
        let mut s = RandomAvailable;
        let cs = cand_mask(&["a", "b", "c"], &[true, false, true]);
        let idx = s.select(&cs);
        assert!(idx == Some(0) || idx == Some(2));
    }

    #[cfg(feature = "rand")]
    #[test]
    fn random_available_none_when_empty() {
        let mut s = RandomAvailable;
        assert_eq!(s.select(&[]), None);
    }

    // ── BestLeastRecentlyViewed ───────────────────────────────────────────────

    #[test]
    fn blrv_cycles_through_all_candidates() {
        let mut s = BestLeastRecentlyViewed::new();
        let cs = cand(&["a", "b", "c"]);
        assert_eq!(s.select(&cs), Some(0)); // a — unseen
        assert_eq!(s.select(&cs), Some(1)); // b — unseen
        assert_eq!(s.select(&cs), Some(2)); // c — unseen
        assert_eq!(s.select(&cs), Some(0)); // a — oldest seen
    }

    #[test]
    fn blrv_skips_unavailable() {
        let mut s = BestLeastRecentlyViewed::new();
        let cs = cand_mask(&["a", "b", "c"], &[false, true, true]);
        assert_eq!(s.select(&cs), Some(1)); // b — first available
        assert_eq!(s.select(&cs), Some(2)); // c — next unseen
    }

    #[test]
    fn blrv_none_when_all_unavailable() {
        let mut s = BestLeastRecentlyViewed::new();
        let cs = cand_mask(&["a", "b"], &[false, false]);
        assert_eq!(s.select(&cs), None);
    }

    #[test]
    fn blrv_single_candidate_always_picks_it() {
        let mut s = BestLeastRecentlyViewed::new();
        let cs = cand(&["only"]);
        assert_eq!(s.select(&cs), Some(0));
        assert_eq!(s.select(&cs), Some(0));
    }
}
