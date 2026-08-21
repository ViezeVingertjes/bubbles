//! Shared FFI helpers: UTF-8 slices from raw pointers and C string output.

use std::ffi::{CString, c_char, c_int, c_void};

use bubbles::{HashMapStorage, Program, Runner};

use crate::error::set_err;
use crate::{BUBBLES_ERR, BUBBLES_OK};

/// The concrete runner type behind every `runner` handle in this ABI.
pub(crate) type FfiRunner = Runner<HashMapStorage>;

/// Unwraps a `Result<T, impl Into<String>>` inside an FFI entry point: on `Err` the
/// thread-local error is set and the function returns `BUBBLES_ERR`.
macro_rules! ffi_try {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                $crate::error::set_err(e);
                return $crate::BUBBLES_ERR;
            }
        }
    };
}
pub(crate) use ffi_try;

/// Borrows the [`Program`] behind an opaque handle.
///
/// # Safety
///
/// `program` must be null or a live handle from `bubbles_compile*`.
pub(crate) unsafe fn program_ref<'a>(program: *mut c_void) -> Result<&'a Program, &'static str> {
    if program.is_null() {
        return Err("program was null");
    }
    Ok(unsafe { &*program.cast::<Program>() })
}

/// Borrows the runner behind an opaque handle.
///
/// # Safety
///
/// `runner` must be null or a live handle from `bubbles_runner_new*`.
pub(crate) unsafe fn runner_ref<'a>(runner: *mut c_void) -> Result<&'a FfiRunner, &'static str> {
    if runner.is_null() {
        return Err("runner was null");
    }
    Ok(unsafe { &*runner.cast::<FfiRunner>() })
}

/// Mutably borrows the runner behind an opaque handle.
///
/// # Safety
///
/// `runner` must be null or a live handle from `bubbles_runner_new*`, and no other
/// reference to it may be alive.
pub(crate) unsafe fn runner_mut<'a>(
    runner: *mut c_void,
) -> Result<&'a mut FfiRunner, &'static str> {
    if runner.is_null() {
        return Err("runner was null");
    }
    Ok(unsafe { &mut *runner.cast::<FfiRunner>() })
}

/// Boxes `value`, writes the raw handle to `*out`, and returns `BUBBLES_OK`.
///
/// # Safety
///
/// `out` must be non-null and writable.
pub(crate) unsafe fn write_handle_out<T>(out: *mut *mut c_void, value: T) -> c_int {
    unsafe {
        *out = Box::into_raw(Box::new(value)).cast::<c_void>();
    }
    BUBBLES_OK
}

/// Read UTF-8 from `ptr` / `len`. The borrow is valid only for the caller's memory lifetime.
///
/// # Safety
///
/// `ptr` + `len` must be valid for the duration of the returned `str` borrow.
pub(crate) unsafe fn str_from_parts<'a>(
    ptr: *const c_char,
    len: usize,
) -> Result<&'a str, &'static str> {
    if ptr.is_null() {
        return Err("null pointer");
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    std::str::from_utf8(slice).map_err(|_| "invalid utf-8")
}

/// Converts `s` into a NUL-terminated C string, writes the raw pointer to `*out`, and returns
/// `BUBBLES_OK`. Sets the thread-local error and returns `BUBBLES_ERR` if `s` contains an
/// interior NUL byte.
pub(crate) fn write_cstring_out(out: *mut *mut c_char, s: String) -> c_int {
    write_cstring_out_with_error(out, s, "string contained interior NUL")
}

pub(crate) fn write_cstring_out_with_error(
    out: *mut *mut c_char,
    s: String,
    interior_nul_error: &'static str,
) -> c_int {
    let cs = match CString::new(s) {
        Ok(c) => c,
        Err(_) => {
            set_err(interior_nul_error);
            return BUBBLES_ERR;
        }
    };
    unsafe {
        *out = cs.into_raw();
    }
    BUBBLES_OK
}
