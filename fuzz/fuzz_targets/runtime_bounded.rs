//! Fuzz the full compile → Runner → event loop pipeline.
//!
//! Compiles the input, starts at the first available node, then drives
//! `next_event` up to `MAX_EVENTS` times. Options resolve to the first
//! available index so execution is deterministic and jump-loops cannot hang.
#![no_main]

use bubbles::{DialogueEvent, HashMapStorage, Runner};
use libfuzzer_sys::fuzz_target;

const MAX_EVENTS: usize = 64;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let Ok(program) = bubbles::compile(src) else {
        return;
    };

    let Some(start_node) = program.node_titles().next() else {
        return;
    };
    let start_node = start_node.to_owned();

    let mut runner = Runner::new(program, HashMapStorage::new());

    if runner.start(&start_node).is_err() {
        return;
    }

    for _ in 0..MAX_EVENTS {
        match runner.next_event() {
            Ok(Some(DialogueEvent::Options(ref opts))) => {
                let Some(idx) = opts.iter().position(|o| o.available) else {
                    break;
                };
                if runner.select_option(idx).is_err() {
                    break;
                }
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }
});
