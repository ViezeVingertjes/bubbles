//! [`AppState`] — the pure model the renderer and the input layer agree on.
//!
//! Nothing in this module does I/O.  Tests build an `AppState` from a source
//! string, push [`Intent`]s through [`AppState::apply`], and inspect the
//! result.  The real binary does the same thing, just with key events
//! translated into intents upstream.

use bubbles::{DialogueError, DialogueEvent};

use crate::intent::Intent;
use crate::session::Session;

/// A line of dialogue ready to be drawn on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedLine {
    /// Optional speaker prefix (the `Speaker:` part of a line).
    pub speaker: Option<String>,
    /// Fully interpolated line text (all `{expr}` fragments already resolved).
    pub text: String,
}

/// The app's complete state: owns the runtime session and whatever is
/// currently on screen.
pub struct AppState {
    session: Session,
    current_line: Option<DisplayedLine>,
    quit_requested: bool,
}

impl AppState {
    /// Compiles `source` and starts the runner on `start_node`, returning a
    /// fresh, idle state.  No events have been pulled yet.
    ///
    /// # Errors
    /// Propagates any compile or start-time runtime error from
    /// [`bubbles::compile`] / [`bubbles::Runner::start`].
    pub fn from_source(source: &str, start_node: &str) -> Result<Self, DialogueError> {
        Ok(Self {
            session: Session::from_source(source, start_node)?,
            current_line: None,
            quit_requested: false,
        })
    }

    /// The line currently awaiting the next `Intent::Advance`, if any.
    #[must_use]
    pub const fn current_line(&self) -> Option<&DisplayedLine> {
        self.current_line.as_ref()
    }

    /// `true` once the dialogue has finished or the user has asked to quit.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.quit_requested || self.session.is_done()
    }

    /// `true` if the user has explicitly asked to quit (distinct from the
    /// dialogue ending on its own).
    #[must_use]
    pub const fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// Applies a user intent to the state.
    ///
    /// # Errors
    /// Any runtime error produced while pulling events from the runner is
    /// surfaced here; the caller decides whether to recover or abort.
    pub fn apply(&mut self, intent: Intent) -> Result<(), DialogueError> {
        match intent {
            Intent::Advance => self.advance(),
            Intent::Quit => {
                self.quit_requested = true;
                Ok(())
            }
        }
    }

    fn advance(&mut self) -> Result<(), DialogueError> {
        self.current_line = None;
        while let Some(event) = self.session.next_event()? {
            if let DialogueEvent::Line { speaker, text, .. } = event {
                self.current_line = Some(DisplayedLine { speaker, text });
                return Ok(());
            }
        }
        Ok(())
    }
}
