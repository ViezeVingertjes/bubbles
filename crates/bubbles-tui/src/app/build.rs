//! Constructors for [`AppState`].
//!
//! Kept in a separate file to stay within the project's per-file line limit
//! while keeping the runtime-dispatch logic in `mod.rs` easy to scan.

use super::AppState;
use crate::display::FocusPanel;
use crate::session::Session;
use crate::source_set::SourceSet;
use crate::transcript::Transcript;
use bubbles::DialogueError;

impl AppState {
    /// Compiles a single `source` string and starts the runner on `start_node`.
    ///
    /// # Errors
    /// Propagates any compile or start-time runtime error.
    pub fn from_source(source: &str, start_node: &str) -> Result<Self, DialogueError> {
        let sources = SourceSet::single("<source>", source);
        Self::from_source_set(sources, start_node)
    }

    /// Compiles multiple `(filename, source)` pairs together and starts the
    /// runner on `start_node`.  Nodes defined in any file are reachable from
    /// any other file via `<<jump>>` or `<<detour>>`.
    ///
    /// # Errors
    /// Propagates any compile or start-time runtime error.
    pub fn from_sources(sources: &[(&str, &str)], start_node: &str) -> Result<Self, DialogueError> {
        let set = SourceSet::many(sources.iter().copied());
        Self::from_source_set(set, start_node)
    }

    /// Compiles a [`SourceSet`] and starts the runner on `start_node`.
    ///
    /// This is the canonical constructor used by the binary when multiple files
    /// are read from disk and assembled into a set before calling.
    ///
    /// # Errors
    /// Propagates any compile or start-time runtime error.
    pub fn from_source_set(sources: SourceSet, start_node: &str) -> Result<Self, DialogueError> {
        let session = Session::from_source_set(&sources, start_node)?;
        Ok(Self::new(Some(session), sources, start_node))
    }

    /// Infallible counterpart to [`Self::from_source`]: on compile error the
    /// returned state carries an [`crate::overlay::ErrorOverlay`] and has no
    /// active session.
    #[must_use]
    pub fn load(source: &str, start_node: &str) -> Self {
        let sources = SourceSet::single("<source>", source);
        Self::load_source_set(sources, start_node)
    }

    /// Infallible counterpart to [`Self::from_sources`]: on compile error the
    /// returned state carries an [`crate::overlay::ErrorOverlay`] and has no
    /// active session.
    #[must_use]
    pub fn load_many(sources: &[(&str, &str)], start_node: &str) -> Self {
        let set = SourceSet::many(sources.iter().copied());
        Self::load_source_set(set, start_node)
    }

    /// Infallible counterpart to [`Self::from_source_set`].
    #[must_use]
    pub fn load_source_set(sources: SourceSet, start_node: &str) -> Self {
        match Session::from_source_set(&sources, start_node) {
            Ok(session) => Self::new(Some(session), sources, start_node),
            Err(err) => {
                let mut state = Self::new(None, sources, start_node);
                state.fail(&err);
                state
            }
        }
    }

    fn new(session: Option<Session>, sources: SourceSet, start_node: &str) -> Self {
        Self {
            session,
            sources,
            start_node: start_node.to_owned(),
            current_line: None,
            options: Vec::new(),
            focused_option: None,
            transcript: Transcript::new(),
            focus: FocusPanel::Options,
            error_overlay: None,
            history: Vec::new(),
            recording: true,
            quit_requested: false,
        }
    }
}
