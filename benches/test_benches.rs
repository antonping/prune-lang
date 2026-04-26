use criterion::{Criterion, criterion_group, criterion_main};
use prune_lang::cli::{self, args::Heuristic};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const TIMEOUT: u64 = 30;

const HEURISTICS: [Heuristic; 5] = [
    Heuristic::LeftBiased,
    Heuristic::Interleave,
    Heuristic::StructRecur,
    Heuristic::LookAhead,
    Heuristic::Random,
];

fn bench_concat_backward(c: &mut Criterion) {
    let mut group = c.benchmark_group("concat_backward");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            group.bench_function(
                format!("concat_backward({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/concat_backward.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap()
                    })
                },
            );
        }
    }
    group.finish();
}

fn bench_avl_tree_gen(c: &mut Criterion) {
    let mut group = c.benchmark_group("avl_tree_gen");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [20, 22, 24, 26, 28, 30, 32, 34, 36];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            let start = Instant::now();
            group.bench_function(
                format!("avl_tree_gen({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/avl_tree_gen.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap()
                    })
                },
            );
            if start.elapsed().as_secs() > TIMEOUT {
                break; // remaining tests will cost too much time!
            }
        }
    }
    group.finish();
}

fn bench_avl_tree_good(c: &mut Criterion) {
    let mut group = c.benchmark_group("avl_tree_good");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [20, 22, 24, 26, 28, 30, 32, 34, 36];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            let start = Instant::now();
            group.bench_function(
                format!("avl_tree_good({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/avl_tree_good.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap();
                    })
                },
            );
            if start.elapsed().as_secs() > TIMEOUT {
                break; // remaining tests will cost too much time!
            }
        }
    }
    group.finish();
}

fn bench_avl_tree_bad(c: &mut Criterion) {
    let mut group = c.benchmark_group("avl_tree_bad");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [20, 22, 24, 26, 28, 30, 32, 34, 36];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            let start = Instant::now();
            group.bench_function(
                format!("avl_tree_bad({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/avl_tree_bad.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap()
                    })
                },
            );
            if start.elapsed().as_secs() > TIMEOUT {
                break; // remaining tests will cost too much time!
            }
        }
    }
    group.finish();
}

fn bench_unary_arith(c: &mut Criterion) {
    let mut group = c.benchmark_group("unary_arith");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            let start = Instant::now();
            group.bench_function(
                format!("unary_arith({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/unary_arith.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap()
                    })
                },
            );
            if start.elapsed().as_secs() > TIMEOUT {
                break; // remaining tests will cost too much time!
            }
        }
    }
    group.finish();
}

fn bench_binary_arith(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_arith");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let depth_limits = [20, 22, 24, 26, 28, 30, 32, 34, 36];

    for heuristic in HEURISTICS.iter() {
        for depth_limit in depth_limits.iter() {
            let start = Instant::now();
            group.bench_function(
                format!("binary_arith({:?}, {})", heuristic, depth_limit),
                |b| {
                    b.iter(|| {
                        cli::pipeline::run_bench_pipeline(
                            PathBuf::from("./benches/binary_arith.pr"),
                            *heuristic,
                            *depth_limit,
                        )
                        .unwrap()
                    })
                },
            );
            if start.elapsed().as_secs() > TIMEOUT {
                break; // remaining tests will cost too much time!
            }
        }
    }
    group.finish();
}

// todo: more benchmarks in real times

criterion_group!(
    benches,
    bench_concat_backward,
    bench_avl_tree_gen,
    bench_avl_tree_good,
    bench_avl_tree_bad,
    bench_unary_arith,
    bench_binary_arith,
);

criterion_main!(benches);
