//! [`RandomAvailable`] — random choice among eligible candidates (`rand` feature).

use super::candidate::{Candidate, SaliencyStrategy};

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
#[derive(Debug, Clone, Default)]
pub struct RandomAvailable;

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
