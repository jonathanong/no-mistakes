use super::fixtures::{fixture_root, source_files, traverse_args, tsconfig, EXPECTED_SOURCE_FILES};
use super::shard;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::benchmark_support;
use no_mistakes::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes::codebase::dependencies::{self, Direction, RelationshipArg};
use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

pub(super) fn bench_lazy_traversal(c: &mut Criterion) {
    if !shard::should_run(shard::GRAPH) {
        return;
    }
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
    if !shard::should_run(shard::GRAPH) {
        return;
    }
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
    let root_node = no_mistakes::codebase::dependencies::NodeId::file(root.join("src/app.tsx"));
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

pub(super) fn bench_high_fanout_finalization(c: &mut Criterion) {
    if shard::should_run(shard::GRAPH) {
        let mut group = c.benchmark_group("graph_finalization");
        for (name, nodes, fanout) in [("large", 4_096, 16), ("high_fanout", 1_024, 128)] {
            let fixture = benchmark_support::high_fanout_finalization_fixture(nodes, fanout);
            let expected_edges = (nodes * fanout) as usize;
            assert_eq!(
                benchmark_support::finalize_high_fanout_adjacency(fixture.clone()).canonical_edges,
                expected_edges,
                "duplicate input edges must not inflate finalized graph size"
            );
            group.throughput(Throughput::Elements(expected_edges as u64));
            group.bench_with_input(
                BenchmarkId::new(name, expected_edges),
                &fixture,
                |b, fixture| {
                    b.iter_batched(
                        || fixture.clone(),
                        |fixture| {
                            black_box(benchmark_support::finalize_high_fanout_adjacency(
                                black_box(fixture),
                            ));
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
        group.finish();
    }

    if !shard::should_run_any(&[shard::GRAPH, shard::GRAPH_PRODUCTION]) {
        return;
    }
    // The general memory shard excludes the two expensive production cases;
    // dedicated shards run each one under its existing benchmark identity.
    if std::env::var("NO_MISTAKES_BENCH_SHARD").as_deref() == Ok(shard::GENERAL_MEMORY) {
        return;
    }

    let mut production = c.benchmark_group("graph_production_finalization");
    let fixture = benchmark_support::production_graph_fixture(1_024, 128);
    let expected_edges = 1_024 * 128;
    assert_eq!(
        benchmark_support::finalize_production_graph(fixture.clone()).canonical_edges,
        expected_edges
    );
    assert_eq!(
        benchmark_support::append_production_selectors(fixture.clone()).selector_appended_edges,
        expected_edges
    );
    production.throughput(Throughput::Elements(expected_edges as u64));
    production.bench_function("node_id_finalization", |b| {
        b.iter_batched(
            || fixture.clone(),
            |fixture| {
                black_box(benchmark_support::finalize_production_graph(black_box(
                    fixture,
                )))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    production.bench_function("selector_append", |b| {
        b.iter_batched(
            || fixture.clone(),
            |fixture| {
                black_box(benchmark_support::append_production_selectors(black_box(
                    fixture,
                )))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    production.finish();
}
