use clap::Parser;
use count_lines::args::Args;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn benchmark_cli_parsing(c: &mut Criterion) {
    c.bench_function("parse_args_simple", |b| {
        b.iter(|| {
            let args = Args::try_parse_from(black_box(["count_lines", "."])).unwrap();
            black_box(args);
        })
    });
}

criterion_group!(benches, benchmark_cli_parsing);
criterion_main!(benches);
