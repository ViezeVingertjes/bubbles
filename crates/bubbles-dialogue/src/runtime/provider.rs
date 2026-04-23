//! [`LineProvider`] trait for localisation / text substitution.

/// Supplies localised (or otherwise substituted) text for a line.
///
/// When a line carries a `#line:<id>` tag the runner queries the provider with
/// that id **before** evaluating any `{expr}` placeholders (translate-then-format
/// ordering).  The returned string is itself a template: any `{expr}` it
/// contains are evaluated against the current variable storage, so translators
/// can reorder or reshape interpolations however their language requires.
///
/// The same id is also exposed on [`crate::DialogueEvent::Line`] as `line_id`
/// (see [`crate::line_id_from_tags`]).
///
/// If the provider returns `None`, the original source text (with its own
/// `{expr}` segments) is used unchanged.
pub trait LineProvider: Send + Sync + 'static {
    /// Returns a localised template for `line_id`, or `None` to fall back to
    /// the source text.
    ///
    /// The returned string may contain `{expr}` placeholders; they will be
    /// evaluated after translation.
    fn get(&self, line_id: &str) -> Option<String>;
}

/// A no-op provider that always returns `None` (source text is used unchanged).
#[derive(Debug, Clone, Default)]
pub struct PassthroughProvider;

impl LineProvider for PassthroughProvider {
    fn get(&self, _line_id: &str) -> Option<String> {
        None
    }
}

/// A simple in-memory provider backed by a [`std::collections::HashMap`].
#[derive(Debug, Clone, Default)]
pub struct HashMapProvider {
    map: std::collections::HashMap<String, String>,
}

impl HashMapProvider {
    /// Creates an empty provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a localised string for the given id.
    pub fn insert(&mut self, id: impl Into<String>, text: impl Into<String>) {
        self.map.insert(id.into(), text.into());
    }
}

impl LineProvider for HashMapProvider {
    fn get(&self, line_id: &str) -> Option<String> {
        self.map.get(line_id).cloned()
    }
}
