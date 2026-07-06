use amar_core::{
    Meters, TideModel, TideThresholdDirection, UtcDateTime, predict_height, predict_series,
    tide_windows,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn brest_model() -> TideModel {
    let data = amar_data::load_pack_from_str(include_str!(
        "../data/packs/amar-data-brest-experimental.json"
    ))
    .expect("Brest pack must load");
    data.stations()
        .iter()
        .find(|station| station.pack().station_id == "refmar:3")
        .expect("Brest station must exist")
        .model()
        .clone()
}

fn at(value: &str) -> UtcDateTime {
    UtcDateTime::parse_rfc3339(value).expect("valid UTC timestamp")
}

fn bench_core(c: &mut Criterion) {
    let model = brest_model();
    let timestamp = at("2026-08-15T12:00:00Z");

    c.bench_function("predict_height_brest37_one_timestamp", |b| {
        b.iter(|| predict_height(black_box(&model), black_box(timestamp)))
    });

    c.bench_function("predict_series_brest37_72h_step6m_721_points", |b| {
        b.iter(|| {
            black_box(predict_series(
                black_box(&model),
                black_box(timestamp),
                black_box(72),
                black_box(6),
            ))
        })
    });

    let from = at("2026-08-01T00:00:00Z");
    let to = from.add_seconds(31 * 24 * 60 * 60);
    let threshold = Meters::new(4.0).expect("finite threshold");
    c.bench_function("tide_windows_brest37_31d_above_4m", |b| {
        b.iter(|| {
            black_box(tide_windows(
                black_box(&model),
                black_box(from),
                black_box(to),
                black_box(threshold),
                black_box(TideThresholdDirection::Above),
            ))
        })
    });
}

criterion_group!(benches, bench_core);
criterion_main!(benches);
