use super::shard;
use criterion::{black_box, BatchSize, Criterion, Throughput};
use no_mistakes::benchmark_support::construct_annotation_module_indexes;
use no_mistakes::codebase::ts_source::{
    discover_visible_paths,
    facts::{collect_ts_facts, TsFactPlan},
};
use std::path::PathBuf;
use std::sync::Arc;

/// Candidate-only constructor control; full checks measure preparation and
/// evaluation together with an unchanged baseline-compatible harness.
pub(super) fn bench_annotation_module_indexes(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-annotations")
        .canonicalize()
        .expect("saved annotation fixture");
    let paths = discover_visible_paths(&root);
    let syntax = collect_ts_facts(
        &paths,
        TsFactPlan {
            imports: true,
            module_bindings: true,
            ..Default::default()
        },
    );
    let syntax = syntax
        .into_iter()
        .map(|(_, facts)| Arc::new(facts))
        .collect::<Vec<_>>();
    let indexes = construct_annotation_module_indexes(&syntax);
    assert!(indexes.binding_count() > 12);
    let mut group = c.benchmark_group("annotation_module_indexes");
    group.throughput(Throughput::Elements(syntax.len() as u64));
    group.bench_function("preparation", |b| {
        b.iter_batched(
            || (),
            |_| black_box(construct_annotation_module_indexes(black_box(&syntax))),
            BatchSize::SmallInput,
        );
    });
    group.finish();
}
