//! C ABI for **bubbles-dialogue** so native hosts (Unity / C# / etc.) can drive dialogue via P/Invoke.
//!
//! # Contract
//!
//! - All text is **UTF-8**. Every string is passed as a pointer plus **byte length** (not
//!   including a null terminator). You may use null-terminated strings from C# by passing
//!   `strlen` or `Encoding.UTF8.GetByteCount`.
//! - Functions are **not thread-safe** unless documented otherwise. Call everything from one
//!   thread (Unity main thread is fine).
//! - Outputs allocated by this library are released with [`bubbles_string_free`].
//! - After [`bubbles_runner_new`], the program handle is consumed and must not be freed or reused.
//!
//! # JSON events
//!
//! [`bubbles_runner_next_event`] returns a JSON object per [`bubbles::DialogueEvent`]. The `kind`
//! field is always present: `NodeStarted`, `Line`, `Options`, `Command`, `NodeComplete`,
//! `DialogueComplete`, or `Unknown` if a future runtime adds variants this shim does not know yet.
//!
//! # Build
//!
//! ```text
//! cargo build -p bubbles-ffi --release
//! ```
//!
//! The shared library is `target/release/libbubbles_ffi.so` (Linux), `libbubbles_ffi.dylib` (macOS),
//! or `bubbles_ffi.dll` (Windows). Copy it next to your Unity project and `DllImport` it.
//!
//! A minimal .NET 10 smoke test lives under `tests/dotnet_smoke/` (run from that directory after
//! building the `cdylib`).

#![deny(missing_docs)]

mod compile_ffi;
mod error;
mod event_json;
mod runner_ffi;
mod util;

use std::ffi::c_char;

pub use compile_ffi::{bubbles_compile, bubbles_compile_files, bubbles_program_free};
pub use error::{bubbles_last_error, bubbles_string_free};
pub use runner_ffi::{
    bubbles_runner_free, bubbles_runner_new, bubbles_runner_next_event,
    bubbles_runner_select_option, bubbles_runner_start,
};

use std::ffi::c_int;

/// Return value: success and an event JSON string was written (see [`bubbles_runner_next_event`]).
pub const BUBBLES_OK: c_int = 0;
/// Return value: [`bubbles_runner_next_event`] has no more events (dialogue finished).
pub const BUBBLES_DONE: c_int = 1;
/// Return value: error; see [`bubbles_last_error`].
pub const BUBBLES_ERR: c_int = -1;

/// One source file for [`bubbles_compile_files`]. Text is UTF-8 bytes; `path` is only used in
/// diagnostics (errors point at the right file).
#[repr(C)]
pub struct BubblesSourceFile {
    /// UTF-8 path bytes (not necessarily NUL-terminated).
    pub path_ptr: *const c_char,
    /// Length of `path_ptr` in bytes.
    pub path_len: usize,
    /// UTF-8 `.bub` source bytes.
    pub text_ptr: *const c_char,
    /// Length of `text_ptr` in bytes.
    pub text_len: usize,
}

/// ABI version of this shim (bump when breaking the C API).
#[unsafe(no_mangle)]
pub extern "C" fn bubbles_abi_version() -> u32 {
    1
}
