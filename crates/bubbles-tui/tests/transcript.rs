//! Transcript-panel tests: the running log of everything the runner has
//! emitted during this session.

use bubbles_tui::{AppState, Intent, TranscriptEntry, render};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const THREE_LINES_AND_A_COMMAND: &str =
    "title: A\n---\nOne.\nTwo.\n<<play_sfx beep tag1>>\nThree.\n===\n";

const BRANCH: &str =
    "title: A\n---\nPick.\n-> Left\n    Alice: left.\n-> Right\n    Alice: right.\n===\n";

fn advance_until_done(state: &mut AppState, max_steps: usize) {
    for _ in 0..max_steps {
        state.apply(Intent::Advance);
        if state.is_done() {
            return;
        }
    }
}

#[test]
fn transcript_records_nodes_lines_and_commands_in_order() {
    let mut state = AppState::from_source(THREE_LINES_AND_A_COMMAND, "A").unwrap();
    advance_until_done(&mut state, 16);

    let entries = state.transcript().to_vec();
    assert!(
        matches!(entries.first(), Some(TranscriptEntry::NodeStarted(name)) if name == "A"),
        "first entry should be NodeStarted(\"A\"), got: {entries:?}"
    );
    assert!(
        matches!(entries.last(), Some(TranscriptEntry::NodeComplete(name)) if name == "A"),
        "last entry should be NodeComplete(\"A\"), got: {entries:?}"
    );

    let lines: Vec<&str> = entries
        .iter()
        .filter_map(|e| match e {
            TranscriptEntry::Line { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(lines, vec!["One.", "Two.", "Three."]);

    let cmd = entries
        .iter()
        .find_map(|e| match e {
            TranscriptEntry::Command { name, args, tags } => Some((name, args, tags)),
            _ => None,
        })
        .expect("expected a Command entry");
    assert_eq!(cmd.0, "play_sfx");
    assert_eq!(cmd.1, &vec!["beep".to_string(), "tag1".to_string()]);
    assert!(cmd.2.is_empty());
}

#[test]
fn choosing_an_option_is_recorded_in_the_transcript() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    for _ in 0..4 {
        if !state.options().is_empty() {
            break;
        }
        state.apply(Intent::Advance);
    }
    assert!(!state.options().is_empty(), "expected option prompt");

    state.apply(Intent::SelectOption(1));

    let chosen = state
        .transcript()
        .iter()
        .find_map(|e| match e {
            TranscriptEntry::OptionChosen { text, index } => Some((text.as_str(), *index)),
            _ => None,
        })
        .expect("OptionChosen entry missing");
    assert_eq!(chosen, ("Right", 1));
}

#[test]
fn transcript_appears_in_rendered_buffer() {
    let mut state = AppState::from_source(THREE_LINES_AND_A_COMMAND, "A").unwrap();
    for _ in 0..3 {
        state.apply(Intent::Advance);
    }

    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(&state, f)).unwrap();

    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();

    assert!(
        content.contains("transcript"),
        "transcript pane title missing: {content:?}"
    );
    assert!(
        content.contains("One."),
        "previous line missing from transcript: {content:?}"
    );
}

// ── rendered-buffer checks for NodeComplete and OptionChosen ─────────────────
//
// These exercise the format_entry arms in ui/transcript.rs that were not
// previously hit by any render test.

fn render_transcript(state: &AppState) -> String {
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(state, f)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn node_complete_marker_appears_in_rendered_buffer() {
    let mut state = AppState::from_source(THREE_LINES_AND_A_COMMAND, "A").unwrap();
    advance_until_done(&mut state, 16);
    assert!(state.is_done(), "expected dialogue to be done");

    let content = render_transcript(&state);
    // NodeComplete is rendered as "← A" (left-arrow + node name).
    assert!(
        content.contains('\u{2190}'),
        "NodeComplete left-arrow marker missing from rendered buffer"
    );
}

#[test]
fn option_chosen_marker_appears_in_rendered_buffer() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    // Advance until we reach the option prompt.
    for _ in 0..8 {
        if !state.options().is_empty() {
            break;
        }
        state.apply(Intent::Advance);
    }
    state.apply(Intent::SelectOption(0));
    advance_until_done(&mut state, 8);

    let content = render_transcript(&state);
    // OptionChosen is rendered as "→ chose [0] …" (right-arrow marker).
    assert!(
        content.contains("chose"),
        "OptionChosen 'chose' text missing from rendered buffer: {content:?}"
    );
}

#[test]
fn toggle_focus_and_scroll_up_shows_older_entries() {
    let mut state = AppState::from_source(THREE_LINES_AND_A_COMMAND, "A").unwrap();
    advance_until_done(&mut state, 16);

    assert_eq!(state.transcript_scroll(), 0);
    state.apply(Intent::ToggleFocus);
    assert!(state.transcript_focused());

    state.apply(Intent::ScrollUp);
    state.apply(Intent::ScrollUp);
    assert_eq!(state.transcript_scroll(), 2);

    state.apply(Intent::ScrollDown);
    assert_eq!(state.transcript_scroll(), 1);

    state.apply(Intent::ToggleFocus);
    assert!(!state.transcript_focused());
}
