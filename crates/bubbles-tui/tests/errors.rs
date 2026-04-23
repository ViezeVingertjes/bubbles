//! Error-overlay tests: compile errors, runtime errors, and the dismiss/reload
//! flow.

use bubbles::{DialogueError, HashMapStorage, Runner, compile};
use bubbles_tui::{AppState, ErrorOverlay, Intent, render};
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

// ── ErrorOverlay::from_error — one test per DialogueError variant ─────────────
//
// These drive the overlay directly from constructed errors so every match arm
// in overlay.rs is exercised even when a particular error path is hard to
// trigger end-to-end.

fn overlay(err: &DialogueError) -> ErrorOverlay {
    ErrorOverlay::from_error(err, None)
}

#[test]
fn from_error_duplicate_node() {
    let o = overlay(&DialogueError::DuplicateNode("Foo".into()));
    assert!(
        o.title.to_lowercase().contains("duplicate"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("Foo"));
    assert!(o.location.is_none());
}

#[test]
fn from_error_validation() {
    let o = overlay(&DialogueError::Validation("jump target missing".into()));
    assert!(
        o.title.to_lowercase().contains("validation"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("jump target missing"));
}

#[test]
fn from_error_runtime() {
    let o = overlay(&DialogueError::Runtime("something went wrong".into()));
    assert!(
        o.title.to_lowercase().contains("runtime"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("something went wrong"));
}

#[test]
fn from_error_type() {
    let o = overlay(&DialogueError::Type("cannot add".into()));
    assert!(o.title.to_lowercase().contains("type"), "got {:?}", o.title);
    assert!(o.message.contains("cannot add"));
}

#[test]
fn from_error_undefined_variable() {
    let o = overlay(&DialogueError::UndefinedVariable("$hp".into()));
    assert!(
        o.title.to_lowercase().contains("undefined") || o.title.to_lowercase().contains("variable"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("$hp"));
}

#[test]
fn from_error_function() {
    let o = overlay(&DialogueError::Function {
        name: "roll".into(),
        message: "bad args".into(),
    });
    assert!(
        o.title.to_lowercase().contains("function"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("roll"));
    assert!(o.message.contains("bad args"));
}

#[test]
fn from_error_protocol_violation() {
    let o = overlay(&DialogueError::ProtocolViolation(
        "call select_option first".into(),
    ));
    assert!(
        o.title.to_lowercase().contains("protocol"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("select_option"));
}

#[test]
fn from_error_type_mismatch() {
    let o = overlay(&DialogueError::TypeMismatch {
        expected: "number".into(),
        got: "string".into(),
        context: "operator `+`".into(),
    });
    assert!(
        o.title.to_lowercase().contains("type") || o.title.to_lowercase().contains("mismatch"),
        "got {:?}",
        o.title
    );
    assert!(o.message.contains("number"));
    assert!(o.message.contains("string"));
}

#[test]
fn from_error_parse_with_source_attaches_excerpt() {
    let source = "title: A\n---\n<<if>>\n===\n";
    let err = DialogueError::Parse {
        file: "<source>".into(),
        line: 3,
        message: "unexpected token".into(),
    };
    let o = ErrorOverlay::from_error(&err, Some(source));
    assert!(
        o.excerpt.is_some(),
        "expected excerpt for a parse error with source"
    );
    assert_eq!(o.excerpt.as_deref(), Some("<<if>>"));
}

#[test]
fn from_error_parse_without_source_has_no_excerpt() {
    let err = DialogueError::Parse {
        file: "<source>".into(),
        line: 3,
        message: "unexpected token".into(),
    };
    let o = ErrorOverlay::from_error(&err, None);
    assert!(o.excerpt.is_none());
}

#[test]
fn location_string_formats_file_and_line() {
    let err = DialogueError::Parse {
        file: "dialogue.bub".into(),
        line: 7,
        message: "oops".into(),
    };
    let o = ErrorOverlay::from_error(&err, None);
    assert_eq!(o.location_string().as_deref(), Some("dialogue.bub:7"));
}

// ── runtime trigger: type mismatch populates overlay ─────────────────────────

#[test]
fn type_mismatch_in_runner_populates_overlay() {
    // Adding a number to a string at runtime → TypeMismatch → overlay.
    let src = "title: A\n---\n<<set $x = 1 + \"oops\">>\n===\n";
    let mut state = AppState::load(src, "A");
    assert!(state.error_overlay().is_none(), "should compile cleanly");

    state.apply(Intent::Advance); // NodeStarted
    state.apply(Intent::Advance); // triggers the set → type mismatch

    let overlay = state
        .error_overlay()
        .expect("expected TypeMismatch overlay");
    assert!(
        overlay.title.to_lowercase().contains("type"),
        "got {:?}",
        overlay.title
    );
}

#[test]
fn protocol_violation_in_runner_populates_overlay() {
    // Call next_event while awaiting option → ProtocolViolation → overlay.
    let src = "title: A\n---\n-> Only option\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap(); // NodeStarted
    runner.next_event().unwrap(); // Options — now awaiting

    // Build a DialogueError directly from what we know the runner produces.
    let err = runner.next_event().unwrap_err();
    assert!(
        matches!(err, DialogueError::ProtocolViolation(_)),
        "expected ProtocolViolation, got: {err:?}"
    );
    let o = ErrorOverlay::from_error(&err, None);
    assert!(
        o.title.to_lowercase().contains("protocol"),
        "got {:?}",
        o.title
    );
}
