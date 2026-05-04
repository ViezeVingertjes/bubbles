//! Fuzz the expression lexer and recursive-descent parser in isolation.
//!
//! Both operate on expression strings, not full `.bub` scripts, so this target
//! exercises a narrower input space than `compile_bub`.
//!
//! Keep `max_len` low (≈2048) — the recursive-descent parser can stack-overflow
//! on deeply nested expressions before OOM is reached.
#![no_main]

use bubbles::compiler::expr::parse_expr_at;
use bubbles::compiler::tokenise;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let _ = tokenise(src, "<fuzz>", 0);
    let _ = parse_expr_at(src, "<fuzz>", 0);
});
