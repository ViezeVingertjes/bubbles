//! Holds the library source to the determinism claims in `README.md`.
//!
//! The gap: "nothing in the runtime reads the clock or the environment, ever",
//! "it reaches for randomness on its own initiative exactly once" and "no global
//! or thread-local state" are claims about what the code *cannot* do, and about
//! how many places it can do it from. A behavioural test cannot see them
//! — a clock read or an ambient RNG draw produces a plausible value rather than
//! a failure, so a session that acquired one would keep passing every existing
//! test while quietly ceasing to be reproducible. The nondeterminism is only
//! visible from outside the process, or on a target where the facility is
//! missing entirely.
//!
//! That is not hypothetical. A clock read reaching `Runner::new` aborts every
//! runner construction on `wasm32-unknown-unknown`, because `std` has no clock
//! there; the build still succeeds, the panic is a runtime one, and the
//! optimiser is entitled to delete everything downstream of the diverging call.
//! These assertions fail at the commit that introduces the read instead.
//!
//! Scope is `crates/bubbles-dialogue/src` only. The claim is about the library,
//! and a workspace-wide assertion would be a different and weaker one:
//! `bubbles-ffi` and `bubbles-tui` are hosts, and a host is allowed a clock.
//! Inline `#[cfg(test)]` modules are excluded for the same reason — a test may
//! read whatever it needs to observe the thing it is testing.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// A pattern the library source must not contain, with the sentence it would
/// falsify. The claim text is in the failure message so that whoever hits this
/// sees what has to change in public documentation if the code is right and the
/// claim is now wrong.
struct Ban {
    pattern: &'static str,
    claim: &'static str,
    /// Paths, relative to `src/`, where the pattern is part of the design
    /// rather than a regression.
    allowed_in: &'static [&'static str],
}

/// Lines beginning with `//` are skipped, so prose that names a banned pattern
/// — including the rustdoc that explains why it is banned — does not trip this.
/// A trailing comment on a line of code is not stripped; reword it if one ever
/// collides.
fn offending_lines(source: &str, pattern: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter(|(_, line)| line.contains(pattern))
        .map(|(i, line)| (i + 1, line.trim().to_string()))
        .collect()
}

/// Every implementation file in the library crate, as (path relative to `src/`,
/// contents).
fn library_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect(&root, &root, &mut sources);
    sources.sort_by(|a, b| a.0.cmp(&b.0));
    sources
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect(root, &path, out);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let relative = relative_slash_path(root, &path);
        // The repository keeps unit tests in `*_tests.rs` files attached with
        // `#[path]`, plus one `library_tests` directory.
        if relative.ends_with("_tests.rs") || relative.contains("library_tests") {
            continue;
        }
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {relative}: {e}"));
        out.push((relative, source));
    }
}

fn relative_slash_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("path under src/")
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn assert_absent(bans: &[Ban]) {
    let sources = library_sources();
    // A walk that silently found nothing would pass every assertion below, so
    // the file count is part of the test. 40 files at the time of writing.
    assert!(
        sources.len() > 30,
        "only {} library source files found - the scan is not looking where it thinks it is",
        sources.len()
    );

    let mut failures = String::new();
    for ban in bans {
        for (path, source) in &sources {
            if ban.allowed_in.contains(&path.as_str()) {
                continue;
            }
            for (line_no, line) in offending_lines(source, ban.pattern) {
                let _ = write!(
                    failures,
                    "\nsrc/{path}:{line_no} contains `{}`\n    {line}\n  which contradicts: {}\n",
                    ban.pattern, ban.claim
                );
            }
        }
    }
    assert!(failures.is_empty(), "{failures}");
}

#[test]
fn the_library_never_reads_the_clock() {
    const CLAIM: &str =
        "README: \"Nothing in the runtime reads the clock or the environment, ever.\"";
    assert_absent(&[
        Ban {
            pattern: "SystemTime",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "Instant",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "std::time",
            claim: CLAIM,
            allowed_in: &[],
        },
    ]);
}

#[test]
fn the_library_never_reads_the_environment() {
    const CLAIM: &str =
        "README: \"Nothing in the runtime reads the clock or the environment, ever.\"";
    assert_absent(&[
        Ban {
            pattern: "std::env",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "env::var",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "env!(",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "option_env!(",
            claim: CLAIM,
            allowed_in: &[],
        },
    ]);
}

#[test]
fn the_library_holds_no_global_or_thread_local_state() {
    const CLAIM: &str = "README: \"no global or thread-local state, so a `Runner` is just a \
                         value you own and you can hold as many as you like\"";
    // `'static` bounds are everywhere and are not state; only the forms that
    // carry mutable or lazily-initialised process-wide data are banned.
    assert_absent(&[
        Ban {
            pattern: "thread_local!",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "static mut",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "OnceLock",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "OnceCell",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "lazy_static",
            claim: CLAIM,
            allowed_in: &[],
        },
        Ban {
            pattern: "Atomic",
            claim: CLAIM,
            allowed_in: &[],
        },
    ]);
}

#[test]
fn the_global_rng_is_reachable_only_through_what_a_script_opts_into() {
    // `random()`, `random_range()` and `dice()` live in the function registry a
    // script has to call into, and `RandomAvailable` has to be installed by the
    // host. Everywhere else - the compiler, the evaluator, the runner, saliency
    // bookkeeping - a draw from the thread RNG would be the runtime taking
    // nondeterminism on its own initiative, which is the half of the sentence
    // that is easiest to break by accident.
    assert_absent(&[Ban {
        pattern: "rand::",
        claim: "README: the only nondeterminism beyond the one starting seed is what the \
                script or host opts into",
        allowed_in: &[
            "library/rand.rs",
            "saliency/random.rs",
            // The one draw the README does claim, asserted to be the only one
            // by `the_runtime_reaches_for_entropy_from_exactly_one_place`.
            "runtime/rng.rs",
        ],
    }]);
}

/// The README quantifies the runtime's own randomness: *"It reaches for
/// randomness on its own initiative exactly once: an unseeded runner asks the
/// operating system for a starting seed as it is built."*
///
/// "Exactly once" is the load-bearing word, and it is the one a plain ban cannot
/// check — licensing `runtime/rng.rs` so the legitimate draw passes would also
/// license a second draw added beside it. Nothing else can catch that: a hidden
/// read of an unpredictable stream produces an unpredictable value either way,
/// so no behavioural assertion can count the draws from outside. This pins the
/// count instead, and fails whether a new site appears in this file or another.
#[test]
fn the_runtime_reaches_for_entropy_from_exactly_one_place() {
    // Where a draw is the script's or the host's choice rather than the
    // runtime's, and so is not what the sentence is counting.
    const OPT_IN: &[&str] = &["library/rand.rs", "saliency/random.rs"];

    // Occurrences, not lines: `rand::random() ^ rand::random()` is two draws,
    // and counting the lines it is spread over would call it one. Found that
    // the only way such a thing is ever found, by planting it.
    let sites: Vec<String> = library_sources()
        .iter()
        .filter(|(path, _)| !OPT_IN.contains(&path.as_str()))
        .flat_map(|(path, source)| {
            offending_lines(source, "rand::")
                .into_iter()
                .flat_map(move |(line_no, line)| {
                    let draws = line.matches("rand::").count();
                    std::iter::repeat_n(format!("src/{path}:{line_no}  {line}"), draws)
                })
        })
        .collect();

    assert_eq!(
        sites.len(),
        1,
        "the runtime draws from the global RNG in {} places, and the README says it does so \
         \"exactly once\". Either the new draw belongs behind something the caller opted into, \
         or the sentence has to be requantified:\n{}",
        sites.len(),
        sites.join("\n")
    );
}

/// A scanner that cannot see the thing it is looking for would report a clean
/// library forever. This plants each banned form and requires a hit, including
/// the two cases the line filter is allowed to ignore.
#[test]
fn the_scanner_finds_what_it_is_looking_for() {
    let planted = "\
use std::time::SystemTime;
fn now() -> SystemTime { SystemTime::now() }
thread_local! { static SEEN: u32 = 0; }
static mut COUNT: u32 = 0;
let v = std::env::var(\"HOME\");
let x = rand::rng().random::<f64>();
";
    for pattern in [
        "SystemTime",
        "std::time",
        "thread_local!",
        "static mut",
        "std::env",
        "rand::",
    ] {
        assert!(
            !offending_lines(planted, pattern).is_empty(),
            "the scan missed a planted `{pattern}`"
        );
    }

    // Prose about a banned form is not a use of it, and a `'static` bound is
    // not global state.
    let prose = "\
//! The runtime never reads SystemTime, and holds no thread_local! state.
/// Not `static mut` either, nor an OnceLock.
pub trait LineProvider: Send + Sync + 'static {}
";
    for pattern in ["SystemTime", "thread_local!", "static mut", "OnceLock"] {
        assert!(
            offending_lines(prose, pattern).is_empty(),
            "a comment naming `{pattern}` was treated as a use of it"
        );
    }
}
