//! `bubbles check` - compile a set of `.bub` files and report what the
//! compiler found.
//!
//! This is the half of the CLI that builds without the `tui` feature, so it
//! can be dropped into someone else's CI without dragging a terminal UI along
//! with it.
//!
//! All files handed to one invocation are compiled **together**, exactly as
//! the player compiles them, because that is what the language means by a
//! programme: node titles are merged across files and `<<jump>>` targets
//! resolve across them. Checking each file on its own would report broken
//! references that do not exist.

use bubbles::compile_many;

use crate::diagnostic::Diagnostic;
use crate::source_set::SourceSet;

/// The outcome of checking one source set.
#[derive(Debug, Clone)]
pub struct Report {
    /// How many files were compiled together.
    pub files: usize,
    /// Distinct node titles in the compiled programme; `0` when it failed.
    pub nodes: usize,
    /// What the compiler objected to.  Empty when the programme is valid.
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// `true` when nothing was wrong with the programme.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// The one-line summary printed on success (suppressed by `--quiet`).
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "ok: {} {}, {} {}",
            self.files,
            plural(self.files, "file", "files"),
            self.nodes,
            plural(self.nodes, "node", "nodes"),
        )
    }
}

/// Compiles `sources` as a single programme and reports the result.
///
/// [`compile_many`] is the whole check: parsing, cross-file merging, and the
/// `validate` pass over every `<<jump>>` and `<<detour>>` target all happen
/// inside it, and it stops at the first thing it cannot accept.
#[must_use]
pub fn check(sources: &SourceSet) -> Report {
    let slices = sources.as_named_slices();
    match compile_many(&slices) {
        Ok(program) => Report {
            files: sources.len(),
            nodes: program.node_titles().count(),
            diagnostics: Vec::new(),
        },
        Err(err) => Report {
            files: sources.len(),
            nodes: 0,
            diagnostics: vec![Diagnostic::from_error(&err, sources.source_for_error(&err))],
        },
    }
}

const fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}
