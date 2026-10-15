//! Minimal host used by `scripts/measure-footprint.sh` to produce a linked
//! artifact whose size is worth reporting.
//!
//! Not part of the published crate and not a workspace member: the script copies
//! this file into a throwaway package. It exists as a real file rather than a
//! heredoc so it is formatted, reviewable, and greppable like the rest of the
//! source.
//!
//! Everything the runtime offers has to be *reached* from an exported symbol, or
//! the linker discards it and the measurement flatters us. `drive` therefore
//! compiles, runs, and reads a line; `save` is compiled only with the `serde`
//! feature and serialises a snapshot, because the feature adds derives that are
//! stripped entirely unless a host actually serialises something.

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

/// Compiles `source` and pumps every event, returning the number of lines seen.
///
/// # Safety
///
/// `ptr` must point to `len` bytes of valid UTF-8 that stay alive for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn drive(ptr: *const u8, len: usize) -> u32 {
    let source = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
    let Ok(program) = compile(source) else {
        return 0;
    };
    let mut runner = Runner::new(program, HashMapStorage::new());
    if runner.start("Start").is_err() {
        return 0;
    }
    let mut lines = 0;
    while let Ok(Some(event)) = runner.next_event() {
        if matches!(event, DialogueEvent::Line { .. }) {
            lines += 1;
        }
    }
    lines
}

/// Compiles `source`, runs it, and returns the byte length of the serialised
/// snapshot, so the `serde` derives are linked in rather than stripped.
///
/// # Safety
///
/// `ptr` must point to `len` bytes of valid UTF-8 that stay alive for the call.
#[cfg(feature = "serde")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn save(ptr: *const u8, len: usize) -> u32 {
    let source = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
    let Ok(program) = compile(source) else {
        return 0;
    };
    let mut runner = Runner::new(program, HashMapStorage::new());
    if runner.start("Start").is_err() {
        return 0;
    }
    while matches!(runner.next_event(), Ok(Some(_))) {}
    let Ok(json) = serde_json::to_string(&runner.snapshot()) else {
        return 0;
    };
    u32::try_from(json.len()).unwrap_or(u32::MAX)
}
