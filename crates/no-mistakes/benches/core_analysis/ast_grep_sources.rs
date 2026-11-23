use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::check_json;
use std::path::PathBuf;

pub(super) fn bench_ast_grep_source_reuse(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/ast-grep-source-reuse");
    let mut group = c.benchmark_group("ast_grep_source_reuse");
    for (name, applications) in [("single", 1), ("repeated-12", 12), ("repeated-20", 20)] {
        let root = fixtures.join(name).canonicalize().unwrap();
        let preflight = check_json(&root).expect("saved YAML benchmark should run");
        let report: serde_json::Value = serde_json::from_str(&preflight).unwrap();
        for field in ["rules", "warnings", "advisories"] {
            assert_eq!(report[field], serde_json::json!([]), "{name}: {field}");
        }
        group.throughput(Throughput::Elements(10 * applications));
        group.bench_function(name, |b| {
            b.iter(|| black_box(check_json(black_box(&root)).unwrap()));
        });
    }
    group.finish();
}
