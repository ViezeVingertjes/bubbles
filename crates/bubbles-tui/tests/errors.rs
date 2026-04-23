//! Error-overlay tests: compile errors, runtime errors, and the dismiss/reload
//! flow.

use bubbles_tui::{AppState, Intent, render};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const BAD_IF: &str = "title: A\n---\n<<if>>\n===\n";
const BAD_JUMP: &str = "title: A\n---\nHi.\n<<jump Nowhere>>\n===\n";
const GOOD: &str = "title: A\n---\nHi.\n===\n";

#[test]
fn load_captures_parse_errors_with_file_and_line() {
    let state = AppState::load(BAD_IF, "A");

    let overlay = state
        .error_overlay()
        .expect("expected an overlay for a malformed script");

    assert!(
        overlay.title.to_lowercase().contains("parse"),
        "overlay title should mention parse, got {:?}",
        overlay.title
    );
    let loc = overlay
        .location
        .as_ref()
        .expect("parse errors should carry a file+line location");
    assert_eq!(loc.line, 3, "location.line was {:?}", loc.line);
    assert!(
        !overlay.message.is_empty(),
        "overlay message should not be empty"
    );

    // The state exists, but playback is not possible.
    assert!(state.current_line().is_none());
    assert!(state.options().is_empty());
    assert!(state.is_errored());
}

#[test]
fn runtime_errors_during_advance_populate_the_overlay() {
    let mut state = AppState::load(BAD_JUMP, "A");
    assert!(state.error_overlay().is_none(), "script compiles cleanly");

    // Advance past the line - the next advance hits the bogus <<jump>>.
    state.apply(Intent::Advance); // surfaces "Hi."
    state.apply(Intent::Advance); // triggers the jump

    let overlay = state
        .error_overlay()
        .expect("runtime error should populate the overlay");
    assert!(
        overlay.title.to_lowercase().contains("runtime")
            || overlay.title.to_lowercase().contains("unknown"),
        "title was {:?}",
        overlay.title
    );
    assert!(overlay.message.to_lowercase().contains("nowhere"));
    assert!(state.is_errored());
}

#[test]
fn reload_clears_a_transient_error_when_source_becomes_valid() {
    let mut state = AppState::load(BAD_IF, "A");
    assert!(state.error_overlay().is_some());

    // Swap in a good source and reload.
    state.replace_source(GOOD.to_owned());
    state.apply(Intent::Reload);

    assert!(state.error_overlay().is_none());
    assert!(!state.is_errored());

    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "Hi.");
}

#[test]
fn dismiss_hides_the_overlay_without_reloading() {
    let mut state = AppState::load(BAD_IF, "A");
    assert!(state.error_overlay().is_some());

    state.apply(Intent::DismissError);
    assert!(state.error_overlay().is_none());
    // Still errored until we successfully reload - there is no session to
    // drive.
    assert!(state.is_errored());
}

#[test]
fn error_overlay_is_drawn_on_top_of_the_dialogue_pane() {
    let state = AppState::load(BAD_IF, "A");

    let backend = TestBackend::new(100, 20);
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
        content.to_lowercase().contains("error"),
        "rendered buffer should include an error marker: {content:?}"
    );
    assert!(
        content.contains(":3"),
        "rendered buffer should include the :line location: {content:?}"
    );
}
