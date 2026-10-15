//! The guarantees `ABI.md` makes to a host that gets its arguments wrong.
//!
//! The gap: `tests/smoke.rs` and `tests/header.rs` cover the happy path and the
//! header's agreement with the Rust constants. Between them, not one call passes
//! a NULL, a bad length, invalid UTF-8, or an out-of-range enum — and those are
//! the calls a host written in C, C# or Python actually makes by accident. Every
//! entry point here is `unsafe extern "C"`, so a missing guard is not a wrong
//! return value, it is undefined behaviour in somebody else's process.
//!
//! Each test names the `ABI.md` promise it holds. That matters more here than in
//! the Rust crate: `ABI.md` is rendered into the published docs and is what a
//! host author codes against, so a promise drifting away from the code is a
//! defect in the contract even when the code is self-consistent.

// A test target does not inherit the lib's crate-level attributes, so the
// workspace `unsafe_code = "deny"` applies here on its own. Calling the ABI is
// the point of the file.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_void};

use bubbles_ffi::{
    BUBBLES_DONE, BUBBLES_ERR, BUBBLES_OK, BUBBLES_SALIENCY_FIRST_AVAILABLE, bubbles_compile,
    bubbles_compile_files, bubbles_copy_utf8, bubbles_last_error, bubbles_program_free,
    bubbles_program_node_exists, bubbles_program_node_tags_json, bubbles_program_node_titles_json,
    bubbles_program_variable_declarations_json, bubbles_runner_free, bubbles_runner_new,
    bubbles_runner_new_seeded, bubbles_runner_new_with_saliency, bubbles_runner_next_event,
    bubbles_runner_register_function, bubbles_runner_restore_session_json,
    bubbles_runner_restore_storage_json, bubbles_runner_select_option,
    bubbles_runner_set_locale_json, bubbles_runner_set_saliency,
    bubbles_runner_snapshot_session_json, bubbles_runner_snapshot_storage_json,
    bubbles_runner_start, bubbles_runner_variable_get_json, bubbles_runner_variable_set_json,
    bubbles_string_free,
};

const BASIC: &str = r"title: Start
---
Alice: Hi
===
";

const WITH_OPTIONS: &str = r"title: Start
---
Alice: Pick one.
-> First
    Alice: You picked the first.
-> Second
    Alice: You picked the second.
===
";

const WITH_HOST_CALL: &str = r"title: Start
---
<<set $n = host(1)>>
Alice: done
===
";

/// Bytes that are not valid UTF-8 at any length: a lone continuation byte
/// followed by an invalid start byte.
const BAD_UTF8: &[u8] = &[0x80, 0xFF];

// ---------------------------------------------------------------- helpers

fn compile_ok(src: &str) -> *mut c_void {
    let mut program: *mut c_void = std::ptr::null_mut();
    let text = CString::new(src).expect("fixture has no interior NUL");
    let rc = unsafe { bubbles_compile(text.as_ptr(), text.as_bytes().len(), &raw mut program) };
    assert_eq!(rc, BUBBLES_OK, "fixture failed to compile: {:?}", error());
    assert!(!program.is_null());
    program
}

fn runner_ok(src: &str) -> *mut c_void {
    let program = compile_ok(src);
    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_new(program, &raw mut runner) };
    assert_eq!(rc, BUBBLES_OK);
    assert!(!runner.is_null());
    runner
}

/// A runner positioned at the first event of `Start`.
fn started_runner(src: &str) -> *mut c_void {
    let runner = runner_ok(src);
    let start = CString::new("Start").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK, "fixture failed to start: {:?}", error());
    runner
}

/// The current thread's last error message, if any. Copied immediately, as
/// `ABI.md` requires.
fn error() -> Option<String> {
    let p = bubbles_last_error();
    if p.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

/// Asserts a call reported failure in the way `ABI.md` promises: the status code
/// is `BUBBLES_ERR` and a message is available on this thread.
fn assert_reported(rc: c_int, what: &str) {
    assert_eq!(rc, BUBBLES_ERR, "{what} should have returned BUBBLES_ERR");
    let msg = error();
    assert!(
        msg.is_some(),
        "{what} returned BUBBLES_ERR but left no message for bubbles_last_error()"
    );
    assert!(
        !msg.unwrap().is_empty(),
        "{what} left an empty error message"
    );
}

fn drain(runner: *mut c_void) -> Vec<String> {
    let mut events = Vec::new();
    loop {
        let mut json: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &raw mut json) };
        if rc == BUBBLES_DONE {
            assert!(json.is_null(), "ABI.md: BUBBLES_DONE sets *out to NULL");
            break;
        }
        assert_eq!(rc, BUBBLES_OK, "next_event: {:?}", error());
        events.push(
            unsafe { CStr::from_ptr(json) }
                .to_string_lossy()
                .into_owned(),
        );
        unsafe { bubbles_string_free(json) };
        assert!(events.len() <= 64, "fixture did not terminate");
    }
    events
}

/// Pumps until a call reports failure, returning the message. Panics if the
/// dialogue finishes without one. The leading `NodeStarted` event arrives before
/// any `<<set>>` runs, so a failing host function surfaces a few events in.
fn pump_until_error(runner: *mut c_void) -> String {
    for _ in 0..32 {
        let mut json: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &raw mut json) };
        if rc == BUBBLES_ERR {
            assert!(json.is_null(), "a failed event pump wrote *out_event_json");
            return error().expect("BUBBLES_ERR left no message");
        }
        assert_ne!(rc, BUBBLES_DONE, "the dialogue finished without an error");
        unsafe { bubbles_string_free(json) };
    }
    panic!("no error within 32 events");
}

/// Pumps until the next `Options` event, returning how many options it offered.
fn drain_to_options(runner: *mut c_void) -> usize {
    for _ in 0..32 {
        let mut json: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &raw mut json) };
        assert_eq!(rc, BUBBLES_OK, "ran out of events before Options");
        let text = unsafe { CStr::from_ptr(json) }
            .to_string_lossy()
            .into_owned();
        unsafe { bubbles_string_free(json) };
        let value: serde_json::Value = serde_json::from_str(&text).expect("event JSON");
        if value["kind"] == "Options" {
            return value["options"].as_array().expect("options array").len();
        }
    }
    panic!("no Options event");
}

// ------------------------------------------------- null and bad arguments

/// `ABI.md` documents every handle parameter as a live handle, and the
/// implementation promises a reported error rather than a dereference. A host
/// that loses track of a handle gets `BUBBLES_ERR`, not a segfault.
#[test]
fn null_handles_are_reported_rather_than_dereferenced() {
    let null = std::ptr::null_mut::<c_void>();
    let name = CString::new("Start").unwrap();
    let json = CString::new("{}").unwrap();
    let (n, nl) = (name.as_ptr(), name.as_bytes().len());
    let (j, jl) = (json.as_ptr(), json.as_bytes().len());

    let mut out_json: *mut c_char = std::ptr::null_mut();
    let mut out_exists: c_int = 0;
    let mut out_runner: *mut c_void = std::ptr::null_mut();

    unsafe {
        assert_reported(
            bubbles_program_node_exists(null, n, nl, &raw mut out_exists),
            "node_exists(NULL program)",
        );
        assert_reported(
            bubbles_program_node_titles_json(null, &raw mut out_json),
            "node_titles_json(NULL program)",
        );
        assert_reported(
            bubbles_program_node_tags_json(null, n, nl, &raw mut out_json),
            "node_tags_json(NULL program)",
        );
        assert_reported(
            bubbles_program_variable_declarations_json(null, &raw mut out_json),
            "variable_declarations_json(NULL program)",
        );
        assert_reported(
            bubbles_runner_new(null, &raw mut out_runner),
            "runner_new(NULL program)",
        );
        assert_reported(
            bubbles_runner_new_with_saliency(
                null,
                BUBBLES_SALIENCY_FIRST_AVAILABLE,
                &raw mut out_runner,
            ),
            "runner_new_with_saliency(NULL program)",
        );
        assert_reported(
            bubbles_runner_new_seeded(
                null,
                BUBBLES_SALIENCY_FIRST_AVAILABLE,
                7,
                &raw mut out_runner,
            ),
            "runner_new_seeded(NULL program)",
        );
        assert_reported(
            bubbles_runner_start(null, n, nl),
            "runner_start(NULL runner)",
        );
        assert_reported(
            bubbles_runner_next_event(null, &raw mut out_json),
            "next_event(NULL runner)",
        );
        assert_reported(
            bubbles_runner_select_option(null, 0),
            "select_option(NULL runner)",
        );
        assert_reported(
            bubbles_runner_set_saliency(null, BUBBLES_SALIENCY_FIRST_AVAILABLE),
            "set_saliency(NULL runner)",
        );
        assert_reported(
            bubbles_runner_set_locale_json(null, j, jl),
            "set_locale_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_register_function(null, n, nl, host_returns_one, std::ptr::null_mut()),
            "register_function(NULL runner)",
        );
        assert_reported(
            bubbles_runner_variable_get_json(null, n, nl, &raw mut out_json),
            "variable_get_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_variable_set_json(null, n, nl, j, jl),
            "variable_set_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_snapshot_session_json(null, &raw mut out_json),
            "snapshot_session_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_snapshot_storage_json(null, &raw mut out_json),
            "snapshot_storage_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_restore_storage_json(null, j, jl),
            "restore_storage_json(NULL runner)",
        );
        assert_reported(
            bubbles_runner_restore_session_json(null, j, jl),
            "restore_session_json(NULL runner)",
        );
    }
}

/// Same for the output parameters. `write_handle_out` and `write_cstring_out`
/// write through these without checking, so the guard has to be at the entry
/// point; a host passing a NULL out-parameter is a bug, but it should be a
/// reported one rather than a null write.
#[test]
fn null_out_parameters_are_reported_rather_than_written() {
    let program = compile_ok(BASIC);
    let runner = runner_ok(BASIC);
    let name = CString::new("Start").unwrap();
    let (n, nl) = (name.as_ptr(), name.as_bytes().len());

    unsafe {
        assert_reported(
            bubbles_compile(n, nl, std::ptr::null_mut()),
            "compile(NULL out)",
        );
        assert_reported(
            bubbles_compile_files(std::ptr::null(), 1, std::ptr::null_mut()),
            "compile_files(NULL out)",
        );
        assert_reported(
            bubbles_program_node_exists(program, n, nl, std::ptr::null_mut()),
            "node_exists(NULL out)",
        );
        assert_reported(
            bubbles_program_node_titles_json(program, std::ptr::null_mut()),
            "node_titles_json(NULL out)",
        );
        assert_reported(
            bubbles_program_node_tags_json(program, n, nl, std::ptr::null_mut()),
            "node_tags_json(NULL out)",
        );
        assert_reported(
            bubbles_program_variable_declarations_json(program, std::ptr::null_mut()),
            "variable_declarations_json(NULL out)",
        );
        assert_reported(
            bubbles_runner_next_event(runner, std::ptr::null_mut()),
            "next_event(NULL out)",
        );
        assert_reported(
            bubbles_runner_variable_get_json(runner, n, nl, std::ptr::null_mut()),
            "variable_get_json(NULL out)",
        );
        assert_reported(
            bubbles_runner_snapshot_session_json(runner, std::ptr::null_mut()),
            "snapshot_session_json(NULL out)",
        );
        assert_reported(
            bubbles_runner_snapshot_storage_json(runner, std::ptr::null_mut()),
            "snapshot_storage_json(NULL out)",
        );

        bubbles_runner_free(runner);
        bubbles_program_free(program);
    }
}

/// `ABI.md`: "After `BUBBLES_ERR`, out-parameters are not written. Initialise
/// them to `NULL` / `0` and they stay that way; do not read them on the error
/// path." A host that follows that advice and then frees whatever it finds would
/// free a garbage pointer if this ever stopped holding, so each sentinel here is
/// a recognisable non-NULL value rather than NULL.
#[test]
fn out_parameters_are_untouched_on_error() {
    const SENTINEL_PTR: *mut c_char = 0x1234_usize as *mut c_char;
    const SENTINEL_HANDLE: *mut c_void = 0x1234_usize as *mut c_void;
    const SENTINEL_INT: c_int = 7;

    let program = compile_ok(BASIC);
    let runner = runner_ok(BASIC);

    // A syntactically invalid program.
    let mut out_program = SENTINEL_HANDLE;
    let bad = CString::new("-> dangling option outside a node\n").unwrap();
    let rc = unsafe { bubbles_compile(bad.as_ptr(), bad.as_bytes().len(), &raw mut out_program) };
    assert_reported(rc, "compile(invalid source)");
    assert_eq!(
        out_program, SENTINEL_HANDLE,
        "compile wrote *out_program on the error path"
    );

    // Invalid UTF-8 where a node name is expected.
    let mut out_exists = SENTINEL_INT;
    let rc = unsafe {
        bubbles_program_node_exists(
            program,
            BAD_UTF8.as_ptr().cast::<c_char>(),
            BAD_UTF8.len(),
            &raw mut out_exists,
        )
    };
    assert_reported(rc, "node_exists(invalid utf-8)");
    assert_eq!(
        out_exists, SENTINEL_INT,
        "node_exists wrote *out_exists on the error path"
    );

    let mut out_json = SENTINEL_PTR;
    let rc = unsafe {
        bubbles_program_node_tags_json(
            program,
            BAD_UTF8.as_ptr().cast::<c_char>(),
            BAD_UTF8.len(),
            &raw mut out_json,
        )
    };
    assert_reported(rc, "node_tags_json(invalid utf-8)");
    assert_eq!(
        out_json, SENTINEL_PTR,
        "node_tags_json wrote *out_json on the error path"
    );

    let mut out_json = SENTINEL_PTR;
    let rc = unsafe {
        bubbles_runner_variable_get_json(
            runner,
            BAD_UTF8.as_ptr().cast::<c_char>(),
            BAD_UTF8.len(),
            &raw mut out_json,
        )
    };
    assert_reported(rc, "variable_get_json(invalid utf-8)");
    assert_eq!(
        out_json, SENTINEL_PTR,
        "variable_get_json wrote *out_json on the error path"
    );

    // An event pump that fails before producing an event.
    let mut out_json = SENTINEL_PTR;
    let rc = unsafe { bubbles_runner_next_event(std::ptr::null_mut(), &raw mut out_json) };
    assert_reported(rc, "next_event(NULL runner)");
    assert_eq!(
        out_json, SENTINEL_PTR,
        "next_event wrote *out_event_json on the error path"
    );

    unsafe {
        bubbles_runner_free(runner);
        bubbles_program_free(program);
    }
}

// ----------------------------------------------- program handle ownership

/// `ABI.md`: the program handle "is consumed whenever both `program` and
/// `out_runner` are non-NULL". The contrapositive is the part a host can act on
/// — with a NULL `out_runner` the call fails *before* taking ownership, so the
/// caller still owns the program and must still free it.
///
/// If the NULL check ever moved below `Box::from_raw`, this test would be
/// exercising a freed program and the published rule would be wrong in the
/// direction that double-frees.
#[test]
fn a_rejected_constructor_call_leaves_the_program_owned_by_the_caller() {
    let program = compile_ok(BASIC);

    let rc = unsafe { bubbles_runner_new(program, std::ptr::null_mut()) };
    assert_reported(rc, "runner_new(NULL out_runner)");

    // Still usable, which is only true if nothing took ownership.
    let name = CString::new("Start").unwrap();
    let mut exists: c_int = 0;
    let rc = unsafe {
        bubbles_program_node_exists(
            program,
            name.as_ptr(),
            name.as_bytes().len(),
            &raw mut exists,
        )
    };
    assert_eq!(rc, BUBBLES_OK);
    assert_eq!(exists, 1);

    // And still the caller's to free.
    unsafe { bubbles_program_free(program) };
}

/// The seeded constructor shares `build_runner` with the other two, so what is
/// worth testing is not the shared guard a third time but that the seed
/// parameter did not buy itself an exemption from it: a seed is a plain `u64`
/// with no invalid value, so every bad-argument case here is still a pointer or
/// enum case, and all of them must behave exactly as the unseeded constructors
/// do.
///
/// Both halves of the ownership rule in one test, because for this entry point
/// they are the same claim read in two directions — NULL `out_runner` is
/// rejected before the program is taken, and an invalid saliency kind is
/// rejected after. As in the test above, the program is deliberately not freed
/// after the second call.
#[test]
fn the_seeded_constructor_validates_exactly_as_the_unseeded_ones_do() {
    let program = compile_ok(BASIC);

    let rc = unsafe {
        bubbles_runner_new_seeded(
            program,
            BUBBLES_SALIENCY_FIRST_AVAILABLE,
            7,
            std::ptr::null_mut(),
        )
    };
    assert_reported(rc, "runner_new_seeded(NULL out_runner)");

    // Not taken, so a seed is still choosable for this program.
    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe {
        bubbles_runner_new_seeded(
            program,
            BUBBLES_SALIENCY_FIRST_AVAILABLE,
            7,
            &raw mut runner,
        )
    };
    assert_eq!(rc, BUBBLES_OK, "{:?}", error());
    assert!(!runner.is_null());
    unsafe { bubbles_runner_free(runner) };

    let program = compile_ok(BASIC);
    let mut runner: *mut c_void = 0x1234_usize as *mut c_void;
    let rc = unsafe { bubbles_runner_new_seeded(program, 99, 7, &raw mut runner) };
    assert_reported(rc, "runner_new_seeded(invalid kind)");
    assert_eq!(
        runner, 0x1234_usize as *mut c_void,
        "a failed constructor wrote *out_runner"
    );
    // Deliberately no `bubbles_program_free(program)`: consumed by the failed call.
}

/// The other half of the same rule. An invalid saliency kind is rejected
/// *after* the program has been taken, so the handle is gone even though the
/// call failed — `ABI.md` documents that explicitly, and it is why this test
/// does not free the program afterwards; doing so would be the double-free the
/// documentation warns hosts about.
///
/// What is asserted is what a test can see from this side of the boundary: the
/// failure is reported and `*out_runner` is left alone. The consumption itself
/// is not observable without reaching into freed memory, so if that half of the
/// rule is ever changed — validating the kind before taking ownership would be
/// the kinder contract — this test will keep passing and `ABI.md` is the thing
/// that has to be kept honest.
#[test]
fn an_invalid_saliency_kind_is_reported_without_writing_the_out_handle() {
    let program = compile_ok(BASIC);
    let mut runner: *mut c_void = 0x1234_usize as *mut c_void;

    let rc = unsafe { bubbles_runner_new_with_saliency(program, 99, &raw mut runner) };
    assert_reported(rc, "runner_new_with_saliency(invalid kind)");
    assert_eq!(
        runner, 0x1234_usize as *mut c_void,
        "a failed constructor wrote *out_runner"
    );
    // Deliberately no `bubbles_program_free(program)` here: per ABI.md the
    // handle was consumed by the call that failed.
}

/// `ABI.md`: "`bubbles_string_free(NULL)` and `bubbles_program_free(NULL)` and
/// `bubbles_runner_free(NULL)` are all no-ops, so you do not need null guards in
/// cleanup paths." Hosts write exactly that cleanup code, so the no-op has to be
/// real.
#[test]
fn freeing_a_null_handle_is_a_no_op() {
    unsafe {
        bubbles_string_free(std::ptr::null_mut());
        bubbles_program_free(std::ptr::null_mut());
        bubbles_runner_free(std::ptr::null_mut());
    }
}

// ------------------------------------------------------ argument recovery

/// `ABI.md`: "A call that fails argument validation leaves the handle usable.
/// Invalid UTF-8, malformed JSON, an unknown node name, an out-of-range or
/// unavailable option index: all reported, none fatal. Retry with corrected
/// input on the same runner."
///
/// That is a promise about runner state after a failure, which no round-trip
/// test can see. The option case matters most: a host that validates loosely
/// sends an index the script does not offer, and if that left the pending
/// options poisoned the dialogue would be stuck with a status code that said it
/// was recoverable.
#[test]
fn argument_validation_failures_leave_the_runner_usable() {
    let runner = runner_ok(WITH_OPTIONS);

    // Unknown node, then the real one.
    let missing = CString::new("NoSuchNode").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, missing.as_ptr(), missing.as_bytes().len()) };
    assert_reported(rc, "start(unknown node)");

    let start = CString::new("Start").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK, "start after a failed start: {:?}", error());

    // Malformed JSON, then valid JSON, on the same runner.
    let key = CString::new("$v").unwrap();
    let broken = CString::new("{not json").unwrap();
    let rc = unsafe {
        bubbles_runner_variable_set_json(
            runner,
            key.as_ptr(),
            key.as_bytes().len(),
            broken.as_ptr(),
            broken.as_bytes().len(),
        )
    };
    assert_reported(rc, "variable_set_json(malformed json)");

    let good = CString::new("12").unwrap();
    let rc = unsafe {
        bubbles_runner_variable_set_json(
            runner,
            key.as_ptr(),
            key.as_bytes().len(),
            good.as_ptr(),
            good.as_bytes().len(),
        )
    };
    assert_eq!(rc, BUBBLES_OK, "set after a malformed set: {:?}", error());

    // An out-of-range option index, then a valid one.
    let count = drain_to_options(runner);
    assert_eq!(count, 2, "fixture should offer two options");

    let rc = unsafe { bubbles_runner_select_option(runner, count) };
    assert_reported(rc, "select_option(index == option count)");
    let rc = unsafe { bubbles_runner_select_option(runner, usize::MAX) };
    assert_reported(rc, "select_option(usize::MAX)");

    let rc = unsafe { bubbles_runner_select_option(runner, 1) };
    assert_eq!(
        rc,
        BUBBLES_OK,
        "select_option(1) after two rejected indices: {:?}",
        error()
    );

    let rest = drain(runner);
    let joined = rest.join("\n");
    assert!(
        joined.contains("You picked the second"),
        "the retried selection did not take effect: {joined}"
    );

    unsafe { bubbles_runner_free(runner) };
}

/// `ABI.md`: "Invalid UTF-8 returns `BUBBLES_ERR` rather than reading past the
/// slice or producing replacement characters." The second half is the one worth
/// asserting: lossy decoding would turn a host's encoding bug into a silent
/// lookup miss instead of an error.
#[test]
fn invalid_utf8_is_rejected_rather_than_replaced() {
    let runner = runner_ok(BASIC);
    let ptr = BAD_UTF8.as_ptr().cast::<c_char>();

    let rc = unsafe { bubbles_runner_start(runner, ptr, BAD_UTF8.len()) };
    assert_reported(rc, "start(invalid utf-8)");
    let message = error().unwrap_or_default();
    assert!(
        !message.contains('\u{fffd}'),
        "the error message contains a replacement character, so the bytes were decoded lossily: {message}"
    );

    let copied = unsafe { bubbles_copy_utf8(ptr, BAD_UTF8.len()) };
    assert!(copied.is_null(), "bubbles_copy_utf8 accepted invalid UTF-8");
    assert!(
        error().is_some(),
        "bubbles_copy_utf8 returned NULL without leaving a message"
    );

    unsafe { bubbles_runner_free(runner) };
}

/// `ABI.md`: "On `BUBBLES_ERR`, `bubbles_last_error()` returns a NUL-terminated
/// UTF-8 message." `bubbles_copy_utf8` signals failure with a NULL return rather
/// than a status code, and it had one failure path that set no message: a valid
/// UTF-8 string with an interior NUL, which cannot be represented in a
/// NUL-terminated string. A host building a host-function result out of
/// player-entered or externally sourced text reaches it, sees NULL, and then
/// reads either nothing or a stale message from an earlier call, so the
/// diagnostic points at its control flow when the problem is its data.
#[test]
fn an_interior_nul_is_reported_and_not_merely_refused() {
    let with_nul = "before\0after";

    // Leave a recognisable message in the slot first. If the interior-NUL path
    // sets nothing, the host reads this instead and is told the wrong thing.
    let bad = unsafe { bubbles_copy_utf8(BAD_UTF8.as_ptr().cast::<c_char>(), BAD_UTF8.len()) };
    assert!(bad.is_null(), "invalid UTF-8 was copied");
    let stale = error().unwrap_or_default();

    let copied = unsafe { bubbles_copy_utf8(with_nul.as_ptr().cast::<c_char>(), with_nul.len()) };
    assert!(
        copied.is_null(),
        "a string with an interior NUL was copied, so the result is truncated at the NUL"
    );
    let message = error().unwrap_or_default();
    assert!(
        !message.is_empty(),
        "bubbles_copy_utf8 returned NULL for an interior NUL without leaving a message"
    );
    assert_ne!(
        message, stale,
        "the message left after the interior-NUL failure is the previous call's, \
         so the error slot was never written: {message}"
    );
}

/// `ABI.md` promises a zero `file_count` and a NULL `files` array are reported,
/// and that a bad path is distinguished from a bad body. A host assembling the
/// array from a loop gets one of these wrong before it gets anything else wrong.
#[test]
fn compile_files_validates_its_array() {
    let mut out: *mut c_void = std::ptr::null_mut();
    unsafe {
        assert_reported(
            bubbles_compile_files(std::ptr::null(), 0, &raw mut out),
            "compile_files(count 0)",
        );
        assert_reported(
            bubbles_compile_files(std::ptr::null(), 2, &raw mut out),
            "compile_files(NULL array)",
        );
    }
    assert!(out.is_null(), "compile_files wrote *out on the error path");
}

// ----------------------------------------------------------- host callbacks

/// Returns the number 1 through the documented allocator.
unsafe extern "C" fn host_returns_one(
    _userdata: *mut c_void,
    _args_ptr: *const c_char,
    _args_len: usize,
    out_result_json: *mut *mut c_char,
) -> c_int {
    let s = CString::new("1").unwrap();
    let p = unsafe { bubbles_copy_utf8(s.as_ptr(), s.as_bytes().len()) };
    if p.is_null() {
        return BUBBLES_ERR;
    }
    unsafe { *out_result_json = p };
    BUBBLES_OK
}

/// A conforming failure: `BUBBLES_ERR` plus a `bubbles_copy_utf8` message, as
/// `ABI.md` requires on the error path.
unsafe extern "C" fn host_fails_with_message(
    _userdata: *mut c_void,
    _args_ptr: *const c_char,
    _args_len: usize,
    out_result_json: *mut *mut c_char,
) -> c_int {
    let s = CString::new("inventory service unreachable").unwrap();
    let p = unsafe { bubbles_copy_utf8(s.as_ptr(), s.as_bytes().len()) };
    unsafe { *out_result_json = p };
    BUBBLES_ERR
}

/// A non-conforming failure: claims success and leaves the result pointer NULL.
const unsafe extern "C" fn host_leaves_null(
    _userdata: *mut c_void,
    _args_ptr: *const c_char,
    _args_len: usize,
    _out_result_json: *mut *mut c_char,
) -> c_int {
    BUBBLES_OK
}

/// A host whose callback misbehaves must not take the process with it. Both
/// shapes are documented — a reported failure carries the callback's own
/// message, and "Leaving it `NULL` is itself reported as a function error" —
/// and neither was covered.
#[test]
fn a_failing_host_function_is_reported_through_the_event_pump() {
    let name = CString::new("host").unwrap();

    let runner = runner_ok(WITH_HOST_CALL);
    let rc = unsafe {
        bubbles_runner_register_function(
            runner,
            name.as_ptr(),
            name.as_bytes().len(),
            host_fails_with_message,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, BUBBLES_OK);
    let start = CString::new("Start").unwrap();
    assert_eq!(
        unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) },
        BUBBLES_OK
    );
    let message = pump_until_error(runner);
    assert!(
        message.contains("inventory service unreachable"),
        "the callback's own message did not survive: {message}"
    );
    unsafe { bubbles_runner_free(runner) };

    let runner = runner_ok(WITH_HOST_CALL);
    let rc = unsafe {
        bubbles_runner_register_function(
            runner,
            name.as_ptr(),
            name.as_bytes().len(),
            host_leaves_null,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, BUBBLES_OK);
    assert_eq!(
        unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) },
        BUBBLES_OK
    );
    let message = pump_until_error(runner);
    assert!(
        message.contains("null"),
        "a NULL host result was not reported as such: {message}"
    );
    unsafe { bubbles_runner_free(runner) };
}

// --------------------------------------------------- threading and replay

/// `ABI.md`: "The error slot is thread-local. An error raised on one thread is
/// not visible from another." A host that logs on a worker thread depends on
/// this, and a single shared slot would read plausibly in every single-threaded
/// test.
#[test]
fn the_error_slot_does_not_leak_between_threads() {
    assert!(
        error().is_none(),
        "this thread started with an error already set"
    );

    let child = std::thread::spawn(|| {
        let rc = unsafe { bubbles_runner_select_option(std::ptr::null_mut(), 0) };
        assert_eq!(rc, BUBBLES_ERR);
        error().expect("the child thread should see its own error")
    });
    let seen_by_child = child.join().expect("child thread panicked");
    assert_ne!(seen_by_child, "", "the child saw an empty message");

    assert!(
        error().is_none(),
        "an error raised on another thread became visible here: {:?}",
        error()
    );
}

/// A hand-written table of entry points goes stale the first time someone adds
/// one, and the blind spot it leaves is the same blind spot this file was
/// written to close. So the header — which `header.rs` already treats as the
/// definitive ABI surface — decides what has to be covered: every `bubbles_*`
/// function it declares must be *called* somewhere in this file.
///
/// What that proves is modest, and worth being precise about: it proves the
/// symbol is exercised here, not that it is exercised with a NULL. A mention in
/// a comment does not count, because the match requires the opening parenthesis
/// of a call, but a happy-path call would satisfy it. It is a ratchet against
/// forgetting, not a measure of thoroughness. The value is that a new entry
/// point cannot be added without someone opening this file and deciding what its
/// bad-argument case is.
#[test]
fn every_entry_point_in_the_header_is_exercised_here() {
    const HEADER: &str = include_str!("../include/bubbles_ffi.h");
    const SELF: &str = include_str!("contract.rs");

    /// Entry points with nothing to validate: these take no pointer the library
    /// could reject, and a NULL handle is a documented no-op covered by
    /// `freeing_a_null_handle_is_a_no_op`.
    const NO_ARGUMENTS_TO_GET_WRONG: &[&str] = &["bubbles_abi_version"];

    let mut declared: Vec<&str> = Vec::new();
    for (index, _) in HEADER.match_indices("bubbles_") {
        let rest = &HEADER[index..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        let (name, after) = (&rest[..end], &rest[end..]);
        if after.starts_with('(') && !declared.contains(&name) {
            declared.push(name);
        }
    }

    assert!(
        declared.len() >= 26,
        "only {} entry points found in the header, so the scan is not reading it: {declared:?}",
        declared.len()
    );

    let missing: Vec<&&str> = declared
        .iter()
        .filter(|name| !NO_ARGUMENTS_TO_GET_WRONG.contains(name))
        .filter(|name| !SELF.contains(&format!("{name}(")))
        .collect();

    assert!(
        missing.is_empty(),
        "the header declares entry points that this file never calls, so their \
         bad-argument behaviour is untested: {missing:?}"
    );
}

/// The determinism claim has to hold at this boundary too, since a native host
/// reaches the runtime only through it. Two separately compiled programs and two
/// separately constructed runners must produce the same event JSON, byte for
/// byte — including the serialised form, not just the Rust events behind it.
#[test]
fn the_event_json_stream_is_identical_across_separate_runners() {
    let left = started_runner(BASIC);
    let right = started_runner(BASIC);
    let first = drain(left);
    let second = drain(right);
    unsafe {
        bubbles_runner_free(left);
        bubbles_runner_free(right);
    }
    assert!(
        first.len() > 1,
        "the fixture produced {} events, so this proved nothing",
        first.len()
    );
    assert_eq!(first, second, "event JSON differed between two runners");
}
