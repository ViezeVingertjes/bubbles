//! Seeded reproducibility and canonical snapshot ordering.
//!
//! The cross-process property tests live with the QA suite; these cover what a
//! single process can observe.

mod common;

use bubbles::{HashMapStorage, Runner, compile};
use common::drain;

#[cfg(feature = "rand")]
mod seeded {
    use bubbles::{
        DialogueEvent, HashMapStorage, RandomAvailable, Runner, RunnerBuilder, Value, compile,
    };

    use super::{common, drain};

    /// Exercises every random path the runtime has: the three built-ins and a
    /// line group resolved by the saliency strategy.
    const RANDOM_SRC: &str = "\
title: A
---
<<set $roll = dice(20, 3)>>
<<set $pick = random_range(1, 1000)>>
Rolled {$roll}, picked {$pick}, drew {random()}.
=> One.
=> Two.
=> Three.
<<jump B>>
===
title: B
---
<<set $roll2 = dice(6, 2)>>
And then {$roll2}.
===
";

    fn seeded_runner(seed: u64) -> Runner<HashMapStorage> {
        RunnerBuilder::new(
            compile(RANDOM_SRC).expect("compile failed"),
            HashMapStorage::new(),
        )
        .with_saliency(RandomAvailable)
        .with_seed(seed)
        .build()
    }

    fn play_seeded(seed: u64) -> Vec<DialogueEvent> {
        let mut runner = seeded_runner(seed);
        runner.start("A").expect("start failed");
        drain(&mut runner)
    }

    #[test]
    fn the_same_seed_replays_the_same_event_stream() {
        assert_eq!(play_seeded(20_260_815), play_seeded(20_260_815));
    }

    #[test]
    fn different_seeds_produce_different_event_streams() {
        // Three dice rolls, a 1-in-1000 range and a float: a coincidental
        // match is not a realistic flake.
        assert_ne!(play_seeded(1), play_seeded(2));
    }

    #[test]
    fn unseeded_runners_still_draw_from_the_operating_system() {
        let prog = compile(RANDOM_SRC).unwrap();
        let mut first = Runner::new(prog.clone(), HashMapStorage::new());
        let mut second = Runner::new(prog, HashMapStorage::new());
        first.start("A").unwrap();
        second.start("A").unwrap();
        assert_ne!(drain(&mut first), drain(&mut second));
    }

    #[test]
    fn a_restored_snapshot_resumes_the_same_stream() {
        let mut original = seeded_runner(99);
        original.start("A").unwrap();
        let _ = original.next_event().unwrap(); // NodeStarted
        let _ = original.next_event().unwrap(); // the rolled line
        let snapshot = original.snapshot();

        // Where the stream had reached travels with the save, so two runners
        // with unrelated seeds replay it identically.
        let mut first = seeded_runner(12_345);
        first.restore(snapshot.clone()).unwrap();
        let mut second = seeded_runner(678);
        second.restore(snapshot).unwrap();
        let restored = drain(&mut first);
        assert_eq!(restored, drain(&mut second));

        // And it is genuinely the saved stream, not either runner's own seed.
        assert_ne!(restored, play_seeded(12_345));
    }

    #[test]
    fn host_functions_can_join_the_seeded_stream() {
        let prog = compile("title: A\n---\nHost rolled {host_roll()}.\n===\n").unwrap();
        let text_for = |seed: u64| {
            let mut runner = RunnerBuilder::new(prog.clone(), HashMapStorage::new())
                .with_seed(seed)
                .build();
            let rng = runner.rng().clone();
            runner.library_mut().register("host_roll", move |_args| {
                Ok(Value::Number(f64::from(
                    i32::try_from(rng.range_inclusive(1, 100)).unwrap_or(1),
                )))
            });
            runner.start("A").unwrap();
            common::drain_line_texts(&mut runner)
        };
        assert_eq!(text_for(4), text_for(4));
        assert_ne!(text_for(4), text_for(5));
    }
}

#[test]
fn visit_counts_and_variables_come_back_in_sorted_order() {
    let src = "\
title: Start
---
<<set $zeta = 1>>
<<set $alpha = 2>>
<<set $mid = 3>>
<<jump Zed>>
===
title: Zed
---
<<jump Alpha>>
===
title: Alpha
---
Done.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let _ = drain(&mut runner);

    let names: Vec<String> = runner
        .all_variables()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, vec!["$alpha", "$mid", "$zeta"]);

    let snapshot = runner.snapshot();
    let visited: Vec<&str> = snapshot.visits.keys().map(String::as_str).collect();
    assert_eq!(visited, vec!["Alpha", "Start", "Zed"]);
}

#[cfg(feature = "serde")]
mod canonical_bytes {
    use bubbles::{HashMapStorage, Runner, RunnerBuilder, Value, VariableStorage, compile};

    use super::{common, drain};

    #[test]
    fn storage_filled_in_either_order_serialises_identically() {
        let names = ["$a", "$b", "$c", "$d", "$e", "$f", "$g", "$h"];
        let mut forwards = HashMapStorage::new();
        for name in names {
            forwards.set(name, Value::Text((*name).into()));
        }
        let mut backwards = HashMapStorage::new();
        for name in names.iter().rev() {
            backwards.set(name, Value::Text((*name).into()));
        }

        assert_eq!(
            serde_json::to_string(&forwards).unwrap(),
            serde_json::to_string(&backwards).unwrap()
        );
    }

    #[test]
    fn snapshots_of_the_same_session_serialise_identically() {
        let src = "\
title: Hub
---
<<once>>
    First time only.
<<endonce>>
<<jump Side>>
===
title: Side
---
Elsewhere.
===
";
        let prog = compile(src).unwrap();
        let snapshot_json = || {
            let mut runner = RunnerBuilder::new(prog.clone(), HashMapStorage::new())
                .with_seed(1)
                .build();
            for _ in 0..3 {
                runner.start("Hub").unwrap();
                let _ = drain(&mut runner);
            }
            serde_json::to_string(&runner.snapshot()).unwrap()
        };

        assert_eq!(snapshot_json(), snapshot_json());
    }

    #[test]
    fn a_snapshot_written_before_seeding_still_loads() {
        // The shape 2.x wrote, which has no `rng` field.
        let legacy = r#"{"current_node":"A","visits":{"A":1},"once_seen":[]}"#;
        let snapshot: bubbles::RunnerSnapshot = serde_json::from_str(legacy).unwrap();
        assert_eq!(snapshot.current_node.as_deref(), Some("A"));
        assert_eq!(snapshot.visits.get("A"), Some(&1));

        let prog = compile("title: A\n---\nHi.\n===\n").unwrap();
        let mut runner = Runner::new(prog, HashMapStorage::new());
        runner.restore(snapshot).unwrap();
        let _ = common::drain(&mut runner);
    }
}
