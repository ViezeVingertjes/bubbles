//! Fuzz the full `.bub` compilation pipeline: lexer → parser → AST → Program → validate.
//!
//! Goal: no panic, no stack overflow on any valid UTF-8 input.
//! Compile returns Ok or a structured Err — both are acceptable; only panics are failures.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let _ = bubbles::compile(source);
    }
});
