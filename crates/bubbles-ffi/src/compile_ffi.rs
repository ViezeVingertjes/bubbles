//! Compile and program lifetime.

use std::ffi::{c_char, c_int, c_void};

use bubbles::{Program, compile, compile_many};

use crate::BubblesSourceFile;
use crate::error::{clear_err, set_err};
use crate::util::str_from_parts;
use crate::{BUBBLES_ERR, BUBBLES_OK};

/// Compile a single `.bub` document. On success writes a program handle to `*out_program`.
///
/// # Safety
///
/// `text_ptr` / `text_len` must point to valid UTF-8 for the duration of the call. `out_program`
/// must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_compile(
    text_ptr: *const c_char,
    text_len: usize,
    out_program: *mut *mut c_void,
) -> c_int {
    if out_program.is_null() {
        set_err("out_program was null");
        return BUBBLES_ERR;
    }
    let text = match unsafe { str_from_parts(text_ptr, text_len) } {
        Ok(s) => s,
        Err(e) => {
            set_err(e);
            return BUBBLES_ERR;
        }
    };
    clear_err();
    match compile(text) {
        Ok(program) => {
            let raw = Box::into_raw(Box::new(program)).cast::<c_void>();
            unsafe {
                *out_program = raw;
            }
            BUBBLES_OK
        }
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}

/// Compile multiple `.bub` sources into one program (same as Rust [`bubbles::compile_many`]).
///
/// # Safety
///
/// `files` must point to `file_count` valid [`BubblesSourceFile`] structs. Each pointer/length pair
/// must be valid UTF-8 for the call. `out_program` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_compile_files(
    files: *const BubblesSourceFile,
    file_count: usize,
    out_program: *mut *mut c_void,
) -> c_int {
    if out_program.is_null() {
        set_err("out_program was null");
        return BUBBLES_ERR;
    }
    if file_count == 0 {
        set_err("file_count was zero");
        return BUBBLES_ERR;
    }
    if files.is_null() {
        set_err("files was null");
        return BUBBLES_ERR;
    }

    let mut owned: Vec<(String, String)> = Vec::with_capacity(file_count);
    for i in 0..file_count {
        let f = unsafe { &*files.add(i) };
        let path = match unsafe { str_from_parts(f.path_ptr, f.path_len) } {
            Ok(s) => s,
            Err(e) => {
                set_err(format!("file {i} path: {e}"));
                return BUBBLES_ERR;
            }
        };
        let text = match unsafe { str_from_parts(f.text_ptr, f.text_len) } {
            Ok(s) => s,
            Err(e) => {
                set_err(format!("file {i} ({path}) body: {e}"));
                return BUBBLES_ERR;
            }
        };
        owned.push((path.to_owned(), text.to_owned()));
    }

    let refs: Vec<(&str, &str)> = owned
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();

    clear_err();
    match compile_many(&refs) {
        Ok(program) => {
            let raw = Box::into_raw(Box::new(program)).cast::<c_void>();
            unsafe {
                *out_program = raw;
            }
            BUBBLES_OK
        }
        Err(e) => {
            set_err(e.to_string());
            BUBBLES_ERR
        }
    }
}

/// Drops a program obtained from [`bubbles_compile`] or [`bubbles_compile_files`].
/// Must **not** be called after the program was consumed by [`bubbles_runner_new`](crate::bubbles_runner_new).
///
/// # Safety
///
/// `program` must be null or a valid program handle not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_program_free(program: *mut c_void) {
    if program.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(program.cast::<Program>()) });
}
