//! Variable access, save/load snapshots, and UTF-8 copy helper.

use std::ffi::{CString, c_char, c_int, c_void};
use std::mem;

use bubbles::{HashMapStorage, RunnerSnapshot, VariableStorage};

use crate::error::{clear_err, set_err};
use crate::util::{ffi_try, runner_mut, runner_ref, str_from_parts, write_cstring_out};
use crate::value_json::{value_from_json_slice, value_to_json_string};
use crate::{BUBBLES_ERR, BUBBLES_OK};

/// Copies `len` UTF-8 bytes into a NUL-terminated string owned by this library.
///
/// Free with [`crate::bubbles_string_free`]. Returns null if the bytes are not valid UTF-8,
/// or if they contain an interior NUL, which a NUL-terminated string cannot carry. Either
/// way the reason is in [`crate::bubbles_last_error`].
///
/// # Safety
///
/// `ptr` + `len` must be valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_copy_utf8(ptr: *const c_char, len: usize) -> *mut c_char {
    let s = match unsafe { str_from_parts(ptr, len) } {
        Ok(s) => s,
        Err(e) => {
            set_err(e);
            return std::ptr::null_mut();
        }
    };
    let Ok(cs) = CString::new(s) else {
        set_err("string contained interior NUL");
        return std::ptr::null_mut();
    };
    cs.into_raw()
}

/// Variable as JSON [`bubbles::Value`], or `"null"` if unset. Free with [`crate::bubbles_string_free`].
///
/// # Safety
///
/// `runner`, `out_json` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_variable_get_json(
    runner: *mut c_void,
    name_ptr: *const c_char,
    name_len: usize,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let runner = ffi_try!(unsafe { runner_ref(runner) });
    let key = ffi_try!(unsafe { str_from_parts(name_ptr, name_len) });
    clear_err();
    let j = runner
        .storage()
        .get(key)
        .and_then(|v| value_to_json_string(&v).ok())
        .unwrap_or_else(|| "null".into());
    write_cstring_out(out_json, j)
}

/// Sets a variable from JSON [`bubbles::Value`] (bool, number, string, or `{"Number"|"Text"|"Bool":…}`).
///
/// # Safety
///
/// Pointers must be valid UTF-8 for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_variable_set_json(
    runner: *mut c_void,
    name_ptr: *const c_char,
    name_len: usize,
    value_json_ptr: *const c_char,
    value_json_len: usize,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let key = ffi_try!(unsafe { str_from_parts(name_ptr, name_len) });
    let val_str = ffi_try!(unsafe { str_from_parts(value_json_ptr, value_json_len) });
    clear_err();
    let value = ffi_try!(value_from_json_slice(val_str.as_bytes()));
    runner.storage_mut().set(key, value);
    BUBBLES_OK
}

/// [`RunnerSnapshot`] as JSON (session only; pair with storage snapshot for saves).
///
/// # Safety
///
/// `out_json` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_snapshot_session_json(
    runner: *mut c_void,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let runner = ffi_try!(unsafe { runner_ref(runner) });
    clear_err();
    let snap = runner.snapshot();
    let j = ffi_try!(serde_json::to_string(&snap).map_err(|e| e.to_string()));
    write_cstring_out(out_json, j)
}

/// [`HashMapStorage`] as JSON.
///
/// # Safety
///
/// `out_json` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_snapshot_storage_json(
    runner: *mut c_void,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let runner = ffi_try!(unsafe { runner_ref(runner) });
    clear_err();
    let j = ffi_try!(serde_json::to_string(runner.storage()).map_err(|e| e.to_string()));
    write_cstring_out(out_json, j)
}

/// Restores storage from [`bubbles_runner_snapshot_storage_json`]. Call before [`bubbles_runner_restore_session_json`].
///
/// # Safety
///
/// `json_ptr` / `json_len` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_restore_storage_json(
    runner: *mut c_void,
    json_ptr: *const c_char,
    json_len: usize,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let text = ffi_try!(unsafe { str_from_parts(json_ptr, json_len) });
    clear_err();
    let mut new_storage: HashMapStorage =
        ffi_try!(serde_json::from_str(text).map_err(|e| format!("storage JSON: {e}")));
    mem::swap(runner.storage_mut(), &mut new_storage);
    BUBBLES_OK
}

/// Restores session from [`bubbles_runner_snapshot_session_json`].
///
/// # Safety
///
/// `json_ptr` / `json_len` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_runner_restore_session_json(
    runner: *mut c_void,
    json_ptr: *const c_char,
    json_len: usize,
) -> c_int {
    let runner = ffi_try!(unsafe { runner_mut(runner) });
    let text = ffi_try!(unsafe { str_from_parts(json_ptr, json_len) });
    clear_err();
    let snap: RunnerSnapshot =
        ffi_try!(serde_json::from_str(text).map_err(|e| format!("session JSON: {e}")));
    match runner.restore(snap) {
        Ok(()) => BUBBLES_OK,
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}
