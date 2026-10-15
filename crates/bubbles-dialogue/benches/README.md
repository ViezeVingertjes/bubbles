# Benchmarks

```sh
cargo bench -p bubbles-dialogue --all-features

# one group
cargo bench -p bubbles-dialogue --all-features --bench runtime -- next_event

# compile and run each benchmark once without timing it, which is what CI does
cargo bench -p bubbles-dialogue --all-features --bench compile --bench runtime -- --test
```

`compile.rs` covers the compile path, `runtime.rs` the runner. Inputs come from
`scripts/mod.rs`: `SMALL` and the generators describe their input by shape, so a
number can be read against a known node and line count, and `HARBOUR` is the
real script under `examples/harbour/`.

Runtime benchmarks report **events per second**. The runner is built in
`iter_batched` setup, so compiling, cloning the `Program`, and `start` stay out
of the timed region — including them overstated per-event cost by 10–40%.
Compile benchmarks report **bytes per second**.

`examples/harbour/` appears under `compile_many` and not `compile`, because
`harbour.bub` jumps into `services.bub` and does not pass validation on its own.

## Recorded run

Read these as a baseline for comparison on the same machine, not as a figure to
quote. See [On the numbers](#on-the-numbers) below.

```
AMD Ryzen 7 5825U, 16 threads, 14 GiB RAM
Linux 7.0.0-38-generic x86_64
rustc 1.99.0 (b940084d7 2026-09-28), release profile, --all-features
2026-10-15
```

Compiling:

| Benchmark | Median | Throughput |
|---|---|---|
| `compile/small` (1 node, 1 line) | 701 ns | 108 MiB/s |
| `compile/medium` (40 nodes, 10 lines each, conditionals) | 217 µs | 72 MiB/s |
| `compile/large` (100 nodes, 20 lines each) | 632 µs | 127 MiB/s |
| `compile_many/harbour_2_files` | 34.7 µs | 115 MiB/s |
| `compile_many/16_small_files` | 222 µs | 94 MiB/s |
| `program/clone_harbour` | 1.09 µs | |

Running:

| Benchmark | Median | Throughput |
|---|---|---|
| `next_event/plain_lines` | 43.5 µs | 9.7 M events/s |
| `next_event/conditionals_and_sets` | 57.5 µs | 8.4 M events/s |
| `next_event/interpolation` | 139 µs | 1.46 M events/s |
| `next_event/markup` | 59.7 µs | 3.40 M events/s |
| `next_event/options_select_first` | 29.5 µs | 5.16 M events/s |
| `next_event/harbour_to_completion` | 2.77 µs | 6.13 M events/s |
| `saliency/first_available` | 29.9 µs | 5.09 M events/s |
| `saliency/best_least_recently_viewed` | 31.0 µs | 4.90 M events/s |
| `runner/new_harbour` | 707 ns | |
| `runner/start_harbour` | 1.16 µs | |
| `snapshot/snapshot_40_visits` | 1.16 µs | |
| `snapshot/restore_40_visits` | 7.11 µs | |
| `snapshot/round_trip_40_visits` | 10.5 µs | |

What the numbers say:

- Plain line emission runs at roughly 10 M events/s. A frame budget of 1 ms
  would allow ten thousand events, and a conversation turn is a handful, so the
  runner is not where a host's frame goes.
- Interpolation is the slowest path by a factor of six against plain lines.
  That is where to look first if anything ever needs to be faster.
- `BestLeastRecentlyViewed` costs about 4% over `FirstAvailable` on the same
  script. Choosing the stateful strategy is not a performance decision.
- A `Program` clone is 1.09 µs against 34.7 µs to compile the same two files, so
  compiling once and cloning per conversation is worth roughly 30x. The ratio is
  the point rather than either figure; `tests/allocation_budget.rs` asserts it in
  allocation terms, which holds on any machine.

## On the numbers

The machine above was not quiescent — it had a browser and other builds running,
and load average sat near 4. Repeated runs of the same commit moved by 10–40%,
which is wider than most regressions worth catching. Treat the table as an
order-of-magnitude record of what the hardware and toolchain were, not as a
precise result.

This is also why **CI does not enforce timing thresholds.** On a shared runner a
threshold tight enough to catch a real regression fires constantly, and one
loose enough to be quiet catches nothing. CI instead runs every benchmark once
via `-- --test`, which catches the failure that actually happens: a benchmark
that stopped compiling or started panicking.

What is gated are the two things that do not depend on how busy the machine is:

- `tests/allocation_budget.rs` — allocation counts on the hot paths, which are
  machine-independent.
- `scripts/measure-footprint.sh --check` — wasm and binary size, which are
  deterministic for a given toolchain.

To compare two commits, run `cargo bench` on both on the same idle machine and
read criterion's own change report, which is what it is designed for. Do not
compare against the table above across different hardware.
