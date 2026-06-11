//! Shared FFI helpers: UTF-8 slices from raw pointers and C string output.

use std::ffi::{CString, c_char, c_int};

use crate::error::set_err;
use crate::{BUBBLES_ERR, BUBBLES_OK};

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
