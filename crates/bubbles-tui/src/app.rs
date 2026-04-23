//! [`AppState`] - the pure model the renderer and the input layer agree on.
//!
//! Nothing in this module does I/O.  Tests build an `AppState` from a source
//! string, push [`Intent`]s through [`AppState::apply`], and inspect the
//! result.  The real binary does the same thing, just with key events
//! translated into intents upstream.

use crate::display::{DisplayedLine, DisplayedOption, FocusPanel, FocusShift};
use crate::history::HistoryStep;
use crate::ingest::apply_event;
use crate::intent::Intent;
use crate::overlay::ErrorOverlay;
use crate::session::Session;
use crate::transcript::{Transcript, TranscriptEntry};
use bubbles::DialogueError;

/// The app's complete state: owns the runtime session and whatever is
/// currently on screen.
pub struct AppState {
    session: Option<Session>,
    source: String,
    start_node: String,
    current_line: Option<DisplayedLine>,
    options: Vec<DisplayedOption>,
    focused_option: Option<usize>,
    transcript: Transcript,
    focus: FocusPanel,
    error_overlay: Option<ErrorOverlay>,
    history: Vec<HistoryStep>,
    recording: bool,
    quit_requested: bool,
}

impl AppState {
    /// Compiles `source` and starts the runner on `start_node`.
    ///
    /// # Errors
    /// Propagates any compile or start-time runtime error.
    pub fn from_source(source: &str, start_node: &str) -> Result<Self, DialogueError> {
        let session = Session::from_source(source, start_node)?;
        Ok(Self::new(Some(session), source, start_node, None))
    }

    /// Infallible counterpart to [`Self::from_source`]: on compile error the
    /// returned state carries an [`ErrorOverlay`] and has no active session.
    #[must_use]
    pub fn load(source: &str, start_node: &str) -> Self {
        match Session::from_source(source, start_node) {
            Ok(session) => Self::new(Some(session), source, start_node, None),
            Err(err) => {
                let overlay = ErrorOverlay::from_error(&err, Some(source));
                Self::new(None, source, start_node, Some(overlay))
            }
        }
    }

    fn new(
        session: Option<Session>,
        source: &str,
        start_node: &str,
        error_overlay: Option<ErrorOverlay>,
    ) -> Self {
        Self {
            session,
            source: source.to_owned(),
            start_node: start_node.to_owned(),
            current_line: None,
            options: Vec::new(),
            focused_option: None,
            transcript: Transcript::new(),
            focus: FocusPanel::Dialogue,
            error_overlay,
            history: Vec::new(),
            recording: true,
            quit_requested: false,
        }
    }

    /// `true` when there is a previous visible step to rewind to.
    #[must_use]
    pub const fn can_step_back(&self) -> bool {
        self.history.len() > 1
    }

    /// Replaces the source the next [`Intent::Reload`] will use.
    pub fn replace_source(&mut self, source: String) {
        self.source = source;
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

    /// The active error overlay, when one is pending.
    #[must_use]
    pub const fn error_overlay(&self) -> Option<&ErrorOverlay> {
        self.error_overlay.as_ref()
    }

    /// `true` if the state is in a non-playable error state - either an
    /// overlay is active or the runner was torn down by a failed load.
    #[must_use]
    pub const fn is_errored(&self) -> bool {
        self.error_overlay.is_some() || self.session.is_none()
    }

    /// `true` once the dialogue has finished, the state is errored, or the
    /// user has asked to quit.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.quit_requested
            || self.is_errored()
            || self.session.as_ref().is_some_and(Session::is_done)
    }

    /// `true` if the user has explicitly asked to quit.
    #[must_use]
    pub const fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// Applies a user intent to the state.
    ///
    /// Runtime errors from the dialogue runner are caught internally and
    /// converted into an [`ErrorOverlay`]; they are never propagated to the
    /// caller.  Inspect [`AppState::is_errored`] or
    /// [`AppState::error_overlay`] after this call if you need to react to
    /// runner failures.
    pub fn apply(&mut self, intent: Intent) {
        match intent {
            Intent::Advance => {
                if self.options.is_empty() {
                    self.run_advance();
                } else if let Some(idx) = self.focused_option {
                    self.run_commit(idx);
                    if !self.is_errored() && self.options.is_empty() {
                        self.run_advance();
                    }
                }
            }
            Intent::Quit => self.quit_requested = true,
            Intent::FocusNext => self.move_focus(FocusShift::Next),
            Intent::FocusPrev => self.move_focus(FocusShift::Prev),
            Intent::SelectOption(idx) => self.run_commit(idx),
            Intent::ToggleFocus => self.toggle_focus(),
            Intent::ScrollUp => self.transcript.scroll_up(),
            Intent::ScrollDown => self.transcript.scroll_down(),
            Intent::Reload => self.reload(),
            Intent::DismissError => self.error_overlay = None,
            Intent::StepBack => self.step_back(),
        }
    }

    fn run_advance(&mut self) {
        self.guarded(Self::advance);
        if !self.is_errored() && self.recording {
            self.history.push(HistoryStep::Advance);
        }
    }

    fn run_commit(&mut self, idx: usize) {
        let will_commit = self.options.get(idx).is_some_and(|o| o.available);
        self.guarded(|s| s.commit_option(idx));
        if will_commit && !self.is_errored() && self.recording {
            self.history.push(HistoryStep::SelectOption(idx));
        }
    }

    fn step_back(&mut self) {
        if !self.can_step_back() {
            return;
        }
        let mut history = std::mem::take(&mut self.history);
        history.pop();
        self.reset_runtime_state();
        self.recording = false;
        for step in &history {
            match *step {
                HistoryStep::Advance => self.guarded(Self::advance),
                HistoryStep::SelectOption(i) => self.guarded(|s| s.commit_option(i)),
            }
        }
        self.recording = true;
        self.history = history;
    }

    fn reset_runtime_state(&mut self) {
        match Session::from_source(&self.source, &self.start_node) {
            Ok(session) => {
                self.session = Some(session);
                self.error_overlay = None;
            }
            Err(err) => {
                self.error_overlay = Some(ErrorOverlay::from_error(&err, Some(&self.source)));
                self.session = None;
            }
        }
        self.transcript = Transcript::new();
        self.current_line = None;
        self.options.clear();
        self.focused_option = None;
    }

    fn guarded<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self) -> Result<(), DialogueError>,
    {
        if let Err(err) = f(self) {
            self.error_overlay = Some(ErrorOverlay::from_error(&err, Some(&self.source)));
            self.session = None;
            self.current_line = None;
            self.options.clear();
            self.focused_option = None;
        }
    }

    fn advance(&mut self) -> Result<(), DialogueError> {
        let Some(session) = self.session.as_mut() else {
            return Ok(());
        };
        self.current_line = None;
        while let Some(event) = session.next_event()? {
            if apply_event(
                &mut self.transcript,
                &mut self.current_line,
                &mut self.options,
                &mut self.focused_option,
                event,
            ) {
                return Ok(());
            }
        }
        Ok(())
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
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| DialogueError::ProtocolViolation("no active session".into()))?;
        session.select_option(index)?;
        self.transcript
            .push(TranscriptEntry::OptionChosen { text, index });
        self.options.clear();
        self.focused_option = None;
        self.current_line = None;
        Ok(())
    }

    fn reload(&mut self) {
        self.reset_runtime_state();
        self.history.clear();
    }
}
