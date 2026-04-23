//! [`AppState`] — the pure model the renderer and the input layer agree on.
//!
//! Nothing in this module does I/O.  Tests build an `AppState` from a source
//! string, push [`Intent`]s through [`AppState::apply`], and inspect the
//! result.  The real binary does the same thing, just with key events
//! translated into intents upstream.

use bubbles::{DialogueError, DialogueEvent, DialogueOption};

use crate::intent::Intent;
use crate::session::Session;
use crate::transcript::{Transcript, TranscriptEntry};

/// A line of dialogue ready to be drawn on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedLine {
    /// Optional speaker prefix (the `Speaker:` part of a line).
    pub speaker: Option<String>,
    /// Fully interpolated line text (all `{expr}` fragments already resolved).
    pub text: String,
}

/// An option ready to be drawn on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedOption {
    /// Fully interpolated option text.
    pub text: String,
    /// Whether the option's guard currently passes; false options are
    /// displayed but not selectable.
    pub available: bool,
}

impl From<DialogueOption> for DisplayedOption {
    fn from(opt: DialogueOption) -> Self {
        Self {
            text: opt.text,
            available: opt.available,
        }
    }
}

/// Which pane currently owns the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    /// The dialogue / options pane.
    Dialogue,
    /// The transcript pane.
    Transcript,
}

/// The app's complete state: owns the runtime session and whatever is
/// currently on screen.
pub struct AppState {
    session: Session,
    current_line: Option<DisplayedLine>,
    options: Vec<DisplayedOption>,
    focused_option: Option<usize>,
    transcript: Transcript,
    focus: FocusPanel,
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
            options: Vec::new(),
            focused_option: None,
            transcript: Transcript::new(),
            focus: FocusPanel::Dialogue,
            quit_requested: false,
        })
    }

    /// The line currently awaiting the next `Intent::Advance`, if any.
    #[must_use]
    pub const fn current_line(&self) -> Option<&DisplayedLine> {
        self.current_line.as_ref()
    }

    /// The current option set, or an empty slice when no prompt is active.
    #[must_use]
    pub fn options(&self) -> &[DisplayedOption] {
        &self.options
    }

    /// Index of the focused option, when an option prompt is active.
    #[must_use]
    pub const fn focused_option(&self) -> Option<usize> {
        self.focused_option
    }

    /// The running session transcript, oldest to newest.
    #[must_use]
    pub fn transcript(&self) -> &[TranscriptEntry] {
        self.transcript.as_slice()
    }

    /// How many entries the transcript view is scrolled back from the tail.
    /// Zero means "newest entry visible".
    #[must_use]
    pub const fn transcript_scroll(&self) -> usize {
        self.transcript.scroll()
    }

    /// `true` when the transcript pane currently owns the keyboard focus.
    #[must_use]
    pub const fn transcript_focused(&self) -> bool {
        matches!(self.focus, FocusPanel::Transcript)
    }

    /// Which pane currently owns the keyboard focus.
    #[must_use]
    pub const fn focus(&self) -> FocusPanel {
        self.focus
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
            Intent::Advance => self.advance_or_commit(),
            Intent::Quit => {
                self.quit_requested = true;
                Ok(())
            }
            Intent::FocusNext => {
                self.move_focus(FocusShift::Next);
                Ok(())
            }
            Intent::FocusPrev => {
                self.move_focus(FocusShift::Prev);
                Ok(())
            }
            Intent::SelectOption(idx) => self.commit_option(idx),
            Intent::ToggleFocus => {
                self.toggle_focus();
                Ok(())
            }
            Intent::ScrollUp => {
                self.transcript.scroll_up();
                Ok(())
            }
            Intent::ScrollDown => {
                self.transcript.scroll_down();
                Ok(())
            }
        }
    }

    fn advance_or_commit(&mut self) -> Result<(), DialogueError> {
        if self.options.is_empty() {
            return self.advance();
        }
        if let Some(idx) = self.focused_option {
            self.commit_option(idx)?;
            if self.options.is_empty() {
                self.advance()?;
            }
        }
        Ok(())
    }

    fn advance(&mut self) -> Result<(), DialogueError> {
        self.current_line = None;
        while let Some(event) = self.session.next_event()? {
            if self.ingest_event(event) {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Records `event` in the transcript and updates derived state.  Returns
    /// `true` when the caller should stop pulling (a line or option prompt
    /// is now visible and awaits user input).
    fn ingest_event(&mut self, event: DialogueEvent) -> bool {
        match event {
            DialogueEvent::NodeStarted(name) => {
                self.transcript.push(TranscriptEntry::NodeStarted(name));
                false
            }
            DialogueEvent::NodeComplete(name) => {
                self.transcript.push(TranscriptEntry::NodeComplete(name));
                false
            }
            DialogueEvent::Line { speaker, text, .. } => {
                self.transcript.push(TranscriptEntry::Line {
                    speaker: speaker.clone(),
                    text: text.clone(),
                });
                self.current_line = Some(DisplayedLine { speaker, text });
                true
            }
            DialogueEvent::Options(opts) => {
                self.options = opts.into_iter().map(DisplayedOption::from).collect();
                self.focused_option = self
                    .options
                    .iter()
                    .position(|o| o.available)
                    .or(Some(0))
                    .filter(|_| !self.options.is_empty());
                true
            }
            DialogueEvent::Command { name, args, tags } => {
                self.transcript
                    .push(TranscriptEntry::Command { name, args, tags });
                false
            }
            // `DialogueComplete` and any future non-exhaustive variant are
            // handled by `Session::is_done()`; nothing to record here.
            _ => false,
        }
    }

    fn move_focus(&mut self, delta: FocusShift) {
        match self.focus {
            FocusPanel::Dialogue => self.shift_option_focus(delta),
            FocusPanel::Transcript => match delta {
                FocusShift::Next => self.transcript.scroll_down(),
                FocusShift::Prev => self.transcript.scroll_up(),
            },
        }
    }

    fn shift_option_focus(&mut self, delta: FocusShift) {
        let len = self.options.len();
        if len == 0 {
            return;
        }
        let current = self.focused_option.unwrap_or(0);
        let next = match delta {
            FocusShift::Next => (current + 1) % len,
            FocusShift::Prev => (current + len - 1) % len,
        };
        self.focused_option = Some(next);
    }

    const fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Dialogue => FocusPanel::Transcript,
            FocusPanel::Transcript => FocusPanel::Dialogue,
        };
    }

    fn commit_option(&mut self, index: usize) -> Result<(), DialogueError> {
        let Some(opt) = self.options.get(index) else {
            return Ok(());
        };
        if !opt.available {
            return Ok(());
        }
        let text = opt.text.clone();
        self.session.select_option(index)?;
        self.transcript
            .push(TranscriptEntry::OptionChosen { text, index });
        self.options.clear();
        self.focused_option = None;
        self.current_line = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum FocusShift {
    Next,
    Prev,
}
