//! Introspection on a compiled [`Program`] before it is consumed by [`crate::bubbles_runner_new`].

use std::ffi::{CString, c_char, c_int, c_void};

use bubbles::Program;
use serde_json::json;

use crate::error::{clear_err, set_err};
use crate::util::str_from_parts;
use crate::{BUBBLES_ERR, BUBBLES_OK};

unsafe fn program_ref<'a>(program: *mut c_void) -> Option<&'a Program> {
    if program.is_null() {
        None
    } else {
        Some(unsafe { &*program.cast::<Program>() })
    }
}

/// Returns whether `node_name` exists. Writes `0` or `1` to `*out_exists`.
///
/// # Safety
///
/// `program` must be a valid program handle. `out_exists` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_program_node_exists(
    program: *mut c_void,
    node_ptr: *const c_char,
    node_len: usize,
    out_exists: *mut c_int,
) -> c_int {
    if out_exists.is_null() {
        set_err("out_exists was null");
        return BUBBLES_ERR;
    }
    let Some(prog) = (unsafe { program_ref(program) }) else {
        set_err("program was null");
        return BUBBLES_ERR;
    };
    let name = match unsafe { str_from_parts(node_ptr, node_len) } {
        Ok(s) => s,
        Err(e) => {
            set_err(e);
            return BUBBLES_ERR;
        }
    };
    clear_err();
    unsafe {
        *out_exists = if prog.node_exists(name) { 1 } else { 0 };
    }
    BUBBLES_OK
}

/// Writes a newly allocated JSON array of node title strings to `*out_json` (free with
/// [`crate::bubbles_string_free`]).
///
/// # Safety
///
/// `program` and `out_json` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_program_node_titles_json(
    program: *mut c_void,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let Some(prog) = (unsafe { program_ref(program) }) else {
        set_err("program was null");
        return BUBBLES_ERR;
    };
    clear_err();
    let titles: Vec<&str> = prog.node_titles().collect();
    let j = serde_json::to_string(&titles).unwrap_or_else(|_| "[]".into());
    write_cstring_out(out_json, j)
}

/// Tags for the first node with `title`, as a JSON string array, or `"null"` if unknown.
///
/// # Safety
///
/// See other program FFI functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_program_node_tags_json(
    program: *mut c_void,
    title_ptr: *const c_char,
    title_len: usize,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let Some(prog) = (unsafe { program_ref(program) }) else {
        set_err("program was null");
        return BUBBLES_ERR;
    };
    let title = match unsafe { str_from_parts(title_ptr, title_len) } {
        Ok(s) => s,
        Err(e) => {
            set_err(e);
            return BUBBLES_ERR;
        }
    };
    clear_err();
    let j = match prog.node_tags(title) {
        Some(tags) => serde_json::to_string(tags).unwrap_or_else(|_| "null".into()),
        None => "null".into(),
    };
    write_cstring_out(out_json, j)
}

/// All `<<declare>>` entries as JSON `[{"name":"$x","default_src":"0"}, ...]`.
///
/// # Safety
///
/// See other program FFI functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bubbles_program_variable_declarations_json(
    program: *mut c_void,
    out_json: *mut *mut c_char,
) -> c_int {
    if out_json.is_null() {
        set_err("out_json was null");
        return BUBBLES_ERR;
    }
    let Some(prog) = (unsafe { program_ref(program) }) else {
        set_err("program was null");
        return BUBBLES_ERR;
    };
    clear_err();
    let decls: Vec<_> = prog
        .variable_declarations()
        .iter()
        .map(|d| {
            json!({
                "name": d.name,
                "default_src": d.default_src,
            })
        })
        .collect();
    let j = serde_json::to_string(&decls).unwrap_or_else(|_| "[]".into());
    write_cstring_out(out_json, j)
}

fn write_cstring_out(out: *mut *mut c_char, s: String) -> c_int {
    let cs = match CString::new(s) {
        Ok(c) => c,
        Err(_) => {
            set_err("JSON contained interior NUL");
            return BUBBLES_ERR;
        }
    };
    unsafe {
        *out = cs.into_raw();
    }
    BUBBLES_OK
}
