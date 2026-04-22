//! A minimal command-line dialogue runner for `.bub` script files.
//!
//! Usage:
//!   cargo run --example cli_runner -- path/to/script.bub StartNode

use std::io::{self, BufRead as _, Write as _};

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| {
        eprintln!("usage: cli_runner <file.bub> [StartNode]");
        std::process::exit(1);
    });
    let start = args.next().unwrap_or_else(|| "Start".to_owned());

    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read `{path}`: {e}"));

    let prog = compile(&source).unwrap_or_else(|e| panic!("compile error: {e}"));
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start(&start).unwrap_or_else(|e| panic!("{e}"));

    let stdin = io::stdin();
    loop {
        match runner.next_event().expect("runtime error") {
            Some(DialogueEvent::NodeStarted(n)) => println!("[ node: {n} ]"),
            Some(DialogueEvent::Line { speaker, text, .. }) => {
                if let Some(spk) = speaker {
                    println!("{spk}: {text}");
                } else {
                    println!("{text}");
                }
                // wait for enter
                print!("(press enter)");
                io::stdout().flush().ok();
                stdin.lock().lines().next();
            }
            Some(DialogueEvent::Options(opts)) => {
                println!();
                for (i, opt) in opts.iter().enumerate() {
                    let marker = if opt.available { "→" } else { "✗" };
                    println!("  {marker} [{i}] {}", opt.text);
                }
                let choice = loop {
                    print!("choose: ");
                    io::stdout().flush().ok();
                    let line = stdin.lock().lines().next()
                        .and_then(|l| l.ok())
                        .unwrap_or_default();
                    if let Ok(n) = line.trim().parse::<usize>() {
                        if n < opts.len() && opts[n].available {
                            break n;
                        }
                    }
                    println!("invalid choice");
                };
                runner.select_option(choice).expect("select_option failed");
            }
            Some(DialogueEvent::Command { name, args, .. }) => {
                println!("[command] {name} {}", args.join(" "));
            }
            Some(DialogueEvent::NodeComplete(n)) => println!("[ /{n} ]"),
            Some(DialogueEvent::DialogueComplete) | None => {
                println!("\n[end]");
                break;
            }
            Some(_) => {} // forward-compatible with future event kinds
        }
    }
}
