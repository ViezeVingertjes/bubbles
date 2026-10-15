// Same rationale as in `lib.rs` - ratatui's transitive graph duplicates a
// handful of crate versions we can't de-dup from our side.
#![allow(clippy::multiple_crate_versions)]

//! Entry point for the `bubbles` command.
//!
//! ```text
//! bubbles check <file.bub>… [--quiet]
//! bubbles play  <file.bub>… [StartNode]
//! bubbles       <file.bub>… [StartNode]
//! ```
//!
//! Every file in one invocation is compiled together into a single programme.
//! For `play`, an optional trailing argument that does not end in `.bub` is
//! the start node (defaults to `"Start"`).

mod play;

use std::path::PathBuf;
use std::process::ExitCode;

use bubbles_cli::{SourceSet, check};

const USAGE: &str = "\
bubbles - check and play .bub dialogue scripts

Usage:
  bubbles check <file.bub>... [--quiet]     compile and validate; non-zero exit on failure
  bubbles play  <file.bub>... [StartNode]   play the script in a terminal UI
  bubbles       <file.bub>... [StartNode]   same as `play`

Every file in one invocation is compiled together as a single programme, so
`<<jump>>` and `<<detour>>` targets resolve across them.

Options:
  -q, --quiet      check: print diagnostics only, no success summary
  -h, --help       show this message
  -V, --version    show the version";

/// Why the command stopped.  Only [`Fault::Usage`] is worth reprinting the
/// usage block for; everything else is a plain message.
enum Fault {
    Usage(String),
    Fatal(String),
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv) {
        Ok(code) => code,
        Err(Fault::Usage(message)) => {
            eprintln!("bubbles: {message}\n");
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
        Err(Fault::Fatal(message)) => {
            eprintln!("bubbles: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(argv: &[String]) -> Result<ExitCode, Fault> {
    match argv.first().map(String::as_str) {
        None => Err(Fault::Usage(
            "expected a subcommand or a .bub file".to_owned(),
        )),
        Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        Some("-V" | "--version") => {
            println!("bubbles {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Some("check") => run_check(&argv[1..]),
        Some("play") => play::run(&argv[1..]),
        // A bare path list keeps every invocation that predates `check` working.
        _ => play::run(argv),
    }
}

/// `bubbles check <file.bub>… [--quiet]`.
///
/// Diagnostics go to stdout: they are the command's result, not its own
/// failure, and CI configs pipe them.  Only unusable input goes to stderr.
fn run_check(args: &[String]) -> Result<ExitCode, Fault> {
    let mut quiet = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-q" | "--quiet" => quiet = true,
            other if other.starts_with('-') => {
                return Err(Fault::Usage(format!("unknown option `{other}`")));
            }
            other => paths.push(PathBuf::from(other)),
        }
    }
    if paths.is_empty() {
        return Err(Fault::Usage(
            "check: expected at least one .bub file".to_owned(),
        ));
    }

    let report = check::check(&read_sources(&paths)?);
    for diagnostic in &report.diagnostics {
        println!("{}", diagnostic.one_line());
    }
    if !report.is_ok() {
        return Ok(ExitCode::FAILURE);
    }
    if !quiet {
        println!("{}", report.summary());
    }
    Ok(ExitCode::SUCCESS)
}

/// Reads every path into a [`SourceSet`], keyed by the path as written so the
/// filenames in diagnostics match what the user typed.
fn read_sources(paths: &[PathBuf]) -> Result<SourceSet, Fault> {
    let mut files: Vec<(String, String)> = Vec::new();
    for path in paths {
        let name = path.display().to_string();
        let source = std::fs::read_to_string(path)
            .map_err(|e| Fault::Fatal(format!("cannot read `{name}`: {e}")))?;
        files.push((name, source));
    }
    Ok(SourceSet::many(files))
}
