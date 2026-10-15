//! Determinism properties that only a second process can observe.
//!
//! `tests/determinism.rs` compares runs inside one process, which is the right
//! instrument for everything except the two pieces of state a process shares
//! with itself: the hasher seed `std` installs once at startup, and the
//! operating-system entropy an unseeded runner falls back to. Both are fixed
//! for the lifetime of a process, so an in-process test cannot tell
//! *reproducible* apart from *identical because nothing varied*. Every test
//! here re-executes the test binary and compares separate processes.
//!
//! The gap is not hypothetical. While session state was held in `HashMap` and
//! `HashSet`, serialising one session twice in a single process produced
//! identical bytes, while two processes produced different ones, because key
//! order follows the per-process hash seed. The whole suite passed. A save file
//! is content-hashed, diffed, or deduplicated *across* processes, which is
//! exactly where that defect lived and the only place a test can see it.
//!
//! Each test spawns the child four times rather than twice: two runs agreeing
//! is weak evidence when the thing being ruled out is a per-process value, and
//! `an_unseeded_run_differs_between_processes` is the control that proves these
//! children really are distinct processes and that this harness can see the
//! difference when there is one.

use std::env;
use std::process::Command;

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

/// Names which payload a spawned child should print.
const MODE: &str = "BUBBLES_CROSS_PROCESS_MODE";
/// Marks a payload line in the child's stdout, so the test harness's own
/// output ("running 1 test", timings) cannot be mistaken for data.
const MARK: &str = "cross-process-payload:";
/// The child entry point, selected by exact name so the child runs nothing else.
const WORKER: &str = "the_child_worker_prints_one_payload";

/// How many separate processes each property compares.
const PROCESSES: usize = 4;

/// A script with no randomness whose output still depends on session state:
/// option selection, a conditional over a variable, a `visited()` check, a
/// `<<once>>` block, and a line group. Node titles are deliberately not in
/// alphabetical order, so sorted and insertion order differ.
const SESSION: &str = "\
title: Zulu
---
<<declare $tally = 0>>
<<declare $name = \"Aurelia\">>
<<once>>
    Keeper: First time only, {$name}.
<<endonce>>
<<set $tally = $tally + 1>>
=> Keeper: The tide is in.
=> Keeper: The tide is out.
<<jump Mike>>
===
title: Mike
---
<<set $tally = $tally + 2>>
<<if visited(\"Zulu\")>>
    Cooper: You came from the common room, tally {$tally}.
<<else>>
    Cooper: A stranger.
<<endif>>
<<once>>
    Cooper: And only once.
<<endonce>>
<<jump Alpha>>
===
title: Alpha
---
<<set $tally = $tally * 3>>
Clerk: Tally stands at {$tally}.
-> Ask about the harbour
    <<jump Bravo>>
-> Leave
    Clerk: Safe roads.
===
title: Bravo
---
<<once>>
    Clerk: The harbour, then.
<<endonce>>
Clerk: Nine berths, two free.
===
";

/// A script that draws from every randomness source the runtime exposes, and
/// interpolates each draw into text so the value reaches the event stream.
#[cfg(feature = "rand")]
const RANDOM: &str = "\
title: Tavern
---
<<declare $roll = 0>>
<<declare $luck = 0>>
<<set $roll = dice(20, 3)>>
<<set $luck = random()>>
Keeper: Three d20 gave {$roll}, on luck {$luck}.
Keeper: A thousand-sided draw is {random_range(1, 1000)}, then {random()}.
=> Keeper: The lamps are low.
=> Keeper: The lamps are lit.
=> Keeper: The lamps are out.
<<jump Cellar>>
===
title: Cellar
---
<<set $roll = dice(6, 2)>>
Cooper: Two d6 down here: {$roll}, and {random()} besides.
===
";

/// An arbitrary fixed seed. Any value works; this one is written out so a
/// failure report can be reproduced by hand.
#[cfg(feature = "rand")]
const SEED: u64 = 0x5EED_0FFA_CED1_u64;

/// The seed used by the save-file tests.
///
/// A save carries the generator's position, so a save is only byte-identical
/// between processes when the runner that produced it was seeded. Seeding is
/// not a `rand`-feature concept - `RunnerSnapshot` records the stream whether
/// or not the random built-ins are compiled in - so this constant is not gated.
#[cfg(feature = "serde")]
const SAVE_SEED: u64 = 0xB00C_u64;

// ── child side ────────────────────────────────────────────────────────────────

/// The child entry point.
///
/// Returns immediately when `MODE` is unset, which is how it behaves during an
/// ordinary `cargo test` run: the parent tests below are what drive it.
#[test]
fn the_child_worker_prints_one_payload() {
    let Ok(mode) = env::var(MODE) else {
        return;
    };
    // `--nocapture` leaves the harness's own "test <name> ... " on the same
    // line as the first thing the test prints, so start a fresh line first.
    println!();
    for line in payload(&mode).lines() {
        println!("{MARK}{line}");
    }
}

fn payload(mode: &str) -> String {
    match mode {
        "session" => session_payload(),
        #[cfg(feature = "rand")]
        "seeded" => random_payload(Some(SEED)),
        #[cfg(feature = "rand")]
        "unseeded" => random_payload(None),
        #[cfg(feature = "serde")]
        "control" => control_payload(),
        #[cfg(feature = "serde")]
        "save" => save_payload(Some(SAVE_SEED)),
        #[cfg(feature = "serde")]
        "save-unseeded" => save_payload(None),
        other => panic!("child asked for unknown payload {other:?}"),
    }
}

/// Renders a run of [`SESSION`], taking the second option at every choice.
fn session_payload() -> String {
    let mut runner = Runner::new(compile(SESSION).unwrap(), HashMapStorage::new());
    runner.start("Zulu").unwrap();
    render(&drive(&mut runner, &[1]))
}

/// Renders a run of [`RANDOM`], seeded or not.
#[cfg(feature = "rand")]
fn random_payload(seed: Option<u64>) -> String {
    use bubbles::RunnerBuilder;
    use bubbles::saliency::RandomAvailable;

    let prog = compile(RANDOM).unwrap();
    let builder = RunnerBuilder::new(prog, HashMapStorage::new()).with_saliency(RandomAvailable);
    let mut runner = match seed {
        Some(seed) => builder.with_seed(seed).build(),
        None => builder.build(),
    };
    runner.start("Tavern").unwrap();
    render(&drive(&mut runner, &[0]))
}

/// Serialises a `HashMap` holding the same keys a session would.
///
/// This is session state as it was held before it became ordered, and nothing
/// in the library reaches it. It is here as the positive control for the test
/// below: it is the shape of the defect, so it must still vary between
/// processes.
#[cfg(feature = "serde")]
fn control_payload() -> String {
    let map: std::collections::HashMap<String, u32> = (0..16)
        .map(|i| (format!("Harbour_Node_{i:02}"), i))
        .collect();
    serde_json::to_string(&map).unwrap()
}

/// Serialises what a host actually writes to disk: the snapshot, the variable
/// store, and the variable list a save editor would show.
///
/// The session is played first so the snapshot carries four visited nodes and
/// three `<<once>>` marks, and the store is filled in an order that does not
/// match sorted order. Without that, key ordering is never exercised and the
/// test would pass over an empty map.
#[cfg(feature = "serde")]
fn save_payload(seed: Option<u64>) -> String {
    use bubbles::{RunnerBuilder, Value, VariableStorage};

    let builder = RunnerBuilder::new(compile(SESSION).unwrap(), HashMapStorage::new());
    let mut runner = match seed {
        Some(seed) => builder.with_seed(seed).build(),
        None => builder.build(),
    };
    runner.start("Zulu").unwrap();
    let _ = drive(&mut runner, &[0]);
    let snapshot = runner.snapshot();
    assert!(
        snapshot.visits.len() >= 4 && snapshot.once_seen.len() >= 3,
        "payload must exercise key ordering, got {} visits and {} marks",
        snapshot.visits.len(),
        snapshot.once_seen.len()
    );

    let mut storage = HashMapStorage::new();
    for (name, value) in [
        ("$zulu", 1.0),
        ("$alpha", 2.0),
        ("$mike", 3.0),
        ("$bravo", 4.0),
        ("$yankee", 5.0),
        ("$charlie", 6.0),
    ] {
        storage.set(name, Value::Number(value));
    }
    storage.set("$name", Value::Text("Aurelia".into()));

    format!(
        "snapshot={}\nstorage={}\nvariables={:?}",
        serde_json::to_string(&snapshot).unwrap(),
        serde_json::to_string(&storage).unwrap(),
        runner.all_variables()
    )
}

/// Pumps a runner to completion, cycling `choices` at every option event, and
/// records the error if one stops the run.
///
/// The error is part of the payload because a replay needs the same failure at
/// the same point, not merely the same prefix of events.
fn drive(runner: &mut Runner<HashMapStorage>, choices: &[usize]) -> Vec<String> {
    let mut out = Vec::new();
    let mut taken = 0;
    loop {
        match runner.next_event() {
            Ok(None) => return out,
            Err(error) => {
                out.push(format!("Error({error})"));
                return out;
            }
            Ok(Some(event)) => {
                out.push(format!("{event:?}"));
                if let DialogueEvent::Options(options) = &event {
                    let wanted = choices[taken % choices.len()];
                    taken += 1;
                    let idx = options
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.available)
                        .map(|(i, _)| i)
                        .nth(wanted)
                        .or_else(|| options.iter().position(|o| o.available));
                    if let Some(idx) = idx
                        && let Err(error) = runner.select_option(idx)
                    {
                        out.push(format!("Error({error})"));
                        return out;
                    }
                }
            }
        }
    }
}

fn render(lines: &[String]) -> String {
    lines.join("\n")
}

// ── parent side ───────────────────────────────────────────────────────────────

/// Runs this test binary again in a fresh process and returns the payload it
/// printed.
fn payload_from_child(mode: &str) -> String {
    let exe = env::current_exe().expect("the test binary knows its own path");
    let output = Command::new(exe)
        .args([WORKER, "--exact", "--nocapture", "--test-threads=1"])
        .env(MODE, mode)
        .output()
        .expect("spawning the test binary again");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "child process for {mode:?} failed ({}):\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    // Taken from wherever the marker appears rather than from the start of the
    // line, so anything the harness prints ahead of it cannot drop a line.
    let payload: Vec<&str> = stdout
        .lines()
        .filter_map(|l| l.find(MARK).map(|at| &l[at + MARK.len()..]))
        .collect();
    assert!(
        !payload.is_empty(),
        "child process for {mode:?} printed no payload. Its stdout was:\n{stdout}"
    );
    payload.join("\n")
}

/// Collects the same payload from [`PROCESSES`] separate processes.
fn payloads_from_children(mode: &str) -> Vec<String> {
    (0..PROCESSES).map(|_| payload_from_child(mode)).collect()
}

/// Asserts every payload matches the first, reporting the first difference.
fn assert_all_identical(mode: &str, payloads: &[String]) {
    for (i, payload) in payloads.iter().enumerate().skip(1) {
        assert_eq!(
            payloads[0], *payload,
            "process 0 and process {i} disagree on the {mode:?} payload"
        );
    }
}

#[test]
fn the_event_stream_is_identical_in_separate_processes() {
    let payloads = payloads_from_children("session");
    assert_all_identical("session", &payloads);
    assert_eq!(
        payloads[0],
        session_payload(),
        "a child process and this process disagree on the same session"
    );
}

#[cfg(feature = "rand")]
#[test]
fn the_same_seed_replays_the_same_stream_in_separate_processes() {
    let payloads = payloads_from_children("seeded");
    assert_all_identical("seeded", &payloads);
    assert_eq!(
        payloads[0],
        random_payload(Some(SEED)),
        "a child process and this process disagree on seed {SEED:#x}"
    );
}

/// The control for the test above.
///
/// If this ever fails, the seeded test is passing for the wrong reason: either
/// the children are not separate processes, or nothing in the payload actually
/// draws from the generator. Four processes agreeing on three `f64` draws by
/// chance is not a case worth guarding against.
#[cfg(feature = "rand")]
#[test]
fn an_unseeded_run_differs_between_processes() {
    let payloads = payloads_from_children("unseeded");
    assert!(
        payloads.iter().any(|p| *p != payloads[0]),
        "every unseeded process produced the same stream, so this harness \
         cannot see per-process variation and the seeded test proves nothing:\n{}",
        payloads[0]
    );
}

#[cfg(feature = "serde")]
#[test]
fn a_saved_session_serialises_to_identical_bytes_in_separate_processes() {
    let payloads = payloads_from_children("save");
    assert_all_identical("save", &payloads);
    assert_eq!(
        payloads[0],
        save_payload(Some(SAVE_SEED)),
        "a child process and this process write different save bytes"
    );
}

/// The control for the test above, and a record of the defect it closed.
///
/// Serialising a `HashMap` with the same keys still produces different bytes in
/// different processes, which is what session state did before it became
/// ordered. So this harness can see the failure it is asserting the absence of,
/// and the test above is not passing because nothing was being compared.
#[cfg(feature = "serde")]
#[test]
fn hash_ordered_state_is_what_this_harness_can_still_catch() {
    let payloads = payloads_from_children("control");
    assert!(
        payloads.iter().any(|p| *p != payloads[0]),
        "16 keys serialised identically in {PROCESSES} processes, so either \
         `HashMap` is no longer randomly seeded or these are not separate \
         processes. Either way the save-file test above proves nothing:\n{}",
        payloads[0]
    );
}

/// Pins the one thing that still varies between processes, and that only this
/// does.
///
/// An unseeded runner takes its stream from the operating system, and the
/// snapshot records where that stream had reached, so its save bytes cannot be
/// identical between processes - and should not be, because the state genuinely
/// differs. Redacting that single field has to be enough to make the bytes
/// match: if anything else ever varies again (a map that stopped being
/// ordered, a set that started iterating), this fails rather than hiding
/// inside a difference that was expected anyway.
#[cfg(feature = "serde")]
#[test]
fn an_unseeded_save_differs_between_processes_only_in_the_recorded_stream() {
    let payloads = payloads_from_children("save-unseeded");
    assert!(
        payloads.iter().any(|p| *p != payloads[0]),
        "unseeded saves were identical between processes, so this test cannot \
         see the field it exists to isolate:\n{}",
        payloads[0]
    );

    let redacted: Vec<String> = payloads.iter().map(|p| redact_rng(p)).collect();
    assert_all_identical("save-unseeded (stream redacted)", &redacted);
}

/// Replaces the `"rng"` value in a serialised snapshot with a fixed marker.
#[cfg(feature = "serde")]
fn redact_rng(payload: &str) -> String {
    const KEY: &str = "\"rng\":";
    let at = payload
        .find(KEY)
        .expect("a snapshot records the stream state");
    let rest = &payload[at + KEY.len()..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .expect("the stream state is followed by more json");
    format!("{}{KEY}<redacted>{}", &payload[..at], &rest[end..])
}
