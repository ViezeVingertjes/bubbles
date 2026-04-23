//! [`AppState`] - the pure model the renderer and the input layer agree on.
//!
//! Nothing in this module does I/O.  Tests build an `AppState` from a source
//! string, push [`Intent`]s through [`AppState::apply`], and inspect the
//! result.  The real binary does the same thing, just with key events
//! translated into intents upstream.

mod build;

use crate::display::{DisplayedLine, DisplayedOption, FocusPanel, FocusShift};
use crate::history::HistoryStep;
use crate::ingest::apply_event;
use crate::intent::Intent;
use crate::overlay::ErrorOverlay;
use crate::session::Session;
use crate::source_set::SourceSet;
use crate::transcript::{Transcript, TranscriptEntry};
use bubbles::DialogueError;

/// The app's complete state: owns the runtime session and whatever is
/// currently on screen.
pub struct AppState {
    pub(super) session: Option<Session>,
    pub(super) sources: SourceSet,
    pub(super) start_node: String,
    pub(super) current_line: Option<DisplayedLine>,
    pub(super) options: Vec<DisplayedOption>,
    pub(super) focused_option: Option<usize>,
    pub(super) transcript: Transcript,
    pub(super) focus: FocusPanel,
    pub(super) error_overlay: Option<ErrorOverlay>,
    pub(super) history: Vec<HistoryStep>,
    pub(super) recording: bool,
    pub(super) quit_requested: bool,
}

impl AppState {
    /// `true` when there is a previous visible step to rewind to.
    #[must_use]
    pub const fn can_step_back(&self) -> bool {
        self.history.len() > 1
    }

    /// Replaces the single source the next [`Intent::Reload`] will use.
    ///
    /// For multi-file workflows use [`Self::replace_sources`].
    pub fn replace_source(&mut self, source: String) {
        self.sources = SourceSet::single("<source>", source);
    }

    /// Replaces the full source set the next [`Intent::Reload`] will use.
    pub fn replace_sources(&mut self, sources: SourceSet) {
        self.sources = sources;
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

    /// `true` when the transcript pane has keyboard focus (arrow keys scroll).
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
            Intent::Restart => self.do_restart(),
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
        match Session::from_source_set(&self.sources, &self.start_node) {
            Ok(session) => {
                self.session = Some(session);
                self.error_overlay = None;
            }
            Err(err) => {
                let excerpt = self.sources.source_for_error(&err);
                self.error_overlay = Some(ErrorOverlay::from_error(&err, excerpt));
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
            let excerpt = self.sources.source_for_error(&err);
            self.error_overlay = Some(ErrorOverlay::from_error(&err, excerpt));
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
            FocusPanel::Options => self.shift_option_focus(delta),
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
            FocusPanel::Options => FocusPanel::Transcript,
            FocusPanel::Transcript => FocusPanel::Options,
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

    fn do_restart(&mut self) {
        let result = self.session.as_mut().map(|s| s.restart(&self.start_node));

        match result {
            Some(Ok(())) => self.error_overlay = None,
            Some(Err(err)) => {
                let excerpt = self.sources.source_for_error(&err);
                self.error_overlay = Some(ErrorOverlay::from_error(&err, excerpt));
                self.session = None;
            }
            None => {
                // No active session (compile error on load) - full reload.
                self.reset_runtime_state();
            }
        }
        self.transcript = Transcript::new();
        self.current_line = None;
        self.options.clear();
        self.focused_option = None;
        self.history.clear();
    }
}
