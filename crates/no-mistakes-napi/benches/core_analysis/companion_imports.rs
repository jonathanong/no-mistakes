use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{check_json, check_json_observed};
use std::path::PathBuf;

pub(super) fn bench_companion_imports(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/required-companion-imports")
        .canonicalize()
        .expect("companion import performance fixture should exist");
    let preflight = check_json(&root).expect("companion import preflight should succeed");
    let report: serde_json::Value = serde_json::from_str(&preflight).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["rule"], "required-companion-imports");
    assert_eq!(findings[0]["file"], "src/Component31.tsx");
    assert_eq!(findings[0]["target"], "@/Component31");
    let (observed, _) = check_json_observed(&root, true).unwrap();
    assert_eq!(observed, preflight);

    let mut group = c.benchmark_group("companion_imports");
    group.throughput(Throughput::Elements(32));
    group.bench_function("32_sources_four_shared_companions", |b| {
        b.iter(|| black_box(check_json(black_box(&root)).unwrap()));
    });
    group.finish();
}
