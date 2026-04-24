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

/** bubbles_runner_new_with_saliency / bubbles_runner_set_saliency */
#define BUBBLES_SALIENCY_FIRST_AVAILABLE 0
#define BUBBLES_SALIENCY_BLRV 1
#define BUBBLES_SALIENCY_RANDOM_AVAILABLE 2

typedef struct BubblesSourceFile {
    const char *path_ptr;
    size_t path_len;
    const char *text_ptr;
    size_t text_len;
} BubblesSourceFile;

/** Host function for bubbles_runner_register_function. */
typedef int32_t (*BubblesHostFn)(void *userdata, const char *args_json_ptr,
                                 size_t args_json_len, char **out_result_json);

uint32_t bubbles_abi_version(void);

/** NUL-terminated UTF-8; valid until the next bubbles_* call on this thread. May be NULL. */
const char *bubbles_last_error(void);

/** Free strings returned by this library (e.g. event JSON from bubbles_runner_next_event). */
void bubbles_string_free(char *p);

/** Copy UTF-8 bytes into a library-owned NUL-terminated string; free with bubbles_string_free. */
char *bubbles_copy_utf8(const char *ptr, size_t len);

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

/** Before bubbles_runner_new: 1 if node exists, else 0. */
int32_t bubbles_program_node_exists(void *program, const char *node_ptr, size_t node_len,
                                      int32_t *out_exists);

/** JSON array of node title strings; free with bubbles_string_free. */
int32_t bubbles_program_node_titles_json(void *program, char **out_json);

/** JSON string array of tags, or JSON null; free with bubbles_string_free. */
int32_t bubbles_program_node_tags_json(void *program, const char *title_ptr, size_t title_len,
                                       char **out_json);

/** JSON array of {"name":"$x","default_src":"..."}; free with bubbles_string_free. */
int32_t bubbles_program_variable_declarations_json(void *program, char **out_json);

/**
 * Create a runner (default saliency: FIRST_AVAILABLE). Consumes program; do not free the program afterward.
 */
int32_t bubbles_runner_new(void *program, void **out_runner);

/** Create a runner with BUBBLES_SALIENCY_*. Consumes program. */
int32_t bubbles_runner_new_with_saliency(void *program, int32_t saliency_kind, void **out_runner);

void bubbles_runner_free(void *runner);

int32_t bubbles_runner_set_saliency(void *runner, int32_t saliency_kind);

/** Merge JSON object of line_id -> template strings into HashMapProvider. Prefer before bubbles_runner_start. */
int32_t bubbles_runner_set_locale_json(void *runner, const char *json_ptr, size_t json_len);

int32_t bubbles_runner_register_function(void *runner, const char *name_ptr, size_t name_len,
                                         BubblesHostFn cb, void *userdata);

/** Start at node_name (UTF-8, byte length). */
int32_t bubbles_runner_start(void *runner, const char *node_ptr, size_t node_len);

/**
 * Next event. Returns BUBBLES_OK and sets *out_event_json to a malloc'd NUL-terminated string,
 * BUBBLES_DONE if finished (*out_event_json = NULL), or BUBBLES_ERR.
 */
int32_t bubbles_runner_next_event(void *runner, char **out_event_json);

int32_t bubbles_runner_select_option(void *runner, size_t index);

/** Variable as JSON value or null; free with bubbles_string_free. */
int32_t bubbles_runner_variable_get_json(void *runner, const char *name_ptr, size_t name_len,
                                         char **out_json);

/** Set variable from JSON value (bool, number, or string). */
int32_t bubbles_runner_variable_set_json(void *runner, const char *name_ptr, size_t name_len,
                                         const char *value_json_ptr, size_t value_json_len);

/** RunnerSnapshot JSON (not storage); free with bubbles_string_free. */
int32_t bubbles_runner_snapshot_session_json(void *runner, char **out_json);

/** HashMapStorage JSON; free with bubbles_string_free. */
int32_t bubbles_runner_snapshot_storage_json(void *runner, char **out_json);

/** Restore storage first when loading; json from bubbles_runner_snapshot_storage_json. */
int32_t bubbles_runner_restore_storage_json(void *runner, const char *json_ptr, size_t json_len);

/** Then restore session; json from bubbles_runner_snapshot_session_json. */
int32_t bubbles_runner_restore_session_json(void *runner, const char *json_ptr, size_t json_len);

#ifdef __cplusplus
}
#endif

#endif /* BUBBLES_FFI_H */
