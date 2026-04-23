//! [`SaliencyStrategy`] trait and built-in implementations.

mod blrv;
mod candidate;
mod first;
#[cfg(feature = "rand")]
mod random;

#[cfg(test)]
mod tests;

pub use blrv::BestLeastRecentlyViewed;
pub use candidate::{Candidate, SaliencyStrategy};
pub use first::FirstAvailable;
#[cfg(feature = "rand")]
pub use random::RandomAvailable;
