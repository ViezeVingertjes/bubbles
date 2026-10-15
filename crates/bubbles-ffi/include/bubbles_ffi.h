/**
 * bubbles_ffi.h - C ABI for bubbles-dialogue.
 *
 * Drive `.bub` dialogue from Unity, Godot .NET, C, C++, or any host that can load a
 * native library and call a C function.
 *
 * ABI version: 1
 * Library version: 2.0.0
 *
 * The full stability contract - what you may rely on across releases, who frees what,
 * and the event JSON schema - is in ABI.md, next to this header:
 * https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/ABI.md
 *
 * Quick rules:
 *   - All text is UTF-8. Inputs are a pointer plus a BYTE LENGTH, never required to be
 *     NUL-terminated. Outputs are NUL-terminated.
 *   - Every string the library returns is freed with bubbles_string_free(), never free().
 *   - Status is BUBBLES_OK / BUBBLES_DONE / BUBBLES_ERR. BUBBLES_DONE is a success.
 *     On BUBBLES_ERR, out-parameters are not written; call bubbles_last_error().
 *   - Not thread-safe. Drive one runner from one thread.
 *   - Check bubbles_abi_version() == BUBBLES_ABI_VERSION at startup.
 *
 * Build the shared library:
 *   cargo build -p bubbles-ffi --release
 *
 * Linux:   target/release/libbubbles_ffi.so
 * macOS:   target/release/libbubbles_ffi.dylib
 * Windows: target/release/bubbles_ffi.dll
 *
 * Prebuilt binaries for Windows x64, macOS x64, macOS arm64, and Linux x64 are attached
 * to GitHub releases.
 *
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#ifndef BUBBLES_FFI_H
#define BUBBLES_FFI_H

#include <stddef.h>
#include <stdint.h>

/**
 * Version of the C ABI contract described in ABI.md.
 *
 * Bumped only for a breaking change to this header's contract, independently of the
 * library version below. Compare against bubbles_abi_version() at startup and refuse to
 * run on a mismatch.
 */
#define BUBBLES_ABI_VERSION 1

/** Version of the bubbles-ffi package this header shipped with. */
#define BUBBLES_FFI_VERSION_MAJOR 2
#define BUBBLES_FFI_VERSION_MINOR 0
#define BUBBLES_FFI_VERSION_PATCH 0
#define BUBBLES_FFI_VERSION_STRING "2.0.0"

#ifdef __cplusplus
extern "C" {
#endif

/* ── Status codes ──────────────────────────────────────────────────────────────
 * These three values are frozen for the lifetime of the ABI version. No function
 * returns any other value. Compare against the macros: BUBBLES_DONE is a success,
 * so `rc != BUBBLES_OK` is not an error test.
 */

/** Success. Any out-parameter has been written. */
#define BUBBLES_OK 0
/** bubbles_runner_next_event only: dialogue finished, *out_event_json set to NULL. */
#define BUBBLES_DONE 1
/** Failure. Out-parameters are untouched; call bubbles_last_error() for a message. */
#define BUBBLES_ERR (-1)

/* ── Saliency strategies ───────────────────────────────────────────────────────
 * For bubbles_runner_new_with_saliency / bubbles_runner_set_saliency. These values
 * are frozen; an unrecognised value returns BUBBLES_ERR rather than silently
 * falling back to a default.
 */

/** Pick the first candidate whose guard passes. The default. */
#define BUBBLES_SALIENCY_FIRST_AVAILABLE 0
/** Best least-recently-viewed: prefer high-condition-count, least recently seen. */
#define BUBBLES_SALIENCY_BLRV 1
/** Uniformly random among available candidates. */
#define BUBBLES_SALIENCY_RANDOM_AVAILABLE 2

/* ── Types ─────────────────────────────────────────────────────────────────────*/

/**
 * One source file for bubbles_compile_files.
 *
 * Layout is stable: no field is reordered, retyped, removed, or added. `path` is used
 * only in diagnostics, so compile errors point at the right file; it need not exist on
 * disk. Both pointer/length pairs are UTF-8 and are borrowed for the duration of the
 * call only.
 */
typedef struct BubblesSourceFile {
    const char *path_ptr;
    size_t      path_len;
    const char *text_ptr;
    size_t      text_len;
} BubblesSourceFile;

/**
 * Host function callback for bubbles_runner_register_function.
 *
 * Called when a `.bub` expression invokes the registered name.
 *
 * @param userdata        the pointer passed at registration, verbatim.
 * @param args_json_ptr   JSON array of dialogue Values, e.g. `[{"Number":41.0}]`.
 *                        UTF-8, NOT NUL-terminated - use args_json_len.
 * @param args_json_len   byte length of args_json_ptr.
 * @param out_result_json MUST be set, on every path, to a string allocated with
 *                        bubbles_copy_utf8(). The library frees it with its own
 *                        allocator, so any other allocation is undefined behaviour,
 *                        and leaving it NULL is reported as a function error.
 *                        On BUBBLES_OK: a JSON dialogue Value (`42`, `"text"`, `true`,
 *                        or the tagged form). On BUBBLES_ERR: a UTF-8 error message.
 * @return BUBBLES_OK or BUBBLES_ERR.
 *
 * Do NOT call any bubbles_* function from inside this callback; the runner is mutably
 * borrowed for its duration. bubbles_copy_utf8 is the one exception and is safe.
 */
typedef int32_t (*BubblesHostFn)(void *userdata, const char *args_json_ptr,
                                 size_t args_json_len, char **out_result_json);

/* ── Version handshake ─────────────────────────────────────────────────────────*/

/**
 * Returns the ABI version of the loaded library. Compare with BUBBLES_ABI_VERSION.
 *
 * The one symbol guaranteed to exist, keep its signature, and be safe to call before
 * anything else in every release that ships this ABI.
 */
uint32_t bubbles_abi_version(void);

/* ── Errors and strings ────────────────────────────────────────────────────────*/

/**
 * Last error on the CALLING THREAD as a NUL-terminated UTF-8 string, or NULL if none.
 *
 * Valid only until the next bubbles_* call on this thread - copy it if you need to keep
 * it. Library-owned: do not free. The message text is a diagnostic for logs and is not
 * stable; never parse it. Only meaningful immediately after a BUBBLES_ERR return.
 */
const char *bubbles_last_error(void);

/**
 * Frees any string this library returned (event JSON, variable JSON, snapshots, ...).
 *
 * Use this and nothing else - not free(), not Marshal.FreeHGlobal. NULL is a no-op.
 */
void bubbles_string_free(char *p);

/**
 * Copies `len` UTF-8 bytes into a library-owned NUL-terminated string.
 *
 * Returns NULL on invalid UTF-8 or an interior NUL, with the reason in
 * bubbles_last_error(). This is the required allocator for BubblesHostFn results; for
 * those, the library takes ownership and frees it. If you call it for any other reason,
 * free the result with bubbles_string_free.
 */
char *bubbles_copy_utf8(const char *ptr, size_t len);

/* ── Compile ───────────────────────────────────────────────────────────────────*/

/**
 * Compiles one .bub document. On BUBBLES_OK, writes an opaque program handle to
 * *out_program. On BUBBLES_ERR, *out_program is left unchanged.
 */
int32_t bubbles_compile(const char *text_ptr, size_t text_len, void **out_program);

/**
 * Compiles and merges several .bub sources into one program, so nodes in one file can
 * jump to nodes in another. Equivalent to the Rust `compile_many`.
 *
 * `file_count` must be non-zero.
 */
int32_t bubbles_compile_files(const BubblesSourceFile *files, size_t file_count,
                              void **out_program);

/**
 * Frees a program handle from bubbles_compile / bubbles_compile_files. NULL is a no-op.
 *
 * Do NOT call this after passing the handle to any bubbles_runner_new* constructor -
 * those consume it.
 */
void bubbles_program_free(void *program);

/* ── Program introspection ─────────────────────────────────────────────────────
 * Valid only before the program is consumed by a runner constructor.
 */

/** Writes 1 to *out_exists if a node with this title exists, else 0. */
int32_t bubbles_program_node_exists(void *program, const char *node_ptr, size_t node_len,
                                    int32_t *out_exists);

/** JSON array of node title strings. Free *out_json with bubbles_string_free. */
int32_t bubbles_program_node_titles_json(void *program, char **out_json);

/**
 * Tags of the first node with this title, as a JSON array of strings, or the JSON
 * literal `null` if the node is unknown. Free *out_json with bubbles_string_free.
 */
int32_t bubbles_program_node_tags_json(void *program, const char *title_ptr,
                                       size_t title_len, char **out_json);

/**
 * All `<<declare>>` entries as JSON `[{"name":"$x","default_src":"0"}, ...]`.
 * Free *out_json with bubbles_string_free.
 */
int32_t bubbles_program_variable_declarations_json(void *program, char **out_json);

/* ── Runner lifecycle ──────────────────────────────────────────────────────────*/

/**
 * Creates a runner using BUBBLES_SALIENCY_FIRST_AVAILABLE.
 *
 * CONSUMES `program`: after this call the program handle is gone, on success and on
 * failure alike. Do not free it and do not reuse it.
 *
 * Randomness is seeded from the OS. For a reproducible session use
 * bubbles_runner_new_seeded.
 */
int32_t bubbles_runner_new(void *program, void **out_runner);

/**
 * Creates a runner with an explicit BUBBLES_SALIENCY_* strategy.
 *
 * CONSUMES `program`, exactly as bubbles_runner_new does.
 */
int32_t bubbles_runner_new_with_saliency(void *program, int32_t saliency_kind,
                                         void **out_runner);

/**
 * Creates a runner whose randomness is seeded, making the session reproducible.
 *
 * Same program, same starting variables, same `seed`, same option selections gives the
 * same sequence of events - every random(), random_range() and dice() result and every
 * BUBBLES_SALIENCY_RANDOM_AVAILABLE choice - in every process and on every platform.
 * Pass BUBBLES_SALIENCY_FIRST_AVAILABLE for the default strategy.
 *
 * There is no set_seed counterpart: a seed belongs to a session, so choose it here. A
 * snapshot records where the stream had reached, so a restored save continues it.
 *
 * CONSUMES `program`, exactly as bubbles_runner_new does.
 */
int32_t bubbles_runner_new_seeded(void *program, int32_t saliency_kind, uint64_t seed,
                                  void **out_runner);

/** Frees a runner handle. NULL is a no-op. Always safe, including after an error. */
void bubbles_runner_free(void *runner);

/** Replaces the saliency strategy. Prefer calling before bubbles_runner_start. */
int32_t bubbles_runner_set_saliency(void *runner, int32_t saliency_kind);

/**
 * Installs line localisation from a JSON object of `{"line_id": "template", ...}`,
 * replacing any previously set table. Call before bubbles_runner_start.
 */
int32_t bubbles_runner_set_locale_json(void *runner, const char *json_ptr, size_t json_len);

/**
 * Registers a host function callable from `.bub` expressions as `name(...)`.
 *
 * `userdata` is stored and passed back to `cb` unchanged; it must outlive the runner.
 * Registering the same name twice replaces the first callback.
 */
int32_t bubbles_runner_register_function(void *runner, const char *name_ptr, size_t name_len,
                                         BubblesHostFn cb, void *userdata);

/* ── Dialogue loop ─────────────────────────────────────────────────────────────*/

/** Starts dialogue at the node titled `node_ptr` (UTF-8, byte length). */
int32_t bubbles_runner_start(void *runner, const char *node_ptr, size_t node_len);

/**
 * Pumps one event.
 *
 * BUBBLES_OK:   *out_event_json is a library-owned NUL-terminated UTF-8 JSON object.
 *               Free it with bubbles_string_free (not free()). Every event has a string
 *               `kind` field; see ABI.md for the schema. Ignore kinds you do not know,
 *               including "Unknown".
 * BUBBLES_DONE: dialogue finished; *out_event_json is set to NULL. Stop looping.
 * BUBBLES_ERR:  the script faulted. Stop pumping this dialogue and free the runner.
 *
 * After an Options event the runner waits: call bubbles_runner_select_option before the
 * next bubbles_runner_next_event.
 */
int32_t bubbles_runner_next_event(void *runner, char **out_event_json);

/**
 * Selects an option by zero-based index into the `options` array of the most recent
 * Options event.
 *
 * Returns BUBBLES_ERR if no Options event is pending, if the index is out of range, or
 * if that option's `available` field was false. All three leave the runner usable.
 */
int32_t bubbles_runner_select_option(void *runner, size_t index);

/* ── Variables ─────────────────────────────────────────────────────────────────*/

/**
 * Reads a variable (name includes the `$`) as a JSON dialogue Value in tagged form -
 * `{"Number":42.0}`, `{"Text":"sword"}`, `{"Bool":true}` - or the JSON literal `null`
 * if unset. Free *out_json with bubbles_string_free.
 */
int32_t bubbles_runner_variable_get_json(void *runner, const char *name_ptr, size_t name_len,
                                         char **out_json);

/**
 * Writes a variable from a JSON dialogue Value. Accepts both plain scalars (`42`,
 * `"sword"`, `true`) and the tagged form (`{"Number":42}`, `{"Text":"sword"}`,
 * `{"Bool":true}`). Rejects `null`, arrays, and unrecognised objects.
 */
int32_t bubbles_runner_variable_set_json(void *runner, const char *name_ptr, size_t name_len,
                                         const char *value_json_ptr, size_t value_json_len);

/* ── Save and load ─────────────────────────────────────────────────────────────
 * Treat both snapshot strings as OPAQUE blobs: store them, hand them back, do not
 * inspect or synthesise them. A full save is both snapshots; restore storage first,
 * then session.
 */

/** Session state (position, visit counts, once-flags). Free with bubbles_string_free. */
int32_t bubbles_runner_snapshot_session_json(void *runner, char **out_json);

/** Variable storage. Free with bubbles_string_free. */
int32_t bubbles_runner_snapshot_storage_json(void *runner, char **out_json);

/** Restores storage. Call this BEFORE bubbles_runner_restore_session_json. */
int32_t bubbles_runner_restore_storage_json(void *runner, const char *json_ptr, size_t json_len);

/** Restores session state. Call this AFTER bubbles_runner_restore_storage_json. */
int32_t bubbles_runner_restore_session_json(void *runner, const char *json_ptr, size_t json_len);

#ifdef __cplusplus
}
#endif

#endif /* BUBBLES_FFI_H */
