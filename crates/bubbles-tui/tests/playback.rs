//! Playback-model tests: drive `AppState` through `Intent`s with no terminal
//! involved - the whole point of the model/view split.

use bubbles_tui::{AppState, Intent};

const ONE_LINE: &str = "title: A\n---\nHi.\n===\n";
const SPEAKER_LINE: &str = "title: A\n---\nAlice: Hi.\n===\n";
const TWO_LINES: &str = "title: A\n---\nOne.\nTwo.\n===\n";

#[test]
fn advance_reveals_first_line() {
    let mut state = AppState::from_source(ONE_LINE, "A").expect("compile+start failed");
    state.apply(Intent::Advance);

    let line = state.current_line().expect("expected a visible line");
    assert_eq!(line.text, "Hi.");
    assert_eq!(line.speaker, None);
}

#[test]
fn speaker_is_captured_on_the_line() {
    let mut state = AppState::from_source(SPEAKER_LINE, "A").unwrap();
    state.apply(Intent::Advance);

    let line = state.current_line().expect("expected a visible line");
    assert_eq!(line.speaker.as_deref(), Some("Alice"));
    assert_eq!(line.text, "Hi.");
}

#[test]
fn advance_past_the_last_line_marks_dialogue_complete() {
    let mut state = AppState::from_source(ONE_LINE, "A").unwrap();
    assert!(!state.is_done());

    state.apply(Intent::Advance); // surface "Hi."
    assert!(!state.is_done());

    state.apply(Intent::Advance); // consume it; runner closes
    assert!(state.is_done());
    assert!(state.current_line().is_none());
}

#[test]
fn advance_progresses_through_multiple_lines() {
    let mut state = AppState::from_source(TWO_LINES, "A").unwrap();
    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "One.");

    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "Two.");

    state.apply(Intent::Advance);
    assert!(state.is_done());
}
