use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::check_json;
use std::path::PathBuf;

pub(super) fn bench_suppression_index(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/suppression-index");
    let mut group = c.benchmark_group("suppression_index");
    group.throughput(Throughput::Elements(3_000));
    for (name, expected) in [
        ("dense", 2_800),
        ("no-directives", 3_000),
        ("single", 0),
        ("file-disable", 0),
    ] {
        let root = fixtures.join(name).canonicalize().unwrap();
        let preflight = check_json(&root).expect("saved suppression benchmark should run");
        let report: serde_json::Value = serde_json::from_str(&preflight).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected,
            "{name}"
        );
        assert_eq!(report["warnings"], serde_json::json!([]), "{name}");
        assert_eq!(report["advisories"], serde_json::json!([]), "{name}");
        group.bench_function(name, |b| {
            b.iter(|| black_box(check_json(black_box(&root)).unwrap()));
        });
    }
    group.finish();
}
