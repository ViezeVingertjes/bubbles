//! Integration tests for the C ABI (via the `bubbles_ffi` rlib).

use std::ffi::{CStr, CString, c_char, c_int, c_void};

use bubbles_ffi::{
    BUBBLES_DONE, BUBBLES_ERR, BUBBLES_OK, BUBBLES_SALIENCY_BLRV, bubbles_compile,
    bubbles_copy_utf8, bubbles_program_node_exists, bubbles_program_node_titles_json,
    bubbles_program_variable_declarations_json, bubbles_runner_free, bubbles_runner_new,
    bubbles_runner_new_with_saliency, bubbles_runner_next_event, bubbles_runner_register_function,
    bubbles_runner_restore_session_json, bubbles_runner_restore_storage_json,
    bubbles_runner_set_locale_json, bubbles_runner_snapshot_session_json,
    bubbles_runner_snapshot_storage_json, bubbles_runner_start, bubbles_runner_variable_get_json,
    bubbles_runner_variable_set_json, bubbles_string_free,
};

const BASIC: &str = r"title: Start
---
Alice: Hi
===
";

const WITH_HOST: &str = r"title: T
---
<<set $n = add_one(41)>>
NPC: done
===
";

const WITH_LINE_ID: &str = r"title: L
---
Shopkeep: Hello #line:greet
===
";

#[test]
fn compile_start_and_drain_events() {
    let mut program: *mut c_void = std::ptr::null_mut();
    let text = CString::new(BASIC).unwrap();
    let rc = unsafe {
        bubbles_compile(
            text.as_ptr(),
            text.as_bytes().len(),
            &mut program as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK, "compile failed");
    assert!(!program.is_null());

    let mut exists: c_int = 0;
    let start = CString::new("Start").unwrap();
    let rc = unsafe {
        bubbles_program_node_exists(program, start.as_ptr(), start.as_bytes().len(), &mut exists)
    };
    assert_eq!(rc, BUBBLES_OK);
    assert_eq!(exists, 1);

    let mut titles_json: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { bubbles_program_node_titles_json(program, &mut titles_json) };
    assert_eq!(rc, BUBBLES_OK);
    let t = unsafe { CStr::from_ptr(titles_json) }.to_str().unwrap();
    assert!(t.contains("Start"));
    unsafe { bubbles_string_free(titles_json) };

    let mut decl_json: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { bubbles_program_variable_declarations_json(program, &mut decl_json) };
    assert_eq!(rc, BUBBLES_OK);
    let d = unsafe { CStr::from_ptr(decl_json) }.to_str().unwrap();
    assert_eq!(d, "[]");
    unsafe { bubbles_string_free(decl_json) };

    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_new(program, &mut runner as *mut *mut c_void) };
    assert_eq!(rc, BUBBLES_OK);
    assert!(!runner.is_null());

    let rc = unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK);

    let mut saw_line = false;
    loop {
        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &mut json_ptr as *mut *mut c_char) };
        match rc {
            x if x == BUBBLES_OK => {
                assert!(!json_ptr.is_null());
                let s = unsafe { CStr::from_ptr(json_ptr) }
                    .to_str()
                    .expect("utf-8 json");
                if s.contains("\"kind\":\"Line\"") {
                    saw_line = true;
                }
                unsafe { bubbles_string_free(json_ptr) };
            }
            x if x == BUBBLES_DONE => break,
            x if x == BUBBLES_ERR => panic!("next_event error"),
            x => panic!("unexpected status {x}"),
        }
    }
    assert!(saw_line, "expected a Line event");

    unsafe { bubbles_runner_free(runner) };
}

#[test]
fn saliency_blrv_locale_host_variables_and_snapshot() {
    let mut program: *mut c_void = std::ptr::null_mut();
    let src = CString::new(WITH_HOST).unwrap();
    let rc = unsafe {
        bubbles_compile(
            src.as_ptr(),
            src.as_bytes().len(),
            &mut program as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK);

    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe {
        bubbles_runner_new_with_saliency(
            program,
            BUBBLES_SALIENCY_BLRV,
            &mut runner as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK);

    let fname = CString::new("add_one").unwrap();
    let rc = unsafe {
        bubbles_runner_register_function(
            runner,
            fname.as_ptr(),
            fname.as_bytes().len(),
            host_add_one,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, BUBBLES_OK);

    let node = CString::new("T").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, node.as_ptr(), node.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK);

    drain_until_line_or_done(runner);

    let mut vjson: *mut c_char = std::ptr::null_mut();
    let key = CString::new("$n").unwrap();
    let rc = unsafe {
        bubbles_runner_variable_get_json(runner, key.as_ptr(), key.as_bytes().len(), &mut vjson)
    };
    assert_eq!(rc, BUBBLES_OK);
    let vs = unsafe { CStr::from_ptr(vjson) }.to_str().unwrap();
    assert_eq!(vs, "{\"Number\":42.0}");
    unsafe { bubbles_string_free(vjson) };

    let mut snap_s: *mut c_char = std::ptr::null_mut();
    let mut snap_st: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_snapshot_session_json(runner, &mut snap_s) };
    assert_eq!(rc, BUBBLES_OK);
    let rc = unsafe { bubbles_runner_snapshot_storage_json(runner, &mut snap_st) };
    assert_eq!(rc, BUBBLES_OK);
    let session_json = unsafe { CStr::from_ptr(snap_s) }
        .to_str()
        .unwrap()
        .to_owned();
    let storage_json = unsafe { CStr::from_ptr(snap_st) }
        .to_str()
        .unwrap()
        .to_owned();
    unsafe {
        bubbles_string_free(snap_s);
        bubbles_string_free(snap_st);
    }

    unsafe { bubbles_runner_free(runner) };

    let mut program2: *mut c_void = std::ptr::null_mut();
    let rc = unsafe {
        bubbles_compile(
            src.as_ptr(),
            src.as_bytes().len(),
            &mut program2 as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK);
    let mut runner2: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_new(program2, &mut runner2 as *mut *mut c_void) };
    assert_eq!(rc, BUBBLES_OK);

    let stor = CString::new(storage_json).unwrap();
    let rc = unsafe {
        bubbles_runner_restore_storage_json(runner2, stor.as_ptr(), stor.as_bytes().len())
    };
    assert_eq!(rc, BUBBLES_OK);
    let sess = CString::new(session_json).unwrap();
    let rc = unsafe {
        bubbles_runner_restore_session_json(runner2, sess.as_ptr(), sess.as_bytes().len())
    };
    assert_eq!(rc, BUBBLES_OK);

    let mut vjson2: *mut c_char = std::ptr::null_mut();
    let rc = unsafe {
        bubbles_runner_variable_get_json(runner2, key.as_ptr(), key.as_bytes().len(), &mut vjson2)
    };
    assert_eq!(rc, BUBBLES_OK);
    let vs2 = unsafe { CStr::from_ptr(vjson2) }.to_str().unwrap();
    assert_eq!(vs2, "{\"Number\":42.0}");
    unsafe { bubbles_string_free(vjson2) };

    unsafe { bubbles_runner_free(runner2) };
}

#[test]
fn locale_json_replaces_line_text() {
    let mut program: *mut c_void = std::ptr::null_mut();
    let src = CString::new(WITH_LINE_ID).unwrap();
    let rc = unsafe {
        bubbles_compile(
            src.as_ptr(),
            src.as_bytes().len(),
            &mut program as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK);

    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_new(program, &mut runner as *mut *mut c_void) };
    assert_eq!(rc, BUBBLES_OK);

    let loc = CString::new(r#"{"greet":"Salut"}"#).unwrap();
    let rc = unsafe { bubbles_runner_set_locale_json(runner, loc.as_ptr(), loc.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK);

    let node = CString::new("L").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, node.as_ptr(), node.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK);

    let mut saw_salut = false;
    loop {
        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &mut json_ptr as *mut *mut c_char) };
        match rc {
            x if x == BUBBLES_OK => {
                let s = unsafe { CStr::from_ptr(json_ptr) }.to_str().unwrap();
                if s.contains("Salut") {
                    saw_salut = true;
                }
                unsafe { bubbles_string_free(json_ptr) };
            }
            x if x == BUBBLES_DONE => break,
            _ => panic!("unexpected"),
        }
    }
    assert!(saw_salut);

    unsafe { bubbles_runner_free(runner) };
}

#[test]
fn variable_set_json_round_trip() {
    let mut program: *mut c_void = std::ptr::null_mut();
    let src = CString::new(BASIC).unwrap();
    unsafe {
        bubbles_compile(
            src.as_ptr(),
            src.as_bytes().len(),
            &mut program as *mut *mut c_void,
        )
    };
    let mut runner: *mut c_void = std::ptr::null_mut();
    unsafe { bubbles_runner_new(program, &mut runner as *mut *mut c_void) };
    let k = CString::new("$k").unwrap();
    let v = CString::new(r#""hello""#).unwrap();
    let rc = unsafe {
        bubbles_runner_variable_set_json(
            runner,
            k.as_ptr(),
            k.as_bytes().len(),
            v.as_ptr(),
            v.as_bytes().len(),
        )
    };
    assert_eq!(rc, BUBBLES_OK);
    let mut out: *mut c_char = std::ptr::null_mut();
    let rc = unsafe {
        bubbles_runner_variable_get_json(runner, k.as_ptr(), k.as_bytes().len(), &mut out)
    };
    assert_eq!(rc, BUBBLES_OK);
    let s = unsafe { CStr::from_ptr(out) }.to_str().unwrap();
    assert_eq!(s, r#"{"Text":"hello"}"#);
    unsafe {
        bubbles_string_free(out);
        bubbles_runner_free(runner);
    }
}

fn drain_until_line_or_done(runner: *mut c_void) {
    loop {
        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { bubbles_runner_next_event(runner, &mut json_ptr as *mut *mut c_char) };
        match rc {
            x if x == BUBBLES_OK => unsafe { bubbles_string_free(json_ptr) },
            x if x == BUBBLES_DONE => break,
            _ => panic!("next_event"),
        }
    }
}

unsafe extern "C" fn host_add_one(
    _userdata: *mut c_void,
    args_json_ptr: *const c_char,
    args_json_len: usize,
    out_result_json: *mut *mut c_char,
) -> c_int {
    let slice = unsafe { std::slice::from_raw_parts(args_json_ptr.cast::<u8>(), args_json_len) };
    let Ok(arr) = serde_json::from_slice::<Vec<serde_json::Value>>(slice) else {
        return BUBBLES_ERR;
    };
    let n = arr
        .first()
        .map(|v| {
            v.as_f64()
                .or_else(|| v.get("Number").and_then(serde_json::Value::as_f64))
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0)
        + 1.0;
    let s = format!("{n}");
    let Ok(cs) = CString::new(s) else {
        return BUBBLES_ERR;
    };
    let p = unsafe { bubbles_copy_utf8(cs.as_ptr(), cs.as_bytes().len()) };
    if p.is_null() {
        return BUBBLES_ERR;
    }
    unsafe {
        *out_result_json = p;
    }
    BUBBLES_OK
}
