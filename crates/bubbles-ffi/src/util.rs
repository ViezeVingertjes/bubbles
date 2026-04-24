//! UTF-8 slices from raw FFI pointers.

use std::ffi::c_char;

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
