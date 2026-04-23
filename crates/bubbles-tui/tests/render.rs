//! Render tests use ratatui's `TestBackend` so no real terminal is needed.

use bubbles_tui::{AppState, Intent, render};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn line_text_appears_in_rendered_buffer() {
    let mut state =
        AppState::from_source("title: A\n---\nAlice: Hello there.\n===\n", "A").unwrap();
    state.apply(Intent::Advance);

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(&state, f)).unwrap();

    let content = buffer_text(&terminal);
    assert!(content.contains("Alice"), "speaker missing: {content:?}");
    assert!(
        content.contains("Hello there."),
        "line text missing: {content:?}"
    );
}

#[test]
fn idle_state_renders_a_hint() {
    let state = AppState::from_source("title: A\n---\nOne.\n===\n", "A").unwrap();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(&state, f)).unwrap();

    let content = buffer_text(&terminal);
    // Nothing has been advanced yet, so some pre-play hint should be visible.
    assert!(
        content.to_lowercase().contains("press"),
        "expected an idle hint mentioning 'press', got: {content:?}"
    );
}
