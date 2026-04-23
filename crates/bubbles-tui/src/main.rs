// Same rationale as in `lib.rs` - ratatui's transitive graph duplicates a
// handful of crate versions we can't de-dup from our side.
#![allow(clippy::multiple_crate_versions)]

//! Entry point for the `bubbles-tui` writer tool.
//!
//! Usage:
//!
//! ```text
//! bubbles-tui <file.bub> [StartNode]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, io};

use bubbles_tui::{AppState, Intent, render, terminal};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let _ = terminal::restore();
            eprintln!("bubbles-tui: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let source = fs::read_to_string(&args.path)
        .map_err(|e| format!("cannot read `{}`: {e}", args.path.display()))?;

    let mut state = AppState::from_source(&source, &args.start_node)?;
    let mut tui = terminal::init()?;

    let loop_result = event_loop(&mut state, &mut tui);

    let _ = terminal::restore();
    loop_result.map_err(Into::into)
}

fn event_loop(state: &mut AppState, tui: &mut terminal::Tui) -> io::Result<()> {
    loop {
        tui.draw(|f| render(state, f))?;
        if state.quit_requested() {
            return Ok(());
        }
        if let Some(intent) = terminal::next_intent()? {
            state.apply(intent);
            if matches!(intent, Intent::Quit) {
                return Ok(());
            }
        }
    }
}

struct Args {
    path: PathBuf,
    start_node: String,
}

fn parse_args() -> Result<Args, String> {
    let mut iter = std::env::args().skip(1);
    let path = iter
        .next()
        .ok_or_else(|| "usage: bubbles-tui <file.bub> [StartNode]".to_owned())?;
    let start_node = iter.next().unwrap_or_else(|| "Start".to_owned());
    Ok(Args {
        path: Path::new(&path).to_path_buf(),
        start_node,
    })
}
