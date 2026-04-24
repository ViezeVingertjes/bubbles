//! Thread-local last error and string free helper.

use std::cell::RefCell;
use std::ffi::{CString, c_char};
use std::ptr;

thread_local! {
    static LAST_ERR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub(crate) fn set_err(msg: impl Into<String>) {
    LAST_ERR.with(|c| {
        let msg = msg.into();
        let cs = CString::new(msg)
            .unwrap_or_else(|_| CString::new("error message contained NUL").unwrap());
        *c.borrow_mut() = Some(cs);
    });
}

pub(crate) fn clear_err() {
    LAST_ERR.with(|c| *c.borrow_mut() = None);
}

/// Returns a pointer to the last error message (NUL-terminated UTF-8), or null if none.
/// Valid until the next call into this library on the **same thread**.
#[unsafe(no_mangle)]
pub extern "C" fn bubbles_last_error() -> *const c_char {
    LAST_ERR.with(|c| c.borrow().as_ref().map_or(ptr::null(), |s| s.as_ptr()))
}

/// Frees a string returned by this library (for example from `bubbles_runner_next_event`).
///
/// # Safety
///
/// `p` must be null or a pointer previously returned by this library and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_string_free(p: *mut c_char) {
    if p.is_null() {
        return;
    }
    drop(unsafe { CString::from_raw(p) });
}
