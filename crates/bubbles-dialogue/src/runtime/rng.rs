//! [`DialogueRng`] - the runner-owned random number generator, and the
//! [`RngState`] value that makes its stream resumable from a save file.

use std::sync::{Arc, Mutex, PoisonError};

/// Golden-ratio increment from `SplitMix64`.
const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// The resumable state of a [`DialogueRng`].
///
/// A snapshot stores one of these so [`Runner::restore`](crate::Runner::restore)
/// continues the same random stream instead of starting a new one. The value is
/// opaque: there is nothing to inspect and nothing to construct by hand, but it
/// serialises (with the `serde` feature) as a single integer, so save files stay
/// small and compare byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct RngState(u64);

impl RngState {
    /// A fresh OS-seeded position, used when loading a save that predates
    /// seeding and therefore carries no stream state.
    #[cfg(feature = "serde")]
    pub(crate) fn from_entropy() -> Self {
        Self(entropy_seed())
    }
}

impl Default for RngState {
    /// The start of the stream for seed `0`.
    ///
    /// Building a [`RunnerSnapshot`](crate::RunnerSnapshot) by hand - in a test
    /// or a save editor - gives a reproducible stream rather than an arbitrary
    /// one.
    fn default() -> Self {
        Self(0)
    }
}

/// The random number generator a [`Runner`](crate::Runner) draws from.
///
/// Every source of randomness inside the runtime - the `random()`,
/// `random_range()` and `dice()` built-ins, and
/// [`RandomAvailable`](crate::saliency::RandomAvailable) - draws from the
/// runner's own generator rather than from a thread-local global. Seed it with
/// [`RunnerBuilder::with_seed`](crate::RunnerBuilder::with_seed) and the whole
/// session replays identically.
///
/// # Reproducibility
///
/// The generator is `SplitMix64`, implemented in this crate rather than taken
/// from a dependency, specifically so the stream is a documented part of the
/// save format: the same seed yields the same sequence on every target, in
/// every build, for as long as the major version holds.
///
/// # Sharing
///
/// [`Clone`] produces another handle to the *same* stream, not an independent
/// copy, which is how the built-in functions and the saliency strategy stay in
/// step. Methods take `&self` for that reason.
///
/// # Example
///
/// ```rust
/// use bubbles::DialogueRng;
///
/// let rng = DialogueRng::from_seed(42);
/// let first: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
///
/// let again = DialogueRng::from_seed(42);
/// assert_eq!(first, (0..4).map(|_| again.next_u64()).collect::<Vec<_>>());
/// ```
#[derive(Clone)]
pub struct DialogueRng {
    state: Arc<Mutex<u64>>,
}

impl DialogueRng {
    /// Creates a generator that produces the same stream for the same `seed`,
    /// on every platform and in every process.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(seed)),
        }
    }

    /// Creates a generator seeded from the operating system, as an unseeded
    /// [`Runner`](crate::Runner) does.
    #[must_use]
    pub fn from_entropy() -> Self {
        Self::from_seed(entropy_seed())
    }

    /// Restarts this generator's stream from `seed`.
    ///
    /// Every handle sharing the stream observes the change.
    pub fn reseed(&self, seed: u64) {
        *self.lock() = seed;
    }

    /// Returns the current position in the stream, for storing in a
    /// [`RunnerSnapshot`](crate::RunnerSnapshot).
    #[must_use]
    pub fn state(&self) -> RngState {
        RngState(*self.lock())
    }

    /// Resumes the stream from a position captured by [`state`](Self::state).
    pub fn set_state(&self, state: RngState) {
        *self.lock() = state.0;
    }

    /// Returns the next value in the stream, uniformly distributed over all
    /// 64-bit integers.
    #[must_use]
    pub fn next_u64(&self) -> u64 {
        let mut state = self.lock();
        *state = state.wrapping_add(GAMMA);
        mix(*state)
    }

    /// Returns a uniform float in `[0, 1)`.
    #[must_use]
    pub fn next_f64(&self) -> f64 {
        // 53 bits is the full mantissa, so every representable value in the
        // range is reachable and the result never rounds up to 1.0.
        #[allow(clippy::cast_precision_loss)]
        {
            (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
        }
    }

    /// Returns a uniform index into a collection of `len` items, or `None` when
    /// `len` is zero.
    ///
    /// Custom [`SaliencyStrategy`](crate::SaliencyStrategy) implementations
    /// should pick candidates through this method, so their choices are part of
    /// the same reproducible stream as everything else.
    #[must_use]
    pub fn choose_index(&self, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let len = u64::try_from(len).unwrap_or(u64::MAX);
        Some(usize::try_from(self.below(len)).unwrap_or(0))
    }

    /// Returns a uniform integer in `[lo, hi]`.
    ///
    /// Returns `lo` when `hi < lo`.
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn range_inclusive(&self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        // The width fits in u64 even for i64::MIN..=i64::MAX, which is why the
        // subtraction and the final add both wrap deliberately.
        let span = hi.wrapping_sub(lo) as u64;
        let offset = self.below(span.wrapping_add(1));
        lo.wrapping_add(offset as i64)
    }

    /// Returns a uniform integer in `[0, n)` with no modulo bias.
    ///
    /// `n` must be non-zero; `below(0)` returns 0.
    #[allow(clippy::cast_possible_truncation)]
    fn below(&self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        // Lemire's multiply-shift, with the rejection step that removes the
        // residual bias when n does not divide 2^64.
        let mut product = u128::from(self.next_u64()) * u128::from(n);
        if (product as u64) < n {
            let threshold = n.wrapping_neg() % n;
            while (product as u64) < threshold {
                product = u128::from(self.next_u64()) * u128::from(n);
            }
        }
        (product >> 64) as u64
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, u64> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for DialogueRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

impl std::fmt::Debug for DialogueRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogueRng")
            .field("state", &self.state())
            .finish()
    }
}

/// The `SplitMix64` finalising mix.
const fn mix(z: u64) -> u64 {
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Draws a seed from the operating system, matching the unseeded behaviour of
/// releases before 3.0.0.
#[cfg(feature = "rand")]
fn entropy_seed() -> u64 {
    rand::random::<u64>()
}

/// Without the `rand` feature there is no OS random source in the dependency
/// tree, so an unseeded generator falls back to the standard library's
/// per-process hash key, which is itself OS-seeded.
#[cfg(not(feature = "rand"))]
#[allow(clippy::cast_possible_truncation)]
fn entropy_seed() -> u64 {
    use std::hash::{BuildHasher as _, Hasher as _};

    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u64(
        std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .map_or(0, |d| d.as_nanos() as u64),
    );
    mix(hasher.finish())
}

#[cfg(test)]
#[path = "rng_tests.rs"]
mod tests;
