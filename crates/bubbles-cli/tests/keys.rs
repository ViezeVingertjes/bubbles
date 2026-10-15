//! Keybinding tests.  The reload/rerun pair used to be `r` and `R`, the one
//! case-sensitive binding in a tool where every other letter is
//! case-insensitive - and the lowercase half silently discarded every
//! variable.  These tests pin the replacement down.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use bubbles_cli::{AppState, Intent, KEYS, render, terminal::key_to_intent};

const fn press(code: KeyCode) -> Option<Intent> {
    key_to_intent(KeyEvent::new(code, KeyModifiers::NONE))
}

fn draw(state: &AppState) -> String {
    let mut terminal = Terminal::new(TestBackend::new(110, 20)).expect("test backend");
    terminal.draw(|f| render(state, f)).expect("draw");
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

const fn ctrl(c: char) -> Option<Intent> {
    key_to_intent(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}

#[test]
fn every_bound_letter_is_case_insensitive() {
    for (lower, upper) in [
        ('q', 'Q'),
        ('j', 'J'),
        ('k', 'K'),
        ('b', 'B'),
        ('x', 'X'),
        ('r', 'R'),
    ] {
        assert_eq!(
            press(KeyCode::Char(lower)),
            press(KeyCode::Char(upper)),
            "`{lower}` and `{upper}` must do the same thing"
        );
    }
}

#[test]
fn the_destructive_reload_is_not_on_a_bare_letter() {
    assert_eq!(press(KeyCode::Char('r')), Some(Intent::Rerun));
    assert_eq!(press(KeyCode::Char('R')), Some(Intent::Rerun));

    assert_eq!(ctrl('r'), Some(Intent::Reload));
    assert_eq!(ctrl('R'), Some(Intent::Reload));
    assert_eq!(press(KeyCode::F(5)), Some(Intent::Reload));
}

#[test]
fn control_does_not_leak_into_the_other_bindings() {
    assert_eq!(ctrl('q'), None);
    assert_eq!(ctrl('j'), None);
}

#[test]
fn question_mark_toggles_the_key_list() {
    assert_eq!(press(KeyCode::Char('?')), Some(Intent::ToggleHelp));

    let mut state = AppState::load("title: A\n---\nHi.\n===\n", "A");
    assert!(!state.help_visible());
    state.apply(Intent::ToggleHelp);
    assert!(state.help_visible());
    state.apply(Intent::ToggleHelp);
    assert!(!state.help_visible());
}

#[test]
fn quit_closes_the_key_list_before_it_quits() {
    let mut state = AppState::load("title: A\n---\nHi.\n===\n", "A");
    state.apply(Intent::ToggleHelp);

    state.apply(Intent::Quit);
    assert!(!state.help_visible());
    assert!(
        !state.quit_requested(),
        "the first Esc only closed the list"
    );

    state.apply(Intent::Quit);
    assert!(state.quit_requested());
}

#[test]
fn the_key_list_names_reload_as_destructive() {
    let reload = KEYS
        .iter()
        .find(|(keys, _)| keys.contains("Ctrl+R"))
        .expect("the key list must document the reload binding");
    assert!(
        reload.1.contains("DESTRUCTIVE"),
        "reload must be labelled destructive: {}",
        reload.1
    );
}

#[test]
fn the_footer_warns_that_reload_wipes_variables() {
    let mut state = AppState::load("title: A\n---\nHi.\n===\n", "A");
    state.apply(Intent::Advance);

    let screen = draw(&state);
    assert!(
        screen.contains("Ctrl+R: reload (wipes variables)"),
        "{screen}"
    );
    assert!(screen.contains("?: keys"), "{screen}");
}

#[test]
fn the_key_list_is_drawn_when_it_is_open() {
    let mut state = AppState::load("title: A\n---\nHi.\n===\n", "A");
    state.apply(Intent::Advance);
    assert!(!draw(&state).contains("DESTRUCTIVE"));

    state.apply(Intent::ToggleHelp);
    let screen = draw(&state);
    assert!(screen.contains("DESTRUCTIVE"), "{screen}");
    assert!(screen.contains("Keys"), "{screen}");
}

#[test]
fn the_key_list_covers_every_binding_the_footer_cannot_show() {
    for expected in ["Enter", "Tab", "PageUp", "Backspace", "?", "Esc"] {
        assert!(
            KEYS.iter().any(|(keys, _)| keys.contains(expected)),
            "the key list is missing `{expected}`"
        );
    }
}
