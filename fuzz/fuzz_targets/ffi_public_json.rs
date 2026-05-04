//! Fuzz the public C ABI JSON entry points in `bubbles-ffi`.
//!
//! A fixed program and runner are built deterministically on each iteration;
//! fuzz bytes are fed into every JSON-accepting function. This covers UTF-8
//! validation, `serde_json` decoding, value mapping, and state-restore logic
//! without involving invalid pointer values.
#![no_main]
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ptr;

use bubbles_ffi::{
    BUBBLES_OK, bubbles_compile, bubbles_runner_free, bubbles_runner_new,
    bubbles_runner_restore_session_json, bubbles_runner_restore_storage_json,
    bubbles_runner_set_locale_json, bubbles_runner_variable_set_json,
};
use libfuzzer_sys::fuzz_target;

const PROG_SRC: &str = "title: Start\n---\nHello.\n===\n";
const VAR_NAME: &[u8] = b"$score";

fuzz_target!(|data: &[u8]| {
    let mut prog_ptr: *mut c_void = ptr::null_mut();
    let ok = unsafe {
        bubbles_compile(
            PROG_SRC.as_ptr().cast(),
            PROG_SRC.len(),
            ptr::addr_of_mut!(prog_ptr),
        )
    };
    if ok != BUBBLES_OK || prog_ptr.is_null() {
        return;
    }

    let mut runner_ptr: *mut c_void = ptr::null_mut();
    // bubbles_runner_new consumes the program handle regardless of outcome.
    let ok = unsafe { bubbles_runner_new(prog_ptr, ptr::addr_of_mut!(runner_ptr)) };
    if ok != BUBBLES_OK || runner_ptr.is_null() {
        return;
    }

    unsafe {
        bubbles_runner_variable_set_json(
            runner_ptr,
            VAR_NAME.as_ptr().cast(),
            VAR_NAME.len(),
            data.as_ptr().cast(),
            data.len(),
        );
        bubbles_runner_set_locale_json(runner_ptr, data.as_ptr().cast(), data.len());
        bubbles_runner_restore_storage_json(runner_ptr, data.as_ptr().cast(), data.len());
        bubbles_runner_restore_session_json(runner_ptr, data.as_ptr().cast(), data.len());
        bubbles_runner_free(runner_ptr);
    }
});
