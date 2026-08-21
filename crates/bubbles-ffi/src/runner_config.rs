//! Runner construction, saliency, localisation, and host function registration.

use std::ffi::{CString, c_char, c_int, c_void};
use std::ptr;

use bubbles::error::DialogueError;
use bubbles::{
    BestLeastRecentlyViewed, FirstAvailable, HashMapProvider, HashMapStorage, Program,
    RandomAvailable, Runner, Value,
};

use crate::error::{clear_err, set_err};
use crate::util::{FfiRunner, ffi_try, runner_mut, str_from_parts, write_handle_out};
use crate::value_json::{value_from_json_slice, values_to_json_array};
use crate::{BUBBLES_ERR, BUBBLES_OK};

/// Saliency kind: pick the first available candidate ([`FirstAvailable`]).
pub const BUBBLES_SALIENCY_FIRST_AVAILABLE: c_int = 0;
/// Saliency kind: best least-recently-viewed candidate ([`BestLeastRecentlyViewed`]).
pub const BUBBLES_SALIENCY_BLRV: c_int = 1;
/// Saliency kind: uniformly random available candidate ([`RandomAvailable`]).
pub const BUBBLES_SALIENCY_RANDOM_AVAILABLE: c_int = 2;

/// Host function: on [`BUBBLES_OK`], set `*out_result_json` to a JSON [`Value`] (bool, number, or string)
/// allocated with [`crate::bubbles_copy_utf8`]. On [`BUBBLES_ERR`], set it to a UTF-8 error message (same
/// allocator).
pub type BubblesHostFn = unsafe extern "C" fn(
    userdata: *mut c_void,
    args_json_ptr: *const c_char,
    args_json_len: usize,
    out_result_json: *mut *mut c_char,
) -> c_int;

pub(crate) unsafe fn build_runner(
    program: *mut c_void,
    saliency: c_int,
    out_runner: *mut *mut c_void,
) -> c_int {
    if out_runner.is_null() {
        set_err("out_runner was null");
        return BUBBLES_ERR;
    }
    if program.is_null() {
        set_err("program was null");
        return BUBBLES_ERR;
    }
    clear_err();
    let program = unsafe { *Box::from_raw(program.cast::<Program>()) };
    let mut runner = Runner::new(program, HashMapStorage::new());
    ffi_try!(apply_saliency(&mut runner, saliency));
    unsafe { write_handle_out(out_runner, runner) }
}

fn apply_saliency(runner: &mut FfiRunner, kind: c_int) -> Result<(), &'static str> {
    match kind {
        BUBBLES_SALIENCY_FIRST_AVAILABLE => runner.set_saliency(FirstAvailable),
        BUBBLES_SALIENCY_BLRV => runner.set_saliency(BestLeastRecentlyViewed::default()),
        BUBBLES_SALIENCY_RANDOM_AVAILABLE => runner.set_saliency(RandomAvailable),
        _ => return Err("invalid saliency kind (use BUBBLES_SALIENCY_*)"),
    }
    Ok(())
}

/// Same as [`crate::bubbles_runner_new`] but selects saliency (`BUBBLES_SALIENCY_*`).
///
/// # Safety
///
/// Same as `bubbles_runner_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_new_with_saliency(
    program: *mut c_void,
    saliency_kind: c_int,
    out_runner: *mut *mut c_void,
) -> c_int {
    unsafe { build_runner(program, saliency_kind, out_runner) }
}

/// Replaces saliency. Prefer before [`crate::bubbles_runner_start`].
///
/// # Safety
///
/// `runner` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_set_saliency(
    runner: *mut c_void,
    saliency_kind: c_int,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    clear_err();
    ffi_try!(apply_saliency(runner, saliency_kind));
    BUBBLES_OK
}

/// Line localisation from JSON `{"line_id":"template", ...}`. Prefer before [`crate::bubbles_runner_start`].
///
/// # Safety
///
/// `json_ptr` / `json_len` must be valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_set_locale_json(
    runner: *mut c_void,
    json_ptr: *const c_char,
    json_len: usize,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let json_str = ffi_try!(unsafe { str_from_parts(json_ptr, json_len) });
    clear_err();
    let map: std::collections::HashMap<String, String> =
        ffi_try!(serde_json::from_str(json_str).map_err(|e| format!("locale JSON: {e}")));
    let mut provider = HashMapProvider::new();
    for (k, v) in map {
        provider.insert(k, v);
    }
    runner.set_provider(provider);
    BUBBLES_OK
}

/// Registers a host function for `.bub` expressions (`name` matches `<<name(...)>>`).
///
/// # Safety
///
/// `cb` follows [`BubblesHostFn`]. Do not call back into `bubbles_*` from inside `cb`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_register_function(
    runner: *mut c_void,
    name_ptr: *const c_char,
    name_len: usize,
    cb: BubblesHostFn,
    userdata: *mut c_void,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let name = ffi_try!(unsafe { str_from_parts(name_ptr, name_len) }).to_owned();
    clear_err();
    let name_for_errors = name.clone();
    let userdata_bits = userdata as usize;
    runner.library_mut().register(name, move |args| {
        let userdata = userdata_bits as *mut c_void;
        host_callback_bridge(&name_for_errors, cb, userdata, args)
    });
    BUBBLES_OK
}

fn host_callback_bridge(
    name: &str,
    cb: BubblesHostFn,
    userdata: *mut c_void,
    args: Vec<Value>,
) -> bubbles::Result<Value> {
    let args_json = values_to_json_array(&args).map_err(|e| DialogueError::Function {
        name: name.to_owned(),
        message: e.to_string(),
    })?;
    let args_cs = CString::new(args_json).map_err(|_| DialogueError::Function {
        name: name.to_owned(),
        message: "args JSON contained NUL".into(),
    })?;
    let mut out_ptr: *mut c_char = ptr::null_mut();
    let st = unsafe {
        cb(
            userdata,
            args_cs.as_ptr(),
            args_cs.as_bytes().len(),
            &mut out_ptr,
        )
    };
    if out_ptr.is_null() {
        return Err(DialogueError::Function {
            name: name.to_owned(),
            message: "host callback left null result pointer".into(),
        });
    }
    let msg = unsafe { CString::from_raw(out_ptr) }
        .into_string()
        .map_err(|_| DialogueError::Function {
            name: name.to_owned(),
            message: "host result was not valid UTF-8".into(),
        })?;
    if st == BUBBLES_ERR {
        return Err(DialogueError::Function {
            name: name.to_owned(),
            message: msg,
        });
    }
    value_from_json_slice(msg.as_bytes()).map_err(|e| DialogueError::Function {
        name: name.to_owned(),
        message: e,
    })
}
