//! [`RandomAvailable`] - random choice among eligible candidates (`rand` feature).

use crate::runtime::DialogueRng;

use super::candidate::{Candidate, SaliencyStrategy};

/// Picks a random available candidate.
///
/// Only available with the `rand` feature.
///
/// Draws from the runner's generator, so a session seeded with
/// [`RunnerBuilder::with_seed`](crate::RunnerBuilder::with_seed) makes the same
/// choices every run.
///
/// # Example
///
/// ```rust
/// use bubbles::DialogueRng;
/// use bubbles::saliency::{Candidate, RandomAvailable, SaliencyStrategy};
///
/// let mut s = RandomAvailable;
/// let candidates = vec![
///     Candidate { id: "a", available: false },
///     Candidate { id: "b", available: true },
///     Candidate { id: "c", available: true },
/// ];
///
/// // The same seed always makes the same choice.
/// let first = s.select(&candidates, &DialogueRng::from_seed(7));
/// assert_eq!(first, s.select(&candidates, &DialogueRng::from_seed(7)));
/// assert!(first == Some(1) || first == Some(2));
/// ```
#[derive(Debug, Clone, Default)]
pub struct RandomAvailable;

impl SaliencyStrategy for RandomAvailable {
    fn select(&mut self, candidates: &[Candidate<'_>], rng: &DialogueRng) -> Option<usize> {
        let available = candidates.iter().filter(|c| c.available).count();
        let nth = rng.choose_index(available)?;
        candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.available)
            .nth(nth)
            .map(|(i, _)| i)
    }
}
