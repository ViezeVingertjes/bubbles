//! Rerun tests: re-running from the start node while keeping runtime state.

use bubbles_tui::{AppState, Intent};

const WITH_ONCE: &str = "title: Start\n---\n<<once>>\n    NPC: First time here.\n<<else>>\n    NPC: Back again.\n<<endonce>>\n===\n";

const WITH_VAR: &str =
    "title: Start\n---\n<<declare $n = 0>>\n<<set $n = $n + 1>>\nCount {$n}\n===\n";

#[test]
fn rerun_preserves_once_seen_state() {
    let mut state = AppState::from_source(WITH_ONCE, "Start").unwrap();
    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "First time here.");

    state.apply(Intent::Advance); // finish
    state.apply(Intent::Rerun);
    state.apply(Intent::Advance);

    assert_eq!(state.current_line().unwrap().text, "Back again.");
}

#[test]
fn rerun_preserves_variables() {
    let mut state = AppState::from_source(WITH_VAR, "Start").unwrap();
    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "Count 1");

    state.apply(Intent::Advance); // finish
    state.apply(Intent::Rerun);
    state.apply(Intent::Advance);

    assert_eq!(state.current_line().unwrap().text, "Count 2");
}

#[test]
fn rerun_clears_transcript_and_history() {
    let mut state = AppState::from_source(WITH_ONCE, "Start").unwrap();
    state.apply(Intent::Advance); // line 1
    state.apply(Intent::Advance); // finish
    assert!(!state.transcript().is_empty());
    assert!(state.can_step_back());

    state.apply(Intent::Rerun);

    assert!(state.transcript().is_empty());
    assert!(!state.can_step_back());
}

#[test]
fn reload_still_resets_once_seen_state() {
    let mut state = AppState::from_source(WITH_ONCE, "Start").unwrap();
    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "First time here.");

    state.apply(Intent::Advance);
    state.apply(Intent::Reload);
    state.apply(Intent::Advance);

    assert_eq!(state.current_line().unwrap().text, "First time here.");
}
