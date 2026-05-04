//! Fuzz `compile_many` with the input split across two synthetic source files.
//!
//! Covers: cross-file duplicate-node detection, jump/detour target validation
//! across files, and file-aware parse error attribution.
//!
//! Splitting strategy: use the first byte (if any) as the cut point modulo
//! input length. This gives libFuzzer a stable, reproducible split with full
//! coverage of both the "all in one file" and "evenly split" cases.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let split = data[0] as usize % data.len();
    let (a, b) = data.split_at(split);

    let Ok(src_a) = std::str::from_utf8(a) else {
        return;
    };
    let Ok(src_b) = std::str::from_utf8(b) else {
        return;
    };

    let _ = bubbles::compile_many(&[("file_a.bub", src_a), ("file_b.bub", src_b)]);
});
