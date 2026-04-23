//! [`FirstAvailable`] — pick the first eligible candidate.

use super::candidate::{Candidate, SaliencyStrategy};

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
