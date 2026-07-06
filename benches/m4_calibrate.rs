#![allow(dead_code)]

#[path = "../crates/amar-calibrate/src/common.rs"]
mod common;
#[path = "../crates/amar-calibrate/src/solve.rs"]
mod solve;

use chrono::{TimeDelta, TimeZone, Utc};
use common::Observation;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use solve::ConstituentSet;

const SYNTHETIC_SAMPLE_COUNT: usize = 45_367;

fn synthetic_samples() -> Vec<Observation> {
    let start = Utc
        .with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
        .single()
        .expect("valid UTC timestamp");
    (0..SYNTHETIC_SAMPLE_COUNT)
        .map(|index| {
            let i = index as f64;
            Observation {
                at: start + TimeDelta::hours(index as i64),
                value_m: 4.2 + (i * 0.091).sin() * 1.7 + (i * 0.037).cos() * 0.4,
                source: 4,
            }
        })
        .collect()
}

fn bench_calibrate(c: &mut Criterion) {
    let samples = synthetic_samples();
    let constituents = solve::prepare_constituents(ConstituentSet::M22Rayleigh37.specs())
        .expect("constituents must prepare");

    let mut group = c.benchmark_group("calibrate_ls_brest37_synthetic");
    group.sample_size(10);

    group.bench_function("assemble_matrix_only", |b| {
        b.iter(|| {
            black_box(
                solve::assemble_design_system(black_box(&samples), black_box(&constituents))
                    .expect("design system must assemble"),
            )
        })
    });

    let design_system = solve::assemble_design_system(&samples, &constituents)
        .expect("design system must assemble");
    let matrix = design_system.matrix;
    let values = design_system.values;
    group.bench_function("svd_only", |b| {
        b.iter_batched(
            || matrix.clone(),
            |matrix| black_box(solve::solve_svd(matrix, black_box(&values)).expect("SVD solves")),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_calibrate);
criterion_main!(benches);
