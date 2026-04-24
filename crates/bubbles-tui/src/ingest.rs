//! Translation layer from raw [`DialogueEvent`]s into the app's display
//! and transcript state.

use bubbles::DialogueEvent;

use crate::display::{DisplayedLine, DisplayedOption};
use crate::transcript::{Transcript, TranscriptEntry};

/// Feeds `event` into the transcript and the displayed line/options buffers.
///
/// Returns `true` when the caller should stop pulling events (a line or
/// option prompt is now visible and awaits user input).
pub fn apply_event(
    transcript: &mut Transcript,
    current_line: &mut Option<DisplayedLine>,
    options: &mut Vec<DisplayedOption>,
    focused_option: &mut Option<usize>,
    event: DialogueEvent,
) -> bool {
    match event {
        DialogueEvent::NodeStarted(name) => {
            transcript.push(TranscriptEntry::NodeStarted(name));
            false
        }
        DialogueEvent::NodeComplete(name) => {
            transcript.push(TranscriptEntry::NodeComplete(name));
            false
        }
        DialogueEvent::Line {
            speaker,
            text,
            tags,
            line_id,
            spans,
        } => {
            transcript.push(TranscriptEntry::Line {
                speaker: speaker.clone(),
                text: text.clone(),
                spans: spans.clone(),
            });
            *current_line = Some(DisplayedLine {
                speaker,
                text,
                spans,
                line_id,
                tags,
            });
            true
        }
        DialogueEvent::Options(opts) => {
            *options = opts.into_iter().map(DisplayedOption::from).collect();
            *focused_option = options
                .iter()
                .position(|o| o.available)
                .or(Some(0))
                .filter(|_| !options.is_empty());
            true
        }
        DialogueEvent::Command { name, args, tags } => {
            transcript.push(TranscriptEntry::Command { name, args, tags });
            false
        }
        // `DialogueComplete` and any future non-exhaustive variant are
        // handled by `Session::is_done()`; nothing to record here.
        _ => false,
    }
}
