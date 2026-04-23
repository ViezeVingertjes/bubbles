//! [`SourceSet`] — one or more named source files that are compiled together.
//!
//! A single-file workflow is just a `SourceSet` of length one.  `AppState`
//! always stores a `SourceSet`; the convenience constructors
//! (`from_source` / `load`) build one internally.

use bubbles::DialogueError;

/// A collection of `(filename, source)` pairs that form a complete programme.
///
/// The filenames are used both by the compiler (they appear in [`DialogueError`]
/// parse locations) and by the error overlay when looking up the source excerpt
/// for the offending line.
#[derive(Debug, Clone, Default)]
pub struct SourceSet {
    files: Vec<(String, String)>,
}

impl SourceSet {
    /// Creates a `SourceSet` from a single source string.
    pub fn single(filename: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            files: vec![(filename.into(), source.into())],
        }
    }

    /// Creates a `SourceSet` from an ordered slice of `(filename, source)` pairs.
    pub fn many<N, S>(files: impl IntoIterator<Item = (N, S)>) -> Self
    where
        N: Into<String>,
        S: Into<String>,
    {
        Self {
            files: files
                .into_iter()
                .map(|(n, s)| (n.into(), s.into()))
                .collect(),
        }
    }

    /// Returns a `Vec` of `(&str, &str)` slices suitable for [`bubbles::compile_many`].
    #[must_use]
    pub fn as_named_slices(&self) -> Vec<(&str, &str)> {
        self.files
            .iter()
            .map(|(n, s)| (n.as_str(), s.as_str()))
            .collect()
    }

    /// Looks up the source text for the file named `filename`.
    ///
    /// Used when building error excerpts: the [`DialogueError::Parse`] variant
    /// carries the filename so we can retrieve exactly the right source text.
    #[must_use]
    pub fn find_source(&self, filename: &str) -> Option<&str> {
        self.files
            .iter()
            .find(|(name, _)| name == filename)
            .map(|(_, src)| src.as_str())
    }

    /// Returns the source text most appropriate for an error excerpt.
    ///
    /// For `Parse` errors the compiler includes the filename, so we look that
    /// up exactly.  For all other errors (runtime, validation…) we fall back
    /// to the first file in the set, which is the natural "primary" script.
    #[must_use]
    pub fn source_for_error(&self, err: &DialogueError) -> Option<&str> {
        if let DialogueError::Parse { file, .. } = err {
            self.find_source(file)
        } else {
            self.files.first().map(|(_, s)| s.as_str())
        }
    }

    /// `true` when the set contains exactly one file (the common case).
    #[must_use]
    pub const fn is_single(&self) -> bool {
        self.files.len() == 1
    }

    /// Number of files in the set.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.files.len()
    }

    /// `true` when the set is empty (should not happen in normal use).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
