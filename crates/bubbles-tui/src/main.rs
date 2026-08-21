// Same rationale as in `lib.rs` - ratatui's transitive graph duplicates a
// handful of crate versions we can't de-dup from our side.
#![allow(clippy::multiple_crate_versions)]

//! Entry point for the `bubbles-tui` writer tool.
//!
//! Usage:
//!
//! ```text
//! bubbles-tui <file.bub> [<file2.bub> …] [StartNode]
//! ```
//!
//! One or more `.bub` files are compiled together into a single programme.
//! An optional trailing argument that does not end in `.bub` is treated as
//! the start node (defaults to `"Start"`).

use std::path::PathBuf;
use std::process::ExitCode;
use std::{fs, io};

use bubbles_tui::{AppState, Intent, SourceSet, render, terminal};

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

    let mut files: Vec<(String, String)> = Vec::new();
    for path in &args.paths {
        let name = path.display().to_string();
        let source = fs::read_to_string(path).map_err(|e| format!("cannot read `{name}`: {e}"))?;
        files.push((name, source));
    }

    let source_set = SourceSet::many(files.iter().map(|(n, s)| (n.as_str(), s.as_str())));
    let mut state = AppState::from_source_set(source_set, &args.start_node)?;
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
    paths: Vec<PathBuf>,
    start_node: String,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.is_empty() {
        return Err("usage: bubbles-tui <file.bub> [<file2.bub> …] [StartNode]".to_owned());
    }

    // The optional trailing start-node argument does not have a .bub extension.
    let is_bub = |s: &String| {
        std::path::Path::new(s)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bub"))
    };
    let (start_node, file_args) = match raw.split_last() {
        Some((last, rest)) if !is_bub(last) => (last.clone(), rest),
        _ => ("Start".to_owned(), raw.as_slice()),
    };

    if file_args.is_empty() {
        return Err("usage: bubbles-tui <file.bub> [<file2.bub> …] [StartNode]".to_owned());
    }

    let paths = file_args.iter().map(PathBuf::from).collect();
    Ok(Args { paths, start_node })
}
