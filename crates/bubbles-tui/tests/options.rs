//! Option-handling tests: focus, direct selection, guard rendering.

use bubbles_tui::{AppState, Intent, render};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const BRANCH: &str =
    "title: A\n---\nPick one.\n-> Left\n    Alice: left.\n-> Right\n    Alice: right.\n===\n";

const GUARDED: &str = "title: A\n---\n<<declare $ok = false>>\nGo on.\n-> Enabled path\n    Alice: through!\n-> Locked path <<if $ok>>\n    Alice: unreachable.\n===\n";

fn advance_until_options(state: &mut AppState) {
    for _ in 0..16 {
        state.apply(Intent::Advance).unwrap();
        if !state.options().is_empty() {
            return;
        }
    }
    panic!("never reached an options prompt");
}

#[test]
fn advancing_into_a_branch_exposes_the_option_list() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    advance_until_options(&mut state);

    let opts = state.options();
    assert_eq!(opts.len(), 2);
    assert_eq!(opts[0].text, "Left");
    assert_eq!(opts[1].text, "Right");
    assert!(opts.iter().all(|o| o.available));
}

#[test]
fn focus_next_wraps_and_select_option_commits_by_index() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    advance_until_options(&mut state);

    assert_eq!(state.focused_option(), Some(0));
    state.apply(Intent::FocusNext).unwrap();
    assert_eq!(state.focused_option(), Some(1));
    state.apply(Intent::FocusNext).unwrap();
    assert_eq!(state.focused_option(), Some(0), "focus should wrap");
    state.apply(Intent::FocusPrev).unwrap();
    assert_eq!(state.focused_option(), Some(1));

    state.apply(Intent::SelectOption(1)).unwrap();
    assert!(state.options().is_empty());

    state.apply(Intent::Advance).unwrap();
    let line = state.current_line().expect("expected line after selection");
    assert_eq!(line.speaker.as_deref(), Some("Alice"));
    assert_eq!(line.text, "right.");
}

#[test]
fn advance_commits_the_focused_option_when_options_are_showing() {
    let mut state = AppState::from_source(BRANCH, "A").unwrap();
    advance_until_options(&mut state);

    state.apply(Intent::FocusNext).unwrap(); // move focus to index 1
    state.apply(Intent::Advance).unwrap();

    let line = state.current_line().expect("expected a line after advance");
    assert_eq!(line.text, "right.");
}

#[test]
fn selecting_an_unavailable_option_is_a_noop() {
    let mut state = AppState::from_source(GUARDED, "A").unwrap();
    advance_until_options(&mut state);

    assert_eq!(state.options().len(), 2);
    assert!(state.options()[0].available);
    assert!(!state.options()[1].available);

    // Try to pick the locked one: the call returns Ok and options stay up.
    state.apply(Intent::SelectOption(1)).unwrap();
    assert_eq!(
        state.options().len(),
        2,
        "unavailable option should not advance the runner"
    );

    state.apply(Intent::SelectOption(0)).unwrap();
    assert!(state.options().is_empty());
}

#[test]
fn options_render_with_focus_marker_and_unavailable_marker() {
    let mut state = AppState::from_source(GUARDED, "A").unwrap();
    advance_until_options(&mut state);

    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(&state, f)).unwrap();

    let buffer = terminal.backend().buffer();
    let content: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();

    assert!(
        content.contains("Enabled path"),
        "first option text missing: {content:?}"
    );
    assert!(
        content.contains("Locked path"),
        "second option text missing: {content:?}"
    );
    // Focus marker on the first option (default focus).
    assert!(
        content.contains("> 1. Enabled path"),
        "focus marker missing: {content:?}"
    );
    // Unavailable marker on the guarded option.
    assert!(
        content.contains("\u{2717}") || content.contains("(locked)"),
        "unavailable marker missing: {content:?}"
    );
}
