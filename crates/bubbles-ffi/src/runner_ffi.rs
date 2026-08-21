//! Runner lifecycle and stepping.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::error::{clear_err, set_err};
use crate::event_json;
use crate::runner_config::{BUBBLES_SALIENCY_FIRST_AVAILABLE, build_runner};
use crate::util::{FfiRunner, ffi_try, runner_mut, str_from_parts, write_cstring_out_with_error};
use crate::{BUBBLES_DONE, BUBBLES_ERR, BUBBLES_OK};

/// Creates a runner. **Consumes** `program`; do not free the program handle afterward.
///
/// # Safety
///
/// `program` must be a live handle from compile. `out_runner` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_new(
    program: *mut c_void,
    out_runner: *mut *mut c_void,
) -> c_int {
    unsafe { build_runner(program, BUBBLES_SALIENCY_FIRST_AVAILABLE, out_runner) }
}

/// Drops a runner.
///
/// # Safety
///
/// `runner` must be null or a valid runner handle not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_free(runner: *mut c_void) {
    if runner.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(runner.cast::<FfiRunner>()) });
}

/// Starts dialogue at `node_name` (UTF-8, byte length).
///
/// # Safety
///
/// `node_ptr` / `node_len` must refer to valid UTF-8 for the call. `runner` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_start(
    runner: *mut c_void,
    node_ptr: *const c_char,
    node_len: usize,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let node = ffi_try!(unsafe { str_from_parts(node_ptr, node_len) });
    clear_err();
    match runner.start(node) {
        Ok(()) => BUBBLES_OK,
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}

/// Advances the runner. On [`BUBBLES_OK`], `*out_event_json` is a newly allocated NUL-terminated
/// UTF-8 JSON string (free with [`crate::bubbles_string_free`]). On [`BUBBLES_DONE`], dialogue has ended
/// and `*out_event_json` is set to null. On [`BUBBLES_ERR`], see [`crate::bubbles_last_error`].
///
/// # Safety
///
/// `runner` and `out_event_json` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_next_event(
    runner: *mut c_void,
    out_event_json: *mut *mut c_char,
) -> c_int {
    if out_event_json.is_null() {
        set_err("out_event_json was null");
        return BUBBLES_ERR;
    }
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    clear_err();
    match runner.next_event() {
        Ok(Some(ev)) => {
            let json = event_json::dialogue_event_to_json(&ev);
            write_cstring_out_with_error(
                out_event_json,
                json,
                "failed to build event JSON (interior NUL)",
            )
        }
        Ok(None) => {
            unsafe {
                *out_event_json = ptr::null_mut();
            }
            BUBBLES_DONE
        }
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}

/// After an `Options` event, select by zero-based index.
///
/// # Safety
///
/// `runner` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_select_option(runner: *mut c_void, index: usize) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    clear_err();
    match runner.select_option(index) {
        Ok(()) => BUBBLES_OK,
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}
