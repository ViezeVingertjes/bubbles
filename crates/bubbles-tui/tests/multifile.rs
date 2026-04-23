//! Multi-file compilation tests: `AppState::from_sources` and `load_many`.
//!
//! These verify that compiling multiple `.bub` files together works end-to-end
//! through the TUI model: cross-file jumps, reload, rewind, and that error
//! overlays point to the correct source file.

use bubbles_tui::{AppState, Intent};

// Two minimal scripts that together form a complete program via a cross-file jump.
const MAIN: &str = "title: Main\n---\nHello from main.\n<<jump Other>>\n===\n";
const OTHER: &str = "title: Other\n---\nHello from other.\n===\n";

// A script with a parse error in a named file.
const BROKEN: &str = "title: Broken\n---\n<<if>>\n===\n";

#[test]
fn from_sources_compiles_two_files() {
    let sources = &[("main.bub", MAIN), ("other.bub", OTHER)];
    let state = AppState::from_sources(sources, "Main").expect("compile+start failed");
    assert!(!state.is_done());
}

#[test]
fn from_sources_drives_a_cross_file_jump() {
    let sources = &[("main.bub", MAIN), ("other.bub", OTHER)];
    let mut state = AppState::from_sources(sources, "Main").unwrap();

    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "Hello from main.");

    state.apply(Intent::Advance);
    assert_eq!(state.current_line().unwrap().text, "Hello from other.");
}

#[test]
fn load_many_with_valid_sources_has_no_error() {
    let sources = &[("main.bub", MAIN), ("other.bub", OTHER)];
    let state = AppState::load_many(sources, "Main");
    assert!(!state.is_errored());
    assert!(state.error_overlay().is_none());
}

#[test]
fn load_many_with_broken_source_shows_error_overlay() {
    let sources = &[("main.bub", MAIN), ("broken.bub", BROKEN)];
    let state = AppState::load_many(sources, "Main");
    assert!(state.is_errored());
    assert!(state.error_overlay().is_some());
}

#[test]
fn error_overlay_in_broken_file_references_correct_filename() {
    let sources = &[("main.bub", MAIN), ("broken.bub", BROKEN)];
    let state = AppState::load_many(sources, "Main");
    let overlay = state.error_overlay().expect("expected error overlay");
    if let Some(loc) = &overlay.location {
        assert!(
            loc.file.contains("broken"),
            "expected location to reference broken.bub, got: {:?}",
            loc.file
        );
    }
}

#[test]
fn error_overlay_for_broken_file_contains_source_excerpt() {
    let sources = &[("main.bub", MAIN), ("broken.bub", BROKEN)];
    let state = AppState::load_many(sources, "Main");
    let overlay = state.error_overlay().expect("expected error overlay");
    // The excerpt for a parse error on line 3 of broken.bub should be "<<if>>"
    assert_eq!(
        overlay.excerpt.as_deref(),
        Some("<<if>>"),
        "expected excerpt from broken.bub, got: {:?}",
        overlay.excerpt
    );
}

#[test]
fn reload_after_from_sources_restarts_from_beginning() {
    let sources = &[("main.bub", MAIN), ("other.bub", OTHER)];
    let mut state = AppState::from_sources(sources, "Main").unwrap();

    state.apply(Intent::Advance); // "Hello from main."
    assert_eq!(state.current_line().unwrap().text, "Hello from main.");

    state.apply(Intent::Reload);
    state.apply(Intent::Advance);
    assert_eq!(
        state.current_line().unwrap().text,
        "Hello from main.",
        "should restart from the beginning after reload"
    );
}

#[test]
fn rewind_works_after_from_sources() {
    let sources = &[("main.bub", MAIN), ("other.bub", OTHER)];
    let mut state = AppState::from_sources(sources, "Main").unwrap();

    state.apply(Intent::Advance); // "Hello from main."
    state.apply(Intent::Advance); // <<jump Other>> + "Hello from other."
    assert_eq!(state.current_line().unwrap().text, "Hello from other.");

    state.apply(Intent::StepBack);
    assert_eq!(
        state.current_line().unwrap().text,
        "Hello from main.",
        "step back should return to the previous line"
    );
}
