//! `bubbles play` - the interactive terminal player.
//!
//! The whole verb sits behind the `tui` feature.  With the feature off the
//! binary still builds and `bubbles check` still works; `play` just says so.

use std::process::ExitCode;

use crate::Fault;

#[cfg(not(feature = "tui"))]
pub fn run(_args: &[String]) -> Result<ExitCode, Fault> {
    Err(Fault::Fatal(
        "this build has no terminal UI (compiled with --no-default-features); \
         `bubbles check` works, and the default build adds `bubbles play`"
            .to_owned(),
    ))
}

#[cfg(feature = "tui")]
pub fn run(args: &[String]) -> Result<ExitCode, Fault> {
    use bubbles_cli::{AppState, terminal};

    let (paths, start_node) = split_args(args)?;
    let sources = crate::read_sources(&paths)?;

    let mut state =
        AppState::from_source_set(sources, &start_node).map_err(|e| Fault::Fatal(e.to_string()))?;
    let mut tui = terminal::init().map_err(|e| Fault::Fatal(e.to_string()))?;

    let outcome = event_loop(&mut state, &mut tui);

    let _ = terminal::restore();
    outcome.map_err(|e| Fault::Fatal(e.to_string()))?;
    Ok(ExitCode::SUCCESS)
}

/// Draws, then applies one intent, until the state asks to quit.
///
/// The quit check reads the state rather than the intent, because `Esc`/`q`
/// closes the key list when it is open instead of quitting.
#[cfg(feature = "tui")]
fn event_loop(
    state: &mut bubbles_cli::AppState,
    tui: &mut bubbles_cli::terminal::Tui,
) -> std::io::Result<()> {
    use bubbles_cli::{render, terminal};

    loop {
        tui.draw(|f| render(state, f))?;
        if state.quit_requested() {
            return Ok(());
        }
        if let Some(intent) = terminal::next_intent()? {
            state.apply(intent);
            if state.quit_requested() {
                return Ok(());
            }
        }
    }
}

/// Splits `args` into source paths and the start node.
///
/// The optional trailing start-node argument is the one that does not have a
/// `.bub` extension.
#[cfg(feature = "tui")]
fn split_args(args: &[String]) -> Result<(Vec<std::path::PathBuf>, String), Fault> {
    let is_bub = |s: &String| {
        std::path::Path::new(s)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bub"))
    };
    let (start_node, file_args) = match args.split_last() {
        Some((last, rest)) if !is_bub(last) => (last.clone(), rest),
        _ => ("Start".to_owned(), args),
    };
    if file_args.is_empty() {
        return Err(Fault::Usage(
            "play: expected at least one .bub file".to_owned(),
        ));
    }
    Ok((
        file_args.iter().map(std::path::PathBuf::from).collect(),
        start_node,
    ))
}
