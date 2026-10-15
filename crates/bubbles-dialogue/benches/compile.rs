//! Compile-path benchmarks: `compile`, `compile_many`, and `Program` cloning.
//!
//! These exist to put a number on the two claims the guide makes about compiling:
//! that recompiling is "fast but not free" and that a `Program` is "cheap to
//! clone". Both are relative claims, so each benchmark here has a counterpart in
//! `runtime.rs` to compare against — a compile is only expensive relative to the
//! runner construction it is contrasted with.

mod scripts;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

use bubbles::{compile, compile_many};

fn bench_compile(c: &mut Criterion) {
    let medium = scripts::medium();
    // `examples/harbour/` is deliberately absent from this group: `harbour.bub`
    // jumps to nodes defined in `services.bub`, so `compile` on it alone fails
    // validation. It is a multi-file program and is measured as one in
    // `compile_many/harbour_2_files`. A third single-file size point comes from a
    // generated script instead.
    let large = scripts::chain("", 100, 20);

    let mut group = c.benchmark_group("compile");
    // Throughput is reported per byte of source so the three sizes are directly
    // comparable; a regression that only affects large inputs shows up as a
    // divergence between them rather than a uniform slowdown.
    group.throughput(Throughput::Bytes(scripts::SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| compile(black_box(scripts::SMALL)).unwrap());
    });

    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| compile(black_box(medium.as_str())).unwrap());
    });

    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| compile(black_box(large.as_str())).unwrap());
    });
    group.finish();
}

fn bench_compile_many(c: &mut Criterion) {
    let total: usize = scripts::HARBOUR.iter().map(|(_, s)| s.len()).sum();

    let mut group = c.benchmark_group("compile_many");
    group.throughput(Throughput::Bytes(total as u64));
    group.bench_function("harbour_2_files", |b| {
        b.iter(|| compile_many(black_box(&scripts::HARBOUR)).unwrap());
    });

    // Many small files is the shape a project with one file per scene has, and it
    // is the case where per-file overhead rather than per-byte cost dominates.
    // Node titles must be unique across files, so each source gets its own prefix.
    let sources: Vec<(String, String)> = (0..16)
        .map(|i| {
            (
                format!("scene{i}.bub"),
                scripts::chain(&format!("F{i}"), 4, 8),
            )
        })
        .collect();
    let pairs: Vec<(&str, &str)> = sources
        .iter()
        .map(|(name, src)| (name.as_str(), src.as_str()))
        .collect();
    let bytes: usize = sources.iter().map(|(_, s)| s.len()).sum();
    group.throughput(Throughput::Bytes(bytes as u64));
    group.bench_function("16_small_files", |b| {
        b.iter(|| compile_many(black_box(&pairs)).unwrap());
    });
    group.finish();
}

fn bench_program_clone(c: &mut Criterion) {
    let prog = compile_many(&scripts::HARBOUR).unwrap();

    let mut group = c.benchmark_group("program");
    group.bench_function("clone_harbour", |b| {
        b.iter(|| black_box(prog.clone()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_compile,
    bench_compile_many,
    bench_program_clone
);
criterion_main!(benches);
