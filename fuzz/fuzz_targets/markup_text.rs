//! Fuzz the markup scanner (`scan_text_segments`). It makes a single forward
//! pass and must never panic on valid UTF-8; returning `Err` is acceptable.
#![no_main]

use bubbles::compiler::markup::scan_text_segments;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };

    let _ = scan_text_segments(src);
});
