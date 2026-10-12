use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{check_json, check_json_observed};
use std::path::PathBuf;

pub(super) fn bench_dependency_checks(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/dependency-checks")
        .canonicalize()
        .expect("dependency check performance fixture should exist");
    let preflight = check_json(&root).expect("dependency check preflight should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&preflight).expect("dependency report should be JSON");
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().expect("rule findings array");
    assert_eq!(
        findings.len(),
        1,
        "one deliberately dev-only runtime import"
    );
    assert_eq!(findings[0]["rule"], "production-dependency-declarations");
    assert_eq!(findings[0]["file"], "packages/p12/local.ts");
    assert_eq!(findings[0]["line"], 1);
    assert_eq!(findings[0]["target"], "fixture-runtime");
    let (observed, _) =
        check_json_observed(&root, true).expect("observed dependency check should succeed");
    assert_eq!(observed, preflight);

    let mut group = c.benchmark_group("dependency_checks");
    group.throughput(Throughput::Elements(48));
    group.bench_function("24_packages_three_lockfile_applications", |b| {
        // Construct fresh request inputs so preparation and traversal costs
        // stay visible rather than warming caches between iterations.
        b.iter(|| {
            black_box(check_json(black_box(&root)).expect("dependency check should succeed"))
        });
    });
    group.finish();
}
