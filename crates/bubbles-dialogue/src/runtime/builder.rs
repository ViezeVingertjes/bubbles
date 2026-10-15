//! [`RunnerBuilder`] - ergonomic one-shot construction of a configured [`Runner`].

use crate::compiler::Program;
use crate::library::{FunctionLibrary, HostFn};
use crate::runtime::provider::{LineProvider, PassthroughProvider};
use crate::runtime::rng::DialogueRng;
use crate::saliency::{FirstAvailable, SaliencyStrategy};
use crate::value::VariableStorage;

use super::Runner;

/// Builds a [`Runner`] with optional saliency, provider, seed, and
/// host-function configuration in a single fluent chain.
///
/// Use this when you need to customise at least one of those at construction
/// time; for the zero-config case [`Runner::new`] is still the fastest path.
///
/// # Example
///
/// ```rust
/// use bubbles::{HashMapStorage, RunnerBuilder, Value, compile};
/// use bubbles::saliency::BestLeastRecentlyViewed;
///
/// let prog = compile("title: A\n---\nHello.\n===\n").unwrap();
/// let mut runner = RunnerBuilder::new(prog, HashMapStorage::new())
///     .with_saliency(BestLeastRecentlyViewed::default())
///     .with_function("greet", |_args| Ok(Value::Text("hi".into())))
///     .build();
/// runner.start("A").unwrap();
/// ```
pub struct RunnerBuilder<S: VariableStorage> {
    program: Program,
    storage: S,
    saliency: Box<dyn SaliencyStrategy>,
    provider: Box<dyn LineProvider>,
    library: FunctionLibrary,
    rng: DialogueRng,
}

impl<S: VariableStorage> RunnerBuilder<S> {
    /// Creates a builder with defaults matching [`Runner::new`]:
    /// [`FirstAvailable`] saliency, [`PassthroughProvider`], and randomness
    /// seeded from the operating system.
    pub fn new(program: Program, storage: S) -> Self {
        let rng = DialogueRng::from_entropy();
        Self {
            program,
            storage,
            saliency: Box::new(FirstAvailable),
            provider: Box::new(PassthroughProvider),
            library: FunctionLibrary::with_rng(&rng),
            rng,
        }
    }

    /// Seeds the runner's randomness, making the session reproducible.
    ///
    /// Two runners built with the same program, the same starting variables,
    /// the same seed, and given the same option selections produce the same
    /// sequence of events - including every `random()`, `random_range()`,
    /// `dice()` and [`RandomAvailable`](crate::saliency::RandomAvailable)
    /// choice - in every process and on every platform.
    ///
    /// Without this call randomness is seeded from the operating system, as in
    /// earlier releases. Either way,
    /// [`Runner::snapshot`](crate::Runner::snapshot) records where the stream
    /// had reached, so a restored save carries on rather than diverging.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "rand")]
    /// # {
    /// use bubbles::{DialogueEvent, HashMapStorage, RunnerBuilder, compile};
    ///
    /// let prog = compile("title: A\n---\nYou rolled {dice(6, 2)}.\n===\n").unwrap();
    ///
    /// let mut line_for = |seed: u64| {
    ///     let mut runner = RunnerBuilder::new(prog.clone(), HashMapStorage::new())
    ///         .with_seed(seed)
    ///         .build();
    ///     runner.start("A").unwrap();
    ///     loop {
    ///         if let Some(DialogueEvent::Line { text, .. }) = runner.next_event().unwrap() {
    ///             return text;
    ///         }
    ///     }
    /// };
    ///
    /// assert_eq!(line_for(7), line_for(7));
    /// # }
    /// ```
    #[must_use]
    pub fn with_seed(self, seed: u64) -> Self {
        self.rng.reseed(seed);
        self
    }

    /// Overrides the saliency strategy used for line and node group selection.
    #[must_use]
    pub fn with_saliency(mut self, strategy: impl SaliencyStrategy) -> Self {
        self.saliency = Box::new(strategy);
        self
    }

    /// Overrides the line provider used for localisation lookup.
    #[must_use]
    pub fn with_provider(mut self, provider: impl LineProvider) -> Self {
        self.provider = Box::new(provider);
        self
    }

    /// Registers a host function available to dialogue expressions.
    ///
    /// This is a convenience wrapper around
    /// [`FunctionLibrary::register`] that keeps configuration on the builder
    /// rather than requiring a post-construction `library_mut()` call.
    #[must_use]
    pub fn with_function<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(Vec<crate::value::Value>) -> crate::error::Result<crate::value::Value>
            + Send
            + Sync
            + 'static,
    {
        self.library.register(name, Box::new(f) as HostFn);
        self
    }

    /// Consumes the builder and returns a fully configured [`Runner`].
    ///
    /// The runner is in the `Idle` state; call [`Runner::start`] to begin
    /// execution.
    #[must_use]
    pub fn build(self) -> Runner<S> {
        Runner::with_parts(
            self.program,
            self.storage,
            self.saliency,
            self.provider,
            self.library,
            self.rng,
        )
    }
}
