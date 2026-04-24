//! [`RunnerBuilder`] - ergonomic one-shot construction of a configured [`Runner`].

use crate::compiler::Program;
use crate::library::{FunctionLibrary, HostFn};
use crate::runtime::provider::{LineProvider, PassthroughProvider};
use crate::saliency::{FirstAvailable, SaliencyStrategy};
use crate::value::VariableStorage;

use super::Runner;

/// Builds a [`Runner`] with optional saliency, provider, and host-function
/// configuration in a single fluent chain.
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
}

impl<S: VariableStorage> RunnerBuilder<S> {
    /// Creates a builder with defaults matching [`Runner::new`]:
    /// [`FirstAvailable`] saliency and [`PassthroughProvider`].
    pub fn new(program: Program, storage: S) -> Self {
        Self {
            program,
            storage,
            saliency: Box::new(FirstAvailable),
            provider: Box::new(PassthroughProvider),
            library: FunctionLibrary::new(),
        }
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
        let mut runner = Runner::new(self.program, self.storage);
        runner.set_saliency_box(self.saliency);
        runner.set_provider_box(self.provider);
        *runner.library_mut() = self.library;
        runner
    }
}
