use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    analyze_project_json, check_json, prepare_dependency_pins, project_dependency_pins,
};
use std::path::PathBuf;

pub(super) fn bench_dependency_pin_jsx(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/dependency-pin-jsx")
        .canonicalize()
        .unwrap();
    let report: serde_json::Value = serde_json::from_str(&check_json(&root).unwrap()).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 128);
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let pin_only_options = serde_json::json!({"root": root, "config": ".pin-only-no-mistakes.yml", "reports": [{"type": "check"}]}).to_string();
    let control: serde_json::Value =
        serde_json::from_str(&analyze_project_json(pin_only_options.clone()).unwrap()).unwrap();
    assert_eq!(
        control["reports"][0]["result"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        128
    );
    let prepared = prepare_dependency_pins(&root).unwrap();
    assert_eq!(project_dependency_pins(&prepared), 128);
    let mut group = c.benchmark_group("dependency_pin_jsx");
    group.throughput(Throughput::Elements(32));
    group.bench_function("check_32_saved_sources", |b| {
        b.iter(|| black_box(check_json(black_box(&root)).unwrap()))
    });
    group.bench_function("pin_only_control_32_saved_sources", |b| {
        b.iter(|| black_box(analyze_project_json(black_box(pin_only_options.clone())).unwrap()))
    });
    group.bench_function("compile_and_extract_32_saved_sources", |b| {
        b.iter(|| black_box(prepare_dependency_pins(black_box(&root)).unwrap()))
    });
    group.bench_function("project_32_saved_sources", |b| {
        b.iter(|| black_box(project_dependency_pins(black_box(&prepared))))
    });
    group.finish();
}
