use super::fixtures::{fixture_root, source_files, traverse_args, tsconfig, EXPECTED_SOURCE_FILES};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes::codebase::dependencies::{self, Direction, RelationshipArg};
use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

pub(super) fn bench_lazy_traversal(c: &mut Criterion) {
    let root = fixture_root();
    let mut group = c.benchmark_group("lazy_traversal");
    for roots in [
        &["src/app.tsx"][..],
        &["src/app.tsx", "src/jobs/send.ts"][..],
    ] {
        let args = traverse_args(&root, roots, RelationshipArg::Import);
        let expected = dependencies::run_json(args, Direction::Deps)
            .expect("lazy traversal preflight should succeed");
        assert!(expected.contains("packages/core/src/index.ts"));
        group.throughput(Throughput::Elements(roots.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(roots.len()),
            &roots.len(),
            |b, _| {
                b.iter(|| {
                    let args = traverse_args(&root, roots, RelationshipArg::Import);
                    black_box(
                        dependencies::run_json(black_box(args), Direction::Deps)
                            .expect("lazy traversal should succeed"),
                    )
                });
            },
        );
    }
    group.finish();
}

pub(super) fn bench_facts_graph_and_query(c: &mut Criterion) {
    let root = fixture_root();
    let files = source_files(&root);
    let config = tsconfig(&root);

    let facts_preflight = collect_ts_facts(&files, TsFactPlan::imports_and_symbols());
    assert_eq!(
        facts_preflight.len(),
        EXPECTED_SOURCE_FILES,
        "fact extraction must preserve one result per fixture source"
    );

    c.bench_function("facts/imports_and_symbols", |b| {
        b.iter(|| {
            black_box(collect_ts_facts(
                black_box(&files),
                black_box(TsFactPlan::imports_and_symbols()),
            ))
        });
    });

    let preflight = DepGraph::build_with_plan_and_config(
        &root,
        &config,
        GraphBuildPlan::all(),
        Some(&root.join(".no-mistakes.yml")),
    )
    .expect("graph preflight should succeed");
    let root_node = no_mistakes::codebase::dependencies::NodeId::File(root.join("src/app.tsx"));
    assert!(!preflight
        .deps_of(std::slice::from_ref(&root_node), None, None)
        .is_empty());
    assert!(preflight.all_files().count() >= EXPECTED_SOURCE_FILES);

    c.bench_function("graph/all_domains_build", |b| {
        b.iter(|| {
            black_box(
                DepGraph::build_with_plan_and_config(
                    black_box(&root),
                    black_box(&config),
                    black_box(GraphBuildPlan::all()),
                    Some(&root.join(".no-mistakes.yml")),
                )
                .expect("graph build should succeed"),
            )
        });
    });

    c.bench_function("graph/forward_reverse_query", |b| {
        b.iter(|| {
            let deps = preflight.deps_of(black_box(std::slice::from_ref(&root_node)), None, None);
            let dependents =
                preflight.dependents_of(black_box(std::slice::from_ref(&root_node)), None, None);
            black_box((deps.len(), dependents.len()))
        });
    });
}
