//! [`BestLeastRecentlyViewed`] - favour content not shown for the longest time.

use std::collections::HashMap;

use super::candidate::{Candidate, SaliencyStrategy};

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
/// // First call - all unseen, picks index 0.
/// assert_eq!(s.select(&candidates), Some(0));
/// // Second call - "a" was just seen, picks "b" at index 1.
/// assert_eq!(s.select(&candidates), Some(1));
/// // Third call - picks "c" at index 2.
/// assert_eq!(s.select(&candidates), Some(2));
/// // Fourth call - all seen, wraps back to "a" (oldest).
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
