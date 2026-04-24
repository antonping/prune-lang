use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use prune_lang::cli::{self, args::Heuristic};

fn bench_concat_backward(c: &mut Criterion) {
    let heuristics = [
        Heuristic::LeftBiased,
        Heuristic::Interleave,
        Heuristic::LookAhead,
        Heuristic::Random,
    ];

    let mut group = c.benchmark_group("concat_backward");

    for heuristic in heuristics.iter() {
        group.bench_function(format!("concat_backward({:?})", heuristic), |b| {
            b.iter(|| {
                cli::pipeline::run_bench_pipeline(
                    PathBuf::from("./benches/concat_backward.pr"),
                    *heuristic,
                )
                .unwrap()
            })
        });
    }
    group.finish();
}

fn bench_unary_arith(c: &mut Criterion) {
    let heuristics = [
        Heuristic::LeftBiased,
        Heuristic::Interleave,
        Heuristic::LookAhead,
        Heuristic::Random,
    ];

    let mut group = c.benchmark_group("unary_arith");

    for heuristic in heuristics.iter() {
        group.bench_function(format!("unary_arith({:?})", heuristic), |b| {
            b.iter(|| {
                cli::pipeline::run_bench_pipeline(
                    PathBuf::from("./benches/unary_arith.pr"),
                    *heuristic,
                )
                .unwrap()
            })
        });
    }
    group.finish();
}

// todo: more benchmarks in real times

criterion_group!(benches, bench_unary_arith, bench_concat_backward);

criterion_main!(benches);
