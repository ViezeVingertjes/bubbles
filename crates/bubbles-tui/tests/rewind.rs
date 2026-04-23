//! Rewind tests: stepping backward through the visible dialogue history.

use bubbles_tui::{AppState, Intent};

const THREE_LINES: &str = "title: A\n---\nOne.\nTwo.\nThree.\n===\n";
const BRANCH: &str =
    "title: A\n---\nPick.\n-> Left\n    Alice: left.\n-> Right\n    Alice: right.\n===\n";
const TWO_NODES: &str = "title: A\n---\nA-line.\n<<jump B>>\n===\ntitle: B\n---\nB-line.\n===\n";

#[test]
fn step_back_from_second_line_returns_to_first() {
    let mut state = AppState::from_source(THREE_LINES, "A").unwrap();
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "One.");
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "Two.");

    assert!(state.can_step_back());
    state.apply(Intent::StepBack).unwrap();

    assert_eq!(state.current_line().unwrap().text, "One.");
}

#[test]
fn step_back_at_the_first_line_is_a_noop() {
    let mut state = AppState::from_source(THREE_LINES, "A").unwrap();
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "One.");
    assert!(!state.can_step_back());

    state.apply(Intent::StepBack).unwrap();
    assert_eq!(state.current_line().unwrap().text, "One.");
}

#[test]
fn step_back_undoes_an_option_choice_and_shows_the_prompt_again() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    for _ in 0..4 {
        if !state.options().is_empty() {
            break;
        }
        state.apply(Intent::Advance).unwrap();
    }
    assert!(!state.options().is_empty(), "expected option prompt");
    state.apply(Intent::SelectOption(1)).unwrap();
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "right.");

    // Two steps back: undo "advance to right line" + undo "pick option 1".
    state.apply(Intent::StepBack).unwrap();
    state.apply(Intent::StepBack).unwrap();

    assert_eq!(state.options().len(), 2, "options should reappear");
}

#[test]
fn step_back_crosses_node_boundaries() {
    let mut state = AppState::from_source(TWO_NODES, "A").unwrap();
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "A-line.");
    state.apply(Intent::Advance).unwrap();
    assert_eq!(state.current_line().unwrap().text, "B-line.");

    state.apply(Intent::StepBack).unwrap();
    assert_eq!(state.current_line().unwrap().text, "A-line.");
}

#[test]
fn step_back_also_rewinds_the_transcript() {
    let mut state = AppState::from_source(THREE_LINES, "A").unwrap();
    state.apply(Intent::Advance).unwrap(); // One.
    state.apply(Intent::Advance).unwrap(); // Two.
    state.apply(Intent::Advance).unwrap(); // Three.
    let before_len = state.transcript().len();
    assert!(before_len > 0);

    state.apply(Intent::StepBack).unwrap();
    assert!(state.transcript().len() < before_len);
    assert_eq!(state.current_line().unwrap().text, "Two.");
}
