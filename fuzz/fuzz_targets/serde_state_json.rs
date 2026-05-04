//! Fuzz serde JSON deserialization for `Value`, `HashMapStorage`, and
//! `RunnerSnapshot`. For valid snapshots, restore is attempted against a fixed
//! small program to surface logic bugs on semantically incoherent state.
//!
//! Requires the `serde` feature on `bubbles-dialogue` (enabled via `full` in
//! `fuzz/Cargo.toml`).
#![no_main]

use bubbles::{HashMapStorage, Runner, RunnerSnapshot, Value};
use libfuzzer_sys::fuzz_target;

const FIXED_PROGRAM_SRC: &str = "title: Start\n---\nHello.\n===\n";

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<Value>(data);
    let _ = serde_json::from_slice::<HashMapStorage>(data);

    if let Ok(snap) = serde_json::from_slice::<RunnerSnapshot>(data)
        && let Ok(program) = bubbles::compile(FIXED_PROGRAM_SRC)
    {
        let mut runner = Runner::new(program, HashMapStorage::new());
        let _ = runner.restore(snap);
    }
});
