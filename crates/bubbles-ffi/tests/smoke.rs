//! Integration smoke test for the C ABI (via the `bubbles_ffi` rlib).

use std::ffi::{CStr, CString, c_char, c_void};

use bubbles_ffi::{
    BUBBLES_DONE, BUBBLES_ERR, BUBBLES_OK, bubbles_compile, bubbles_runner_free,
    bubbles_runner_new, bubbles_runner_next_event, bubbles_runner_start, bubbles_string_free,
};

const SCRIPT: &str = r"title: Start
---
Alice: Hi
===
";

#[test]
fn compile_start_and_drain_events() {
    let mut program: *mut c_void = std::ptr::null_mut();
    let text = CString::new(SCRIPT).expect("script has no interior NUL");
    let rc = unsafe {
        bubbles_compile(
            text.as_ptr(),
            text.as_bytes().len(),
            &mut program as *mut *mut c_void,
        )
    };
    assert_eq!(rc, BUBBLES_OK, "compile failed");
    assert!(!program.is_null());

    let mut runner: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { bubbles_runner_new(program, &mut runner as *mut *mut c_void) };
    assert_eq!(rc, BUBBLES_OK, "runner_new failed");
    assert!(!runner.is_null());

    let start = CString::new("Start").unwrap();
    let rc = unsafe { bubbles_runner_start(runner, start.as_ptr(), start.as_bytes().len()) };
    assert_eq!(rc, BUBBLES_OK, "start failed");

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
