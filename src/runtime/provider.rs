//! [`LineProvider`] trait for localisation / text substitution.

/// Supplies localised (or otherwise substituted) text for a line.
///
/// When a line carries a `#line:<id>` tag, the runner queries the provider with that id.
/// If the provider returns `Some(text)`, that replaces the original line text in the event.
/// If it returns `None`, the original source text is used as-is.
pub trait LineProvider: Send + Sync + 'static {
    /// Returns localised text for `line_id`, or `None` to use the source text.
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
