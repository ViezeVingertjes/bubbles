/**
 * C ABI for bubbles-dialogue (Unity / P/Invoke, other native hosts).
 *
 * All strings are UTF-8. Lengths are byte counts (not including a trailing NUL).
 * Call from a single thread unless you add your own synchronization.
 *
 * Build the shared library:
 *   cargo build -p bubbles-ffi --release
 *
 * Linux:   target/release/libbubbles_ffi.so
 * macOS:   target/release/libbubbles_ffi.dylib
 * Windows: target/release/bubbles_ffi.dll
 */

#ifndef BUBBLES_FFI_H
#define BUBBLES_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Success: an event JSON string was produced (see bubbles_runner_next_event). */
#define BUBBLES_OK 0
/** bubbles_runner_next_event: dialogue finished (no more events). */
#define BUBBLES_DONE 1
/** Error: use bubbles_last_error() for a UTF-8 message. */
#define BUBBLES_ERR (-1)

typedef struct BubblesSourceFile {
    const char *path_ptr;
    size_t path_len;
    const char *text_ptr;
    size_t text_len;
} BubblesSourceFile;

uint32_t bubbles_abi_version(void);

/** NUL-terminated UTF-8; valid until the next bubbles_* call on this thread. May be NULL. */
const char *bubbles_last_error(void);

/** Free strings returned by this library (e.g. event JSON from bubbles_runner_next_event). */
void bubbles_string_free(char *p);

/**
 * Compile one .bub document. On success, writes a program handle to *out_program.
 * On failure, leaves *out_program unchanged.
 */
int32_t bubbles_compile(const char *text_ptr, size_t text_len, void **out_program);

/**
 * Merge multiple .bub sources (same as Rust compile_many). Paths are used in diagnostics only.
 */
int32_t bubbles_compile_files(const BubblesSourceFile *files, size_t file_count, void **out_program);

/** Free a program from bubbles_compile / bubbles_compile_files. Do not call after bubbles_runner_new. */
void bubbles_program_free(void *program);

/**
 * Create a runner. Consumes program; do not free the program afterward.
 */
int32_t bubbles_runner_new(void *program, void **out_runner);

void bubbles_runner_free(void *runner);

/** Start at node_name (UTF-8, byte length). */
int32_t bubbles_runner_start(void *runner, const char *node_ptr, size_t node_len);

/**
 * Next event. Returns BUBBLES_OK and sets *out_event_json to a malloc'd NUL-terminated string,
 * BUBBLES_DONE if finished (*out_event_json = NULL), or BUBBLES_ERR.
 */
int32_t bubbles_runner_next_event(void *runner, char **out_event_json);

int32_t bubbles_runner_select_option(void *runner, size_t index);

#ifdef __cplusplus
}
#endif

#endif /* BUBBLES_FFI_H */
