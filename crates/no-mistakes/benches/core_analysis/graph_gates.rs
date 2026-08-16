//! Step-up corpus for visitor-fusion and outer-graph-parallelism gates.
//! Sized above the 14-file core-analysis fixture and below the 246-file
//! large-graph-monorepo acceptance fixture so CI benches stay in-process.

#[path = "graph_gates_support.rs"]
mod support;

use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::dependencies::NodeId;
use no_mistakes::codebase::ts_resolver::load_tsconfig;
use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use support::{
    build_graph, expect_count, expect_kind_counts, fact_totals, file_nodes, fixture_root,
    gate_plan, source_files, traversal_snapshot, EXPECTED_FORWARD_DEPS, EXPECTED_GRAPH_NODES,
    EXPECTED_IMPORTS, EXPECTED_REVERSE_DEPENDENTS, EXPECTED_SOURCE_FILES, EXPECTED_SYMBOL_EXPORTS,
    EXPECTED_SYMBOL_IMPORTS, EXPECTED_SYMBOL_NODES, EXPECTED_SYMBOL_REFS, FORWARD_ROOTS,
    REVERSE_ROOTS,
};

pub(super) fn bench_graph_gates(c: &mut Criterion) {
    let root = fixture_root();
    let files = source_files(&root);
    let config = load_tsconfig(&root.join("tsconfig.json")).expect("graph-gates tsconfig");
    let config_path = root.join(".no-mistakes.yml");
    let plan = TsFactPlan::imports_and_symbols();

    {
        let facts_preflight = collect_ts_facts(&files, plan);
        expect_count("fact files", facts_preflight.len(), EXPECTED_SOURCE_FILES);
        assert!(facts_preflight
            .values()
            .all(|facts| facts.operational_error.is_none() && facts.parse_error.is_none()));
        let (imports, symbol_imports, symbol_exports, symbol_refs) = fact_totals(&facts_preflight);
        expect_count("imports", imports, EXPECTED_IMPORTS);
        expect_count("symbol imports", symbol_imports, EXPECTED_SYMBOL_IMPORTS);
        expect_count("symbol exports", symbol_exports, EXPECTED_SYMBOL_EXPORTS);
        expect_count("symbol refs", symbol_refs, EXPECTED_SYMBOL_REFS);
    }

    let serial = build_graph(&root, &config, &config_path, 1);
    let parallel = build_graph(&root, &config, &config_path, 4);
    assert_eq!(
        traversal_snapshot(&serial),
        traversal_snapshot(&parallel),
        "serial and 4-thread graph builds must preserve traversal order"
    );
    drop(serial);
    let preflight = parallel;
    let forward_roots = file_nodes(&root, FORWARD_ROOTS);
    let reverse_roots = file_nodes(&root, REVERSE_ROOTS);
    let deps = preflight.deps_of(&forward_roots, None, None);
    let dependents = preflight.dependents_of(&reverse_roots, None, None);
    let symbol_nodes = preflight
        .all_files()
        .filter(|node| matches!(node, NodeId::Symbol { .. }))
        .count();
    expect_count(
        "graph nodes",
        preflight.all_files().count(),
        EXPECTED_GRAPH_NODES,
    );
    expect_count("symbol nodes", symbol_nodes, EXPECTED_SYMBOL_NODES);
    expect_kind_counts(&preflight);
    expect_count("forward deps", deps.len(), EXPECTED_FORWARD_DEPS);
    expect_count(
        "reverse dependents",
        dependents.len(),
        EXPECTED_REVERSE_DEPENDENTS,
    );

    let mut facts_group = c.benchmark_group("graph_gates_facts");
    facts_group.throughput(Throughput::Elements(EXPECTED_SOURCE_FILES as u64));
    facts_group.bench_function("extract", |b| {
        b.iter(|| black_box(collect_ts_facts(black_box(&files), black_box(plan))));
    });
    facts_group.finish();

    let mut build_group = c.benchmark_group("graph_gates_build");
    build_group.throughput(Throughput::Elements(EXPECTED_GRAPH_NODES as u64));
    for threads in [1usize, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("graph-gates rayon pool");
        build_group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, _| {
            b.iter(|| {
                pool.install(|| {
                    black_box(
                        DepGraph::build_with_plan_and_config(
                            black_box(&root),
                            black_box(&config),
                            black_box(gate_plan()),
                            Some(black_box(&config_path)),
                        )
                        .expect("graph-gates build should succeed"),
                    )
                })
            });
        });
    }
    build_group.finish();

    let mut query_group = c.benchmark_group("graph_gates_query");
    query_group.throughput(Throughput::Elements(
        (FORWARD_ROOTS.len() + REVERSE_ROOTS.len()) as u64,
    ));
    query_group.bench_function("forward_reverse", |b| {
        b.iter(|| {
            let deps = preflight.deps_of(black_box(&forward_roots), None, None);
            let dependents = preflight.dependents_of(black_box(&reverse_roots), None, None);
            black_box((deps.len(), dependents.len()))
        });
    });
    query_group.finish();
}
