//! [`FirstAvailable`] - pick the first eligible candidate.

use crate::runtime::DialogueRng;

use super::candidate::{Candidate, SaliencyStrategy};

/// Picks the first available candidate.
///
/// This is the default strategy when no other is configured.
///
/// # Example
///
/// ```rust
/// use bubbles::DialogueRng;
/// use bubbles::saliency::{Candidate, FirstAvailable, SaliencyStrategy};
///
/// let mut s = FirstAvailable;
/// let candidates = vec![
///     Candidate { id: "a", available: false },
///     Candidate { id: "b", available: true },
///     Candidate { id: "c", available: true },
/// ];
/// assert_eq!(s.select(&candidates, &DialogueRng::from_seed(1)), Some(1));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FirstAvailable;

impl SaliencyStrategy for FirstAvailable {
    fn select(&mut self, candidates: &[Candidate<'_>], _rng: &DialogueRng) -> Option<usize> {
        candidates.iter().position(|c| c.available)
    }
}
