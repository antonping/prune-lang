use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use prune_lang::cli::args::Heuristic::LookAhead;
use prune_lang::cli::{self, args::Heuristic};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const TIMEOUT: u64 = 30;

const HEURISTICS: [Heuristic; 6] = [
    Heuristic::LeftBiased,
    Heuristic::Interleave,
    Heuristic::SmallFirst,
    Heuristic::Hybrid,
    Heuristic::LookAhead,
    Heuristic::Random,
];

fn run_benchmark(c: &mut Criterion, name: &str, depth_limits: &[usize]) {
    let mut group = c.benchmark_group(name);
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for heuristic in HEURISTICS.iter() {
        for reduction in [false, true] {
            if *heuristic == LookAhead && reduction == false {
                continue;
            }
            for depth_limit in depth_limits.iter() {
                let start = Instant::now();
                group.bench_with_input(
                    BenchmarkId::new(
                        format!("{}({:?}, {})", name, heuristic, reduction),
                        depth_limit,
                    ),
                    depth_limit,
                    |b, depth_limit| {
                        b.iter(|| {
                            cli::pipeline::run_bench_pipeline(
                                PathBuf::from(format!("./benches/{name}.pr")),
                                *heuristic,
                                reduction,
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
    }
    group.finish();
}

fn bench_concat_backward(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 10] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    run_benchmark(c, "concat_backward", &DEPTH_LIMITS);
}

fn bench_avl_tree_gen(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 9] = [20, 22, 24, 26, 28, 30, 32, 34, 36];
    run_benchmark(c, "avl_tree_gen", &DEPTH_LIMITS);
}

fn bench_avl_tree_good(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 9] = [20, 22, 24, 26, 28, 30, 32, 34, 36];
    run_benchmark(c, "avl_tree_good", &DEPTH_LIMITS);
}

fn bench_avl_tree_bad(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 9] = [20, 22, 24, 26, 28, 30, 32, 34, 36];
    run_benchmark(c, "avl_tree_bad", &DEPTH_LIMITS);
}

fn bench_unary_arith(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 10] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    run_benchmark(c, "unary_arith", &DEPTH_LIMITS);
}

fn bench_binary_arith(c: &mut Criterion) {
    const DEPTH_LIMITS: [usize; 9] = [20, 22, 24, 26, 28, 30, 32, 34, 36];
    run_benchmark(c, "binary_arith", &DEPTH_LIMITS);
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
