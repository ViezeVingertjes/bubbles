//! Runtime-path benchmarks: `next_event` throughput, interpolation, markup,
//! option selection, runner construction, and snapshot/restore.
//!
//! Every `next_event` benchmark reports throughput in **events per second** by
//! declaring `Throughput::Elements(n)` for a run that produces `n` events, so the
//! headline number is the one a host cares about: how many events it can pull per
//! frame. Runs are measured end to end (`start` through the final `None`) because
//! a per-event timing loop would measure criterion's own overhead against a body
//! of work that is only tens of nanoseconds.

mod scripts;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

use bubbles::{
    BestLeastRecentlyViewed, DialogueEvent, HashMapStorage, Program, Runner, RunnerBuilder,
    RunnerSnapshot, compile, compile_many,
};

/// A runner positioned at `start` and ready to be pumped.
///
/// Built in `iter_batched` setup so that compiling, cloning the `Program`, and
/// `start` stay outside the timed region. Without this the `next_event` numbers
/// would silently include a `Program` clone, which `compile.rs` shows costs more
/// than several hundred events.
fn ready(prog: &Program, start: &str) -> Runner<HashMapStorage> {
    let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
    runner.start(start).unwrap();
    runner
}

/// Pumps a runner to completion, returning the event count.
fn drain(runner: &mut Runner<HashMapStorage>) -> usize {
    let mut n = 0;
    while runner.next_event().unwrap().is_some() {
        n += 1;
    }
    n
}

/// Pumps a runner to completion, always selecting the first available option.
fn drain_selecting_first(runner: &mut Runner<HashMapStorage>) -> usize {
    let mut n = 0;
    while let Some(event) = runner.next_event().unwrap() {
        n += 1;
        if let DialogueEvent::Options(options) = event
            && let Some(idx) = options.iter().position(|o| o.available)
        {
            runner.select_option(idx).unwrap();
        }
    }
    n
}

/// Registers a benchmark whose throughput is the number of events it produces.
///
/// The event count is measured once up front and declared as
/// `Throughput::Elements`, so criterion reports events per second directly.
fn bench_events(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    prog: &Program,
    start: &str,
    drive: fn(&mut Runner<HashMapStorage>) -> usize,
) {
    let events = drive(&mut ready(prog, start));
    assert!(events > 0, "{name} produced no events");
    group.throughput(Throughput::Elements(events as u64));
    group.bench_function(name, |b| {
        b.iter_batched_ref(
            || ready(prog, start),
            |runner| black_box(drive(runner)),
            BatchSize::SmallInput,
        );
    });
}

fn bench_next_event(c: &mut Criterion) {
    let plain = compile(&scripts::chain("", 20, 20)).unwrap();
    let medium = compile(&scripts::medium()).unwrap();
    let interp = compile(&scripts::interpolation(200)).unwrap();
    let markup = compile(&scripts::markup(200)).unwrap();
    let options = compile(&scripts::options(50)).unwrap();
    let harbour = compile_many(&scripts::HARBOUR).unwrap();

    let mut group = c.benchmark_group("next_event");
    bench_events(&mut group, "plain_lines", &plain, "N0", drain);
    bench_events(&mut group, "conditionals_and_sets", &medium, "Entry", drain);
    bench_events(&mut group, "interpolation", &interp, "Interp", drain);
    bench_events(&mut group, "markup", &markup, "Markup", drain);
    bench_events(
        &mut group,
        "options_select_first",
        &options,
        "O0",
        drain_selecting_first,
    );
    bench_events(
        &mut group,
        "harbour_to_completion",
        &harbour,
        "Start",
        drain_selecting_first,
    );
    group.finish();
}

fn bench_saliency(c: &mut Criterion) {
    // Contrasts the stateless default against the stateful strategy on one
    // script, so the difference reported is the strategy's own cost and not a
    // difference in the dialogue being run.
    let prog = compile(&scripts::options(50)).unwrap();
    let events = drain_selecting_first(&mut ready(&prog, "O0"));

    let mut group = c.benchmark_group("saliency");
    group.throughput(Throughput::Elements(events as u64));
    group.bench_function("first_available", |b| {
        b.iter_batched_ref(
            || ready(&prog, "O0"),
            |runner| black_box(drain_selecting_first(runner)),
            BatchSize::SmallInput,
        );
    });
    group.bench_function("best_least_recently_viewed", |b| {
        b.iter_batched_ref(
            || {
                let mut runner = RunnerBuilder::new(prog.clone(), HashMapStorage::new())
                    .with_saliency(BestLeastRecentlyViewed::new())
                    .build();
                runner.start("O0").unwrap();
                runner
            },
            |runner| black_box(drain_selecting_first(runner)),
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_runner_construction(c: &mut Criterion) {
    // The guide claims runners are cheap and programs are the expensive thing.
    // This is the "runner" half; `compile.rs` measures the other half on the
    // same script, which is what makes the comparison meaningful.
    let prog = compile_many(&scripts::HARBOUR).unwrap();

    let mut group = c.benchmark_group("runner");
    group.bench_function("new_harbour", |b| {
        b.iter_batched(
            || prog.clone(),
            |p| black_box(Runner::new(p, HashMapStorage::new())),
            BatchSize::SmallInput,
        );
    });
    group.bench_function("start_harbour", |b| {
        b.iter_batched(
            || Runner::new(prog.clone(), HashMapStorage::new()),
            |mut r| {
                r.start("Start").unwrap();
                black_box(r.phase())
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_snapshot(c: &mut Criterion) {
    // Snapshot cost scales with how much state the session has accumulated, so
    // the runner is driven through every node first rather than snapshotted from
    // a fresh start where the maps would be empty.
    let prog = compile(&scripts::chain("", 40, 4)).unwrap();
    let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
    runner.start("N0").unwrap();
    while runner.next_event().unwrap().is_some() {}
    let snapshot = runner.snapshot();

    let mut group = c.benchmark_group("snapshot");
    group.bench_function("snapshot_40_visits", |b| {
        b.iter(|| black_box(runner.snapshot()));
    });
    group.bench_function("restore_40_visits", |b| {
        b.iter_batched(
            || snapshot.clone(),
            |s| {
                let mut r = Runner::new(prog.clone(), HashMapStorage::new());
                r.restore(s).unwrap();
                black_box(r.phase())
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("round_trip_40_visits", |b| {
        b.iter(|| {
            let s: RunnerSnapshot = runner.snapshot();
            let mut r = Runner::new(prog.clone(), HashMapStorage::new());
            r.restore(s).unwrap();
            black_box(r.phase())
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_next_event,
    bench_saliency,
    bench_runner_construction,
    bench_snapshot
);
criterion_main!(benches);
